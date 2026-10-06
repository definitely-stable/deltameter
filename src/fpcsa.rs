use core::fmt;

const ROW_DOMAIN: u64 = 0x243F_6A88_85A3_08D3;
const LEVEL_DOMAIN: u64 = 0x1319_8A2E_0370_7344;
const COEFFICIENT_DOMAIN: u64 = 0xA409_3822_299F_31D0;
const ATTEMPT_STEP: u64 = 0xD134_2543_DE82_EF95;

const F2_TABLE2_NORMALIZATION_PER_ROW: f64 = 1.079;
const F2_TABLE2_RSE_COEFFICIENT: f64 = 1.638;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FpcsaError {
    RowsMustBePositive,
    LevelsOutOfRange,
    StateSizeOverflow,
    InvalidRow { row: u32, rows: u32 },
    InvalidLevel(u8),
    IncompatibleConfig,
    MiddleRangeRequired { empty_rows: usize },
}

impl fmt::Display for FpcsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RowsMustBePositive => f.write_str("F-PCSA requires at least one row"),
            Self::LevelsOutOfRange => {
                f.write_str("stored level count must be in the inclusive range 1..=64")
            }
            Self::StateSizeOverflow => f.write_str("packed F-PCSA state size overflow"),
            Self::InvalidRow { row, rows } => {
                write!(f, "row {row} is out of range for {rows} rows")
            }
            Self::InvalidLevel(level) => {
                write!(f, "level {level} is invalid; levels are one-based")
            }
            Self::IncompatibleConfig => f.write_str("F-PCSA configurations are incompatible"),
            Self::MiddleRangeRequired { empty_rows } => {
                write!(
                    f,
                    "Table 2 reference estimate is unavailable with {empty_rows} empty rows"
                )
            }
        }
    }
}

impl std::error::Error for FpcsaError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishedFpcsaOracle {
    row_seed: u64,
    level_seed: u64,
    coefficient_seed: u64,
}

impl PublishedFpcsaOracle {
    /// Defines the deterministic pseudo-random oracle used by the M2
    /// engineering reproduction.
    ///
    /// This is not the ideal random oracle assumed by the paper and does not
    /// turn the paper's asymptotic analysis into a finite-sample guarantee.
    pub const fn new(row_seed: u64, level_seed: u64, coefficient_seed: u64) -> Self {
        Self {
            row_seed,
            level_seed,
            coefficient_seed,
        }
    }

    fn sample(self, key: u64, rows: u32) -> FpcsaSample {
        let row = self.sample_row(key, rows);
        let level = self.sample_level(key);
        let coefficient_one =
            oracle_word(self.coefficient_seed, key, COEFFICIENT_DOMAIN, 0) & 1 != 0;

        FpcsaSample {
            row,
            level,
            coefficient_one,
        }
    }

    fn sample_row(self, key: u64, rows: u32) -> u32 {
        let rows_u128 = u128::from(rows);
        let range = 1_u128 << 64;
        let accepted = range - (range % rows_u128);

        let mut attempt = 0_u64;
        loop {
            let word = oracle_word(self.row_seed, key, ROW_DOMAIN, attempt);
            let word_u128 = u128::from(word);
            if word_u128 < accepted {
                return (word_u128 % rows_u128) as u32;
            }
            attempt = attempt.wrapping_add(1);
        }
    }

    fn sample_level(self, key: u64) -> Option<u8> {
        let word = oracle_word(self.level_seed, key, LEVEL_DOMAIN, 0);
        if word == 0 {
            None
        } else {
            Some((word.trailing_zeros() + 1) as u8)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedFpcsaF2Config {
    rows: u32,
    levels: u8,
    oracle: PublishedFpcsaOracle,
    state_bits: usize,
    state_words: usize,
}

impl PublishedFpcsaF2Config {
    /// Creates a finite-state reproduction of Definition 6 for F = GF(2).
    ///
    /// stored_levels = J stores levels 1..=J. For the full u64 key domain,
    /// use J = 64. Levels above J are explicit truncation events.
    pub fn new(
        rows: u32,
        stored_levels: u8,
        oracle: PublishedFpcsaOracle,
    ) -> Result<Self, FpcsaError> {
        if rows == 0 {
            return Err(FpcsaError::RowsMustBePositive);
        }
        if !(1..=64).contains(&stored_levels) {
            return Err(FpcsaError::LevelsOutOfRange);
        }

        let state_bits = usize::try_from(rows)
            .ok()
            .and_then(|rows| rows.checked_mul(usize::from(stored_levels)))
            .ok_or(FpcsaError::StateSizeOverflow)?;
        let state_words = state_bits
            .checked_add(63)
            .ok_or(FpcsaError::StateSizeOverflow)?
            / 64;

        Ok(Self {
            rows,
            levels: stored_levels,
            oracle,
            state_bits,
            state_words,
        })
    }

    pub const fn rows(&self) -> u32 {
        self.rows
    }

    pub const fn levels(&self) -> u8 {
        self.levels
    }

    pub const fn logical_state_bits(&self) -> usize {
        self.state_bits
    }

    pub const fn packed_state_words(&self) -> usize {
        self.state_words
    }

    pub const fn packed_state_bytes(&self) -> usize {
        self.state_words * core::mem::size_of::<u64>()
    }

    #[cfg(test)]
    pub const fn oracle(&self) -> PublishedFpcsaOracle {
        self.oracle
    }

    #[cfg(test)]
    fn sample_key(&self, key: u64) -> FpcsaSample {
        self.oracle.sample(key, self.rows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpcsaUpdate {
    Stored { row: u32, level: u8 },
    ZeroCoefficient { row: u32, level: u8 },
    Truncated { row: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FpcsaTable2ReferenceEstimate {
    pub point: f64,
    pub asymptotic_relative_standard_error: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FpcsaSample {
    row: u32,
    level: Option<u8>,
    coefficient_one: bool,
}

#[derive(Debug, Clone)]
pub struct PublishedFpcsaF2 {
    config: PublishedFpcsaF2Config,
    words: Box<[u64]>,
}

impl PublishedFpcsaF2 {
    pub fn new(config: PublishedFpcsaF2Config) -> Self {
        Self {
            words: vec![0; config.packed_state_words()].into_boxed_slice(),
            config,
        }
    }

    #[cfg(test)]
    pub fn config(&self) -> &PublishedFpcsaF2Config {
        &self.config
    }

    #[cfg(test)]
    pub fn packed_words(&self) -> &[u64] {
        &self.words
    }

    /// Applies the GF(2) update k = 1 from Definition 6.
    ///
    /// Repeating the same key twice cancels because the same h(v) and g(v)
    /// are reused and addition in GF(2) is XOR.
    pub fn toggle(&mut self, key: u64) -> FpcsaUpdate {
        let sample = self.config.oracle.sample(key, self.config.rows);
        self.apply_sample(sample)
            .expect("pseudo-oracle always emits a valid row and level")
    }

    /// XOR-composes two linear GF(2) sketches with identical configuration.
    pub fn xor_assign(&mut self, other: &Self) -> Result<(), FpcsaError> {
        if self.config != other.config {
            return Err(FpcsaError::IncompatibleConfig);
        }

        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            *left ^= *right;
        }

        Ok(())
    }

    pub fn xor_merged(&self, other: &Self) -> Result<Self, FpcsaError> {
        let mut merged = self.clone();
        merged.xor_assign(other)?;
        Ok(merged)
    }

    #[cfg(test)]
    pub fn row_highest_level(&self, row: u32) -> Result<Option<u8>, FpcsaError> {
        if row >= self.config.rows {
            return Err(FpcsaError::InvalidRow {
                row,
                rows: self.config.rows,
            });
        }

        Ok(self.row_highest_level_unchecked(row))
    }

    #[cfg(test)]
    pub fn highest_levels(&self) -> Vec<Option<u8>> {
        (0..self.config.rows)
            .map(|row| self.row_highest_level_unchecked(row))
            .collect()
    }

    /// Reproduces the rounded GF(2) asymptotic constants from Table 2.
    ///
    /// This is a diagnostic reference only. The finite implementation uses a
    /// deterministic pseudo-random oracle, finite levels, and does not
    /// implement the paper's random-offsetting simplification assumption.
    /// Therefore this result is not Coverage::Proven and is not a strict
    /// finite-sample estimator.
    pub fn table2_reference_estimate(&self) -> Result<FpcsaTable2ReferenceEstimate, FpcsaError> {
        let mut empty_rows = 0_usize;
        let mut sum_levels = 0_u64;

        for row in 0..self.config.rows {
            match self.row_highest_level_unchecked(row) {
                Some(level) => sum_levels += u64::from(level),
                None => empty_rows += 1,
            }
        }

        if empty_rows != 0 {
            return Err(FpcsaError::MiddleRangeRequired { empty_rows });
        }

        let rows = f64::from(self.config.rows);
        let mean_level = sum_levels as f64 / rows;

        Ok(FpcsaTable2ReferenceEstimate {
            point: F2_TABLE2_NORMALIZATION_PER_ROW * rows * 2_f64.powf(mean_level),
            asymptotic_relative_standard_error: F2_TABLE2_RSE_COEFFICIENT / rows.sqrt(),
        })
    }

    fn apply_sample(&mut self, sample: FpcsaSample) -> Result<FpcsaUpdate, FpcsaError> {
        if sample.row >= self.config.rows {
            return Err(FpcsaError::InvalidRow {
                row: sample.row,
                rows: self.config.rows,
            });
        }

        let Some(level) = sample.level else {
            return Ok(FpcsaUpdate::Truncated { row: sample.row });
        };
        if level == 0 {
            return Err(FpcsaError::InvalidLevel(level));
        }
        if level > self.config.levels {
            return Ok(FpcsaUpdate::Truncated { row: sample.row });
        }
        if !sample.coefficient_one {
            return Ok(FpcsaUpdate::ZeroCoefficient {
                row: sample.row,
                level,
            });
        }

        let bit = self.bit_offset(sample.row, level);
        let word = bit / 64;
        let mask = 1_u64 << (bit % 64);
        self.words[word] ^= mask;

        Ok(FpcsaUpdate::Stored {
            row: sample.row,
            level,
        })
    }

    fn row_highest_level_unchecked(&self, row: u32) -> Option<u8> {
        if self.config.levels == 64 {
            let word = self.words[row as usize];
            return (word != 0).then(|| (64 - word.leading_zeros()) as u8);
        }

        (1..=self.config.levels)
            .rev()
            .find(|&level| self.bit_is_set(row, level))
    }

    fn bit_is_set(&self, row: u32, level: u8) -> bool {
        let bit = self.bit_offset(row, level);
        let word = bit / 64;
        let mask = 1_u64 << (bit % 64);
        self.words[word] & mask != 0
    }

    fn bit_offset(&self, row: u32, level: u8) -> usize {
        (row as usize) * usize::from(self.config.levels) + usize::from(level - 1)
    }

    #[cfg(test)]
    fn apply_explicit_sample(
        &mut self,
        row: u32,
        level: Option<u8>,
        coefficient_one: bool,
    ) -> Result<FpcsaUpdate, FpcsaError> {
        self.apply_sample(FpcsaSample {
            row,
            level,
            coefficient_one,
        })
    }
}

fn oracle_word(seed: u64, key: u64, domain: u64, attempt: u64) -> u64 {
    mix64(
        key.wrapping_add(seed.rotate_left(17))
            .wrapping_add(domain)
            .wrapping_add(attempt.wrapping_mul(ATTEMPT_STEP)),
    )
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(rows: u32, levels: u8) -> PublishedFpcsaF2Config {
        PublishedFpcsaF2Config::new(rows, levels, PublishedFpcsaOracle::new(1, 2, 3)).unwrap()
    }

    #[test]
    fn one_row_is_valid_for_definition_reproduction() {
        let one = config(1, 1);
        assert_eq!(one.rows(), 1);
        assert_eq!(one.levels(), 1);
        assert_eq!(one.logical_state_bits(), 1);
    }

    #[test]
    fn zero_rows_are_rejected() {
        assert_eq!(
            PublishedFpcsaF2Config::new(0, 8, PublishedFpcsaOracle::new(1, 2, 3)).unwrap_err(),
            FpcsaError::RowsMustBePositive
        );
    }

    #[test]
    fn packed_state_size_matches_m_times_j_bits() {
        let c32 = config(256, 32);
        assert_eq!(c32.logical_state_bits(), 8192);
        assert_eq!(c32.packed_state_words(), 128);
        assert_eq!(c32.packed_state_bytes(), 1024);

        let c64 = config(256, 64);
        assert_eq!(c64.logical_state_bits(), 16384);
        assert_eq!(c64.packed_state_words(), 256);
        assert_eq!(c64.packed_state_bytes(), 2048);
    }

    #[test]
    fn pseudo_oracle_reference_samples_are_stable() {
        let c = config(8, 16);

        assert_eq!(
            c.sample_key(0),
            FpcsaSample {
                row: 3,
                level: Some(1),
                coefficient_one: true,
            }
        );
        assert_eq!(
            c.sample_key(1),
            FpcsaSample {
                row: 0,
                level: Some(1),
                coefficient_one: false,
            }
        );
        assert_eq!(
            c.sample_key(2),
            FpcsaSample {
                row: 2,
                level: Some(2),
                coefficient_one: false,
            }
        );
        assert_eq!(
            c.sample_key(42),
            FpcsaSample {
                row: 6,
                level: Some(3),
                coefficient_one: true,
            }
        );
    }

    #[test]
    fn repeated_toggle_of_same_key_cancels() {
        let mut sketch = PublishedFpcsaF2::new(config(8, 16));

        assert_eq!(sketch.toggle(0), FpcsaUpdate::Stored { row: 3, level: 1 });
        assert!(sketch.packed_words().iter().any(|&word| word != 0));

        sketch.toggle(0);
        assert!(sketch.packed_words().iter().all(|&word| word == 0));
    }

    #[test]
    fn zero_coefficient_does_not_change_state() {
        let mut sketch = PublishedFpcsaF2::new(config(8, 16));
        let before = sketch.packed_words().to_vec();

        assert_eq!(
            sketch.toggle(1),
            FpcsaUpdate::ZeroCoefficient { row: 0, level: 1 }
        );
        assert_eq!(sketch.packed_words(), before);
    }

    #[test]
    fn finite_level_truncation_is_explicit() {
        let mut sketch = PublishedFpcsaF2::new(config(8, 1));
        let before = sketch.packed_words().to_vec();

        assert_eq!(sketch.toggle(2), FpcsaUpdate::Truncated { row: 2 });
        assert_eq!(sketch.packed_words(), before);
    }

    #[test]
    fn highest_level_tracks_gf2_cancellation() {
        let mut sketch = PublishedFpcsaF2::new(config(3, 8));

        sketch.apply_explicit_sample(0, Some(3), true).unwrap();
        sketch.apply_explicit_sample(0, Some(5), true).unwrap();
        assert_eq!(sketch.row_highest_level(0).unwrap(), Some(5));

        sketch.apply_explicit_sample(0, Some(5), true).unwrap();
        assert_eq!(sketch.row_highest_level(0).unwrap(), Some(3));
    }

    #[test]
    fn xor_merge_matches_combined_toggle_stream() {
        let c = config(16, 32);
        let mut left = PublishedFpcsaF2::new(c.clone());
        let mut right = PublishedFpcsaF2::new(c.clone());
        let mut combined = PublishedFpcsaF2::new(c);

        let left_keys = [1, 2, 3, 5, 8, 13, 21];
        let right_keys = [3, 5, 34, 55, 89];

        for key in left_keys {
            left.toggle(key);
            combined.toggle(key);
        }
        for key in right_keys {
            right.toggle(key);
            combined.toggle(key);
        }

        let merged = left.xor_merged(&right).unwrap();
        assert_eq!(merged.packed_words(), combined.packed_words());
        assert_eq!(merged.highest_levels(), combined.highest_levels());
    }

    #[test]
    fn incompatible_config_fails_closed() {
        let left = PublishedFpcsaF2::new(config(8, 16));
        let right = PublishedFpcsaF2::new(
            PublishedFpcsaF2Config::new(8, 16, PublishedFpcsaOracle::new(10, 20, 30)).unwrap(),
        );

        assert_eq!(
            left.xor_merged(&right).unwrap_err(),
            FpcsaError::IncompatibleConfig
        );
    }

    #[test]
    fn table2_reference_requires_middle_range_nonempty_rows() {
        let sketch = PublishedFpcsaF2::new(config(8, 16));

        assert_eq!(
            sketch.table2_reference_estimate().unwrap_err(),
            FpcsaError::MiddleRangeRequired { empty_rows: 8 }
        );
    }

    #[test]
    fn table2_reference_uses_published_rounded_f2_constants() {
        let mut sketch = PublishedFpcsaF2::new(config(3, 8));
        sketch.apply_explicit_sample(0, Some(1), true).unwrap();
        sketch.apply_explicit_sample(1, Some(2), true).unwrap();
        sketch.apply_explicit_sample(2, Some(3), true).unwrap();

        let estimate = sketch.table2_reference_estimate().unwrap();
        let expected_point = 1.079 * 3.0 * 2_f64.powf(2.0);
        let expected_rse = 1.638 / 3_f64.sqrt();

        assert!((estimate.point - expected_point).abs() < 1e-12);
        assert!((estimate.asymptotic_relative_standard_error - expected_rse).abs() < 1e-12);
    }
}
