use core::fmt;

use crate::coverage::Coverage;
use crate::snapshot::{
    BACKEND_ENERGY, Cursor, SnapshotError, decode_envelope, encode_envelope, push_i64, push_u32,
    push_u64,
};

const ENERGY_PROFILE_CUSTOM: u8 = 0;
const ENERGY_PROFILE_PROVEN: u8 = 1;
const ENERGY_SNAPSHOT_METADATA_LEN: usize = 16;
const ENERGY_SNAPSHOT_ROW_LEN: usize = 6 * core::mem::size_of::<u64>();

/// Reduction constant for the irreducible polynomial
/// x^64 + x^4 + x^3 + x + 1 over GF(2).
///
/// The previous bootstrap value 0xC5 was reducible and therefore could not
/// justify the pairwise/4-wise finite-field independence argument.
const GF64_REDUCTION: u64 = 0x1B;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnergyError {
    BucketsMustBePowerOfTwo,
    TablesMustBeOdd,
    EmptyTables,
    RowCountMismatch { expected: usize, actual: usize },
    UniformWordCountMismatch { expected: usize, actual: usize },
    ProfileMismatch,
    StateSizeOverflow,
    CounterOverflow,
    EnergyOverflow,
    CapacityOverflow,
    IncompatibleConfig,
}

impl fmt::Display for EnergyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BucketsMustBePowerOfTwo => {
                f.write_str("bucket count must be a non-zero power of two")
            }
            Self::TablesMustBeOdd => f.write_str("table count must be odd"),
            Self::EmptyTables => f.write_str("at least one table is required"),
            Self::RowCountMismatch { expected, actual } => {
                write!(f, "row count mismatch: expected {expected}, got {actual}")
            }
            Self::UniformWordCountMismatch { expected, actual } => {
                write!(
                    f,
                    "uniform-random word count mismatch: expected {expected}, got {actual}"
                )
            }
            Self::ProfileMismatch => {
                f.write_str("configuration is not bound to the requested proven profile")
            }
            Self::StateSizeOverflow => f.write_str("counter-state size overflow"),
            Self::CounterOverflow => f.write_str("counter overflow"),
            Self::EnergyOverflow => f.write_str("cached energy overflow"),
            Self::CapacityOverflow => f.write_str("capacity calculation overflow"),
            Self::IncompatibleConfig => f.write_str("sketch configurations are incompatible"),
        }
    }
}

impl std::error::Error for EnergyError {}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeError {
    FivePercent,
    TenPercent,
    TwentyPercent,
}

impl RelativeError {
    pub const fn epsilon(self) -> f64 {
        match self {
            Self::FivePercent => 0.05,
            Self::TenPercent => 0.10,
            Self::TwentyPercent => 0.20,
        }
    }

    pub const fn buckets(self) -> usize {
        match self {
            Self::FivePercent => 8192,
            Self::TenPercent => 2048,
            Self::TwentyPercent => 512,
        }
    }

    const fn capacity_extra_denominator(self) -> u128 {
        match self {
            Self::FivePercent => 19,
            Self::TenPercent => 9,
            Self::TwentyPercent => 4,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureTarget {
    OneInThousand,
    OneInMillion,
    OneInBillion,
}

impl FailureTarget {
    pub const fn probability(self) -> f64 {
        match self {
            Self::OneInThousand => 1e-3,
            Self::OneInMillion => 1e-6,
            Self::OneInBillion => 1e-9,
        }
    }

    pub const fn tables(self) -> usize {
        match self {
            Self::OneInThousand => 9,
            Self::OneInMillion => 23,
            Self::OneInBillion => 35,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnergyProfile {
    relative_error: RelativeError,
    failure_target: FailureTarget,
}

impl EnergyProfile {
    /// Balanced v0 strict default: 10% relative error at failure probability <= 1e-6.
    pub const DEFAULT: Self = Self::new(RelativeError::TenPercent, FailureTarget::OneInMillion);

    pub const fn new(relative_error: RelativeError, failure_target: FailureTarget) -> Self {
        Self {
            relative_error,
            failure_target,
        }
    }

    pub const fn relative_error(self) -> RelativeError {
        self.relative_error
    }

    pub const fn failure_target(self) -> FailureTarget {
        self.failure_target
    }

    pub const fn buckets(self) -> usize {
        self.relative_error.buckets()
    }

    pub const fn tables(self) -> usize {
        self.failure_target.tables()
    }

    pub const fn counter_state_bytes(self) -> usize {
        self.buckets() * self.tables() * core::mem::size_of::<i64>()
    }

    /// Number of independent uniformly random u64 words needed to construct
    /// all theorem-facing row hashes: 2 bucket + 4 sign coefficients per row.
    pub const fn uniform_words_required(self) -> usize {
        self.tables() * 6
    }

    pub fn recommended_capacity(self, point: u128) -> Result<u128, EnergyError> {
        let extra = ceil_div(point, self.relative_error.capacity_extra_denominator());
        point
            .checked_add(extra)
            .ok_or(EnergyError::CapacityOverflow)
    }
}

impl Default for EnergyProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyEstimate {
    pub point: u128,
    pub upper_capacity: u128,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnergyRowHash {
    bucket_coefficients: [u64; 2],
    sign_coefficients: [u64; 4],
}

impl EnergyRowHash {
    /// Creates one row hash from explicit GF(2^64) coefficients.
    ///
    /// This constructor does not prove that the coefficients were sampled
    /// randomly. The Coverage::Proven contract is available only through
    /// EnergyConfig::for_profile_assuming_uniform_words, whose caller accepts
    /// the documented uniform-independence precondition.
    pub const fn from_coefficients(
        bucket_coefficients: [u64; 2],
        sign_coefficients: [u64; 4],
    ) -> Self {
        Self {
            bucket_coefficients,
            sign_coefficients,
        }
    }

    fn bucket_index(self, key: u64, buckets: usize) -> usize {
        if buckets == 1 {
            return 0;
        }

        let value = gf64_mul(self.bucket_coefficients[0], key) ^ self.bucket_coefficients[1];
        let bits = buckets.trailing_zeros();
        (value >> (64 - bits)) as usize
    }

    fn sign_masks(self) -> [u64; 3] {
        let [_, c1, c2, c3] = self.sign_coefficients;
        [
            lsb_multiplication_mask(c1),
            lsb_multiplication_mask(c2),
            lsb_multiplication_mask(c3),
        ]
    }

    #[inline]
    fn sign_from_masks(self, key: u64, key_squared: u64, key_cubed: u64, masks: [u64; 3]) -> i64 {
        let [c0, _, _, _] = self.sign_coefficients;
        let bit = (c0 & 1)
            ^ u64::from((masks[0] & key).count_ones() & 1)
            ^ u64::from((masks[1] & key_squared).count_ones() & 1)
            ^ u64::from((masks[2] & key_cubed).count_ones() & 1);

        if bit == 0 { 1 } else { -1 }
    }

    #[cfg(test)]
    fn sign_horner(self, key: u64) -> i64 {
        let [c0, c1, c2, c3] = self.sign_coefficients;
        let value = gf64_mul(gf64_mul(gf64_mul(c3, key) ^ c2, key) ^ c1, key) ^ c0;

        if value & 1 == 0 { 1 } else { -1 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyConfig {
    buckets: usize,
    rows: Box<[EnergyRowHash]>,
    proven_profile: Option<EnergyProfile>,
}

impl EnergyConfig {
    /// Creates a custom configuration for point-estimation/reference work.
    ///
    /// Custom configurations are deliberately not eligible for
    /// Coverage::Proven.
    pub fn new(buckets: usize, rows: Vec<EnergyRowHash>) -> Result<Self, EnergyError> {
        validate_dimensions(buckets, &rows)?;

        Ok(Self {
            buckets,
            rows: rows.into_boxed_slice(),
            proven_profile: None,
        })
    }

    /// Creates a theorem-profile configuration from raw independent uniform
    /// words without imposing an RNG dependency on the crate.
    ///
    /// # Randomness contract
    ///
    /// `words` must contain exactly `profile.uniform_words_required()`
    /// independent uniformly distributed u64 values. Every six words form one
    /// row: two bucket coefficients followed by four sign coefficients.
    ///
    /// The probability in Coverage::Proven is over that random draw. A small
    /// deterministic seed expanded by an ordinary PRNG does not automatically
    /// satisfy this information-theoretic contract.
    pub fn for_profile_assuming_uniform_words(
        profile: EnergyProfile,
        words: &[u64],
    ) -> Result<Self, EnergyError> {
        let expected = profile.uniform_words_required();
        if words.len() != expected {
            return Err(EnergyError::UniformWordCountMismatch {
                expected,
                actual: words.len(),
            });
        }

        let rows = words
            .chunks(6)
            .map(|chunk| {
                EnergyRowHash::from_coefficients(
                    [chunk[0], chunk[1]],
                    [chunk[2], chunk[3], chunk[4], chunk[5]],
                )
            })
            .collect();

        Self::for_profile_assuming_uniform_rows(profile, rows)
    }

    fn for_profile_assuming_uniform_rows(
        profile: EnergyProfile,
        rows: Vec<EnergyRowHash>,
    ) -> Result<Self, EnergyError> {
        if rows.len() != profile.tables() {
            return Err(EnergyError::RowCountMismatch {
                expected: profile.tables(),
                actual: rows.len(),
            });
        }

        validate_dimensions(profile.buckets(), &rows)?;

        Ok(Self {
            buckets: profile.buckets(),
            rows: rows.into_boxed_slice(),
            proven_profile: Some(profile),
        })
    }

    pub const fn buckets(&self) -> usize {
        self.buckets
    }

    pub fn tables(&self) -> usize {
        self.rows.len()
    }

    pub const fn proven_profile(&self) -> Option<EnergyProfile> {
        self.proven_profile
    }
}

fn validate_dimensions(buckets: usize, rows: &[EnergyRowHash]) -> Result<(), EnergyError> {
    if buckets == 0 || !buckets.is_power_of_two() {
        return Err(EnergyError::BucketsMustBePowerOfTwo);
    }
    if rows.is_empty() {
        return Err(EnergyError::EmptyTables);
    }
    if rows.len().is_multiple_of(2) {
        return Err(EnergyError::TablesMustBeOdd);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct PendingUpdate {
    index: usize,
    counter: i64,
    energy: i128,
}

#[derive(Debug, Clone)]
pub struct EnergyDeltaMeter {
    config: EnergyConfig,
    counters: Box<[i64]>,
    energies: Box<[i128]>,
    pending: Box<[PendingUpdate]>,
    sign_masks: Box<[[u64; 3]]>,
}

impl EnergyDeltaMeter {
    pub fn new(config: EnergyConfig) -> Result<Self, EnergyError> {
        let sign_masks = config
            .rows
            .iter()
            .copied()
            .map(EnergyRowHash::sign_masks)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self::new_with_sign_masks(config, sign_masks)
    }

    fn new_with_sign_masks(
        config: EnergyConfig,
        sign_masks: Box<[[u64; 3]]>,
    ) -> Result<Self, EnergyError> {
        let counter_len = config
            .buckets()
            .checked_mul(config.tables())
            .ok_or(EnergyError::StateSizeOverflow)?;
        let tables = config.tables();
        debug_assert_eq!(sign_masks.len(), tables);

        Ok(Self {
            config,
            counters: vec![0; counter_len].into_boxed_slice(),
            energies: vec![0; tables].into_boxed_slice(),
            pending: vec![PendingUpdate::default(); tables].into_boxed_slice(),
            sign_masks,
        })
    }

    pub fn config(&self) -> &EnergyConfig {
        &self.config
    }

    /// Encodes this sketch as the canonical DeltaMeter snapshot v1 byte format.
    ///
    /// The snapshot includes the exact hash-row configuration and primary
    /// counters. Cached row energies are intentionally omitted and recomputed
    /// during decoding.
    pub fn encode_snapshot(&self) -> Result<Vec<u8>, SnapshotError> {
        let buckets =
            u64::try_from(self.config.buckets()).map_err(|_| SnapshotError::UnsupportedConfig)?;
        let tables =
            u32::try_from(self.config.tables()).map_err(|_| SnapshotError::UnsupportedConfig)?;

        let profile_tags = match self.config.proven_profile() {
            None => [ENERGY_PROFILE_CUSTOM, 0, 0, 0],
            Some(profile) => [
                ENERGY_PROFILE_PROVEN,
                relative_error_snapshot_tag(profile.relative_error())?,
                failure_target_snapshot_tag(profile.failure_target())?,
                0,
            ],
        };
        let counter_bytes = self
            .counters
            .len()
            .checked_mul(core::mem::size_of::<i64>())
            .ok_or(SnapshotError::LengthOverflow)?;
        let payload_len = self
            .config
            .tables()
            .checked_mul(ENERGY_SNAPSHOT_ROW_LEN)
            .and_then(|bytes| bytes.checked_add(ENERGY_SNAPSHOT_METADATA_LEN))
            .and_then(|bytes| bytes.checked_add(counter_bytes))
            .ok_or(SnapshotError::LengthOverflow)?;

        encode_envelope(BACKEND_ENERGY, payload_len, |payload| {
            push_u64(payload, buckets);
            push_u32(payload, tables);
            payload.extend_from_slice(&profile_tags);

            for row in &self.config.rows {
                for coefficient in row.bucket_coefficients {
                    push_u64(payload, coefficient);
                }
                for coefficient in row.sign_coefficients {
                    push_u64(payload, coefficient);
                }
            }
            for &counter in &self.counters {
                push_i64(payload, counter);
            }
        })
    }

    /// Decodes a canonical DeltaMeter snapshot v1 custom Energy sketch.
    ///
    /// A snapshot that claims a theorem-backed profile is rejected with
    /// SnapshotError::ProvenanceRequired. CRC32C authenticates nothing and a
    /// byte stream cannot prove that its hash rows originated from the
    /// independent-uniform draw required by Coverage::Proven.
    pub fn decode_snapshot(bytes: &[u8]) -> Result<Self, SnapshotError> {
        Self::decode_snapshot_impl(bytes, false)
    }

    /// Decodes an Energy snapshot while explicitly accepting the theorem-facing
    /// uniform-row provenance precondition.
    ///
    /// If the snapshot carries a Proven profile marker, the caller asserts that
    /// its encoded row coefficients originated from a valid independent-uniform
    /// draw for that profile and were not adversarially substituted. CRC32C is
    /// only accidental-corruption detection.
    pub fn decode_snapshot_assuming_uniform_rows(bytes: &[u8]) -> Result<Self, SnapshotError> {
        Self::decode_snapshot_impl(bytes, true)
    }

    fn decode_snapshot_impl(
        bytes: &[u8],
        allow_proven_profile: bool,
    ) -> Result<Self, SnapshotError> {
        let payload = decode_envelope(bytes, BACKEND_ENERGY)?;
        let mut cursor = Cursor::new(payload);

        let buckets_u64 = cursor.read_u64()?;
        let buckets = usize::try_from(buckets_u64).map_err(|_| SnapshotError::UnsupportedConfig)?;
        let table_count_u32 = cursor.read_u32()?;
        let table_count =
            usize::try_from(table_count_u32).map_err(|_| SnapshotError::UnsupportedConfig)?;

        let profile_kind = cursor.read_u8()?;
        let relative_tag = cursor.read_u8()?;
        let failure_tag = cursor.read_u8()?;
        if cursor.read_u8()? != 0 {
            return Err(SnapshotError::InvalidPayload);
        }
        if profile_kind == ENERGY_PROFILE_PROVEN && !allow_proven_profile {
            return Err(SnapshotError::ProvenanceRequired);
        }

        let row_bytes = table_count
            .checked_mul(6)
            .and_then(|count| count.checked_mul(core::mem::size_of::<u64>()))
            .ok_or(SnapshotError::LengthOverflow)?;
        if cursor.remaining() < row_bytes {
            return Err(SnapshotError::InvalidPayload);
        }

        let mut rows = Vec::with_capacity(table_count);
        for _ in 0..table_count {
            rows.push(EnergyRowHash::from_coefficients(
                [cursor.read_u64()?, cursor.read_u64()?],
                [
                    cursor.read_u64()?,
                    cursor.read_u64()?,
                    cursor.read_u64()?,
                    cursor.read_u64()?,
                ],
            ));
        }

        let config = match profile_kind {
            ENERGY_PROFILE_CUSTOM => {
                if relative_tag != 0 || failure_tag != 0 {
                    return Err(SnapshotError::InvalidPayload);
                }
                EnergyConfig::new(buckets, rows).map_err(|_| SnapshotError::InvalidPayload)?
            }
            ENERGY_PROFILE_PROVEN => {
                let profile = EnergyProfile::new(
                    relative_error_from_snapshot_tag(relative_tag)?,
                    failure_target_from_snapshot_tag(failure_tag)?,
                );
                if buckets != profile.buckets() {
                    return Err(SnapshotError::InvalidPayload);
                }
                EnergyConfig::for_profile_assuming_uniform_rows(profile, rows)
                    .map_err(|_| SnapshotError::InvalidPayload)?
            }
            _ => return Err(SnapshotError::InvalidPayload),
        };

        let counter_len = config
            .buckets()
            .checked_mul(config.tables())
            .ok_or(SnapshotError::LengthOverflow)?;
        let counter_bytes = counter_len
            .checked_mul(core::mem::size_of::<i64>())
            .ok_or(SnapshotError::LengthOverflow)?;
        if cursor.remaining() != counter_bytes {
            return Err(SnapshotError::InvalidPayload);
        }

        let mut meter = Self::new(config).map_err(|_| SnapshotError::InvalidPayload)?;
        for counter in &mut meter.counters {
            *counter = cursor.read_i64()?;
        }
        if !cursor.is_finished() {
            return Err(SnapshotError::InvalidPayload);
        }

        meter.energies = recompute_energies(
            &meter.counters,
            meter.config.buckets(),
            meter.config.tables(),
        )
        .map_err(|_| SnapshotError::InvalidPayload)?;

        Ok(meter)
    }

    /// Adds one unique source-set element.
    ///
    /// The symmetric-difference interpretation requires callers to contribute
    /// each set element at most once to each source sketch.
    pub fn add_unique(&mut self, key: u64) -> Result<(), EnergyError> {
        self.apply_unit(key, 1)
    }

    /// Returns the linear difference self - other.
    ///
    /// If both operands were built from unique source sets using the same
    /// configuration, the resulting frequency vector is in {-1, 0, +1} and
    /// its F2 value equals the set symmetric-difference size.
    pub fn difference(&self, other: &Self) -> Result<Self, EnergyError> {
        if self.config != other.config {
            return Err(EnergyError::IncompatibleConfig);
        }

        let mut result = Self::new_with_sign_masks(self.config.clone(), self.sign_masks.clone())?;

        for ((dst, left), right) in result
            .counters
            .iter_mut()
            .zip(self.counters.iter())
            .zip(other.counters.iter())
        {
            *dst = left
                .checked_sub(*right)
                .ok_or(EnergyError::CounterOverflow)?;
        }

        result.energies = recompute_energies(
            &result.counters,
            result.config.buckets(),
            result.config.tables(),
        )?;

        Ok(result)
    }

    pub fn subtract_assign(&mut self, other: &Self) -> Result<(), EnergyError> {
        *self = self.difference(other)?;
        Ok(())
    }

    pub fn point_estimate(&self) -> u128 {
        let mut values = self.energies.to_vec();
        let middle = values.len() / 2;
        let (_, median, _) = values.select_nth_unstable(middle);
        debug_assert!(*median >= 0);
        *median as u128
    }

    /// Returns the theorem-backed estimate for the exact profile used to
    /// create this configuration.
    pub fn estimate(&self, profile: EnergyProfile) -> Result<EnergyEstimate, EnergyError> {
        if self.config.proven_profile() != Some(profile) {
            return Err(EnergyError::ProfileMismatch);
        }

        let point = self.point_estimate();
        let upper_capacity = profile.recommended_capacity(point)?;

        Ok(EnergyEstimate {
            point,
            upper_capacity,
            coverage: Coverage::Proven {
                failure_probability_upper_bound: profile.failure_target().probability(),
            },
        })
    }

    #[cfg(test)]
    fn remove_unique(&mut self, key: u64) -> Result<(), EnergyError> {
        self.apply_unit(key, -1)
    }

    fn apply_unit(&mut self, key: u64, delta: i64) -> Result<(), EnergyError> {
        let buckets = self.config.buckets();
        let key_squared = gf64_mul(key, key);
        let key_cubed = gf64_mul(key_squared, key);

        for row_index in 0..self.config.tables() {
            let row = self.config.rows[row_index];
            let bucket = row.bucket_index(key, buckets);
            let index = row_index * buckets + bucket;
            let signed_delta = delta
                * row.sign_from_masks(key, key_squared, key_cubed, self.sign_masks[row_index]);
            let old_counter = self.counters[index];
            let new_counter = old_counter
                .checked_add(signed_delta)
                .ok_or(EnergyError::CounterOverflow)?;

            let old_counter = i128::from(old_counter);
            let new_counter_i128 = i128::from(new_counter);
            let old_square = old_counter * old_counter;
            let new_square = new_counter_i128 * new_counter_i128;
            let energy_delta = new_square - old_square;
            let new_energy = self.energies[row_index]
                .checked_add(energy_delta)
                .ok_or(EnergyError::EnergyOverflow)?;

            self.pending[row_index] = PendingUpdate {
                index,
                counter: new_counter,
                energy: new_energy,
            };
        }

        for row_index in 0..self.config.tables() {
            let pending = self.pending[row_index];
            self.counters[pending.index] = pending.counter;
            self.energies[row_index] = pending.energy;
        }

        Ok(())
    }
}

fn recompute_energies(
    counters: &[i64],
    buckets: usize,
    tables: usize,
) -> Result<Box<[i128]>, EnergyError> {
    let mut energies = vec![0_i128; tables];

    for (row_index, row) in counters.chunks_exact(buckets).enumerate() {
        let mut energy = 0_i128;
        for &counter in row {
            let counter = i128::from(counter);
            energy = energy
                .checked_add(counter * counter)
                .ok_or(EnergyError::EnergyOverflow)?;
        }
        energies[row_index] = energy;
    }

    Ok(energies.into_boxed_slice())
}

fn relative_error_snapshot_tag(value: RelativeError) -> Result<u8, SnapshotError> {
    match value {
        RelativeError::FivePercent => Ok(1),
        RelativeError::TenPercent => Ok(2),
        RelativeError::TwentyPercent => Ok(3),
    }
}

fn relative_error_from_snapshot_tag(tag: u8) -> Result<RelativeError, SnapshotError> {
    match tag {
        1 => Ok(RelativeError::FivePercent),
        2 => Ok(RelativeError::TenPercent),
        3 => Ok(RelativeError::TwentyPercent),
        _ => Err(SnapshotError::InvalidPayload),
    }
}

fn failure_target_snapshot_tag(value: FailureTarget) -> Result<u8, SnapshotError> {
    match value {
        FailureTarget::OneInThousand => Ok(1),
        FailureTarget::OneInMillion => Ok(2),
        FailureTarget::OneInBillion => Ok(3),
    }
}

fn failure_target_from_snapshot_tag(tag: u8) -> Result<FailureTarget, SnapshotError> {
    match tag {
        1 => Ok(FailureTarget::OneInThousand),
        2 => Ok(FailureTarget::OneInMillion),
        3 => Ok(FailureTarget::OneInBillion),
        _ => Err(SnapshotError::InvalidPayload),
    }
}

const fn ceil_div(value: u128, divisor: u128) -> u128 {
    if value == 0 {
        0
    } else {
        1 + (value - 1) / divisor
    }
}

fn lsb_multiplication_mask(coefficient: u64) -> u64 {
    let mut mask = 0_u64;
    let mut basis_product = coefficient;

    for bit in 0..64 {
        if basis_product & 1 != 0 {
            mask |= 1_u64 << bit;
        }
        basis_product = gf64_mul_x(basis_product);
    }

    mask
}

#[inline]
fn gf64_mul_x(value: u64) -> u64 {
    let carry = value >> 63;
    let mut shifted = value << 1;
    if carry != 0 {
        shifted ^= GF64_REDUCTION;
    }
    shifted
}

#[inline]
fn gf64_mul(mut left: u64, mut right: u64) -> u64 {
    let mut product = 0_u64;

    for _ in 0..64 {
        if right & 1 != 0 {
            product ^= left;
        }
        right >>= 1;

        let carry = left >> 63;
        left <<= 1;
        if carry != 0 {
            left ^= GF64_REDUCTION;
        }
    }

    product
}

#[cfg(test)]
mod tests {
    use super::*;

    fn splitmix64(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn test_uniform_words(word_count: usize, salt: u64) -> Vec<u64> {
        (0..word_count)
            .map(|index| splitmix64((index as u64) ^ salt.rotate_left(17)))
            .collect()
    }

    fn test_rows(count: usize, salt: u64) -> Vec<EnergyRowHash> {
        test_uniform_words(count * 6, salt)
            .chunks(6)
            .map(|chunk| {
                EnergyRowHash::from_coefficients(
                    [chunk[0], chunk[1]],
                    [chunk[2], chunk[3], chunk[4], chunk[5]],
                )
            })
            .collect()
    }

    fn meter(profile: EnergyProfile, salt: u64) -> EnergyDeltaMeter {
        let words = test_uniform_words(profile.uniform_words_required(), salt);
        let config = EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap();
        EnergyDeltaMeter::new(config).unwrap()
    }

    #[test]
    fn profile_dimensions_match_research_table() {
        let failures = [
            (FailureTarget::OneInThousand, 9),
            (FailureTarget::OneInMillion, 23),
            (FailureTarget::OneInBillion, 35),
        ];
        let errors = [
            (RelativeError::FivePercent, 8192),
            (RelativeError::TenPercent, 2048),
            (RelativeError::TwentyPercent, 512),
        ];

        for (failure, tables) in failures {
            for (error, buckets) in errors {
                let profile = EnergyProfile::new(error, failure);
                assert_eq!(profile.tables(), tables);
                assert_eq!(profile.buckets(), buckets);
                assert_eq!(
                    profile.counter_state_bytes(),
                    buckets * tables * core::mem::size_of::<i64>()
                );
            }
        }
    }

    #[test]
    fn energy_estimate_remains_proven_after_asymptotic_variant_is_added() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let sketch = meter(profile, 404);
        let estimate = sketch.estimate(profile).unwrap();

        assert!(matches!(estimate.coverage, Coverage::Proven { .. }));
    }

    #[test]
    fn ten_percent_capacity_uses_ceiling() {
        let profile = EnergyProfile::new(RelativeError::TenPercent, FailureTarget::OneInMillion);

        assert_eq!(profile.recommended_capacity(0).unwrap(), 0);
        assert_eq!(profile.recommended_capacity(9).unwrap(), 10);
        assert_eq!(profile.recommended_capacity(100).unwrap(), 112);
    }

    #[test]
    fn field_multiplication_has_identity_and_is_distributive() {
        let a = 0x0123_4567_89AB_CDEF;
        let b = 0xF0E1_D2C3_B4A5_9687;
        let c = 0x0F0F_F0F0_AAAA_5555;

        assert_eq!(gf64_mul(a, 0), 0);
        assert_eq!(gf64_mul(a, 1), a);
        assert_eq!(gf64_mul(a, b), gf64_mul(b, a));
        assert_eq!(gf64_mul(a, b ^ c), gf64_mul(a, b) ^ gf64_mul(a, c));
    }

    #[test]
    fn lsb_multiplication_mask_matches_direct_field_multiplication() {
        let coefficients = [
            0,
            1,
            1_u64 << 63,
            u64::MAX,
            0x0123_4567_89AB_CDEF,
            0xDEAD_BEEF_CAFE_BABE,
        ];
        let values = [
            0,
            1,
            2,
            1_u64 << 63,
            u64::MAX,
            0xF0E1_D2C3_B4A5_9687,
            0x1357_9BDF_2468_ACE0,
        ];

        for coefficient in coefficients {
            let mask = lsb_multiplication_mask(coefficient);
            for value in values {
                assert_eq!(
                    u64::from((mask & value).count_ones() & 1),
                    gf64_mul(coefficient, value) & 1,
                    "coefficient={coefficient:#018x} value={value:#018x}"
                );
            }
        }
    }

    #[test]
    fn lsb_mask_recurrence_matches_every_polynomial_basis_vector() {
        for coefficient in [
            0,
            1,
            1_u64 << 63,
            u64::MAX,
            0x0123_4567_89AB_CDEF,
            0xDEAD_BEEF_CAFE_BABE,
        ] {
            let mask = lsb_multiplication_mask(coefficient);
            for bit in 0..64 {
                assert_eq!(
                    (mask >> bit) & 1,
                    gf64_mul(coefficient, 1_u64 << bit) & 1,
                    "coefficient={coefficient:#018x} bit={bit}"
                );
            }
        }
    }

    #[test]
    fn cached_sign_masks_match_horner_for_edge_coefficients_and_keys() {
        let rows = [
            EnergyRowHash::from_coefficients([0, 0], [0, 0, 0, 0]),
            EnergyRowHash::from_coefficients([1, 1], [1, 1, 1, 1]),
            EnergyRowHash::from_coefficients(
                [1_u64 << 63, u64::MAX],
                [1_u64 << 63, 1_u64 << 63, 1_u64 << 63, 1_u64 << 63],
            ),
            EnergyRowHash::from_coefficients(
                [u64::MAX, 0],
                [u64::MAX, u64::MAX, u64::MAX, u64::MAX],
            ),
            EnergyRowHash::from_coefficients(
                [0x0123_4567_89AB_CDEF, 0xF0E1_D2C3_B4A5_9687],
                [
                    0xDEAD_BEEF_CAFE_BABE,
                    0x1357_9BDF_2468_ACE0,
                    0xAAAA_5555_F0F0_0F0F,
                    0x8000_0000_0000_0001,
                ],
            ),
        ];
        let keys = [
            0,
            1,
            2,
            1_u64 << 63,
            u64::MAX,
            0x0123_4567_89AB_CDEF,
            0xF0E1_D2C3_B4A5_9687,
        ];

        for row in rows {
            let masks = row.sign_masks();
            for key in keys {
                let key_squared = gf64_mul(key, key);
                let key_cubed = gf64_mul(key_squared, key);
                assert_eq!(
                    row.sign_from_masks(key, key_squared, key_cubed, masks),
                    row.sign_horner(key),
                    "key={key:#018x} row={row:?}"
                );
            }
        }
    }

    #[test]
    fn cached_sign_masks_match_horner_for_every_profile_shape() {
        for failure in [
            FailureTarget::OneInThousand,
            FailureTarget::OneInMillion,
            FailureTarget::OneInBillion,
        ] {
            for error in [
                RelativeError::FivePercent,
                RelativeError::TenPercent,
                RelativeError::TwentyPercent,
            ] {
                let profile = EnergyProfile::new(error, failure);
                let sketch = meter(profile, 0xCACE_600D);
                for (row, masks) in sketch
                    .config
                    .rows
                    .iter()
                    .copied()
                    .zip(sketch.sign_masks.iter().copied())
                {
                    for key in [0, 1, 1_u64 << 63, u64::MAX, 0xA5A5_5A5A_DEAD_BEEF] {
                        let key_squared = gf64_mul(key, key);
                        let key_cubed = gf64_mul(key_squared, key);
                        assert_eq!(
                            row.sign_from_masks(key, key_squared, key_cubed, masks),
                            row.sign_horner(key)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn incremental_energy_matches_recomputation() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let mut sketch = meter(profile, 1);

        for key in [1, 2, 3, 5, 8, 13, 21, 34] {
            sketch.add_unique(key).unwrap();
        }
        for key in [55, 89, 144] {
            sketch.remove_unique(key).unwrap();
        }

        let recomputed = recompute_energies(
            &sketch.counters,
            sketch.config.buckets(),
            sketch.config.tables(),
        )
        .unwrap();

        assert_eq!(sketch.energies, recomputed);
    }

    #[test]
    fn sketch_difference_matches_direct_signed_difference() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let words = test_uniform_words(profile.uniform_words_required(), 7);
        let config = EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap();

        let mut left = EnergyDeltaMeter::new(config.clone()).unwrap();
        let mut right = EnergyDeltaMeter::new(config.clone()).unwrap();
        let mut direct = EnergyDeltaMeter::new(config).unwrap();

        for key in [1, 2, 3, 4] {
            left.add_unique(key).unwrap();
        }
        for key in [3, 4, 5] {
            right.add_unique(key).unwrap();
        }

        direct.add_unique(1).unwrap();
        direct.add_unique(2).unwrap();
        direct.remove_unique(5).unwrap();

        let difference = left.difference(&right).unwrap();

        assert_eq!(difference.counters, direct.counters);
        assert_eq!(difference.energies, direct.energies);
        assert_eq!(difference.point_estimate(), direct.point_estimate());
    }

    #[test]
    fn incompatible_configs_are_rejected() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let left = meter(profile, 1);
        let right = meter(profile, 2);

        assert_eq!(
            left.difference(&right).unwrap_err(),
            EnergyError::IncompatibleConfig
        );
    }

    #[test]
    fn custom_config_cannot_claim_proven_profile() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let config = EnergyConfig::new(profile.buckets(), test_rows(profile.tables(), 1)).unwrap();
        let sketch = EnergyDeltaMeter::new(config).unwrap();

        assert_eq!(
            sketch.estimate(profile).unwrap_err(),
            EnergyError::ProfileMismatch
        );
    }

    fn polynomial_degree(value: u128) -> u32 {
        127 - value.leading_zeros()
    }

    fn polynomial_mod(mut value: u128, modulus: u128) -> u128 {
        let modulus_degree = polynomial_degree(modulus);
        while value != 0 && polynomial_degree(value) >= modulus_degree {
            value ^= modulus << (polynomial_degree(value) - modulus_degree);
        }
        value
    }

    fn polynomial_gcd(mut left: u128, mut right: u128) -> u128 {
        while right != 0 {
            let remainder = polynomial_mod(left, right);
            left = right;
            right = remainder;
        }
        left
    }

    #[test]
    fn gf64_modulus_is_irreducible() {
        let modulus = (1_u128 << 64) | u128::from(GF64_REDUCTION);
        let x = 2_u64;
        let mut value = x;

        // For degree 64, whose only prime divisor is 2, Rabin's test reduces
        // to gcd(x^(2^32)-x, p)=1 and x^(2^64)=x modulo p.
        for _ in 0..32 {
            value = gf64_mul(value, value);
        }
        assert_eq!(polynomial_gcd(u128::from(value ^ x), modulus), 1);

        for _ in 0..32 {
            value = gf64_mul(value, value);
        }
        assert_eq!(value, x);
    }

    #[test]
    fn gf64_multiplication_matches_reference_vectors() {
        let vectors = [
            (
                0x0123_4567_89AB_CDEF,
                0xF0E1_D2C3_B4A5_9687,
                0x287C_26FE_0540_0BC2,
            ),
            (
                0xFFFF_FFFF_FFFF_FFFF,
                0xFFFF_FFFF_FFFF_FFFF,
                0x5555_5555_5555_5513,
            ),
            (
                0x8000_0000_0000_0000,
                0x0000_0000_0000_0002,
                0x0000_0000_0000_001B,
            ),
            (
                0xDEAD_BEEF_CAFE_BABE,
                0x0123_4567_89AB_CDEF,
                0xFBB6_7120_92FD_6A8C,
            ),
        ];

        for (left, right, expected) in vectors {
            assert_eq!(gf64_mul(left, right), expected);
            assert_eq!(gf64_mul(right, left), expected);
        }
    }

    #[test]
    fn uniform_word_count_mismatch_is_rejected() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let words = test_uniform_words(profile.uniform_words_required() - 1, 11);

        assert_eq!(
            EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap_err(),
            EnergyError::UniformWordCountMismatch {
                expected: profile.uniform_words_required(),
                actual: profile.uniform_words_required() - 1,
            }
        );
    }

    #[test]
    fn source_order_is_invariant() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let mut forward = meter(profile, 23);
        let mut reverse = meter(profile, 23);
        let keys = [3, 5, 8, 13, 21, 34, 55, 89];

        for key in keys {
            forward.add_unique(key).unwrap();
        }
        for key in keys.into_iter().rev() {
            reverse.add_unique(key).unwrap();
        }

        assert_eq!(forward.counters, reverse.counters);
        assert_eq!(forward.energies, reverse.energies);
    }

    #[test]
    fn self_difference_is_zero() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let mut sketch = meter(profile, 31);
        for key in 0..64 {
            if key % 3 != 0 {
                sketch.add_unique(key).unwrap();
            }
        }

        let zero = sketch.difference(&sketch).unwrap();
        assert!(zero.counters.iter().all(|&counter| counter == 0));
        assert!(zero.energies.iter().all(|&energy| energy == 0));
        assert_eq!(zero.point_estimate(), 0);
    }

    #[test]
    fn difference_is_antisymmetric() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let mut left = meter(profile, 41);
        let mut right = meter(profile, 41);

        for key in [1, 2, 5, 8, 13, 21] {
            left.add_unique(key).unwrap();
        }
        for key in [2, 3, 5, 13, 34, 55] {
            right.add_unique(key).unwrap();
        }

        let left_minus_right = left.difference(&right).unwrap();
        let right_minus_left = right.difference(&left).unwrap();

        for (forward, reverse) in left_minus_right
            .counters
            .iter()
            .zip(right_minus_left.counters.iter())
        {
            assert_eq!(*forward, -*reverse);
        }
        assert_eq!(left_minus_right.energies, right_minus_left.energies);
        assert_eq!(
            left_minus_right.point_estimate(),
            right_minus_left.point_estimate()
        );
    }

    #[test]
    fn deterministic_difference_property_grid() {
        let profile =
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);

        for case in 0_u64..16 {
            let mut left = meter(profile, 73);
            let mut right = meter(profile, 73);
            let mut direct = meter(profile, 73);

            for key in 0_u64..96 {
                let left_member = splitmix64(key ^ case.rotate_left(7)) & 3 != 0;
                let right_member =
                    splitmix64(key ^ case.rotate_left(19) ^ 0xA5A5_A5A5_A5A5_A5A5) & 3 != 0;

                if left_member {
                    left.add_unique(key).unwrap();
                }
                if right_member {
                    right.add_unique(key).unwrap();
                }

                match (left_member, right_member) {
                    (true, false) => direct.add_unique(key).unwrap(),
                    (false, true) => direct.remove_unique(key).unwrap(),
                    _ => {}
                }
            }

            let difference = left.difference(&right).unwrap();
            assert_eq!(difference.counters, direct.counters, "case {case}");
            assert_eq!(difference.energies, direct.energies, "case {case}");
            assert_eq!(
                difference.point_estimate(),
                direct.point_estimate(),
                "case {case}"
            );
        }
    }

    #[test]
    fn failed_update_is_transactional_across_rows() {
        let config = EnergyConfig::new(1, test_rows(3, 101)).unwrap();
        let mut sketch = EnergyDeltaMeter::new(config).unwrap();

        sketch.counters[1] = i64::MAX;
        let counters_before = sketch.counters.clone();
        let energies_before = sketch.energies.clone();

        assert_eq!(
            sketch.add_unique(7).unwrap_err(),
            EnergyError::CounterOverflow
        );
        assert_eq!(sketch.counters, counters_before);
        assert_eq!(sketch.energies, energies_before);
    }

    #[test]
    fn profile_mismatch_is_rejected() {
        let stored = EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
        let requested = EnergyProfile::new(RelativeError::TenPercent, FailureTarget::OneInThousand);
        let sketch = meter(stored, 1);

        assert_eq!(
            sketch.estimate(requested).unwrap_err(),
            EnergyError::ProfileMismatch
        );
    }
}
