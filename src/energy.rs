use core::fmt;

/// Reduction constant for the irreducible polynomial
/// x^64 + x^4 + x^3 + x + 1 over GF(2).
///
/// The previous bootstrap value 0xC5 was reducible and therefore could not
/// justify the pairwise/4-wise finite-field independence argument.
const GF64_REDUCTION: u64 = 0x1B;

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
