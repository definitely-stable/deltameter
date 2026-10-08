use blake3::derive_key;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const ROWS: usize = 4096;
const J_VALUES: [u32; 6] = [24, 32, 40, 48, 56, 64];
const TABLE_ENTRIES: usize = 2048;
const SENTINEL: u64 = u64::MAX;
const Q_BITS: u32 = 32;
const DOMAIN_CARDINALITY: u128 = 1_u128 << 64;
const ORACLE_CONTEXT: &str = "deltameter 2026-10-08 strict-compact keyed oracle v1";
const MASTER_KEY: [u8; 32] = [
    0x44, 0x65, 0x6c, 0x74, 0x61, 0x4d, 0x65, 0x74, 0x65, 0x72, 0x2d, 0x53, 0x54, 0x52, 0x49, 0x43,
    0x54, 0x2d, 0x43, 0x4f, 0x4d, 0x50, 0x41, 0x43, 0x54, 0x2d, 0x30, 0x30, 0x34, 0x21, 0x21, 0x21,
];
const UPDATE_TOKENS: u64 = 131_072;
const QUERY_TOKENS: u64 = 65_536;
const WARMUPS: usize = 2;
const SAMPLES: usize = 8;
const QUERY_REPETITIONS: usize = 64;
const XOR_REPETITIONS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    RowMajor,
    LevelMajor,
}

impl Layout {
    const fn name(self) -> &'static str {
        match self {
            Self::RowMajor => "row",
            Self::LevelMajor => "level",
        }
    }
}

struct PackedSketch {
    levels: u32,
    layout: Layout,
    oracle_key: [u8; 32],
    words: Box<[u64]>,
}

impl PackedSketch {
    fn new(levels: u32, layout: Layout) -> Self {
        assert!(J_VALUES.contains(&levels));
        let bits = ROWS.checked_mul(usize::try_from(levels).unwrap()).unwrap();
        assert_eq!(bits % 64, 0);
        let words = bits / 64;
        Self {
            levels,
            layout,
            oracle_key: derive_key(ORACLE_CONTEXT, &MASTER_KEY),
            words: vec![0_u64; words].into_boxed_slice(),
        }
    }

    const fn state_bytes(&self) -> usize {
        self.words.len() * 8
    }

    #[inline]
    fn bit_offset(&self, row: usize, level: u32) -> usize {
        let zero_level = usize::try_from(level - 1).unwrap();
        match self.layout {
            Layout::RowMajor => row * usize::try_from(self.levels).unwrap() + zero_level,
            Layout::LevelMajor => zero_level * ROWS + row,
        }
    }

    #[inline]
    fn toggle(&mut self, token: u64) {
        let hash = blake3::keyed_hash(&self.oracle_key, &token.to_le_bytes());
        let bytes = hash.as_bytes();
        let row_word = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let coefficient_word = u64::from_le_bytes(bytes[16..24].try_into().unwrap());

        if coefficient_word & 1 == 0 || level_word == 0 {
            return;
        }

        let level = level_word.trailing_zeros() + 1;
        if level > self.levels {
            return;
        }

        let row = usize::try_from(row_word & ((ROWS as u64) - 1)).unwrap();
        let bit = self.bit_offset(row, level);
        self.words[bit / 64] ^= 1_u64 << (bit % 64);
    }

    #[inline]
    fn bit_is_set(&self, row: usize, level: u32) -> bool {
        let bit = self.bit_offset(row, level);
        self.words[bit / 64] & (1_u64 << (bit % 64)) != 0
    }

    fn xor_assign(&mut self, other: &Self) {
        assert_eq!(self.levels, other.levels);
        assert_eq!(self.layout, other.layout);
        assert_eq!(self.oracle_key, other.oracle_key);
        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            *left ^= *right;
        }
    }

    fn level_counts(&self) -> Vec<u16> {
        let levels = usize::try_from(self.levels).unwrap();
        let mut counts = vec![0_u16; levels];

        match self.layout {
            Layout::RowMajor => {
                for (word_index, &word) in self.words.iter().enumerate() {
                    let mut bits = word;
                    while bits != 0 {
                        let offset = bits.trailing_zeros() as usize;
                        let logical_bit = word_index * 64 + offset;
                        let level = logical_bit % levels;
                        counts[level] += 1;
                        bits &= bits - 1;
                    }
                }
            }
            Layout::LevelMajor => {
                let words_per_level = ROWS / 64;
                for (level, count) in counts.iter_mut().enumerate() {
                    let start = level * words_per_level;
                    let ones: u32 = self.words[start..start + words_per_level]
                        .iter()
                        .map(|word| word.count_ones())
                        .sum();
                    *count = u16::try_from(ones).unwrap();
                }
            }
        }

        counts
    }

    fn estimate(&self, table: &[u64]) -> u128 {
        strict_upper_bound(&self.level_counts(), table)
    }

    fn canonical_digest(&self) -> blake3::Hash {
        let mut canonical = vec![0_u8; self.state_bytes()];
        for level in 1..=self.levels {
            let zero_level = usize::try_from(level - 1).unwrap();
            for row in 0..ROWS {
                if self.bit_is_set(row, level) {
                    let bit = zero_level * ROWS + row;
                    canonical[bit / 8] |= 1_u8 << (bit % 8);
                }
            }
        }
        blake3::hash(&canonical)
    }
}

fn load_table(path: &Path) -> Vec<u64> {
    let text = std::fs::read_to_string(path).expect("read certified Q32 table");
    let table: Vec<u64> = text
        .lines()
        .map(|line| line.parse::<u64>().expect("Q32 integer"))
        .collect();
    assert_eq!(table.len(), TABLE_ENTRIES);
    table
}

fn q32_level_upper(q: u64, level_one_based: u32) -> u128 {
    if q == SENTINEL {
        return DOMAIN_CARDINALITY;
    }
    let numerator = (q as u128) << level_one_based;
    let rounded_up = numerator.saturating_add((1_u128 << Q_BITS) - 1) >> Q_BITS;
    rounded_up.min(DOMAIN_CARDINALITY)
}

fn strict_upper_bound(counts: &[u16], table: &[u64]) -> u128 {
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
        best = best.min(q32_level_upper(q, u32::try_from(index + 1).unwrap()));
    }
    best
}

fn populate(levels: u32, layout: Layout, start: u64, end: u64) -> PackedSketch {
    let mut sketch = PackedSketch::new(levels, layout);
    for token in start..end {
        sketch.toggle(token);
    }
    sketch
}

fn assert_oracle_vectors() {
    let key = derive_key(ORACLE_CONTEXT, &MASTER_KEY);
    let expected = [
        (0_u64, 3383_u64, 5_u64, 1_u64),
        (1, 2445, 1, 0),
        (2, 248, 1, 1),
        (42, 328, 1, 0),
        (u64::MAX, 1146, 2, 1),
    ];

    for (token, expected_row, expected_level, expected_coefficient) in expected {
        let hash = blake3::keyed_hash(&key, &token.to_le_bytes());
        let bytes = hash.as_bytes();
        let row = u64::from_le_bytes(bytes[0..8].try_into().unwrap()) & ((ROWS as u64) - 1);
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let level = if level_word == 0 {
            0
        } else {
            u64::from(level_word.trailing_zeros() + 1)
        };
        let coefficient = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) & 1;
        assert_eq!(
            (row, level, coefficient),
            (expected_row, expected_level, expected_coefficient)
        );
    }
}

fn correctness(levels: u32, table: &[u64]) {
    let expected_bytes = ROWS * usize::try_from(levels).unwrap() / 8;
    let mut row = PackedSketch::new(levels, Layout::RowMajor);
    let mut level = PackedSketch::new(levels, Layout::LevelMajor);
    assert_eq!(row.state_bytes(), expected_bytes);
    assert_eq!(level.state_bytes(), expected_bytes);

    let row_ptr = row.words.as_ptr();
    let level_ptr = level.words.as_ptr();
    for token in 0_u64..8192 {
        row.toggle(token);
        level.toggle(token);
    }
    assert_eq!(row.words.as_ptr(), row_ptr);
    assert_eq!(level.words.as_ptr(), level_ptr);

    for stored_level in 1..=levels {
        for stored_row in 0..ROWS {
            assert_eq!(
                row.bit_is_set(stored_row, stored_level),
                level.bit_is_set(stored_row, stored_level)
            );
        }
    }
    assert_eq!(row.level_counts(), level.level_counts());
    assert_eq!(row.estimate(table), level.estimate(table));
    assert_eq!(row.canonical_digest(), level.canonical_digest());

    let before_row = row.canonical_digest();
    let before_level = level.canonical_digest();
    row.toggle(42);
    row.toggle(42);
    level.toggle(42);
    level.toggle(42);
    assert_eq!(row.canonical_digest(), before_row);
    assert_eq!(level.canonical_digest(), before_level);

    for layout in [Layout::RowMajor, Layout::LevelMajor] {
        let mut left = populate(levels, layout, 0, 4096);
        let right = populate(levels, layout, 2048, 6144);
        let mut combined = PackedSketch::new(levels, layout);
        for token in 0_u64..4096 {
            combined.toggle(token);
        }
        for token in 2048_u64..6144 {
            combined.toggle(token);
        }
        left.xor_assign(&right);
        assert_eq!(left.words, combined.words);
    }

    println!(
        "STRICT_COMPACT_OPT_A_VECTOR J={} state_bytes={} bound={} digest={}",
        levels,
        expected_bytes,
        row.estimate(table),
        row.canonical_digest()
    );
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn benchmark_update(levels: u32, layout: Layout) -> f64 {
    let mut samples = Vec::with_capacity(SAMPLES);

    for round in 0..(WARMUPS + SAMPLES) {
        let mut sketch = PackedSketch::new(levels, layout);
        let start = Instant::now();
        for token in 0_u64..UPDATE_TOKENS {
            sketch.toggle(black_box(token));
        }
        let elapsed = start.elapsed();
        black_box(&sketch.words);

        if round >= WARMUPS {
            samples.push(elapsed.as_nanos() as f64 / UPDATE_TOKENS as f64);
        }
    }

    median(samples)
}

fn benchmark_estimate(levels: u32, layout: Layout, table: &[u64]) -> f64 {
    let sketch = populate(levels, layout, 0, QUERY_TOKENS);
    let mut samples = Vec::with_capacity(SAMPLES);

    for round in 0..(WARMUPS + SAMPLES) {
        let start = Instant::now();
        let mut sink = 0_u128;
        for _ in 0..QUERY_REPETITIONS {
            sink ^= black_box(sketch.estimate(table));
        }
        let elapsed = start.elapsed();
        black_box(sink);

        if round >= WARMUPS {
            samples.push(elapsed.as_nanos() as f64 / QUERY_REPETITIONS as f64);
        }
    }

    median(samples)
}

fn benchmark_xor(levels: u32, layout: Layout) -> f64 {
    let source = populate(levels, layout, 0, QUERY_TOKENS / 2);
    let mut samples = Vec::with_capacity(SAMPLES);
    let kib = source.state_bytes() as f64 / 1024.0;

    for round in 0..(WARMUPS + SAMPLES) {
        let mut target = populate(levels, layout, QUERY_TOKENS / 2, QUERY_TOKENS);
        let start = Instant::now();
        for _ in 0..XOR_REPETITIONS {
            target.xor_assign(black_box(&source));
        }
        let elapsed = start.elapsed();
        black_box(&target.words);

        if round >= WARMUPS {
            samples.push(elapsed.as_nanos() as f64 / (XOR_REPETITIONS as f64 * kib));
        }
    }

    median(samples)
}

fn main() {
    let table_path = std::env::args_os()
        .nth(1)
        .expect("usage: strict_compact_opt_a_lab <certified-q32-table.txt>");
    let table = load_table(Path::new(&table_path));

    assert_oracle_vectors();

    for levels in J_VALUES {
        correctness(levels, &table);
    }

    for levels in J_VALUES {
        for layout in [Layout::RowMajor, Layout::LevelMajor] {
            let state_bytes = ROWS * usize::try_from(levels).unwrap() / 8;
            let update_ns = benchmark_update(levels, layout);
            let estimate_ns = benchmark_estimate(levels, layout, &table);
            let xor_ns_per_kib = benchmark_xor(levels, layout);

            println!(
                "STRICT_COMPACT_OPT_A_CASE J={} layout={} state_bytes={} update_ns_per_token={:.6} estimate_ns={:.3} xor_ns_per_kib={:.6}",
                levels,
                layout.name(),
                state_bytes,
                update_ns,
                estimate_ns,
                xor_ns_per_kib,
            );
        }
    }

    println!("STRICT_COMPACT_OPT_A_LAB_PASS");
}
