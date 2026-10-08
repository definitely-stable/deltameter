#[path = "../src/fpcsa.rs"]
mod fpcsa;

use blake3::derive_key;
use deltameter::{EnergyConfig, EnergyDeltaMeter, EnergyProfile};
use fpcsa::{PublishedFpcsaF2, PublishedFpcsaF2Config, PublishedFpcsaOracle};
use std::hint::black_box;
use std::time::Instant;

const ROWS: u32 = 4096;
const LEVELS: u8 = 64;
const ORACLE_CONTEXT: &str = "deltameter 2026-10-08 strict-compact keyed oracle v1";
const MASTER_KEY: [u8; 32] = [
    0x44, 0x65, 0x6c, 0x74, 0x61, 0x4d, 0x65, 0x74, 0x65, 0x72, 0x2d, 0x53, 0x54, 0x52, 0x49, 0x43,
    0x54, 0x2d, 0x43, 0x4f, 0x4d, 0x50, 0x41, 0x43, 0x54, 0x2d, 0x30, 0x30, 0x33, 0x21, 0x21, 0x21,
];

const KEYED_ABSOLUTE_GATE_NS: f64 = 500.0;
const KEYED_RELATIVE_GATE: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sample {
    row: u16,
    level: u8,
    coefficient_one: bool,
}

#[derive(Clone)]
struct KeyedBlake3Sketch {
    oracle_key: [u8; 32],
    words: Box<[u64]>,
}

impl KeyedBlake3Sketch {
    fn new(master_key: &[u8; 32]) -> Self {
        let oracle_key = derive_key(ORACLE_CONTEXT, master_key);
        Self {
            oracle_key,
            words: vec![0_u64; ROWS as usize].into_boxed_slice(),
        }
    }

    #[inline]
    fn sample(&self, token: u64) -> Sample {
        let hash = blake3::keyed_hash(&self.oracle_key, &token.to_le_bytes());
        let bytes = hash.as_bytes();

        let row_word = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let coefficient_word = u64::from_le_bytes(bytes[16..24].try_into().unwrap());

        // ROWS=4096=2^12, so masking is exactly uniform under a uniform word.
        let row = (row_word & (u64::from(ROWS) - 1)) as u16;
        let level = if level_word == 0 {
            // Probability 2^-64. The frozen J=64 state treats this as truncation.
            0
        } else {
            (level_word.trailing_zeros() + 1) as u8
        };

        Sample {
            row,
            level,
            coefficient_one: coefficient_word & 1 != 0,
        }
    }

    #[inline]
    fn toggle(&mut self, token: u64) {
        let sample = self.sample(token);
        if sample.level == 0 || !sample.coefficient_one {
            return;
        }

        let mask = 1_u64 << (sample.level - 1);
        self.words[usize::from(sample.row)] ^= mask;
    }

    fn xor_assign(&mut self, other: &Self) {
        assert_eq!(self.oracle_key, other.oracle_key);
        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            *left ^= *right;
        }
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn token(index: usize) -> u64 {
    splitmix64(index as u64 ^ 0xA5A5_5A5A_D13C_0003)
}

fn parity_config() -> PublishedFpcsaF2Config {
    PublishedFpcsaF2Config::new(
        ROWS,
        LEVELS,
        PublishedFpcsaOracle::new(
            0x243F_6A88_85A3_08D3,
            0x1319_8A2E_0370_7344,
            0xA409_3822_299F_31D0,
        ),
    )
    .expect("valid frozen parity config")
}

fn energy_config() -> EnergyConfig {
    let profile = EnergyProfile::DEFAULT;
    let mut state = 0xC0FF_EE11_D13C_0003_u64;
    let words: Vec<u64> = (0..profile.uniform_words_required())
        .map(|_| {
            state = splitmix64(state);
            state
        })
        .collect();

    EnergyConfig::for_profile_assuming_uniform_words(profile, &words)
        .expect("deterministic benchmark config has correct shape")
}

fn check_semantics() {
    let probe = KeyedBlake3Sketch::new(&MASTER_KEY);
    let stored_token = (0_u64..4096)
        .find(|&key| {
            let sample = probe.sample(key);
            sample.level != 0 && sample.coefficient_one
        })
        .expect("find deterministic stored-token fixture");

    let mut once = KeyedBlake3Sketch::new(&MASTER_KEY);
    let before = once.words.clone();
    once.toggle(stored_token);
    assert_ne!(once.words, before, "stored token must toggle one bit");
    once.toggle(stored_token);
    assert_eq!(once.words, before, "repeating the same token must cancel");

    let no_op_token = (0_u64..4096)
        .find(|&key| {
            let sample = probe.sample(key);
            sample.level == 0 || !sample.coefficient_one
        })
        .expect("find deterministic no-op fixture");
    once.toggle(no_op_token);
    assert_eq!(
        once.words, before,
        "zero coefficient/truncation must not mutate state"
    );

    let mut left = KeyedBlake3Sketch::new(&MASTER_KEY);
    let mut right = KeyedBlake3Sketch::new(&MASTER_KEY);
    let mut combined = KeyedBlake3Sketch::new(&MASTER_KEY);

    for key in [1_u64, 2, 3, 5, 8, 13, 21] {
        left.toggle(key);
        combined.toggle(key);
    }
    for key in [3_u64, 5, 34, 55, 89] {
        right.toggle(key);
        combined.toggle(key);
    }

    left.xor_assign(&right);
    assert_eq!(left.words, combined.words);

    let a = KeyedBlake3Sketch::new(&MASTER_KEY);
    let b = KeyedBlake3Sketch::new(&MASTER_KEY);
    for key in [0_u64, 1, 2, 42, u64::MAX] {
        assert_eq!(a.sample(key), b.sample(key));
    }
}

fn distribution_sanity(samples: usize) {
    let sketch = KeyedBlake3Sketch::new(&MASTER_KEY);
    let mut row_counts = vec![0_u32; ROWS as usize];
    let mut coefficient_ones = 0_usize;
    let mut level_one = 0_usize;
    let mut level_two = 0_usize;

    for index in 0..samples {
        let sample = sketch.sample(token(index));
        row_counts[usize::from(sample.row)] += 1;
        coefficient_ones += usize::from(sample.coefficient_one);
        level_one += usize::from(sample.level == 1);
        level_two += usize::from(sample.level == 2);
    }

    let expected_row = samples as f64 / f64::from(ROWS);
    let row_min = *row_counts.iter().min().unwrap() as f64;
    let row_max = *row_counts.iter().max().unwrap() as f64;
    let coefficient_fraction = coefficient_ones as f64 / samples as f64;
    let level_one_fraction = level_one as f64 / samples as f64;
    let level_two_fraction = level_two as f64 / samples as f64;

    // Broad sanity checks only. They are explicitly not theorem evidence.
    assert!(row_min > expected_row * 0.45);
    assert!(row_max < expected_row * 1.75);
    assert!((coefficient_fraction - 0.5).abs() < 0.01);
    assert!((level_one_fraction - 0.5).abs() < 0.01);
    assert!((level_two_fraction - 0.25).abs() < 0.01);

    println!(
        "STRICT_COMPACT_KEYED_SANITY samples={} row_min={} row_max={} coeff={:.6} level1={:.6} level2={:.6}",
        samples,
        row_min as u64,
        row_max as u64,
        coefficient_fraction,
        level_one_fraction,
        level_two_fraction,
    );
}

fn bench_parity(tokens: usize) -> f64 {
    let config = parity_config();
    let mut sketch = PublishedFpcsaF2::new(config);
    let start = Instant::now();
    for index in 0..tokens {
        black_box(sketch.toggle(black_box(token(index))));
    }
    let elapsed = start.elapsed().as_nanos() as f64;
    black_box(sketch);
    elapsed / tokens as f64
}

fn bench_keyed(tokens: usize) -> f64 {
    let mut sketch = KeyedBlake3Sketch::new(&MASTER_KEY);
    let start = Instant::now();
    for index in 0..tokens {
        sketch.toggle(black_box(token(index)));
    }
    let elapsed = start.elapsed().as_nanos() as f64;
    black_box(sketch);
    elapsed / tokens as f64
}

fn bench_energy(tokens: usize) -> f64 {
    let config = energy_config();
    let mut sketch = EnergyDeltaMeter::new(config).expect("energy meter");
    let start = Instant::now();
    for index in 0..tokens {
        sketch
            .add_unique(black_box(token(index)))
            .expect("energy update");
    }
    let elapsed = start.elapsed().as_nanos() as f64;
    black_box(sketch);
    elapsed / tokens as f64
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    values[values.len() / 2]
}

fn main() {
    let tokens: usize = std::env::args()
        .nth(1)
        .map(|value| value.parse().expect("token count"))
        .unwrap_or(250_000);
    let rounds: usize = std::env::args()
        .nth(2)
        .map(|value| value.parse().expect("round count"))
        .unwrap_or(3);
    let result_path = std::env::args_os().nth(3);

    assert!(tokens >= 10_000);
    assert!(rounds >= 3 && rounds % 2 == 1);

    check_semantics();
    distribution_sanity(250_000);

    // Warm-up outside measured rounds.
    black_box(bench_parity(tokens / 10));
    black_box(bench_keyed(tokens / 10));
    black_box(bench_energy((tokens / 100).max(1_000)));

    let mut parity = Vec::with_capacity(rounds);
    let mut keyed = Vec::with_capacity(rounds);
    let mut energy = Vec::with_capacity(rounds);

    // Alternate ordering by round to reduce fixed arm-order bias.
    for round in 0..rounds {
        if round % 2 == 0 {
            parity.push(bench_parity(tokens));
            keyed.push(bench_keyed(tokens));
        } else {
            keyed.push(bench_keyed(tokens));
            parity.push(bench_parity(tokens));
        }
        energy.push(bench_energy((tokens / 10).max(10_000)));
    }

    let parity_ns = median(parity);
    let keyed_ns = median(keyed);
    let energy_ns = median(energy);
    let ratio = keyed_ns / parity_ns;
    let energy_ratio = energy_ns / keyed_ns;

    let result = format!(
        "STRICT_COMPACT_KEYED_RESULT tokens={} rounds={} parity_ns={:.3} keyed_ns={:.3} energy_ns={:.3} keyed_over_parity={:.3} energy_over_keyed={:.3}\n",
        tokens, rounds, parity_ns, keyed_ns, energy_ns, ratio, energy_ratio,
    );
    print!("{result}");
    if let Some(path) = result_path {
        std::fs::write(path, result.as_bytes()).expect("write durable result");
    }

    assert!(
        keyed_ns <= KEYED_ABSOLUTE_GATE_NS,
        "keyed oracle missed absolute gate: {keyed_ns:.3} ns"
    );
    assert!(
        ratio <= KEYED_RELATIVE_GATE,
        "keyed oracle missed relative gate: {ratio:.3}x"
    );

    println!("STRICT_COMPACT_KEYED_ORACLE_PASS");
}
