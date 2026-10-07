//! M6-D4 hosted diagnostic: generic versus characteristic-2 trace squaring.
//!
//! Private research harness only. The D2 protocol and locator computation are
//! invariant controls; only root-factor trace squaring differs.

#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;

use std::hint::black_box;
use std::time::Instant;

use pinsketch64::PinSketch64Lab;
use trace_square::{
    D4Error, decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
    generic_square_mod, poly_square_mod_monic,
};

const SOURCE_KEYS: usize = 8_192;
const SAMPLES: usize = 4;
const SQUARE_REPEATS: u64 = 128;
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
struct DecodeMetrics {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    locator_ns: u128,
    generic_ns: u128,
    specialized_ns: u128,
}

fn main() {
    println!("format=deltameter.m6d4-trace-square.v1");
    println!("contract=frozen_m6d2_protocol_root_trace_square_only");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("square_repeats={SQUARE_REPEATS}");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("payloads=17;25;41;73");
    println!(
        "record,kind,scenario,d,degree,outcome,operations,final_k,attempts,rtts,payload_bytes,generic_ns,specialized_ns,locator_ns,false_success"
    );

    run_square_micro();

    let left = canonical_keys(SOURCE_KEYS, 0xD400_BA5E_0000_0001);
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
            0xD400_D1FF_0000_0000 ^ scenario.d as u64,
            scenario.include_zero,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), scenario.d);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
        assert_eq!(right.len(), SOURCE_KEYS + scenario.d % 2);
        validate_control(&left_full, &right_full, &expected);

        // Untimed warmup of both complete paths, then balanced AB/BA pairs.
        run_total(&left_full, &right_full, &expected, true);
        run_total(&left_full, &right_full, &expected, false);
        for sample in 0..SAMPLES {
            let metrics = run_decode(&left_full, &right_full, &expected, sample % 2 == 0);

            println!(
                "record,decode,{},{},0,{},{},{},{},{},{},{},{},{},{}",
                scenario.name,
                scenario.d,
                metrics.outcome.as_str(),
                1,
                metrics.final_k,
                metrics.attempts,
                metrics.attempts,
                metrics.payload_bytes,
                metrics.generic_ns,
                metrics.specialized_ns,
                metrics.locator_ns,
                u8::from(metrics.outcome == Outcome::FalseSuccess)
            );

            let (generic_total, specialized_total) = if sample % 2 == 0 {
                (
                    run_total(&left_full, &right_full, &expected, true),
                    run_total(&left_full, &right_full, &expected, false),
                )
            } else {
                let specialized = run_total(&left_full, &right_full, &expected, false);
                let generic = run_total(&left_full, &right_full, &expected, true);
                (generic, specialized)
            };
            println!(
                "record,total,{},{},0,{},{},{},{},{},{},{},{},0,0",
                scenario.name,
                scenario.d,
                metrics.outcome.as_str(),
                1,
                metrics.final_k,
                metrics.attempts,
                metrics.attempts,
                metrics.payload_bytes,
                generic_total,
                specialized_total,
            );
        }
    }
}

// Independent frozen decoder validation is isolated from every timed region.
// This is an actual control on the measured corpus, not a dead-code keepalive.
fn validate_control(left: &PinSketch64Lab, right: &PinSketch64Lab, expected: &[u64]) {
    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(left, right, capacity);
        let frozen = sketch.decode_candidate_with_limit(limit);
        let locator = fresh_locator(&sketch, limit).unwrap();
        let generic = decode_with_locator_generic(&sketch, limit, &locator);
        let specialized = decode_with_locator_specialized(&sketch, limit, &locator);
        assert_eq!(generic, frozen.clone().map_err(D4Error::from));
        assert_eq!(specialized, frozen.clone().map_err(D4Error::from));
        if expected.len() <= limit {
            assert_eq!(frozen.unwrap(), expected);
        } else {
            assert!(frozen.is_err());
        }
    }
}

// Separately measured cumulative decoder: prefix extraction/merge, fresh
// locator/BM, root factorization, guard verification and retry loop. Source
// construction, exact oracle comparison and network transport are excluded.
fn run_total(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    expected: &[u64],
    generic: bool,
) -> u128 {
    let started = Instant::now();
    let mut candidate = None;
    let mut final_k = 8;
    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(black_box(left), black_box(right), capacity);
        let locator = fresh_locator(black_box(&sketch), limit).unwrap();
        let result = if generic {
            decode_with_locator_generic(black_box(&sketch), limit, black_box(&locator))
        } else {
            decode_with_locator_specialized(black_box(&sketch), limit, black_box(&locator))
        };
        if let Ok(roots) = result {
            candidate = Some(roots);
            final_k = limit;
            break;
        }
    }
    black_box(&candidate);
    let elapsed = started.elapsed().as_nanos();
    let outcome = classify(candidate.as_ref(), expected, expected.len() <= final_k);
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
    elapsed
}

fn run_square_micro() {
    for degree in [2_usize, 4, 8] {
        let modulus = square_modulus(degree);
        let polynomial = square_polynomial(degree);

        assert_eq!(
            generic_square_mod(&polynomial, &modulus).unwrap(),
            poly_square_mod_monic(&polynomial, &modulus).unwrap()
        );

        for sample in 0..SAMPLES {
            let (generic_ns, specialized_ns) = if sample % 2 == 0 {
                (
                    time_repeated(SQUARE_REPEATS, || {
                        black_box(
                            generic_square_mod(black_box(&polynomial), black_box(&modulus))
                                .unwrap(),
                        );
                    }),
                    time_repeated(SQUARE_REPEATS, || {
                        black_box(
                            poly_square_mod_monic(black_box(&polynomial), black_box(&modulus))
                                .unwrap(),
                        );
                    }),
                )
            } else {
                let specialized = time_repeated(SQUARE_REPEATS, || {
                    black_box(
                        poly_square_mod_monic(black_box(&polynomial), black_box(&modulus)).unwrap(),
                    );
                });
                let generic = time_repeated(SQUARE_REPEATS, || {
                    black_box(
                        generic_square_mod(black_box(&polynomial), black_box(&modulus)).unwrap(),
                    );
                });
                (generic, specialized)
            };

            println!(
                "record,square,degree{degree},0,{degree},na,{SQUARE_REPEATS},0,0,0,0,{generic_ns},{specialized_ns},0,0"
            );
        }
    }
}

fn run_decode(
    left_full: &PinSketch64Lab,
    right_full: &PinSketch64Lab,
    expected: &[u64],
    generic_first: bool,
) -> DecodeMetrics {
    let mut metrics = DecodeMetrics {
        outcome: Outcome::Rejected,
        final_k: 8,
        attempts: 0,
        payload_bytes: 73,
        locator_ns: 0,
        generic_ns: 0,
        specialized_ns: 0,
    };

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(left_full, right_full, capacity);
        metrics.attempts += 1;
        metrics.final_k = limit;
        metrics.payload_bytes = payload_bytes(capacity);

        let locator_started = Instant::now();
        let locator = fresh_locator(black_box(&sketch), limit).unwrap();
        metrics.locator_ns += locator_started.elapsed().as_nanos();

        let (generic, specialized, generic_ns, specialized_ns) = if generic_first {
            let generic_started = Instant::now();
            let generic =
                decode_with_locator_generic(black_box(&sketch), limit, black_box(&locator));
            let generic_ns = generic_started.elapsed().as_nanos();

            let specialized_started = Instant::now();
            let specialized =
                decode_with_locator_specialized(black_box(&sketch), limit, black_box(&locator));
            let specialized_ns = specialized_started.elapsed().as_nanos();

            (generic, specialized, generic_ns, specialized_ns)
        } else {
            let specialized_started = Instant::now();
            let specialized =
                decode_with_locator_specialized(black_box(&sketch), limit, black_box(&locator));
            let specialized_ns = specialized_started.elapsed().as_nanos();

            let generic_started = Instant::now();
            let generic =
                decode_with_locator_generic(black_box(&sketch), limit, black_box(&locator));
            let generic_ns = generic_started.elapsed().as_nanos();

            (generic, specialized, generic_ns, specialized_ns)
        };

        metrics.generic_ns += generic_ns;
        metrics.specialized_ns += specialized_ns;

        let generic_outcome = classify(generic.as_ref().ok(), expected, expected.len() <= limit);
        let specialized_outcome =
            classify(specialized.as_ref().ok(), expected, expected.len() <= limit);

        assert_eq!(
            generic_outcome,
            specialized_outcome,
            "root outcome mismatch d={} limit={limit}",
            expected.len()
        );
        assert_eq!(
            generic.as_ref().ok(),
            specialized.as_ref().ok(),
            "root candidate mismatch d={} limit={limit}",
            expected.len()
        );

        match generic_outcome {
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

fn square_modulus(degree: usize) -> Vec<u64> {
    let mut modulus = Vec::with_capacity(degree + 1);
    for index in 0..degree {
        modulus.push(splitmix64(
            0xD400_6000_0000_0000 ^ (degree as u64) << 48 ^ index as u64,
        ));
    }
    modulus.push(1);
    modulus
}

fn square_polynomial(degree: usize) -> Vec<u64> {
    (0..degree)
        .map(|index| splitmix64(0xD400_7000_0000_0000 ^ (degree as u64) << 48 ^ index as u64))
        .collect()
}

fn time_repeated(mut operations: u64, mut operation: impl FnMut()) -> u128 {
    let started = Instant::now();
    while operations != 0 {
        operation();
        operations -= 1;
    }
    started.elapsed().as_nanos()
}

const fn payload_bytes(capacity: usize) -> usize {
    1 + capacity * 8
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
        values.push(splitmix64(salt ^ 0xD400_5A5A_0000_0000 ^ index as u64));
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
