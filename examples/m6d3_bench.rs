//! M6-D3 hosted diagnostic: incremental BM reuse versus frozen D2 decoder.
//!
//! Private research harness only. D2 communication bytes/RTTs are invariant
//! controls; only decoder computation is under test.

#[path = "support/m6d3_incremental_bm.rs"]
mod incremental_bm;
#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;

use std::hint::black_box;
use std::time::Instant;

use incremental_bm::{
    IncrementalBmDecoder, decode_with_locator, fresh_connection_from_sequence, fresh_full_sequence,
    prefix,
};
use pinsketch64::{LabError, PinSketch64Lab};

const SOURCE_KEYS: usize = 8_192;
const SAMPLES: usize = 3;
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
struct Metrics {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    merge_ns: u128,
    reference_decode_ns: u128,
    incremental_bm_ns: u128,
    factor_verify_ns: u128,
    fresh_sequence_ns: u128,
    fresh_bm_ns: u128,
}

fn main() {
    println!("format=deltameter.m6d3-incremental-bm.v1");
    println!("contract=frozen_m6d2_bytes_rtts_decoder_cpu_only");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("payloads=17;25;41;73");
    println!(
        "result,arm,scenario,d,outcome,final_k,attempts,rtts,payload_bytes,merge_ns,reference_decode_ns,incremental_bm_ns,factor_verify_ns,fresh_sequence_ns,fresh_bm_ns,candidate_decode_ns,cold_build_ns,false_success"
    );

    let left = canonical_keys(SOURCE_KEYS, 0xD300_BA5E_0000_0001);
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
            0xD300_D1FF_0000_0000 ^ scenario.d as u64,
            scenario.include_zero,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), scenario.d);

        for sample in 0..SAMPLES {
            let build_started = Instant::now();
            let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
            let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
            let cold_build_ns = build_started.elapsed().as_nanos();

            let fixed = run_fixed_reference(&left_full, &right_full, &expected);
            emit("fixed-reference", scenario, fixed, cold_build_ns);

            let incremental_reference =
                run_incremental_reference(&left_full, &right_full, &expected);
            emit(
                "incremental-reference",
                scenario,
                incremental_reference,
                cold_build_ns,
            );

            let incremental_bm = run_incremental_bm(&left_full, &right_full, &expected);
            emit("incremental-bm", scenario, incremental_bm, cold_build_ns);

            assert_eq!(incremental_reference.outcome, incremental_bm.outcome);
            assert_eq!(incremental_reference.final_k, incremental_bm.final_k);
            assert_eq!(incremental_reference.attempts, incremental_bm.attempts);
            assert_eq!(
                incremental_reference.payload_bytes,
                incremental_bm.payload_bytes
            );

            black_box(sample);
        }
    }
}

fn run_fixed_reference(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
) -> Metrics {
    let (limit, stored_capacity) = fixed_stage(expected.len());

    let merge_started = Instant::now();
    let sketch = difference_prefix(left_full, right_full, stored_capacity);
    let merge_ns = merge_started.elapsed().as_nanos();

    let decode_started = Instant::now();
    let decoded = sketch.decode_candidate_with_limit(limit);
    let reference_decode_ns = decode_started.elapsed().as_nanos();
    let outcome = classify_reference(decoded, expected, expected.len() <= limit);

    Metrics {
        outcome,
        final_k: limit,
        attempts: 1,
        payload_bytes: payload_bytes(stored_capacity),
        merge_ns,
        reference_decode_ns,
        incremental_bm_ns: 0,
        factor_verify_ns: 0,
        fresh_sequence_ns: 0,
        fresh_bm_ns: 0,
    }
}

fn run_incremental_reference(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
) -> Metrics {
    let mut metrics = Metrics {
        outcome: Outcome::Rejected,
        final_k: 8,
        attempts: 0,
        payload_bytes: 73,
        merge_ns: 0,
        reference_decode_ns: 0,
        incremental_bm_ns: 0,
        factor_verify_ns: 0,
        fresh_sequence_ns: 0,
        fresh_bm_ns: 0,
    };

    for (limit, stored_capacity) in STAGES {
        let merge_started = Instant::now();
        let sketch = difference_prefix(left_full, right_full, stored_capacity);
        metrics.merge_ns += merge_started.elapsed().as_nanos();
        metrics.attempts += 1;
        metrics.final_k = limit;
        metrics.payload_bytes = payload_bytes(stored_capacity);

        let sequence_started = Instant::now();
        let sequence = fresh_full_sequence(&sketch, limit).unwrap();
        metrics.fresh_sequence_ns += sequence_started.elapsed().as_nanos();

        let bm_started = Instant::now();
        black_box(fresh_connection_from_sequence(black_box(&sequence)).unwrap());
        metrics.fresh_bm_ns += bm_started.elapsed().as_nanos();

        let decode_started = Instant::now();
        let decoded = sketch.decode_candidate_with_limit(limit);
        metrics.reference_decode_ns += decode_started.elapsed().as_nanos();

        match classify_reference(decoded, expected, expected.len() <= limit) {
            Outcome::Exact => {
                metrics.outcome = Outcome::Exact;
                break;
            }
            Outcome::FalseSuccess => {
                metrics.outcome = Outcome::FalseSuccess;
                break;
            }
            Outcome::Rejected => {}
        }
    }

    metrics
}

fn run_incremental_bm(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
) -> Metrics {
    let mut decoder = IncrementalBmDecoder::new();
    let mut metrics = Metrics {
        outcome: Outcome::Rejected,
        final_k: 8,
        attempts: 0,
        payload_bytes: 73,
        merge_ns: 0,
        reference_decode_ns: 0,
        incremental_bm_ns: 0,
        factor_verify_ns: 0,
        fresh_sequence_ns: 0,
        fresh_bm_ns: 0,
    };

    for (limit, stored_capacity) in STAGES {
        let merge_started = Instant::now();
        let sketch = difference_prefix(left_full, right_full, stored_capacity);
        metrics.merge_ns += merge_started.elapsed().as_nanos();
        metrics.attempts += 1;
        metrics.final_k = limit;
        metrics.payload_bytes = payload_bytes(stored_capacity);

        let bm_started = Instant::now();
        decoder.extend_to(black_box(&sketch), limit).unwrap();
        metrics.incremental_bm_ns += bm_started.elapsed().as_nanos();
        black_box(decoder.sequence().len());
        black_box(decoder.linear_complexity());

        let locator = decoder.connection_polynomial();
        let factor_started = Instant::now();
        let decoded = decode_with_locator(&sketch, limit, black_box(&locator));
        metrics.factor_verify_ns += factor_started.elapsed().as_nanos();

        match classify_d3(decoded, expected, expected.len() <= limit) {
            Outcome::Exact => {
                metrics.outcome = Outcome::Exact;
                break;
            }
            Outcome::FalseSuccess => {
                metrics.outcome = Outcome::FalseSuccess;
                break;
            }
            Outcome::Rejected => {}
        }
    }

    metrics
}

fn difference_prefix(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    stored_capacity: usize,
) -> PinSketch64Lab {
    let mut sketch = prefix(left, stored_capacity).unwrap();
    sketch
        .merge(&prefix(right, stored_capacity).unwrap())
        .unwrap();
    sketch
}

fn classify_reference(
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

fn classify_d3(
    decoded: Result<Vec<u64>, incremental_bm::D3Error>,
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

const fn payload_bytes(stored_capacity: usize) -> usize {
    1 + stored_capacity * 8
}

fn emit(arm: &str, scenario: Scenario, metrics: Metrics, cold_build_ns: u128) {
    let candidate_decode_ns = metrics.incremental_bm_ns + metrics.factor_verify_ns;
    println!(
        "result,{arm},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        scenario.name,
        scenario.d,
        metrics.outcome.as_str(),
        metrics.final_k,
        metrics.attempts,
        metrics.attempts,
        metrics.payload_bytes,
        metrics.merge_ns,
        metrics.reference_decode_ns,
        metrics.incremental_bm_ns,
        metrics.factor_verify_ns,
        metrics.fresh_sequence_ns,
        metrics.fresh_bm_ns,
        candidate_decode_ns,
        cold_build_ns,
        u8::from(metrics.outcome == Outcome::FalseSuccess)
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
        values.push(splitmix64(salt ^ 0xD300_5A5A_0000_0000 ^ index as u64));
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
