//! M6-D12 hosted diagnostic: post-D11 whole-decode residual profile.
//!
//! Profile-only research harness. Accepted D11 is the uninstrumented control.

#[path = "support/m6d11_fixed_reduction.rs"]
mod fixed_reduction;
#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "support/m6d12_residual_profile.rs"]
mod residual_profile;
#[path = "support/m6d8_square.rs"]
mod square_candidate;
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;

use std::hint::black_box;
use std::time::Instant;

use fixed_reduction::{
    FixedMultiplier, decode_with_locator_fixed_reduction, gf64_mul_reference_for_d11,
};
use pinsketch64::PinSketch64Lab;
use quadratic::decode_with_locator_quadratic;
use residual_profile::{
    ResidualProfile, decode_with_locator_profiled_d11,
    gf64_mul_reference_for_d11 as profiled_gf64_mul_reference,
};
use square_candidate::{
    decode_with_locator_square_candidate, gf64_square_candidate, gf64_square_scalar_reference,
};
use trace_square::{
    D4Error, decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
    generic_square_mod, poly_square_mod_monic,
};

const SOURCE_KEYS: usize = 8_192;
const SAMPLES: usize = 4;
const STAGES: [(usize, usize); 4] = [(1, 2), (2, 3), (4, 5), (8, 9)];

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    d: usize,
    include_zero: bool,
}

#[derive(Clone, Copy)]
struct Corpus {
    name: &'static str,
    left_salt: u64,
    right_salt: u64,
    add_mask: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Exact,
    Rejected,
    FalseSuccess,
}

impl Outcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Rejected => "rejected",
            Self::FalseSuccess => "false_success",
        }
    }
}

#[derive(Debug)]
struct ControlResult {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    candidate: Option<Vec<u64>>,
    elapsed_ns: u128,
}

#[derive(Debug)]
struct ProfileResult {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    candidate: Option<Vec<u64>>,
    elapsed_ns: u128,
    prefix_ns: u128,
    locator_ns: u128,
    decode_wall_ns: u128,
    profile: ResidualProfile,
}

fn main() {
    println!("format=deltameter.m6d12-post-d11-profile.v1");
    println!("contract=frozen_m6d11_whole_decode_profile");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("corpora=d4;d5;d6;d7a;d7b");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("payloads=17;25;41;73");
    println!(
        "record,kind,corpus,scenario,d,sample,degree,outcome,final_k,attempts,rtts,payload_bytes,control_ns,profile_wall_ns,prefix_ns,locator_ns,decode_wall_ns,validation_ns,factor_wall_ns,verification_ns,factor_calls,plan_build_calls,trace_attempts,quadratic_calls,self_ns,plan_build_ns,quadratic_ns,trace_ns,gcd_ns,division_ns,false_success"
    );

    validate_arithmetic_controls();

    let corpora = [
        Corpus {
            name: "d4",
            left_salt: 0xD400_BA5E_0000_0001,
            right_salt: 0xD400_D1FF_0000_0000,
            add_mask: 0xD400_5A5A_0000_0000,
        },
        Corpus {
            name: "d5",
            left_salt: 0xD500_BA5E_0000_0001,
            right_salt: 0xD500_D1FF_0000_0000,
            add_mask: 0xD500_5A5A_0000_0000,
        },
        Corpus {
            name: "d6",
            left_salt: 0xD600_BA5E_0000_0001,
            right_salt: 0xD600_D1FF_0000_0000,
            add_mask: 0xD600_5A5A_0000_0000,
        },
        Corpus {
            name: "d7a",
            left_salt: 0xD700_BA5E_0000_0001,
            right_salt: 0xD700_D1FF_0000_0000,
            add_mask: 0xD700_5A5A_0000_0000,
        },
        Corpus {
            name: "d7b",
            left_salt: 0xD701_BA5E_0000_0001,
            right_salt: 0xD701_D1FF_0000_0000,
            add_mask: 0xD701_5A5A_0000_0000,
        },
    ];

    let scenarios = [
        Scenario {
            name: "d0",
            d: 0,
            include_zero: false,
        },
        Scenario {
            name: "d1-zero",
            d: 1,
            include_zero: true,
        },
        Scenario {
            name: "d2",
            d: 2,
            include_zero: false,
        },
        Scenario {
            name: "d3",
            d: 3,
            include_zero: false,
        },
        Scenario {
            name: "d4",
            d: 4,
            include_zero: false,
        },
        Scenario {
            name: "d5",
            d: 5,
            include_zero: false,
        },
        Scenario {
            name: "d8",
            d: 8,
            include_zero: false,
        },
        Scenario {
            name: "d9",
            d: 9,
            include_zero: false,
        },
        Scenario {
            name: "d10",
            d: 10,
            include_zero: false,
        },
        Scenario {
            name: "d16",
            d: 16,
            include_zero: false,
        },
    ];

    for corpus in corpora {
        let left = canonical_keys(SOURCE_KEYS, corpus.left_salt);

        for scenario in scenarios {
            let right = derive_source(
                &left,
                scenario.d,
                corpus.right_salt ^ scenario.d as u64,
                corpus.add_mask,
                scenario.include_zero,
            );
            let expected = symmetric_difference(&left, &right);
            assert_eq!(expected.len(), scenario.d);

            let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
            let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
            validate_controls(&left_full, &right_full, &expected);

            let warm_control = run_control(&left_full, &right_full, &expected);
            let warm_profile = run_profiled(&left_full, &right_full, &expected);
            assert_equivalent(&warm_control, &warm_profile);
            black_box(warm_control.elapsed_ns);
            black_box(warm_profile.elapsed_ns);

            for sample in 0..SAMPLES {
                let (control, profiled) = if sample % 2 == 0 {
                    (
                        run_control(&left_full, &right_full, &expected),
                        run_profiled(&left_full, &right_full, &expected),
                    )
                } else {
                    let profiled = run_profiled(&left_full, &right_full, &expected);
                    let control = run_control(&left_full, &right_full, &expected);
                    (control, profiled)
                };

                assert_equivalent(&control, &profiled);
                profiled.profile.validate_accounting().unwrap();

                emit_scenario(corpus, scenario, sample, &control, &profiled);
                for degree in 1..=8 {
                    emit_degree(corpus, scenario, sample, degree, &control, &profiled);
                }
            }
        }
    }
}

fn validate_arithmetic_controls() {
    let constants = [0_u64, 1, 0x1B, 1_u64 << 63, u64::MAX, 0xDEAD_BEEF_CAFE_BABE];
    for constant in constants {
        let multiplier = FixedMultiplier::new(constant);
        for bit in 0..64 {
            let variable = 1_u64 << bit;
            let reference = gf64_mul_reference_for_d11(variable, constant);
            assert_eq!(multiplier.multiply(variable), reference);
            assert_eq!(profiled_gf64_mul_reference(variable, constant), reference);
        }
    }

    for bit in 0..64 {
        let value = 1_u64 << bit;
        assert_eq!(
            gf64_square_candidate(value),
            gf64_square_scalar_reference(value)
        );
    }

    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}

fn validate_controls(left: &PinSketch64Lab, right: &PinSketch64Lab, expected: &[u64]) {
    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(left, right, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();

        let frozen = sketch.decode_candidate_with_limit(limit);
        let generic = decode_with_locator_generic(&sketch, limit, &locator);
        let d4 = decode_with_locator_specialized(&sketch, limit, &locator);
        let d6 = decode_with_locator_quadratic(&sketch, limit, &locator);
        let control = decode_with_locator_square_candidate(&sketch, limit, &locator);
        let candidate = decode_with_locator_fixed_reduction(&sketch, limit, &locator);

        assert_eq!(generic, frozen.clone().map_err(D4Error::from));
        assert_eq!(d4, frozen.clone().map_err(D4Error::from));
        assert_eq!(d6, d4);
        assert_eq!(control, d6);
        assert_eq!(candidate, control);

        if expected.len() <= limit {
            assert_eq!(frozen.unwrap(), expected);
        } else {
            assert!(frozen.is_err());
        }
    }
}

fn run_control(left: &PinSketch64Lab, right: &PinSketch64Lab, expected: &[u64]) -> ControlResult {
    let started = Instant::now();
    let mut candidate = None;
    let mut final_k = 8;
    let mut attempts = 0;
    let mut payload_bytes = 73;

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(black_box(left), black_box(right), capacity);
        let locator = fresh_locator(black_box(&sketch), limit).unwrap();
        let result =
            decode_with_locator_fixed_reduction(black_box(&sketch), limit, black_box(&locator));

        attempts += 1;
        final_k = limit;
        payload_bytes = prefix_payload_bytes(capacity);

        if let Ok(roots) = result {
            candidate = Some(roots);
            break;
        }
    }

    black_box(&candidate);
    let elapsed_ns = started.elapsed().as_nanos();
    let outcome = classify(candidate.as_ref(), expected, expected.len() <= final_k);
    validate_final(outcome, final_k, expected);

    ControlResult {
        outcome,
        final_k,
        attempts,
        payload_bytes,
        candidate,
        elapsed_ns,
    }
}

fn run_profiled(left: &PinSketch64Lab, right: &PinSketch64Lab, expected: &[u64]) -> ProfileResult {
    let started = Instant::now();
    let mut aggregate = ResidualProfile::default();
    let mut prefix_ns = 0_u128;
    let mut locator_ns = 0_u128;
    let mut decode_wall_ns = 0_u128;
    let mut candidate = None;
    let mut final_k = 8;
    let mut attempts = 0;
    let mut payload_bytes = 73;

    for (limit, capacity) in STAGES {
        let prefix_started = Instant::now();
        let sketch = difference_prefix(black_box(left), black_box(right), capacity);
        prefix_ns += prefix_started.elapsed().as_nanos();

        let locator_started = Instant::now();
        let locator = fresh_locator(black_box(&sketch), limit).unwrap();
        locator_ns += locator_started.elapsed().as_nanos();

        let decode_started = Instant::now();
        let profiled = decode_with_locator_profiled_d11(&sketch, limit, &locator);
        decode_wall_ns += decode_started.elapsed().as_nanos();
        aggregate.merge(&profiled.profile);

        attempts += 1;
        final_k = limit;
        payload_bytes = prefix_payload_bytes(capacity);

        if let Ok(roots) = profiled.result {
            candidate = Some(roots);
            break;
        }
    }

    black_box(&candidate);
    aggregate.validate_accounting().unwrap();
    let elapsed_ns = started.elapsed().as_nanos();
    let outcome = classify(candidate.as_ref(), expected, expected.len() <= final_k);
    validate_final(outcome, final_k, expected);

    ProfileResult {
        outcome,
        final_k,
        attempts,
        payload_bytes,
        candidate,
        elapsed_ns,
        prefix_ns,
        locator_ns,
        decode_wall_ns,
        profile: aggregate,
    }
}

fn assert_equivalent(control: &ControlResult, profiled: &ProfileResult) {
    assert_eq!(control.outcome, profiled.outcome);
    assert_eq!(control.final_k, profiled.final_k);
    assert_eq!(control.attempts, profiled.attempts);
    assert_eq!(control.payload_bytes, profiled.payload_bytes);
    assert_eq!(control.candidate, profiled.candidate);
}

fn emit_scenario(
    corpus: Corpus,
    scenario: Scenario,
    sample: usize,
    control: &ControlResult,
    profiled: &ProfileResult,
) {
    let fields = vec![
        "record".to_owned(),
        "scenario".to_owned(),
        corpus.name.to_owned(),
        scenario.name.to_owned(),
        scenario.d.to_string(),
        sample.to_string(),
        "0".to_owned(),
        control.outcome.as_str().to_owned(),
        control.final_k.to_string(),
        control.attempts.to_string(),
        control.attempts.to_string(),
        control.payload_bytes.to_string(),
        control.elapsed_ns.to_string(),
        profiled.elapsed_ns.to_string(),
        profiled.prefix_ns.to_string(),
        profiled.locator_ns.to_string(),
        profiled.decode_wall_ns.to_string(),
        profiled.profile.validation_ns.to_string(),
        profiled.profile.factor_wall_ns.to_string(),
        profiled.profile.verification_ns.to_string(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        u8::from(control.outcome == Outcome::FalseSuccess).to_string(),
    ];
    println!("{}", fields.join(","));
}

fn emit_degree(
    corpus: Corpus,
    scenario: Scenario,
    sample: usize,
    degree: usize,
    control: &ControlResult,
    profiled: &ProfileResult,
) {
    let fields = vec![
        "record".to_owned(),
        "degree".to_owned(),
        corpus.name.to_owned(),
        scenario.name.to_owned(),
        scenario.d.to_string(),
        sample.to_string(),
        degree.to_string(),
        control.outcome.as_str().to_owned(),
        control.final_k.to_string(),
        control.attempts.to_string(),
        control.attempts.to_string(),
        control.payload_bytes.to_string(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        profiled.profile.factor_calls[degree].to_string(),
        profiled.profile.plan_build_calls[degree].to_string(),
        profiled.profile.trace_attempts[degree].to_string(),
        profiled.profile.quadratic_calls[degree].to_string(),
        profiled.profile.self_ns[degree].to_string(),
        profiled.profile.plan_build_ns[degree].to_string(),
        profiled.profile.quadratic_ns[degree].to_string(),
        profiled.profile.trace_ns[degree].to_string(),
        profiled.profile.gcd_ns[degree].to_string(),
        profiled.profile.division_ns[degree].to_string(),
        u8::from(control.outcome == Outcome::FalseSuccess).to_string(),
    ];
    println!("{}", fields.join(","));
}

fn validate_final(outcome: Outcome, final_k: usize, expected: &[u64]) {
    let expected_outcome = if expected.len() <= 8 {
        Outcome::Exact
    } else {
        Outcome::Rejected
    };
    assert_eq!(outcome, expected_outcome);

    let expected_k = STAGES
        .iter()
        .find(|&&(limit, _)| expected.len() <= limit)
        .map_or(8, |&(limit, _)| limit);
    assert_eq!(final_k, expected_k);
}

fn classify(candidate: Option<&Vec<u64>>, expected: &[u64], within_limit: bool) -> Outcome {
    match candidate {
        Some(candidate) if candidate.as_slice() == expected && within_limit => Outcome::Exact,
        Some(_) => Outcome::FalseSuccess,
        None => Outcome::Rejected,
    }
}

fn difference_prefix(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    capacity: usize,
) -> PinSketch64Lab {
    let mut difference = prefix(left, capacity);
    difference.merge(&prefix(right, capacity)).unwrap();
    difference
}

fn prefix(source: &PinSketch64Lab, capacity: usize) -> PinSketch64Lab {
    let mut bytes = source.encode();
    let expected = PinSketch64Lab::encoded_len_for_capacity(capacity).unwrap();
    bytes[8..10].copy_from_slice(&(capacity as u16).to_le_bytes());
    bytes.truncate(expected);
    PinSketch64Lab::decode(&bytes).unwrap()
}

const fn prefix_payload_bytes(stored_capacity: usize) -> usize {
    1 + stored_capacity * 8
}

fn derive_source(
    base: &[u64],
    difference: usize,
    salt: u64,
    add_mask: u64,
    include_zero: bool,
) -> Vec<u64> {
    if difference == 0 {
        return base.to_vec();
    }
    if include_zero {
        assert_eq!(difference, 1);
        let mut values = base.to_vec();
        values.push(0);
        values.sort_unstable();
        return values;
    }

    let remove = difference / 2;
    let add = difference - remove;
    let mut values = base[remove..].to_vec();
    for index in 0..add {
        values.push(splitmix64(salt ^ add_mask ^ index as u64));
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn symmetric_difference(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                result.push(left[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(right[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}

fn canonical_keys(count: usize, salt: u64) -> Vec<u64> {
    let mut keys: Vec<_> = (0..count)
        .map(|index| {
            let mut key = splitmix64(salt ^ index as u64);
            if key == 0 {
                key = u64::MAX;
            }
            key
        })
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), count);
    keys
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
