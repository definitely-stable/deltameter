//! M6-D10 hosted trace-internal replay measurement.
//!
//! Measurement only. Accepted D8 remains the decoder control. Timing occurs only
//! after exact trace operands have been collected.

#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "support/m6d8_square.rs"]
mod square_candidate;
#[path = "support/m6d10_trace_internal.rs"]
mod trace_internal;
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;

use std::hint::black_box;
use std::time::Instant;

use pinsketch64::PinSketch64Lab;
use quadratic::decode_with_locator_quadratic;
use square_candidate::decode_with_locator_square_candidate;
use trace_internal::{
    TraceReplay, decode_with_locator_collect, full_square_mod_replay, full_trace_replay,
    prepare_modulus_replay, reduce_unreduced_replay, square_unreduced_replay,
    trace_accumulate_replay,
};
use trace_square::{
    D4Error, decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
    generic_square_mod, poly_square_mod_monic,
};

const SOURCE_KEYS: usize = 8_192;
const SAMPLES: usize = 4;
const REPEATS: usize = 8;
const STAGES: [(usize, usize); 4] = [(1, 2), (2, 3), (4, 5), (8, 9)];

#[derive(Clone, Copy)]
struct Corpus {
    name: &'static str,
    left_salt: u64,
    right_salt: u64,
    add_mask: u64,
}

struct ReplayDataset {
    replays: Vec<TraceReplay>,
    prepared_moduli: Vec<Vec<u64>>,
    unreduced: Vec<Vec<Vec<u64>>>,
}

#[derive(Clone, Copy)]
enum Component {
    FullTrace,
    FullSquareMod,
    Prepare,
    SquareBuild,
    ReductionWithCopy,
    CloneOnly,
    TraceAccumulate,
}

#[derive(Default)]
struct Timings {
    full_trace_ns: u128,
    full_square_mod_ns: u128,
    prepare_ns: u128,
    square_build_ns: u128,
    reduction_with_copy_ns: u128,
    clone_ns: u128,
    trace_accumulate_ns: u128,
}

fn main() {
    println!("format=deltameter.m6d10-trace-internal.v1");
    println!("contract=frozen_m6d8_offline_replay");
    println!("source_keys={SOURCE_KEYS}");
    println!("samples={SAMPLES}");
    println!("repeats={REPEATS}");
    println!("corpora=d4;d5;d6;d7a;d7b");
    println!("scenario=d8");
    println!("schedule=1:2;2:3;4:5;8:9");
    println!("payload_bytes=73");
    println!(
        "record,corpus,sample,trace_attempts,term_cases,repeats,full_trace_ns,full_square_mod_ns,prepare_ns,square_build_ns,reduction_with_copy_ns,clone_ns,trace_accumulate_ns"
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

    for corpus in corpora {
        let dataset = build_dataset(corpus);
        let trace_attempts = dataset.replays.len();
        let term_cases: usize = dataset.replays.iter().map(|replay| replay.terms.len()).sum();

        assert!(trace_attempts > 0);
        assert_eq!(term_cases, trace_attempts * 64);

        // Warm every component once before measured samples.
        for component in all_components() {
            black_box(measure_component(component, &dataset));
        }

        for sample in 0..SAMPLES {
            let mut timings = Timings::default();
            let order = if sample % 2 == 0 {
                all_components()
            } else {
                let mut order = all_components();
                order.reverse();
                order
            };

            for component in order {
                let elapsed = measure_component(component, &dataset);
                match component {
                    Component::FullTrace => timings.full_trace_ns = elapsed,
                    Component::FullSquareMod => timings.full_square_mod_ns = elapsed,
                    Component::Prepare => timings.prepare_ns = elapsed,
                    Component::SquareBuild => timings.square_build_ns = elapsed,
                    Component::ReductionWithCopy => timings.reduction_with_copy_ns = elapsed,
                    Component::CloneOnly => timings.clone_ns = elapsed,
                    Component::TraceAccumulate => timings.trace_accumulate_ns = elapsed,
                }
            }

            assert!(timings.reduction_with_copy_ns > timings.clone_ns);

            println!(
                "record,{},{},{},{},{},{},{},{},{},{},{},{}",
                corpus.name,
                sample,
                trace_attempts,
                term_cases,
                REPEATS,
                timings.full_trace_ns,
                timings.full_square_mod_ns,
                timings.prepare_ns,
                timings.square_build_ns,
                timings.reduction_with_copy_ns,
                timings.clone_ns,
                timings.trace_accumulate_ns
            );
        }
    }
}

fn all_components() -> Vec<Component> {
    vec![
        Component::FullTrace,
        Component::FullSquareMod,
        Component::Prepare,
        Component::SquareBuild,
        Component::ReductionWithCopy,
        Component::CloneOnly,
        Component::TraceAccumulate,
    ]
}

fn measure_component(component: Component, dataset: &ReplayDataset) -> u128 {
    let started = Instant::now();

    for _ in 0..REPEATS {
        match component {
            Component::FullTrace => {
                for replay in &dataset.replays {
                    black_box(full_trace_replay(black_box(replay)).unwrap());
                }
            }
            Component::FullSquareMod => {
                for replay in &dataset.replays {
                    for term in &replay.terms {
                        black_box(
                            full_square_mod_replay(black_box(term), black_box(&replay.modulus))
                                .unwrap(),
                        );
                    }
                }
            }
            Component::Prepare => {
                for replay in &dataset.replays {
                    for _ in &replay.terms {
                        black_box(prepare_modulus_replay(black_box(&replay.modulus)).unwrap());
                    }
                }
            }
            Component::SquareBuild => {
                for replay in &dataset.replays {
                    for term in &replay.terms {
                        black_box(square_unreduced_replay(black_box(term)).unwrap());
                    }
                }
            }
            Component::ReductionWithCopy => {
                for (replay_index, remainders) in dataset.unreduced.iter().enumerate() {
                    let modulus = &dataset.prepared_moduli[replay_index];
                    for remainder in remainders {
                        let working = black_box(remainder).clone();
                        black_box(
                            reduce_unreduced_replay(working, black_box(modulus)).unwrap(),
                        );
                    }
                }
            }
            Component::CloneOnly => {
                for remainders in &dataset.unreduced {
                    for remainder in remainders {
                        black_box(black_box(remainder).clone());
                    }
                }
            }
            Component::TraceAccumulate => {
                for replay in &dataset.replays {
                    black_box(trace_accumulate_replay(black_box(&replay.terms)));
                }
            }
        }
    }

    started.elapsed().as_nanos()
}

fn build_dataset(corpus: Corpus) -> ReplayDataset {
    let left = canonical_keys(SOURCE_KEYS, corpus.left_salt);
    let right = derive_d8(&left, corpus.right_salt ^ 8, corpus.add_mask);
    let expected = symmetric_difference(&left, &right);
    assert_eq!(expected.len(), 8);

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    let mut replays = Vec::new();
    let mut candidate = None;

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();

        let d6 = decode_with_locator_quadratic(&sketch, limit, &locator);
        let d8 = decode_with_locator_square_candidate(&sketch, limit, &locator);
        let collected = decode_with_locator_collect(&sketch, limit, &locator);

        assert_eq!(d8, d6);
        assert_eq!(collected.result, d8);

        for replay in &collected.replays {
            validate_replay(replay);
        }
        replays.extend(collected.replays);

        if let Ok(roots) = d8 {
            candidate = Some(roots);
            break;
        }
    }

    assert_eq!(candidate.unwrap(), expected);

    let mut prepared_moduli = Vec::with_capacity(replays.len());
    let mut unreduced = Vec::with_capacity(replays.len());

    for replay in &replays {
        let prepared = prepare_modulus_replay(&replay.modulus).unwrap();
        let mut replay_unreduced = Vec::with_capacity(replay.terms.len());

        for term in &replay.terms {
            replay_unreduced.push(square_unreduced_replay(term).unwrap());
        }

        prepared_moduli.push(prepared);
        unreduced.push(replay_unreduced);
    }

    ReplayDataset {
        replays,
        prepared_moduli,
        unreduced,
    }
}

fn validate_replay(replay: &TraceReplay) {
    assert_eq!(replay.terms.len(), 64);
    assert_eq!(full_trace_replay(replay).unwrap(), replay.trace);
    assert_eq!(trace_accumulate_replay(&replay.terms), replay.trace);

    let prepared = prepare_modulus_replay(&replay.modulus).unwrap();
    for (index, term) in replay.terms.iter().enumerate() {
        let full = full_square_mod_replay(term, &replay.modulus).unwrap();
        let unreduced = square_unreduced_replay(term).unwrap();
        let split = reduce_unreduced_replay(unreduced, &prepared).unwrap();
        assert_eq!(split, full);
        if index + 1 < replay.terms.len() {
            assert_eq!(full, replay.terms[index + 1]);
        }
    }
}

fn validate_arithmetic_controls() {
    let sketch = PinSketch64Lab::from_sorted_unique(3, &[1, 2]).unwrap();
    let locator = fresh_locator(&sketch, 2).unwrap();
    let frozen = sketch.decode_candidate_with_limit(2).map_err(D4Error::from);
    assert_eq!(decode_with_locator_generic(&sketch, 2, &locator), frozen);
    assert_eq!(
        decode_with_locator_specialized(&sketch, 2, &locator),
        decode_with_locator_quadratic(&sketch, 2, &locator)
    );

    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}

fn derive_d8(base: &[u64], salt: u64, add_mask: u64) -> Vec<u64> {
    let mut values = base[4..].to_vec();
    for index in 0..4 {
        values.push(splitmix64(salt ^ add_mask ^ index as u64));
    }
    values.sort_unstable();
    values.dedup();
    values
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
