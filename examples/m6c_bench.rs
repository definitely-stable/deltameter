//! M6-C diagnostic Energy sign-mask measurements.
//!
//! The harness is injected unchanged into base/head by the hosted workflow.
use std::hint::black_box;
use std::time::{Duration, Instant};

use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, RelativeError,
};

const UPDATE_KEYS: u64 = 2_000;
const DIFFERENCE_REPEATS: u64 = 32;
const QUERY_REPEATS: u64 = 512;
const DECODE_REPEATS: u64 = 16;

fn main() {
    let samples = arg_usize(1, 5);
    assert!(samples > 0, "sample count must be positive");

    println!("format=deltameter.m6c-perf.v1");
    println!("contract=diagnostic_only_not_a_performance_promise");
    println!("samples={samples}");
    println!("update_keys={UPDATE_KEYS}");
    println!("difference_repeats={DIFFERENCE_REPEATS}");
    println!("query_repeats={QUERY_REPEATS}");
    println!("decode_repeats={DECODE_REPEATS}");
    println!("metric,profile,content,operations,sample,total_ns");

    for (profile_name, profile, salt) in [
        ("default", EnergyProfile::DEFAULT, 0xC600_D001),
        (
            "small",
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand),
            0xC600_5001,
        ),
    ] {
        let config = benchmark_config(profile, salt);
        println!(
            "profile,{profile_name},buckets={},tables={},primary_counter_bytes={},candidate_derived_mask_bytes={}",
            profile.buckets(),
            profile.tables(),
            profile.counter_state_bytes(),
            profile.tables() * 3 * core::mem::size_of::<u64>()
        );

        measure(samples, "construct", profile_name, "empty", 1, || {
            let owned = config.clone();
            timed(|| {
                black_box(EnergyDeltaMeter::new(owned).unwrap());
            })
        });

        for content in ["sequential", "full-width"] {
            measure(
                samples,
                "update",
                profile_name,
                content,
                UPDATE_KEYS,
                || {
                    let mut meter = EnergyDeltaMeter::new(config.clone()).unwrap();
                    timed(|| {
                        for ordinal in 0..UPDATE_KEYS {
                            meter.add_unique(black_box(key(content, ordinal))).unwrap();
                        }
                        black_box(&meter);
                    })
                },
            );
        }

        let mut snapshot_meter = EnergyDeltaMeter::new(config.clone()).unwrap();
        for ordinal in 0..UPDATE_KEYS {
            snapshot_meter
                .add_unique(key("full-width", ordinal))
                .unwrap();
        }
        let snapshot = snapshot_meter.encode_snapshot().unwrap();

        measure(
            samples,
            "decode",
            profile_name,
            "full-width",
            DECODE_REPEATS,
            || {
                timed(|| {
                    for _ in 0..DECODE_REPEATS {
                        black_box(EnergyDeltaMeter::decode_snapshot(black_box(&snapshot)).unwrap());
                    }
                })
            },
        );

        let (left, right) = build_pair(config.clone());
        measure(
            samples,
            "difference",
            profile_name,
            "full-width",
            DIFFERENCE_REPEATS,
            || {
                timed(|| {
                    for _ in 0..DIFFERENCE_REPEATS {
                        black_box(left.difference(&right).unwrap());
                    }
                })
            },
        );

        measure(
            samples,
            "query",
            profile_name,
            "full-width",
            QUERY_REPEATS,
            || {
                timed(|| {
                    for _ in 0..QUERY_REPEATS {
                        black_box(left.point_estimate());
                    }
                })
            },
        );
    }
}

fn build_pair(config: EnergyConfig) -> (EnergyDeltaMeter, EnergyDeltaMeter) {
    let mut left = EnergyDeltaMeter::new(config.clone()).unwrap();
    let mut right = EnergyDeltaMeter::new(config).unwrap();

    for ordinal in 0..UPDATE_KEYS {
        let key = key("full-width", ordinal);
        if ordinal % 3 != 0 {
            left.add_unique(key).unwrap();
        }
        if ordinal % 5 != 0 {
            right.add_unique(key).unwrap();
        }
    }
    (left, right)
}

fn measure(
    samples: usize,
    metric: &str,
    profile: &str,
    content: &str,
    operations: u64,
    mut sample: impl FnMut() -> Duration,
) {
    black_box(sample());
    for ordinal in 0..samples {
        let elapsed = sample().as_nanos();
        println!("{metric},{profile},{content},{operations},{ordinal},{elapsed}");
    }
}

fn timed(operation: impl FnOnce()) -> Duration {
    let start = Instant::now();
    operation();
    start.elapsed()
}

fn benchmark_config(profile: EnergyProfile, salt: u64) -> EnergyConfig {
    let words = pseudo_words(profile.uniform_words_required(), salt);
    let rows = words
        .chunks(6)
        .map(|chunk| {
            EnergyRowHash::from_coefficients(
                [chunk[0], chunk[1]],
                [chunk[2], chunk[3], chunk[4], chunk[5]],
            )
        })
        .collect();
    EnergyConfig::new(profile.buckets(), rows).unwrap()
}

fn key(content: &str, ordinal: u64) -> u64 {
    match content {
        "sequential" => ordinal,
        "full-width" => splitmix64(ordinal ^ 0xA5A5_5A5A_DEAD_BEEF),
        _ => unreachable!("known benchmark content"),
    }
}

fn pseudo_words(count: usize, salt: u64) -> Vec<u64> {
    (0..count)
        .map(|index| splitmix64((index as u64) ^ salt.rotate_left(17)))
        .collect()
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn arg_usize(index: usize, default: usize) -> usize {
    std::env::args()
        .nth(index)
        .map(|value| value.parse().expect("argument must be an integer"))
        .unwrap_or(default)
}
