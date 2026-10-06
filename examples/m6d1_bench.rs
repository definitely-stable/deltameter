//! M6-D1 diagnostic PinSketch64 lab measurements.
//!
//! This is a private research harness. It does not benchmark a public API.

#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;

use std::hint::black_box;
use std::time::{Duration, Instant};

use pinsketch64::PinSketch64Lab;

const SOURCE_KEYS: usize = 8_192;
const BUILD_SAMPLES: usize = 3;
const MERGE_REPEATS: u64 = 4_096;
const DECODE_SAMPLES: usize = 3;
const OVER_CAPACITY_TRIALS: usize = 64;

fn main() {
    println!("format=deltameter.m6d1-perf.v2");
    println!("contract=private_lab_diagnostic_not_product_promise");
    println!("source_keys={SOURCE_KEYS}");
    println!("build_samples={BUILD_SAMPLES}");
    println!("merge_repeats={MERGE_REPEATS}");
    println!("decode_samples={DECODE_SAMPLES}");
    println!("over_capacity_trials={OVER_CAPACITY_TRIALS}");
    println!("guard_field_bits=64");
    println!("guard_extra_syndromes=1");
    println!(
        "metric,stored_capacity,max_elements,d,operations,sample,total_ns,sketch_bytes,direct_set_payload_bytes"
    );

    let base = canonical_keys(SOURCE_KEYS, 0xD100_BA5E_0000_0001);
    let direct_bytes = 4 + base.len() * 8;

    for max_elements in [1_usize, 2, 4, 8] {
        let stored_capacity = max_elements + 1;
        let sketch_bytes = PinSketch64Lab::encoded_len_for_capacity(stored_capacity).unwrap();

        for sample in 0..BUILD_SAMPLES {
            let elapsed = timed(|| {
                black_box(
                    PinSketch64Lab::from_sorted_unique(stored_capacity, black_box(&base)).unwrap(),
                );
            });
            emit(
                "cold_build",
                stored_capacity,
                max_elements,
                0,
                base.len() as u64,
                sample,
                elapsed,
                sketch_bytes,
                direct_bytes,
            );
        }

        let left = PinSketch64Lab::from_sorted_unique(stored_capacity, &base).unwrap();
        assert_eq!(left.capacity(), stored_capacity);
        assert_eq!(left.odd_syndromes().len(), stored_capacity);
        assert!(!left.zero_present());

        let encoded_left = left.encode();
        assert_eq!(encoded_left.len(), sketch_bytes);
        assert_eq!(PinSketch64Lab::decode(&encoded_left).unwrap(), left);

        for sample in 0..BUILD_SAMPLES {
            let encode_elapsed = timed(|| {
                for _ in 0..MERGE_REPEATS {
                    black_box(black_box(&left).encode());
                }
            });
            emit(
                "encode_state",
                stored_capacity,
                max_elements,
                0,
                MERGE_REPEATS,
                sample,
                encode_elapsed,
                sketch_bytes,
                direct_bytes,
            );

            let decode_state_elapsed = timed(|| {
                for _ in 0..MERGE_REPEATS {
                    black_box(PinSketch64Lab::decode(black_box(&encoded_left)).unwrap());
                }
            });
            emit(
                "decode_state",
                stored_capacity,
                max_elements,
                0,
                MERGE_REPEATS,
                sample,
                decode_state_elapsed,
                sketch_bytes,
                direct_bytes,
            );
        }

        let mut right_keys = base.clone();
        right_keys.remove(0);
        right_keys.push(splitmix64(0xF00D_0000_0000_0001 ^ max_elements as u64));
        right_keys.sort_unstable();
        right_keys.dedup();
        assert_eq!(right_keys.len(), base.len());
        let right = PinSketch64Lab::from_sorted_unique(stored_capacity, &right_keys).unwrap();

        for sample in 0..BUILD_SAMPLES {
            let merge_elapsed = timed(|| {
                for _ in 0..MERGE_REPEATS {
                    let mut combined = left.clone();
                    combined.merge(black_box(&right)).unwrap();
                    black_box(combined);
                }
            });
            emit(
                "merge",
                stored_capacity,
                max_elements,
                2,
                MERGE_REPEATS,
                sample,
                merge_elapsed,
                sketch_bytes,
                direct_bytes,
            );

            let direct_elapsed = timed(|| {
                for _ in 0..MERGE_REPEATS {
                    black_box(symmetric_difference_count(
                        black_box(&base),
                        black_box(&right_keys),
                    ));
                }
            });
            emit(
                "direct_diff",
                stored_capacity,
                max_elements,
                2,
                MERGE_REPEATS,
                sample,
                direct_elapsed,
                sketch_bytes,
                direct_bytes,
            );
        }

        for d in decode_sizes(max_elements) {
            let candidate = difference_sketch(
                stored_capacity,
                max_elements,
                d,
                0xDEC0_DE00 ^ max_elements as u64,
            );
            for sample in 0..DECODE_SAMPLES {
                let elapsed = timed(|| {
                    let decoded = black_box(&candidate)
                        .decode_candidate_with_limit(max_elements)
                        .unwrap();
                    assert_eq!(decoded.len(), d);
                    black_box(decoded);
                });
                emit(
                    "decode_guarded",
                    stored_capacity,
                    max_elements,
                    d,
                    1,
                    sample,
                    elapsed,
                    sketch_bytes,
                    direct_bytes,
                );
            }
        }

        for d in over_capacity_sizes(stored_capacity, max_elements) {
            inventory_over_capacity("unguarded", max_elements, max_elements, d);
            inventory_over_capacity("guarded", stored_capacity, max_elements, d);
        }
    }
}

fn decode_sizes(max_elements: usize) -> Vec<usize> {
    let mut values = vec![0, 1, max_elements / 2, max_elements];
    values.sort_unstable();
    values.dedup();
    values
}

fn over_capacity_sizes(stored_capacity: usize, max_elements: usize) -> Vec<usize> {
    let mut values = vec![
        max_elements + 1,
        max_elements + 2,
        max_elements * 2,
        stored_capacity * 2 + 1,
    ];
    values.sort_unstable();
    values.dedup();
    values
}

fn difference_sketch(
    stored_capacity: usize,
    max_elements: usize,
    d: usize,
    salt: u64,
) -> PinSketch64Lab {
    let keys = canonical_keys(d, salt);
    let sketch = PinSketch64Lab::from_sorted_unique(stored_capacity, &keys).unwrap();
    let decoded = sketch.decode_candidate_with_limit(max_elements).unwrap();
    assert_eq!(decoded, keys);
    sketch
}

fn inventory_over_capacity(mode: &str, stored_capacity: usize, max_elements: usize, d: usize) {
    let mut rejected = 0_usize;
    let mut false_success = 0_usize;

    for trial in 0..OVER_CAPACITY_TRIALS {
        let keys = canonical_keys(
            d,
            0xBAD0_0000_0000_0000
                ^ ((stored_capacity as u64) << 40)
                ^ ((max_elements as u64) << 32)
                ^ trial as u64,
        );
        let sketch = PinSketch64Lab::from_sorted_unique(stored_capacity, &keys).unwrap();
        match sketch.decode_candidate_with_limit(max_elements) {
            Ok(candidate) => {
                assert_ne!(candidate, keys);
                false_success += 1;
            }
            Err(_) => rejected += 1,
        }
    }

    println!(
        "inventory,mode={mode},stored_capacity={stored_capacity},max_elements={max_elements},d={d},trials={OVER_CAPACITY_TRIALS},rejected={rejected},false_success={false_success}"
    );
}

#[allow(clippy::too_many_arguments)]
fn emit(
    metric: &str,
    stored_capacity: usize,
    max_elements: usize,
    d: usize,
    operations: u64,
    sample: usize,
    elapsed: Duration,
    sketch_bytes: usize,
    direct_bytes: usize,
) {
    println!(
        "{metric},{stored_capacity},{max_elements},{d},{operations},{sample},{},{sketch_bytes},{direct_bytes}",
        elapsed.as_nanos()
    );
}

fn timed(operation: impl FnOnce()) -> Duration {
    let started = Instant::now();
    operation();
    started.elapsed()
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

fn symmetric_difference_count(left: &[u64], right: &[u64]) -> u64 {
    let mut i = 0;
    let mut j = 0;
    let mut difference = 0_u64;

    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                difference += 1;
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                difference += 1;
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    difference + (left.len() - i) as u64 + (right.len() - j) as u64
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
