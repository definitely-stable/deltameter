#![allow(dead_code)]

#[path = "../src/fpcsa.rs"]
mod fpcsa;

use fpcsa::{
    PublishedFpcsaF2,
    PublishedFpcsaF2Config,
    PublishedFpcsaOracle,
};
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const ROWS: u32 = 4096;
const LEVELS: u8 = 64;
const TABLE_ENTRIES: usize = 2048;
const Q_BITS: u32 = 32;
const SENTINEL: u64 = u64::MAX;
const DOMAIN_CARDINALITY: u128 = 1_u128 << 64;

fn load_table(path: &Path) -> Vec<u64> {
    let text = std::fs::read_to_string(path).expect("read Q32 table");
    let table: Vec<u64> = text
        .lines()
        .map(|line| line.parse::<u64>().expect("Q32 integer"))
        .collect();

    assert_eq!(table.len(), TABLE_ENTRIES);

    let mut previous = 0_u64;
    let mut saw_sentinel = false;
    for &value in &table {
        if value == SENTINEL {
            saw_sentinel = true;
            continue;
        }
        assert!(!saw_sentinel, "finite threshold after sentinel");
        assert!(value >= previous, "threshold table must be monotone");
        previous = value;
    }

    table
}

fn level_counts(words: &[u64]) -> [u16; 64] {
    assert_eq!(words.len(), ROWS as usize);
    let mut counts = [0_u16; 64];

    for &word in words {
        let mut bits = word;
        while bits != 0 {
            let level = bits.trailing_zeros() as usize;
            counts[level] += 1;
            bits &= bits - 1;
        }
    }

    assert!(counts.iter().all(|&count| u32::from(count) <= ROWS));
    counts
}

fn q32_level_upper(q: u64, level_one_based: u32) -> u128 {
    if q == SENTINEL {
        return DOMAIN_CARDINALITY;
    }
    assert!((1..=64).contains(&level_one_based));

    let numerator = (q as u128) << level_one_based;
    let denominator_mask = (1_u128 << Q_BITS) - 1;
    let rounded_up = numerator.saturating_add(denominator_mask) >> Q_BITS;
    rounded_up.min(DOMAIN_CARDINALITY)
}

fn strict_upper_bound(counts: &[u16; 64], table: &[u64]) -> u128 {
    let mut best = DOMAIN_CARDINALITY;

    for (index, &count) in counts.iter().enumerate() {
        let observed = usize::from(count);
        if observed >= TABLE_ENTRIES {
            continue;
        }

        let q = table[observed];
        if q == SENTINEL {
            continue;
        }

        let candidate = q32_level_upper(q, (index + 1) as u32);
        best = best.min(candidate);
    }

    best
}

fn build_sketch(d: u64) -> PublishedFpcsaF2 {
    let oracle = PublishedFpcsaOracle::new(
        0x243F_6A88_85A3_08D3,
        0x1319_8A2E_0370_7344,
        0xA409_3822_299F_31D0,
    );
    let config =
        PublishedFpcsaF2Config::new(ROWS, LEVELS, oracle).expect("valid lab config");
    assert_eq!(config.packed_state_bytes(), 32_768);

    let mut sketch = PublishedFpcsaF2::new(config);
    for key in 0..d {
        black_box(sketch.toggle(black_box(key)));
    }
    sketch
}

fn bench_counts(words: &[u64], iterations: u32) -> (u128, u64) {
    let start = Instant::now();
    let mut checksum = 0_u64;

    for _ in 0..iterations {
        let counts = level_counts(black_box(words));
        checksum ^= u64::from(black_box(counts[0]));
        checksum = checksum.rotate_left(7) ^ u64::from(black_box(counts[31]));
    }

    let elapsed = start.elapsed().as_nanos();
    (elapsed / u128::from(iterations), checksum)
}

fn bench_lookup(counts: &[u16; 64], table: &[u64], iterations: u32) -> (u128, u128) {
    let start = Instant::now();
    let mut checksum = 0_u128;

    for _ in 0..iterations {
        let bound = strict_upper_bound(black_box(counts), black_box(table));
        checksum ^= black_box(bound);
        checksum = checksum.rotate_left(11);
    }

    let elapsed = start.elapsed().as_nanos();
    (elapsed / u128::from(iterations), checksum)
}

fn bench_total(words: &[u64], table: &[u64], iterations: u32) -> (u128, u128) {
    let start = Instant::now();
    let mut checksum = 0_u128;

    for _ in 0..iterations {
        let counts = level_counts(black_box(words));
        let bound = strict_upper_bound(black_box(&counts), black_box(table));
        checksum ^= black_box(bound);
        checksum = checksum.rotate_left(13);
    }

    let elapsed = start.elapsed().as_nanos();
    (elapsed / u128::from(iterations), checksum)
}

fn main() {
    let table_path = std::env::args_os()
        .nth(1)
        .expect("usage: strict_compact_lookup_lab <q32-table.txt>");
    let table = load_table(Path::new(&table_path));

    let finite_entries = table.iter().take_while(|&&value| value != SENTINEL).count();
    assert_eq!(finite_entries, 1853);
    assert_eq!(table.len() * std::mem::size_of::<u64>(), 16_384);

    println!(
        "STRICT_COMPACT_LOOKUP_READY rows={} levels={} state_bytes={} table_bytes={} finite_entries={}",
        ROWS,
        LEVELS,
        32_768,
        table.len() * std::mem::size_of::<u64>(),
        finite_entries,
    );

    for d in [4096_u64, 65_536, 262_144] {
        let sketch = build_sketch(d);
        let words = sketch.snapshot_words();
        assert_eq!(words.len(), ROWS as usize);

        let counts = level_counts(words);
        let bound = strict_upper_bound(&counts, &table);

        if bound < u128::from(d) {
            panic!("deterministic lab bound underflow: d={d} bound={bound}");
        }

        let (counts_ns, counts_checksum) = bench_counts(words, 400);
        let (lookup_ns, lookup_checksum) = bench_lookup(&counts, &table, 20_000);
        let (total_ns, total_checksum) = bench_total(words, &table, 300);

        println!(
            "STRICT_COMPACT_LOOKUP_CASE d={} bound={} ratio={:.6} counts_ns={} lookup_ns={} total_ns={} checksums={:x}/{:x}/{:x}",
            d,
            bound,
            bound as f64 / d as f64,
            counts_ns,
            lookup_ns,
            total_ns,
            counts_checksum,
            lookup_checksum,
            total_checksum,
        );
    }

    println!("STRICT_COMPACT_LOOKUP_PASS");
}
