//! M6-D8 hosted diagnostic: accepted D6 versus scalar-square candidate.
//!
//! Private research only. D8 changes only coefficient squaring inside the
//! degree>=3 trace square/mod path.

#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "support/m6d8_square.rs"]
mod square_candidate;
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;

use std::hint::black_box;
use std::time::Instant;

use pinsketch64::PinSketch64Lab;
use quadratic::{decode_with_locator_quadratic, gf64_square_reference_for_d8};
use square_candidate::{decode_with_locator_square_candidate, gf64_square_candidate};
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
struct DecodeResult {
    outcome: Outcome,
    final_k: usize,
    attempts: usize,
    payload_bytes: usize,
    candidate: Option<Vec<u64>>,
    elapsed_ns: u128,
}

fn main() {
    println!("format=deltameter.m6d8-square.v1");
    println!("contract=frozen_m6d7_square_only");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("corpora=d4;d5;d6;d7a;d7b");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("payloads=17;25;41;73");
    println!(
        "record,corpus,scenario,d,sample,outcome,final_k,attempts,rtts,payload_bytes,control_ns,candidate_ns,false_success"
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

            black_box(run_complete(&left_full, &right_full, &expected, false));
            black_box(run_complete(&left_full, &right_full, &expected, true));

            for sample in 0..SAMPLES {
                let (control, candidate) = if sample % 2 == 0 {
                    (
                        run_complete(&left_full, &right_full, &expected, false),
                        run_complete(&left_full, &right_full, &expected, true),
                    )
                } else {
                    let candidate = run_complete(&left_full, &right_full, &expected, true);
                    let control = run_complete(&left_full, &right_full, &expected, false);
                    (control, candidate)
                };

                assert_eq!(candidate.outcome, control.outcome);
                assert_eq!(candidate.final_k, control.final_k);
                assert_eq!(candidate.attempts, control.attempts);
                assert_eq!(candidate.payload_bytes, control.payload_bytes);
                assert_eq!(candidate.candidate, control.candidate);

                println!(
                    "record,{},{},{},{},{},{},{},{},{},{},{},{}",
                    corpus.name,
                    scenario.name,
                    scenario.d,
                    sample,
                    control.outcome.as_str(),
                    control.final_k,
                    control.attempts,
                    control.attempts,
                    control.payload_bytes,
                    control.elapsed_ns,
                    candidate.elapsed_ns,
                    u8::from(control.outcome == Outcome::FalseSuccess)
                );
            }
        }
    }
}

fn validate_arithmetic_controls() {
    for bit in 0..64 {
        let value = 1_u64 << bit;
        assert_eq!(
            gf64_square_candidate(value),
            gf64_square_reference_for_d8(value)
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
        let control = decode_with_locator_quadratic(&sketch, limit, &locator);
        let candidate = decode_with_locator_square_candidate(&sketch, limit, &locator);

        assert_eq!(generic, frozen.clone().map_err(D4Error::from));
        assert_eq!(d4, frozen.clone().map_err(D4Error::from));
        assert_eq!(control, d4);
        assert_eq!(candidate, control);

        if expected.len() <= limit {
            assert_eq!(frozen.unwrap(), expected);
        } else {
            assert!(frozen.is_err());
        }
    }
}

fn run_complete(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    expected: &[u64],
    use_candidate: bool,
) -> DecodeResult {
    let started = Instant::now();
    let mut candidate = None;
    let mut final_k = 8;
    let mut attempts = 0;
    let mut payload_bytes = 73;

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(black_box(left), black_box(right), capacity);
        let locator = fresh_locator(black_box(&sketch), limit).unwrap();
        let result = if use_candidate {
            decode_with_locator_square_candidate(black_box(&sketch), limit, black_box(&locator))
        } else {
            decode_with_locator_quadratic(black_box(&sketch), limit, black_box(&locator))
        };

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

    DecodeResult {
        outcome,
        final_k,
        attempts,
        payload_bytes,
        candidate,
        elapsed_ns,
    }
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
