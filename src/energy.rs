use core::fmt;

/// Reduction constant for the irreducible polynomial
/// x^64 + x^7 + x^6 + x^2 + 1 over GF(2).
const GF64_REDUCTION: u64 = 0xC5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnergyError {
    BucketsMustBePowerOfTwo,
    TablesMustBeOdd,
    EmptyTables,
    RowCountMismatch { expected: usize, actual: usize },
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

    pub fn recommended_capacity(self, point: u128) -> Result<u128, EnergyError> {
        let extra = ceil_div(point, self.relative_error.capacity_extra_denominator());
        point
            .checked_add(extra)
            .ok_or(EnergyError::CapacityOverflow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coverage {
    Proven {
        failure_probability_upper_bound: f64,
    },
}

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
    /// EnergyConfig::for_profile_assuming_uniform_rows, whose caller accepts
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

    fn sign(self, key: u64) -> i64 {
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

    /// Creates a theorem-profile configuration.
    ///
    /// # Randomness contract
    ///
    /// For every row, the two bucket coefficients must be sampled uniformly
    /// and independently from GF(2^64). The four sign coefficients must also
    /// be sampled uniformly and independently from GF(2^64), independently of
    /// the bucket coefficients and independently across rows.
    ///
    /// The probability in Coverage::Proven is over that random draw.
    pub fn for_profile_assuming_uniform_rows(
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

    pub fn rows(&self) -> &[EnergyRowHash] {
        &self.rows
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
}

impl EnergyDeltaMeter {
    pub fn new(config: EnergyConfig) -> Result<Self, EnergyError> {
        let counter_len = config
            .buckets()
            .checked_mul(config.tables())
            .ok_or(EnergyError::StateSizeOverflow)?;
        let tables = config.tables();

        Ok(Self {
            config,
            counters: vec![0; counter_len].into_boxed_slice(),
            energies: vec![0; tables].into_boxed_slice(),
            pending: vec![PendingUpdate::default(); tables].into_boxed_slice(),
        })
    }

    pub fn config(&self) -> &EnergyConfig {
        &self.config
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

        let mut result = Self::new(self.config.clone())?;

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

        for row_index in 0..self.config.tables() {
            let row = self.config.rows[row_index];
            let bucket = row.bucket_index(key, buckets);
            let index = row_index * buckets + bucket;
            let signed_delta = delta * row.sign(key);
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
            let square = counter * counter;
            energy = energy
                .checked_add(square)
                .ok_or(EnergyError::EnergyOverflow)?;
        }
        energies[row_index] = energy;
    }

    Ok(energies.into_boxed_slice())
}

const fn ceil_div(value: u128, divisor: u128) -> u128 {
    if value == 0 {
        0
    } else {
        1 + (value - 1) / divisor
    }
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

    fn test_rows(count: usize, salt: u64) -> Vec<EnergyRowHash> {
        (0..count)
            .map(|index| {
                let i = index as u64 + 1;
                EnergyRowHash::from_coefficients(
                    [
                        0x9E37_79B9_7F4A_7C15 ^ i.rotate_left(7) ^ salt,
                        0xD1B5_4A32_D192_ED03 ^ i.rotate_left(19),
                    ],
                    [
                        0x94D0_49BB_1331_11EB ^ i,
                        0xBF58_476D_1CE4_E5B9 ^ i.rotate_left(11),
                        0xA24B_AED4_963E_E407 ^ i.rotate_left(23),
                        0x9FB2_1C65_1E98_DF25 ^ i.rotate_left(37) ^ salt,
                    ],
                )
            })
            .collect()
    }

    fn meter(profile: EnergyProfile, salt: u64) -> EnergyDeltaMeter {
        let config = EnergyConfig::for_profile_assuming_uniform_rows(
            profile,
            test_rows(profile.tables(), salt),
        )
        .unwrap();
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
        let rows = test_rows(profile.tables(), 7);
        let config = EnergyConfig::for_profile_assuming_uniform_rows(profile, rows).unwrap();

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
