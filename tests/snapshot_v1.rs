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
    let mut decoded = ParityDeltaMeter::decode_snapshot(&encoded).unwrap();

    assert_eq!(decoded.config(), meter.config());
    assert_eq!(decoded.encode_snapshot().unwrap(), encoded);

    let cancelled = decoded.xor_merged(&meter).unwrap();
    assert!(matches!(
        cancelled.estimate(),
        Err(ParityError::EstimateUnavailable { empty_rows: 17 })
    ));

    // A paired single update checks continued oracle/config interpretation;
    // double toggles alone would pass even if both updates were no-ops.
    for key in [0, 1 << 63, u64::MAX] {
        assert_eq!(meter.toggle(key), decoded.toggle(key));
        assert_eq!(
            decoded.encode_snapshot().unwrap(),
            meter.encode_snapshot().unwrap()
        );
    }
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

#[test]
fn valid_crc_energy_shape_mismatch_reaches_backend_validation() {
    let mut bytes = hex_bytes(ENERGY_EMPTY_V1_HEX);
    bytes[24..32].copy_from_slice(&2_u64.to_le_bytes());

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::ChecksumMismatch)
    ));

    rewrite_snapshot_crc32c(&mut bytes);
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::InvalidPayload)
    ));
}

#[test]
fn valid_crc_energy_row_shape_mismatch_fails_closed() {
    let mut bytes = hex_bytes(ENERGY_EMPTY_V1_HEX);
    bytes[32..36].copy_from_slice(&3_u32.to_le_bytes());
    rewrite_snapshot_crc32c(&mut bytes);

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::InvalidPayload)
    ));
}

#[test]
fn valid_crc_energy_invalid_custom_tags_fail_closed() {
    let mut bytes = hex_bytes(ENERGY_EMPTY_V1_HEX);
    bytes[37] = 1;
    rewrite_snapshot_crc32c(&mut bytes);

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::InvalidPayload)
    ));
}

#[test]
fn proven_energy_keeps_provenance_error_precedence_before_shape_validation() {
    let profile = EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
    let words: Vec<_> = (0..profile.uniform_words_required())
        .map(|index| splitmix64(index as u64 ^ 0x0D15_EA5E))
        .collect();
    let meter = EnergyDeltaMeter::new(
        EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap(),
    )
    .unwrap();
    let mut bytes = meter.encode_snapshot().unwrap();

    bytes[32..36].copy_from_slice(&1_u32.to_le_bytes());
    rewrite_snapshot_crc32c(&mut bytes);

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::ProvenanceRequired)
    ));
    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot_assuming_uniform_rows(&bytes),
        Err(SnapshotError::InvalidPayload)
    ));
}

#[test]
fn valid_crc_parity_shape_mismatch_reaches_backend_validation() {
    let mut bytes = hex_bytes(PARITY_EMPTY_V1_HEX);
    bytes[24..28].copy_from_slice(&65_u32.to_le_bytes());

    assert!(matches!(
        ParityDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::ChecksumMismatch)
    ));

    rewrite_snapshot_crc32c(&mut bytes);
    assert!(matches!(
        ParityDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::InvalidPayload)
    ));
}

fn rewrite_snapshot_crc32c(bytes: &mut [u8]) {
    let checksum_offset = bytes
        .len()
        .checked_sub(core::mem::size_of::<u32>())
        .expect("snapshot contains CRC32C");
    let checksum = crc32c_bitwise_reference(&bytes[..checksum_offset]);
    bytes[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

fn crc32c_bitwise_reference(bytes: &[u8]) -> u32 {
    const POLYNOMIAL: u32 = 0x82F6_3B78;

    let mut crc = !0_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (POLYNOMIAL & mask);
        }
    }
    !crc
}

#[test]
fn valid_crc_energy_row_energy_overflow_remains_invalid_payload() {
    let mut bytes = hex_bytes(ENERGY_NEGATIVE_V1_HEX);
    let counter_offset = 24 + 16 + 48;
    bytes[counter_offset..counter_offset + 8].copy_from_slice(&i64::MIN.to_le_bytes());
    bytes[counter_offset + 8..counter_offset + 16].copy_from_slice(&i64::MIN.to_le_bytes());
    rewrite_snapshot_crc32c(&mut bytes);

    assert!(matches!(
        EnergyDeltaMeter::decode_snapshot(&bytes),
        Err(SnapshotError::InvalidPayload)
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

// Independently assembled from the M5 field layout (Python struct + bitwise
// Castagnoli CRC), not captured from the encoder being tested. Energy: B=2,
// one all-zero custom row, counters [-1, 0]. Parity: m=17, J=13, seed=7,
// words [1, 1<<63, 0, 1<<28]; the last used bit is bit 220.
const ENERGY_NEGATIVE_V1_HEX: &str = "44454c54414d54520100010100000000500000000000000002000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffff000000000000000064568660";
const PARITY_MULTIWORD_V1_HEX: &str = "44454c54414d545201000201000000003000000000000000110000000d00000007000000000000000100000000000000000000000000008000000000000000000000001000000000d820d11d";

#[test]
fn negative_energy_fixed_vector_matches_independent_state_and_continues() {
    let row = EnergyRowHash::from_coefficients([0; 2], [0; 4]);
    let config = EnergyConfig::new(2, vec![row]).unwrap();
    let left = EnergyDeltaMeter::new(config.clone()).unwrap();
    let mut right = EnergyDeltaMeter::new(config).unwrap();
    right.add_unique(7).unwrap();
    let difference = left.difference(&right).unwrap();
    let bytes = hex_bytes(ENERGY_NEGATIVE_V1_HEX);
    assert_eq!(difference.encode_snapshot().unwrap(), bytes);
    let mut restored = EnergyDeltaMeter::decode_snapshot(&bytes).unwrap();
    assert_eq!(restored.point_estimate(), 1);
    restored.add_unique(7).unwrap();
    assert_eq!(
        restored.encode_snapshot().unwrap(),
        left.encode_snapshot().unwrap()
    );
}

#[test]
fn nonempty_multiword_parity_fixed_vector_preserves_padding_and_updates() {
    let bytes = hex_bytes(PARITY_MULTIWORD_V1_HEX);
    let mut restored = ParityDeltaMeter::decode_snapshot(&bytes).unwrap();
    assert_eq!(restored.encode_snapshot().unwrap(), bytes);
    for key in [0, 1, 1 << 63, u64::MAX] {
        restored.toggle(key);
        restored.toggle(key);
    }
    assert_eq!(restored.encode_snapshot().unwrap(), bytes);
}

#[test]
fn every_energy_profile_preserves_snapshot_length_and_continuation() {
    for error in [
        RelativeError::FivePercent,
        RelativeError::TenPercent,
        RelativeError::TwentyPercent,
    ] {
        for failure in [
            FailureTarget::OneInThousand,
            FailureTarget::OneInMillion,
            FailureTarget::OneInBillion,
        ] {
            let profile = EnergyProfile::new(error, failure);
            // Deterministic compatibility fixture, not evidence of uniform randomness.
            let words: Vec<_> = (0..profile.uniform_words_required())
                .map(|i| splitmix64(i as u64))
                .collect();
            let config = EnergyConfig::for_profile_assuming_uniform_words(profile, &words).unwrap();
            let mut original = EnergyDeltaMeter::new(config).unwrap();
            original.add_unique(u64::MAX).unwrap();
            let bytes = original.encode_snapshot().unwrap();
            assert_eq!(
                bytes.len(),
                44 + 48 * profile.tables() + profile.counter_state_bytes()
            );
            let mut restored =
                EnergyDeltaMeter::decode_snapshot_assuming_uniform_rows(&bytes).unwrap();
            assert_eq!(restored.encode_snapshot().unwrap(), bytes);
            for key in [0, 1, 1 << 63] {
                original.add_unique(key).unwrap();
                restored.add_unique(key).unwrap();
            }
            assert_eq!(
                restored.encode_snapshot().unwrap(),
                original.encode_snapshot().unwrap()
            );
            assert_eq!(restored.estimate(profile), original.estimate(profile));
        }
    }
}
