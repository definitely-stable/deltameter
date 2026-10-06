use std::hint::black_box;
use std::time::{Duration, Instant};

use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, ParityConfig,
    ParityDeltaMeter, ParityProfile, RelativeError,
};

fn main() {
    let keys = std::env::args()
        .nth(1)
        .map(|value| value.parse::<u64>().expect("key count must be an integer"))
        .unwrap_or(10_000);

    let energy_profile = EnergyProfile::new(RelativeError::TenPercent, FailureTarget::OneInMillion);
    let energy_config = energy_benchmark_config(energy_profile, 0xE11E);

    let parity_config = ParityConfig::for_profile(ParityProfile::Standard, 0x0A11_CE55).unwrap();

    println!("DeltaMeter M3 comparison harness");
    println!("keys={keys}");
    println!(
        "Energy counter state={} bytes; theorem profile=Proven(epsilon=10%, delta=1e-6)",
        energy_profile.counter_state_bytes()
    );
    println!(
        "Parity packed cell state={} bytes; guarantee=Asymptotic(RSE≈{:.4}%)",
        parity_config.packed_state_bytes(),
        parity_config.asymptotic_relative_standard_error() * 100.0
    );
    println!("WARNING: these guarantees are not equivalent.");
    println!(
        "Benchmark Energy coefficients are deterministic timing inputs, not theorem randomness."
    );

    let energy_update = bench_energy_update(keys, energy_config.clone());
    let parity_update = bench_parity_update(keys, parity_config.clone());

    let (energy_left, energy_right) = build_energy_pair(keys, energy_config);
    let (parity_left, parity_right) = build_parity_pair(keys, parity_config);

    let energy_merge = elapsed(|| {
        black_box(energy_left.difference(&energy_right).unwrap());
    });
    let parity_merge = elapsed(|| {
        black_box(parity_left.xor_merged(&parity_right).unwrap());
    });

    let energy_query = repeated(100, || {
        black_box(energy_left.point_estimate());
    });
    let parity_query = repeated(100, || {
        black_box(parity_left.estimate().ok());
    });

    println!("update:");
    println!(
        "  Energy: {:?} total, {:.2} ns/update",
        energy_update,
        nanos_per_item(energy_update, keys)
    );
    println!(
        "  Parity: {:?} total, {:.2} ns/update",
        parity_update,
        nanos_per_item(parity_update, keys)
    );
    println!("merge:");
    println!("  Energy: {:?}", energy_merge);
    println!("  Parity: {:?}", parity_merge);
    println!("query (100 iterations):");
    println!("  Energy: {:?}", energy_query);
    println!("  Parity: {:?}", parity_query);
}

fn bench_energy_update(keys: u64, config: EnergyConfig) -> Duration {
    let mut meter = EnergyDeltaMeter::new(config).unwrap();
    elapsed(|| {
        for key in 0..keys {
            meter.add_unique(black_box(key)).unwrap();
        }
        black_box(meter.point_estimate());
    })
}

fn bench_parity_update(keys: u64, config: ParityConfig) -> Duration {
    let mut meter = ParityDeltaMeter::new(config);
    elapsed(|| {
        for key in 0..keys {
            black_box(meter.toggle(black_box(key)));
        }
        black_box(&meter);
    })
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

fn elapsed(operation: impl FnOnce()) -> Duration {
    let start = Instant::now();
    operation();
    start.elapsed()
}

fn repeated(iterations: u32, mut operation: impl FnMut()) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    start.elapsed()
}

fn nanos_per_item(duration: Duration, items: u64) -> f64 {
    if items == 0 {
        return 0.0;
    }
    duration.as_nanos() as f64 / items as f64
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
