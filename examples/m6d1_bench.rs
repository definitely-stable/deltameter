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
const OVER_CAPACITY_TRIALS: usize = 128;

fn main() {
    println!("format=deltameter.m6d1-perf.v1");
    println!("contract=private_lab_diagnostic_not_product_promise");
    println!("source_keys={SOURCE_KEYS}");
    println!("build_samples={BUILD_SAMPLES}");
    println!("merge_repeats={MERGE_REPEATS}");
    println!("decode_samples={DECODE_SAMPLES}");
    println!("over_capacity_trials={OVER_CAPACITY_TRIALS}");
    println!("metric,capacity,d,operations,sample,total_ns,sketch_bytes,direct_set_payload_bytes");

    let base = canonical_keys(SOURCE_KEYS, 0xD100_BA5E_0000_0001);
    let direct_bytes = 4 + base.len() * 8;

    for capacity in [1_usize, 2, 4, 8] {
        let sketch_bytes = PinSketch64Lab::encoded_len_for_capacity(capacity).unwrap();

        for sample in 0..BUILD_SAMPLES {
            let elapsed = timed(|| {
                black_box(PinSketch64Lab::from_sorted_unique(capacity, black_box(&base)).unwrap());
            });
            emit(
                "cold_build",
                capacity,
                0,
                base.len() as u64,
                sample,
                elapsed,
                sketch_bytes,
                direct_bytes,
            );
        }

        let left = PinSketch64Lab::from_sorted_unique(capacity, &base).unwrap();
        let mut right_keys = base.clone();
        right_keys.remove(0);
        right_keys.push(splitmix64(0xF00D_0000_0000_0001 ^ capacity as u64));
        right_keys.sort_unstable();
        right_keys.dedup();
        assert_eq!(right_keys.len(), base.len());
        let right = PinSketch64Lab::from_sorted_unique(capacity, &right_keys).unwrap();

        for sample in 0..BUILD_SAMPLES {
            let elapsed = timed(|| {
                for _ in 0..MERGE_REPEATS {
                    let mut combined = left.clone();
                    combined.merge(black_box(&right)).unwrap();
                    black_box(combined);
                }
            });
            emit(
                "merge",
                capacity,
                2,
                MERGE_REPEATS,
                sample,
                elapsed,
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
                capacity,
                2,
                MERGE_REPEATS,
                sample,
                direct_elapsed,
                sketch_bytes,
                direct_bytes,
            );
        }

        for d in decode_sizes(capacity) {
            let candidate = difference_sketch(capacity, d, 0xDEC0_DE00 ^ capacity as u64);
            for sample in 0..DECODE_SAMPLES {
                let elapsed = timed(|| {
                    let decoded = black_box(&candidate).decode_candidate().unwrap();
                    assert_eq!(decoded.len(), d);
                    black_box(decoded);
                });
                emit(
                    "decode",
                    capacity,
                    d,
                    1,
                    sample,
                    elapsed,
                    sketch_bytes,
                    direct_bytes,
                );
            }
        }

        for d in over_capacity_sizes(capacity) {
            inventory_over_capacity(capacity, d);
        }
    }
}

fn decode_sizes(capacity: usize) -> Vec<usize> {
    let mut values = vec![0, 1, capacity / 2, capacity];
    values.sort_unstable();
    values.dedup();
    values
}

fn difference_sketch(capacity: usize, d: usize, salt: u64) -> PinSketch64Lab {
    let keys = canonical_keys(d, salt);
    let sketch = PinSketch64Lab::from_sorted_unique(capacity, &keys).unwrap();
    let decoded = sketch.decode_candidate().unwrap();
    assert_eq!(decoded, keys);
    sketch
}

fn over_capacity_sizes(capacity: usize) -> Vec<usize> {
    let mut values = vec![capacity + 1, capacity + 2, capacity * 2];
    values.sort_unstable();
    values.dedup();
    values
}

fn inventory_over_capacity(capacity: usize, d: usize) {
    let mut rejected = 0_usize;
    let mut false_success = 0_usize;

    for trial in 0..OVER_CAPACITY_TRIALS {
        let keys = canonical_keys(
            d,
            0xBAD0_0000_0000_0000 ^ ((capacity as u64) << 32) ^ trial as u64,
        );
        let sketch = PinSketch64Lab::from_sorted_unique(capacity, &keys).unwrap();
        match sketch.decode_candidate() {
            Ok(candidate) => {
                assert_ne!(candidate, keys);
                false_success += 1;
            }
            Err(_) => rejected += 1,
        }
    }

    println!(
        "inventory,capacity={capacity},d={d},trials={OVER_CAPACITY_TRIALS},rejected={rejected},false_success={false_success}"
    );
}

#[allow(clippy::too_many_arguments)]
fn emit(
    metric: &str,
    capacity: usize,
    d: usize,
    operations: u64,
    sample: usize,
    elapsed: Duration,
    sketch_bytes: usize,
    direct_bytes: usize,
) {
    println!(
        "{metric},{capacity},{d},{operations},{sample},{},{sketch_bytes},{direct_bytes}",
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
