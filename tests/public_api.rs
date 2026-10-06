use deltameter::{
    Coverage, EnergyConfig, EnergyDeltaMeter, EnergyError, EnergyProfile, EnergyRowHash,
    FailureTarget, ParityConfig, ParityDeltaMeter, ParityProfile, ParityUpdate, RelativeError,
    SnapshotError,
};

#[test]
fn supported_root_api_contract_smoke() {
    let energy_profile = EnergyProfile::DEFAULT;
    assert_eq!(energy_profile.relative_error(), RelativeError::TenPercent);
    assert_eq!(energy_profile.failure_target(), FailureTarget::OneInMillion);

    let row = EnergyRowHash::from_coefficients([1, 2], [3, 4, 5, 6]);
    let config = EnergyConfig::new(energy_profile.buckets(), vec![row; energy_profile.tables()])
        .expect("default dimensions are valid");
    let mut left = EnergyDeltaMeter::new(config.clone()).unwrap();
    let right = EnergyDeltaMeter::new(config).unwrap();

    left.add_unique(7).unwrap();
    let difference = left.difference(&right).unwrap();
    assert!(difference.point_estimate() > 0);
    let custom_snapshot = difference.encode_snapshot().unwrap();
    let decoded_custom = EnergyDeltaMeter::decode_snapshot(&custom_snapshot).unwrap();
    assert_eq!(decoded_custom.config(), difference.config());
    let _decode_proven: fn(&[u8]) -> Result<EnergyDeltaMeter, SnapshotError> =
        EnergyDeltaMeter::decode_snapshot_assuming_uniform_rows;
    assert_eq!(
        difference.estimate(energy_profile),
        Err(EnergyError::ProfileMismatch)
    );

    let parity_profile = ParityProfile::DEFAULT;
    assert_eq!(parity_profile, ParityProfile::Standard);

    let parity_config = ParityConfig::for_profile(parity_profile, 11).unwrap();
    let mut parity = ParityDeltaMeter::new(parity_config);
    let update = parity.toggle(7);
    match update {
        ParityUpdate::Stored { .. }
        | ParityUpdate::ZeroCoefficient { .. }
        | ParityUpdate::Truncated { .. } => {}
        _ => unreachable!("future update variants remain source-compatible"),
    }

    assert_eq!(parity.packed_state_bytes(), 2048);
    let parity_snapshot = parity.encode_snapshot().unwrap();
    let decoded_parity = ParityDeltaMeter::decode_snapshot(&parity_snapshot).unwrap();
    assert_eq!(decoded_parity.config(), parity.config());

    let _snapshot_error_type: Option<SnapshotError> = None;

    let coverage = Coverage::Asymptotic {
        relative_standard_error: parity_profile.asymptotic_relative_standard_error(),
    };
    match coverage {
        Coverage::Asymptotic {
            relative_standard_error,
        } => assert!(relative_standard_error > 0.0),
        _ => unreachable!("future coverage variants remain source-compatible"),
    }
}
