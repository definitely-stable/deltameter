use core::fmt;

use crate::energy::Coverage;
use crate::fpcsa::{
    FpcsaError, FpcsaUpdate, PublishedFpcsaF2, PublishedFpcsaF2Config, PublishedFpcsaOracle,
};

const PARITY_RSE_COEFFICIENT: f64 = 1.638;
const ROW_SEED_DOMAIN: u64 = 0x6A09_E667_F3BC_C909;
const LEVEL_SEED_DOMAIN: u64 = 0xBB67_AE85_84CA_A73B;
const COEFFICIENT_SEED_DOMAIN: u64 = 0x3C6E_F372_FE94_F82B;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityProfile {
    Compact,
    Standard,
    Accurate,
}

impl ParityProfile {
    pub const fn rows(self) -> u32 {
        match self {
            Self::Compact => 64,
            Self::Standard => 256,
            Self::Accurate => 1024,
        }
    }

    pub const fn stored_levels(self) -> u8 {
        64
    }

    pub const fn packed_state_bytes(self) -> usize {
        match self {
            Self::Compact => 512,
            Self::Standard => 2048,
            Self::Accurate => 8192,
        }
    }

    pub const fn asymptotic_relative_standard_error(self) -> f64 {
        match self {
            Self::Compact => 0.20475,
            Self::Standard => 0.102375,
            Self::Accurate => 0.0511875,
        }
    }

    fn from_shape(rows: u32, stored_levels: u8) -> Option<Self> {
        if stored_levels != 64 {
            return None;
        }

        match rows {
            64 => Some(Self::Compact),
            256 => Some(Self::Standard),
            1024 => Some(Self::Accurate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParityError {
    Backend(FpcsaError),
    IncompatibleConfig,
    EstimateUnavailable { empty_rows: usize },
}

impl fmt::Display for ParityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(error) => write!(f, "F-PCSA backend error: {error}"),
            Self::IncompatibleConfig => f.write_str("Parity configurations are incompatible"),
            Self::EstimateUnavailable { empty_rows } => {
                write!(
                    f,
                    "asymptotic Parity estimate unavailable with {empty_rows} empty rows"
                )
            }
        }
    }
}

impl std::error::Error for ParityError {}

impl From<FpcsaError> for ParityError {
    fn from(value: FpcsaError) -> Self {
        match value {
            FpcsaError::IncompatibleConfig => Self::IncompatibleConfig,
            FpcsaError::MiddleRangeRequired { empty_rows } => {
                Self::EstimateUnavailable { empty_rows }
            }
            other => Self::Backend(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityConfig {
    seed: u64,
    backend: PublishedFpcsaF2Config,
}

impl ParityConfig {
    /// Builds an experimental Parity configuration.
    ///
    /// The seed is expanded deterministically into the three pseudo-oracle
    /// domains used by M2. This is an engineering/reproducibility mechanism,
    /// not a claim that a 64-bit seed instantiates the ideal random oracle
    /// assumed by the published F-PCSA analysis.
    pub fn new(rows: u32, stored_levels: u8, seed: u64) -> Result<Self, ParityError> {
        let oracle = PublishedFpcsaOracle::new(
            derive_seed(seed, ROW_SEED_DOMAIN),
            derive_seed(seed, LEVEL_SEED_DOMAIN),
            derive_seed(seed, COEFFICIENT_SEED_DOMAIN),
        );
        let backend = PublishedFpcsaF2Config::new(rows, stored_levels, oracle)?;

        Ok(Self { seed, backend })
    }

    pub fn for_profile(profile: ParityProfile, seed: u64) -> Result<Self, ParityError> {
        Self::new(profile.rows(), profile.stored_levels(), seed)
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub const fn rows(&self) -> u32 {
        self.backend.rows()
    }

    pub const fn stored_levels(&self) -> u8 {
        self.backend.levels()
    }

    pub const fn logical_state_bits(&self) -> usize {
        self.backend.logical_state_bits()
    }

    pub const fn packed_state_bytes(&self) -> usize {
        self.backend.packed_state_bytes()
    }

    pub fn profile(&self) -> Option<ParityProfile> {
        ParityProfile::from_shape(self.rows(), self.stored_levels())
    }

    pub fn asymptotic_relative_standard_error(&self) -> f64 {
        PARITY_RSE_COEFFICIENT / f64::from(self.rows()).sqrt()
    }

    fn backend(&self) -> &PublishedFpcsaF2Config {
        &self.backend
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityUpdate {
    Stored { row: u32, level: u8 },
    ZeroCoefficient { row: u32, level: u8 },
    Truncated { row: u32 },
}

impl From<FpcsaUpdate> for ParityUpdate {
    fn from(value: FpcsaUpdate) -> Self {
        match value {
            FpcsaUpdate::Stored { row, level } => Self::Stored { row, level },
            FpcsaUpdate::ZeroCoefficient { row, level } => Self::ZeroCoefficient { row, level },
            FpcsaUpdate::Truncated { row } => Self::Truncated { row },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParityEstimate {
    pub point: f64,
    pub coverage: Coverage,
}

#[derive(Debug, Clone)]
pub struct ParityDeltaMeter {
    config: ParityConfig,
    backend: PublishedFpcsaF2,
}

impl ParityDeltaMeter {
    pub fn new(config: ParityConfig) -> Self {
        let backend = PublishedFpcsaF2::new(config.backend().clone());
        Self { config, backend }
    }

    pub fn config(&self) -> &ParityConfig {
        &self.config
    }

    pub fn packed_words(&self) -> &[u64] {
        self.backend.packed_words()
    }

    pub fn packed_state_bytes(&self) -> usize {
        self.config.packed_state_bytes()
    }

    /// Applies one GF(2) set toggle.
    ///
    /// Repeating the same key twice cancels. A Truncated result is surfaced
    /// rather than silently clamped when a sampled level lies above J.
    pub fn toggle(&mut self, key: u64) -> ParityUpdate {
        self.backend.toggle(key).into()
    }

    pub fn xor_assign(&mut self, other: &Self) -> Result<(), ParityError> {
        if self.config != other.config {
            return Err(ParityError::IncompatibleConfig);
        }

        self.backend.xor_assign(&other.backend)?;
        Ok(())
    }

    pub fn xor_merged(&self, other: &Self) -> Result<Self, ParityError> {
        if self.config != other.config {
            return Err(ParityError::IncompatibleConfig);
        }

        let backend = self.backend.xor_merged(&other.backend)?;
        Ok(Self {
            config: self.config.clone(),
            backend,
        })
    }

    /// Returns the published middle-range point estimate with explicitly
    /// asymptotic coverage metadata.
    ///
    /// M3 intentionally exposes no recommended-capacity API.
    pub fn estimate(&self) -> Result<ParityEstimate, ParityError> {
        let reference = self.backend.table2_reference_estimate()?;

        Ok(ParityEstimate {
            point: reference.point,
            coverage: Coverage::Asymptotic {
                relative_standard_error: reference.asymptotic_relative_standard_error,
            },
        })
    }
}

fn derive_seed(seed: u64, domain: u64) -> u64 {
    mix64(seed ^ domain)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_match_published_rse_scale_and_state_sizes() {
        let cases = [
            (ParityProfile::Compact, 64, 512, 0.20475),
            (ParityProfile::Standard, 256, 2048, 0.102375),
            (ParityProfile::Accurate, 1024, 8192, 0.0511875),
        ];

        for (profile, rows, bytes, rse) in cases {
            assert_eq!(profile.rows(), rows);
            assert_eq!(profile.stored_levels(), 64);
            assert_eq!(profile.packed_state_bytes(), bytes);
            assert!((profile.asymptotic_relative_standard_error() - rse).abs() < 1e-15);

            let config = ParityConfig::for_profile(profile, 7).unwrap();
            assert_eq!(config.rows(), rows);
            assert_eq!(config.stored_levels(), 64);
            assert_eq!(config.packed_state_bytes(), bytes);
            assert_eq!(config.profile(), Some(profile));
            assert!((config.asymptotic_relative_standard_error() - rse).abs() < 1e-15);
        }
    }

    #[test]
    fn deterministic_seed_controls_compatibility() {
        let left = ParityConfig::for_profile(ParityProfile::Standard, 11).unwrap();
        let same = ParityConfig::for_profile(ParityProfile::Standard, 11).unwrap();
        let different_seed = ParityConfig::for_profile(ParityProfile::Standard, 12).unwrap();

        assert_eq!(left, same);
        assert_ne!(left, different_seed);
    }

    #[test]
    fn repeated_stored_toggle_cancels() {
        let config = ParityConfig::new(8, 16, 23).unwrap();
        let mut meter = ParityDeltaMeter::new(config);

        let mut stored_key = None;
        for key in 0..1024 {
            if matches!(meter.toggle(key), ParityUpdate::Stored { .. }) {
                stored_key = Some(key);
                break;
            }
        }

        let key = stored_key.expect("test seed should produce a stored update");
        let after_first = meter.packed_words().to_vec();
        assert!(after_first.iter().any(|&word| word != 0));

        assert!(matches!(meter.toggle(key), ParityUpdate::Stored { .. }));
        assert!(meter.packed_words().iter().all(|&word| word == 0));
    }

    #[test]
    fn finite_level_truncation_is_visible() {
        let config = ParityConfig::new(4, 1, 29).unwrap();
        let mut meter = ParityDeltaMeter::new(config);

        let saw_truncation =
            (0..1024).any(|key| matches!(meter.toggle(key), ParityUpdate::Truncated { .. }));

        assert!(saw_truncation);
    }

    #[test]
    fn xor_merge_matches_combined_toggle_stream() {
        let config = ParityConfig::for_profile(ParityProfile::Compact, 31).unwrap();
        let mut left = ParityDeltaMeter::new(config.clone());
        let mut right = ParityDeltaMeter::new(config.clone());
        let mut combined = ParityDeltaMeter::new(config);

        for key in [1, 2, 3, 5, 8, 13, 21] {
            left.toggle(key);
            combined.toggle(key);
        }
        for key in [3, 5, 34, 55, 89] {
            right.toggle(key);
            combined.toggle(key);
        }

        let merged = left.xor_merged(&right).unwrap();
        assert_eq!(merged.packed_words(), combined.packed_words());
    }

    #[test]
    fn incompatible_configs_fail_closed() {
        let left =
            ParityDeltaMeter::new(ParityConfig::for_profile(ParityProfile::Compact, 37).unwrap());
        let right =
            ParityDeltaMeter::new(ParityConfig::for_profile(ParityProfile::Compact, 38).unwrap());

        assert_eq!(
            left.xor_merged(&right).unwrap_err(),
            ParityError::IncompatibleConfig
        );
    }

    #[test]
    fn estimate_is_asymptotic_and_never_proven() {
        let config = ParityConfig::new(4, 64, 41).unwrap();
        let expected_rse = config.asymptotic_relative_standard_error();
        let mut meter = ParityDeltaMeter::new(config);

        for key in 0..10_000 {
            meter.toggle(key);
        }

        let estimate = meter.estimate().unwrap();
        assert!(estimate.point.is_finite());
        assert!(estimate.point > 0.0);

        match estimate.coverage {
            Coverage::Asymptotic {
                relative_standard_error,
            } => {
                assert!((relative_standard_error - expected_rse).abs() < 1e-15);
            }
            Coverage::Proven { .. } => panic!("Parity must not expose Proven coverage in M3"),
        }
    }
}
