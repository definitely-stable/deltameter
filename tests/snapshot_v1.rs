use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, ParityConfig,
    ParityDeltaMeter, ParityError, RelativeError, SnapshotError,
};

const ENERGY_EMPTY_V1_HEX: &str = "44454c54414d5452010001010000000048000000000000000100000000000000010000000000000001000000000000000200000000000000030000000000000004000000000000000500000000000000060000000000000000000000000000009b7dc4a1";
const PARITY_EMPTY_V1_HEX: &str = "44454c54414d545201000201000000001800000000000000010000000100000007000000000000000000000000000000073d291e";
const PARITY_NONCANONICAL_PADDING_V1_HEX: &str = "44454c54414d54520100020100000000180000000000000001000000010000000700000000000000020000000000000049c7518c";

#[test]
fn energy_empty_snapshot_matches_committed_v1_vector() {
    let row = EnergyRowHash::from_coefficients([1, 2], [3, 4, 5, 6]);
    let meter = EnergyDeltaMeter::new(EnergyConfig::new(1, vec![row]).unwrap()).unwrap();

    assert_eq!(
        meter.encode_snapshot().unwrap(),
        hex_bytes(ENERGY_EMPTY_V1_HEX)
    );
}

#[test]
fn parity_empty_snapshot_matches_committed_v1_vector() {
    let meter = ParityDeltaMeter::new(ParityConfig::new(1, 1, 7).unwrap());

    assert_eq!(
        meter.encode_snapshot().unwrap(),
        hex_bytes(PARITY_EMPTY_V1_HEX)
    );
}

#[test]
fn proven_energy_round_trip_preserves_config_state_and_coverage() {
    let profile = EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
    let words: Vec<u64> = (0..profile.uniform_words_required())
        .map(|index| splitmix64(index as u64 ^ 0xA5A5_5A5A))
        .collect();
    let config = EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap();
    let mut meter = EnergyDeltaMeter::new(config).unwrap();

    for key in [1, 2, 3, 5, 8, 13, 21, 34] {
        meter.add_unique(key).unwrap();
    }

    let encoded = meter.encode_snapshot().unwrap();
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&encoded),
        Err(SnapshotError::ProvenanceRequired)
    ));
    let decoded = EnergyDeltaMeter::decode_snapshot_assuming_uniform_rows(&encoded).unwrap();

    assert_eq!(decoded.config(), meter.config());
    assert_eq!(decoded.estimate(profile), meter.estimate(profile));
    assert_eq!(decoded.difference(&meter).unwrap().point_estimate(), 0);
    assert_eq!(decoded.encode_snapshot().unwrap(), encoded);
}

#[test]
fn custom_energy_round_trip_preserves_signed_difference_state() {
    let rows = vec![EnergyRowHash::from_coefficients([11, 12], [13, 14, 15, 16])];
    let config = EnergyConfig::new(1, rows).unwrap();
    let left = EnergyDeltaMeter::new(config.clone()).unwrap();
    let mut right = EnergyDeltaMeter::new(config).unwrap();
    right.add_unique(99).unwrap();
    let meter = left.difference(&right).unwrap();

    let decoded = EnergyDeltaMeter::decode_snapshot(&meter.encode_snapshot().unwrap()).unwrap();

    assert_eq!(decoded.config(), meter.config());
    assert_eq!(decoded.config().proven_profile(), None);
    assert_eq!(decoded.difference(&meter).unwrap().point_estimate(), 0);
}

#[test]
fn parity_round_trip_preserves_config_state_and_merge_behavior() {
    let config = ParityConfig::new(17, 13, 0x1234_5678_9ABC_DEF0).unwrap();
    let mut meter = ParityDeltaMeter::new(config);

    for key in 0..500 {
        meter.toggle(key);
    }

    let encoded = meter.encode_snapshot().unwrap();
    let decoded = ParityDeltaMeter::decode_snapshot(&encoded).unwrap();

    assert_eq!(decoded.config(), meter.config());
    assert_eq!(decoded.encode_snapshot().unwrap(), encoded);

    let cancelled = decoded.xor_merged(&meter).unwrap();
    assert!(matches!(
        cancelled.estimate(),
        Err(ParityError::EstimateUnavailable { empty_rows: 17 })
    ));
}

#[test]
fn wrong_backend_fails_closed() {
    let energy = hex_bytes(ENERGY_EMPTY_V1_HEX);
    let parity = hex_bytes(PARITY_EMPTY_V1_HEX);

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&parity),
        Err(SnapshotError::BackendMismatch {
            expected: 1,
            actual: 2
        })
    ));
    assert!(matches!(
        ParityDeltaMeter::decode_snapshot(&energy),
        Err(SnapshotError::BackendMismatch {
            expected: 2,
            actual: 1
        })
    ));
}

#[test]
fn corruption_and_trailing_bytes_fail_closed() {
    let mut corrupt = hex_bytes(ENERGY_EMPTY_V1_HEX);
    corrupt[32] ^= 1;
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&corrupt),
        Err(SnapshotError::ChecksumMismatch)
    ));

    let mut trailing = hex_bytes(ENERGY_EMPTY_V1_HEX);
    trailing.push(0);
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&trailing),
        Err(SnapshotError::LengthMismatch { .. })
    ));
}

#[test]
fn unknown_version_and_flags_fail_closed() {
    let mut version = hex_bytes(ENERGY_EMPTY_V1_HEX);
    version[8] = 2;
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&version),
        Err(SnapshotError::UnsupportedVersion(2))
    ));

    let mut domain = hex_bytes(ENERGY_EMPTY_V1_HEX);
    domain[11] = 2;
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&domain),
        Err(SnapshotError::UnsupportedDomain(2))
    ));

    let mut flags = hex_bytes(ENERGY_EMPTY_V1_HEX);
    flags[12] = 1;
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&flags),
        Err(SnapshotError::UnsupportedFlags(1))
    ));

    let mut reserved = hex_bytes(ENERGY_EMPTY_V1_HEX);
    reserved[13] = 1;
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&reserved),
        Err(SnapshotError::InvalidPayload)
    ));
}

#[test]
fn every_strict_snapshot_prefix_is_rejected() {
    let energy = hex_bytes(ENERGY_EMPTY_V1_HEX);
    for end in 0..energy.len() {
        assert!(EnergyDeltaMeter::decode_snapshot(&energy[..end]).is_err());
    }

    let parity = hex_bytes(PARITY_EMPTY_V1_HEX);
    for end in 0..parity.len() {
        assert!(ParityDeltaMeter::decode_snapshot(&parity[..end]).is_err());
    }
}

#[test]
fn parity_non_zero_padding_bits_are_rejected() {
    let bytes = hex_bytes(PARITY_NONCANONICAL_PADDING_V1_HEX);

    assert!(matches!(
        ParityDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::NonCanonicalPadding)
    ));
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
