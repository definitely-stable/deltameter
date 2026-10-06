use deltameter::{EnergyProfile, FailureTarget, RelativeError};

const ORACLE_CSV: &str = include_str!("../research/energy_profiles.csv");

fn approx_eq(left: f64, right: f64) -> bool {
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= 8.0 * f64::EPSILON * scale
}

#[test]
fn rust_profiles_match_theorem_oracle_csv() {
    let mut rows = 0_usize;

    for line in ORACLE_CSV.lines().skip(1) {
        if line.is_empty() {
            continue;
        }

        let fields: Vec<_> = line.split(',').collect();
        assert_eq!(fields.len(), 8, "unexpected oracle row: {line}");

        let epsilon: f64 = fields[0].parse().unwrap();
        let delta: f64 = fields[1].parse().unwrap();
        let buckets: usize = fields[2].parse().unwrap();
        let tables: usize = fields[3].parse().unwrap();
        let counters: usize = fields[6].parse().unwrap();
        let state_bytes: usize = fields[7].parse().unwrap();

        let relative_error = if approx_eq(epsilon, 0.05) {
            RelativeError::FivePercent
        } else if approx_eq(epsilon, 0.10) {
            RelativeError::TenPercent
        } else if approx_eq(epsilon, 0.20) {
            RelativeError::TwentyPercent
        } else {
            panic!("unknown epsilon in oracle: {epsilon}");
        };

        let failure_target = if approx_eq(delta, 1e-3) {
            FailureTarget::OneInThousand
        } else if approx_eq(delta, 1e-6) {
            FailureTarget::OneInMillion
        } else if approx_eq(delta, 1e-9) {
            FailureTarget::OneInBillion
        } else {
            panic!("unknown delta in oracle: {delta}");
        };

        let profile = EnergyProfile::new(relative_error, failure_target);

        assert!(approx_eq(profile.relative_error().epsilon(), epsilon));
        assert!(approx_eq(profile.failure_target().probability(), delta));
        assert_eq!(profile.buckets(), buckets);
        assert_eq!(profile.tables(), tables);
        assert_eq!(profile.buckets() * profile.tables(), counters);
        assert_eq!(profile.counter_state_bytes(), state_bytes);
        assert_eq!(profile.uniform_words_required(), tables * 6);

        rows += 1;
    }

    assert_eq!(rows, 9);
}
