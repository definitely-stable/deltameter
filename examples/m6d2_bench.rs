//! M6-D2 diagnostic guarded incremental prefix experiment.
//!
//! Private research harness only. This is not a network protocol or public API.

#[path = "support/m6d2_prefix.rs"]
mod prefix_lab;

use std::hint::black_box;
use std::time::Instant;

use prefix_lab::{LabError, PinSketch64Lab, extend_prefix, prefix, prefix_payload_bytes};

const SOURCE_KEYS: usize = 8_192;
const SAMPLES: usize = 3;
const DIRECT_REPEATS: u64 = 128;
const STAGES: [(usize, usize); 4] = [(1, 2), (2, 3), (4, 5), (8, 9)];

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    d: usize,
    include_zero: bool,
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

#[derive(Debug, Clone, Copy)]
struct ProtocolResult {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    decode_ns: u128,
    extension_ns: u128,
    merge_ns: u128,
}

fn main() {
    println!("format=deltameter.m6d2-prefix.v1");
    println!("contract=private_lab_application_bytes_no_framing");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("direct_repeats={DIRECT_REPEATS}");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("first_zero_metadata_bytes=1");
    println!("syndrome_word_bytes=8");
    println!(
        "result,arm,scenario,d,source_n,outcome,final_k,attempts,rtts,payload_bytes,decode_ns,extension_ns,merge_ns,cold_build_ns,direct_diff_ns,false_success"
    );

    let left = canonical_keys(SOURCE_KEYS, 0xD200_BA5E_0000_0001);
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

    for scenario in scenarios {
        let right = derive_source(
            &left,
            scenario.d,
            0xD200_D1FF_0000_0000 ^ scenario.d as u64,
            scenario.include_zero,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), scenario.d, "scenario={}", scenario.name);

        for sample in 0..SAMPLES {
            let build_started = Instant::now();
            let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
            let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
            let cold_build_ns = build_started.elapsed().as_nanos();

            let direct_started = Instant::now();
            for _ in 0..DIRECT_REPEATS {
                black_box(symmetric_difference(black_box(&left), black_box(&right)));
            }
            let direct_diff_ns = direct_started.elapsed().as_nanos() / u128::from(DIRECT_REPEATS);
            let direct_bytes = 4 + right.len() * 8;

            emit(
                "direct-exact",
                scenario,
                right.len(),
                Outcome::Exact,
                0,
                1,
                direct_bytes,
                0,
                0,
                0,
                0,
                direct_diff_ns,
            );

            let fixed = run_fixed(&left_full, &right_full, &expected);
            emit_protocol(
                "fixed-guarded",
                scenario,
                right.len(),
                fixed,
                cold_build_ns,
                direct_diff_ns,
            );

            let incremental = run_incremental(&left_full, &right_full, &expected);
            emit_protocol(
                "incremental-prefix",
                scenario,
                right.len(),
                incremental,
                cold_build_ns,
                direct_diff_ns,
            );

            let naive_bytes = naive_resend_bytes(incremental.attempts);
            let naive = ProtocolResult {
                payload_bytes: naive_bytes,
                ..incremental
            };
            emit_protocol(
                "naive-resend",
                scenario,
                right.len(),
                naive,
                cold_build_ns,
                direct_diff_ns,
            );

            black_box(sample);
        }
    }
}

fn run_fixed(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
) -> ProtocolResult {
    let exact_d = expected.len();
    let (limit, stored_capacity) = fixed_stage(exact_d);

    let merge_started = Instant::now();
    let mut difference = prefix(left_full, stored_capacity).unwrap();
    difference
        .merge(&prefix(right_full, stored_capacity).unwrap())
        .unwrap();
    let merge_ns = merge_started.elapsed().as_nanos();

    let decode_started = Instant::now();
    let decoded = difference.decode_candidate_with_limit(limit);
    let decode_ns = decode_started.elapsed().as_nanos();
    let outcome = classify(decoded, expected, exact_d <= limit);

    ProtocolResult {
        outcome,
        final_k: limit,
        attempts: 1,
        payload_bytes: prefix_payload_bytes(stored_capacity),
        decode_ns,
        extension_ns: 0,
        merge_ns,
    }
}

fn run_incremental(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
) -> ProtocolResult {
    let mut remote = prefix(right_full, STAGES[0].1).unwrap();
    let mut payload_bytes = prefix_payload_bytes(STAGES[0].1);
    let mut attempts = 0_usize;
    let mut decode_ns = 0_u128;
    let mut extension_ns = 0_u128;
    let mut merge_ns = 0_u128;

    for (stage_index, (limit, stored_capacity)) in STAGES.into_iter().enumerate() {
        if stage_index != 0 {
            let extend_started = Instant::now();
            let added = extend_prefix(&mut remote, right_full, stored_capacity).unwrap();
            extension_ns += extend_started.elapsed().as_nanos();
            payload_bytes += added * 8;
        }

        let merge_started = Instant::now();
        let mut difference = prefix(left_full, stored_capacity).unwrap();
        difference.merge(&remote).unwrap();
        merge_ns += merge_started.elapsed().as_nanos();

        attempts += 1;
        let decode_started = Instant::now();
        let decoded = difference.decode_candidate_with_limit(limit);
        decode_ns += decode_started.elapsed().as_nanos();

        match classify(decoded, expected, expected.len() <= limit) {
            Outcome::Exact => {
                return ProtocolResult {
                    outcome: Outcome::Exact,
                    final_k: limit,
                    attempts,
                    payload_bytes,
                    decode_ns,
                    extension_ns,
                    merge_ns,
                };
            }
            Outcome::FalseSuccess => {
                return ProtocolResult {
                    outcome: Outcome::FalseSuccess,
                    final_k: limit,
                    attempts,
                    payload_bytes,
                    decode_ns,
                    extension_ns,
                    merge_ns,
                };
            }
            Outcome::Rejected => {}
        }
    }

    ProtocolResult {
        outcome: Outcome::Rejected,
        final_k: STAGES.last().unwrap().0,
        attempts,
        payload_bytes,
        decode_ns,
        extension_ns,
        merge_ns,
    }
}

fn classify(
    decoded: Result<Vec<u64>, LabError>,
    expected: &[u64],
    within_limit: bool,
) -> Outcome {
    match decoded {
        Ok(candidate) if candidate == expected && within_limit => Outcome::Exact,
        Ok(_) => Outcome::FalseSuccess,
        Err(_) => Outcome::Rejected,
    }
}

fn fixed_stage(d: usize) -> (usize, usize) {
    STAGES
        .into_iter()
        .find(|(limit, _)| d <= *limit)
        .unwrap_or_else(|| *STAGES.last().unwrap())
}

fn naive_resend_bytes(attempts: usize) -> usize {
    STAGES
        .iter()
        .take(attempts)
        .map(|(_, stored_capacity)| prefix_payload_bytes(*stored_capacity))
        .sum()
}

#[allow(clippy::too_many_arguments)]
fn emit(
    arm: &str,
    scenario: Scenario,
    source_n: usize,
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    decode_ns: u128,
    extension_ns: u128,
    merge_ns: u128,
    cold_build_ns: u128,
    direct_diff_ns: u128,
) {
    println!(
        "result,{arm},{},{},{source_n},{},{final_k},{attempts},{attempts},{payload_bytes},{decode_ns},{extension_ns},{merge_ns},{cold_build_ns},{direct_diff_ns},{}",
        scenario.name,
        scenario.d,
        outcome.as_str(),
        u8::from(outcome == Outcome::FalseSuccess)
    );
}

fn emit_protocol(
    arm: &str,
    scenario: Scenario,
    source_n: usize,
    result: ProtocolResult,
    cold_build_ns: u128,
    direct_diff_ns: u128,
) {
    emit(
        arm,
        scenario,
        source_n,
        result.outcome,
        result.final_k,
        result.attempts,
        result.payload_bytes,
        result.decode_ns,
        result.extension_ns,
        result.merge_ns,
        cold_build_ns,
        direct_diff_ns,
    );
}

fn derive_source(base: &[u64], difference: usize, salt: u64, include_zero: bool) -> Vec<u64> {
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
        values.push(splitmix64(salt ^ 0xA5A5_5A5A_0000_0000 ^ index as u64));
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
