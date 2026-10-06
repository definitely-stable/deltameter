use std::hint::black_box;
use std::time::{Duration, Instant};

use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, ParityConfig,
    ParityDeltaMeter, ParityProfile, RelativeError,
};

const MERGE_REPEATS: u64 = 64;
const QUERY_REPEATS: u64 = 1024;

fn main() {
    let keys = arg_u64(1, 10_000);
    let samples = arg_usize(2, 7);
    assert!(samples > 0, "sample count must be positive");

    let energy_profile = EnergyProfile::new(RelativeError::TenPercent, FailureTarget::OneInMillion);
    let energy_config = energy_benchmark_config(energy_profile, 0xE11E);
    let parity_profile = ParityProfile::Standard;
    let parity_config = ParityConfig::for_profile(parity_profile, 0x0A11_CE55).unwrap();

    println!("format=deltameter.m4-perf.v1");
    println!("keys={keys}");
    println!("samples={samples}");
    println!("merge_repeats={MERGE_REPEATS}");
    println!("query_repeats={QUERY_REPEATS}");
    println!("note=diagnostic_only_not_a_performance_promise");
    println!(
        "profile,energy,epsilon=0.10,delta=1e-6,primary_state_bytes={}",
        energy_profile.counter_state_bytes()
    );
    println!(
        "profile,parity,standard,rse={},primary_state_bytes={}",
        parity_profile.asymptotic_relative_standard_error(),
        parity_profile.packed_state_bytes()
    );

    let energy_update = median_ns(samples, || {
        let mut meter = EnergyDeltaMeter::new(energy_config.clone()).unwrap();
        elapsed(|| {
            for key in 0..keys {
                meter.add_unique(black_box(key)).unwrap();
            }
            black_box(&meter);
        })
    });
    emit("update", "energy", keys, energy_update);

    let parity_update = median_ns(samples, || {
        let mut meter = ParityDeltaMeter::new(parity_config.clone());
        elapsed(|| {
            for key in 0..keys {
                black_box(meter.toggle(black_box(key)));
            }
            black_box(&meter);
        })
    });
    emit("update", "parity", keys, parity_update);

    let (energy_left, energy_right) = build_energy_pair(keys, energy_config);
    let (parity_left, parity_right) = build_parity_pair(keys, parity_config);

    let energy_merge = median_ns(samples, || {
        elapsed(|| {
            for _ in 0..MERGE_REPEATS {
                black_box(energy_left.difference(&energy_right).unwrap());
            }
        })
    });
    emit("merge", "energy", MERGE_REPEATS, energy_merge);

    let parity_merge = median_ns(samples, || {
        elapsed(|| {
            for _ in 0..MERGE_REPEATS {
                black_box(parity_left.xor_merged(&parity_right).unwrap());
            }
        })
    });
    emit("merge", "parity", MERGE_REPEATS, parity_merge);

    let energy_query = median_ns(samples, || {
        elapsed(|| {
            for _ in 0..QUERY_REPEATS {
                black_box(energy_left.point_estimate());
            }
        })
    });
    emit("query", "energy", QUERY_REPEATS, energy_query);

    let parity_query = median_ns(samples, || {
        elapsed(|| {
            for _ in 0..QUERY_REPEATS {
                black_box(parity_left.estimate().ok());
            }
        })
    });
    emit("query", "parity", QUERY_REPEATS, parity_query);
}

fn emit(metric: &str, backend: &str, operations: u64, median_total_ns: u128) {
    let ns_per_op = median_total_ns as f64 / operations.max(1) as f64;
    println!(
        "metric,{metric},backend,{backend},operations,{operations},median_total_ns,{median_total_ns},median_ns_per_op,{ns_per_op:.3}"
    );
}

fn median_ns(samples: usize, mut sample: impl FnMut() -> Duration) -> u128 {
    black_box(sample());

    let mut values = Vec::with_capacity(samples);
    for _ in 0..samples {
        values.push(sample().as_nanos());
    }
    values.sort_unstable();
    values[values.len() / 2]
}

fn elapsed(operation: impl FnOnce()) -> Duration {
    let start = Instant::now();
    operation();
    start.elapsed()
}

fn build_energy_pair(keys: u64, config: EnergyConfig) -> (EnergyDeltaMeter, EnergyDeltaMeter) {
    let mut left = EnergyDeltaMeter::new(config.clone()).unwrap();
    let mut right = EnergyDeltaMeter::new(config).unwrap();

    for key in 0..keys {
        if key % 3 != 0 {
            left.add_unique(key).unwrap();
        }
        if key % 5 != 0 {
            right.add_unique(key).unwrap();
        }
    }

    (left, right)
}

fn build_parity_pair(keys: u64, config: ParityConfig) -> (ParityDeltaMeter, ParityDeltaMeter) {
    let mut left = ParityDeltaMeter::new(config.clone());
    let mut right = ParityDeltaMeter::new(config);

    for key in 0..keys {
        if key % 3 != 0 {
            left.toggle(key);
        }
        if key % 5 != 0 {
            right.toggle(key);
        }
    }

    (left, right)
}

fn energy_benchmark_config(profile: EnergyProfile, salt: u64) -> EnergyConfig {
    let words = pseudo_uniform_words(profile.uniform_words_required(), salt);
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

fn pseudo_uniform_words(count: usize, salt: u64) -> Vec<u64> {
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

fn arg_u64(index: usize, default: u64) -> u64 {
    std::env::args()
        .nth(index)
        .map(|value| value.parse().expect("argument must be an integer"))
        .unwrap_or(default)
}

fn arg_usize(index: usize, default: usize) -> usize {
    std::env::args()
        .nth(index)
        .map(|value| value.parse().expect("argument must be an integer"))
        .unwrap_or(default)
}
