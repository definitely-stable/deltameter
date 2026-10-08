use blake3::derive_key;
use std::fmt;
use std::path::Path;

const ROWS: usize = 4096;
const LEVELS: u32 = 64;
const TABLE_ENTRIES: usize = 2048;
const SENTINEL: u64 = u64::MAX;
const Q_BITS: u32 = 32;
const DOMAIN_CARDINALITY: u128 = 1_u128 << 64;
const USEFUL_RANGE_MIN: u128 = 4096;
const IDEAL_FAILURE: f64 = 1e-6;
const Q95_WIDTH: f64 = 1.5;
const ORACLE_CONTEXT: &str = "deltameter 2026-10-08 strict-compact keyed oracle v1";
const KEY_ID_CONTEXT: &str = "deltameter 2026-10-08 strict-compact key id v1";
const MASTER_KEY: [u8; 32] = [
    0x44, 0x65, 0x6c, 0x74, 0x61, 0x4d, 0x65, 0x74, 0x65, 0x72, 0x2d, 0x53, 0x54, 0x52, 0x49, 0x43,
    0x54, 0x2d, 0x43, 0x4f, 0x4d, 0x50, 0x41, 0x43, 0x54, 0x2d, 0x30, 0x30, 0x34, 0x21, 0x21, 0x21,
];

#[derive(PartialEq, Eq)]
struct SecretKey([u8; 32]);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputModel {
    NonAdaptiveIndependentOfSecretKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeyedPrfCoverage {
    primitive: &'static str,
    context_version: u8,
    input_model: InputModel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StrictCompactEstimate {
    upper_bound: u128,
    ideal_statistical_failure_upper_bound: f64,
    coverage_model: KeyedPrfCoverage,
    useful_range_min: u128,
    useful_range_max: u128,
    domain_cardinality: u128,
    q95_width_ratio_upper_bound: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrivateError {
    IncompatibleOracleKey,
}

#[derive(PartialEq, Eq)]
struct PrivateConfig {
    oracle_key: [u8; 32],
    key_id: [u8; 16],
}

impl fmt::Debug for PrivateConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateConfig")
            .field("key_id", &self.key_id)
            .field("oracle_key", &"[REDACTED]")
            .finish()
    }
}

impl PrivateConfig {
    fn new(secret: SecretKey) -> Self {
        let oracle_key = derive_key(ORACLE_CONTEXT, &secret.0);
        let key_id_full = derive_key(KEY_ID_CONTEXT, &oracle_key);
        let mut key_id = [0_u8; 16];
        key_id.copy_from_slice(&key_id_full[..16]);
        Self { oracle_key, key_id }
    }
}

struct PrivateStrictCompact {
    config: PrivateConfig,
    words: Box<[u64]>,
}

impl fmt::Debug for PrivateStrictCompact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateStrictCompact")
            .field("config", &self.config)
            .field("state_bytes", &(self.words.len() * 8))
            .finish_non_exhaustive()
    }
}

impl PrivateStrictCompact {
    fn new(secret: SecretKey) -> Self {
        Self {
            config: PrivateConfig::new(secret),
            words: vec![0_u64; ROWS].into_boxed_slice(),
        }
    }

    #[inline]
    fn toggle(&mut self, token: u64) {
        let hash = blake3::keyed_hash(&self.config.oracle_key, &token.to_le_bytes());
        let bytes = hash.as_bytes();
        let row_word = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let coefficient_word = u64::from_le_bytes(bytes[16..24].try_into().unwrap());

        if coefficient_word & 1 == 0 || level_word == 0 {
            return;
        }

        let level = level_word.trailing_zeros() + 1;
        if level > LEVELS {
            return;
        }

        let row = (row_word & ((ROWS as u64) - 1)) as usize;
        self.words[row] ^= 1_u64 << (level - 1);
    }

    fn xor_assign(&mut self, other: &Self) -> Result<(), PrivateError> {
        if self.config.oracle_key != other.config.oracle_key {
            return Err(PrivateError::IncompatibleOracleKey);
        }
        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            *left ^= *right;
        }
        Ok(())
    }

    fn estimate(&self, table: &[u64]) -> StrictCompactEstimate {
        let counts = level_counts(&self.words);
        StrictCompactEstimate {
            upper_bound: strict_upper_bound(&counts, table),
            ideal_statistical_failure_upper_bound: IDEAL_FAILURE,
            coverage_model: KeyedPrfCoverage {
                primitive: "BLAKE3 keyed hash",
                context_version: 1,
                input_model: InputModel::NonAdaptiveIndependentOfSecretKey,
            },
            useful_range_min: USEFUL_RANGE_MIN,
            useful_range_max: u128::from(u64::MAX),
            domain_cardinality: DOMAIN_CARDINALITY,
            q95_width_ratio_upper_bound: Q95_WIDTH,
        }
    }
}

fn load_table(path: &Path) -> Vec<u64> {
    let text = std::fs::read_to_string(path).expect("read certified Q32 table");
    let table: Vec<u64> = text
        .lines()
        .map(|line| line.parse::<u64>().expect("Q32 integer"))
        .collect();
    assert_eq!(table.len(), TABLE_ENTRIES);

    let finite = table.iter().take_while(|&&value| value != SENTINEL).count();
    assert_eq!(finite, 1853);
    assert!(table[finite..].iter().all(|&value| value == SENTINEL));

    let mut previous = 0_u64;
    for &value in &table[..finite] {
        assert!(value >= previous);
        previous = value;
    }
    table
}

fn level_counts(words: &[u64]) -> [u16; 64] {
    assert_eq!(words.len(), ROWS);
    let mut counts = [0_u16; 64];
    for &word in words {
        let mut bits = word;
        while bits != 0 {
            let level = bits.trailing_zeros() as usize;
            counts[level] += 1;
            bits &= bits - 1;
        }
    }
    counts
}

fn q32_level_upper(q: u64, level_one_based: u32) -> u128 {
    if q == SENTINEL {
        return DOMAIN_CARDINALITY;
    }
    let numerator = (q as u128) << level_one_based;
    let rounded_up = numerator.saturating_add((1_u128 << Q_BITS) - 1) >> Q_BITS;
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
        best = best.min(q32_level_upper(q, (index + 1) as u32));
    }
    best
}

fn check_debug_redaction() {
    let secret = SecretKey(MASTER_KEY);
    assert_eq!(format!("{secret:?}"), "SecretKey([REDACTED])");

    let sketch = PrivateStrictCompact::new(secret);
    let debug = format!("{sketch:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("68, 101, 108, 116"));
}

fn check_merge_contract() {
    let mut left = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
    let mut right = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
    let mut combined = PrivateStrictCompact::new(SecretKey(MASTER_KEY));

    for key in [1_u64, 2, 3, 5, 8, 13, 21] {
        left.toggle(key);
        combined.toggle(key);
    }
    for key in [3_u64, 5, 34, 55, 89] {
        right.toggle(key);
        combined.toggle(key);
    }
    left.xor_assign(&right).unwrap();
    assert_eq!(left.words, combined.words);

    let mut wrong_key = MASTER_KEY;
    wrong_key[31] ^= 0x80;
    let other = PrivateStrictCompact::new(SecretKey(wrong_key));
    assert_ne!(left.config.key_id, other.config.key_id);
    assert_eq!(
        left.xor_assign(&other),
        Err(PrivateError::IncompatibleOracleKey)
    );
}

fn check_toggle_cancellation() {
    let mut sketch = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
    let before = sketch.words.clone();
    for key in 0_u64..4096 {
        sketch.toggle(key);
        sketch.toggle(key);
    }
    assert_eq!(sketch.words, before);
}

fn check_domain_ceiling(table: &[u64]) {
    let saturated = [u16::try_from(ROWS).unwrap(); 64];
    assert_eq!(strict_upper_bound(&saturated, table), DOMAIN_CARDINALITY);
    assert_eq!(DOMAIN_CARDINALITY, u128::from(u64::MAX) + 1);
}

fn check_reference_vectors() {
    let sketch = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
    assert_eq!(
        sketch.config.key_id,
        [
            0x10, 0x9c, 0x85, 0x91, 0xb5, 0xe0, 0x02, 0x24, 0x12, 0x53, 0x49, 0x12, 0x9e, 0xd0,
            0xe5, 0xeb,
        ]
    );

    let expected = [
        (0_u64, 3383_u64, 5_u64, 1_u64),
        (1, 2445, 1, 0),
        (2, 248, 1, 1),
        (42, 328, 1, 0),
        (u64::MAX, 1146, 2, 1),
    ];

    for (token, expected_row, expected_level, expected_coefficient) in expected {
        let hash = blake3::keyed_hash(&sketch.config.oracle_key, &token.to_le_bytes());
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
        println!(
            "STRICT_COMPACT_PRIVATE_VECTOR token={token} row={row} level={level} coefficient={coefficient}"
        );
    }
}

fn main() {
    let table_path = std::env::args_os()
        .nth(1)
        .expect("usage: strict_compact_private_api_lab <certified-q32-table.txt>");
    let table = load_table(Path::new(&table_path));

    check_debug_redaction();
    check_toggle_cancellation();
    check_merge_contract();
    check_domain_ceiling(&table);

    let empty = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
    let empty_estimate = empty.estimate(&table);
    assert!(empty_estimate.upper_bound <= DOMAIN_CARDINALITY);
    assert_eq!(empty_estimate.useful_range_min, USEFUL_RANGE_MIN);
    assert_eq!(empty_estimate.useful_range_max, u128::from(u64::MAX));
    assert_eq!(empty_estimate.domain_cardinality, DOMAIN_CARDINALITY);
    assert_eq!(
        empty_estimate.coverage_model.input_model,
        InputModel::NonAdaptiveIndependentOfSecretKey
    );

    for d in [4096_u64, 65_536, 262_144] {
        let mut sketch = PrivateStrictCompact::new(SecretKey(MASTER_KEY));
        for token in 0..d {
            sketch.toggle(token);
        }
        let estimate = sketch.estimate(&table);
        assert!(estimate.upper_bound <= DOMAIN_CARDINALITY);
        println!(
            "STRICT_COMPACT_PRIVATE_ESTIMATE d={} upper={} ratio={:.6} ideal_delta={} key_id={:02x?}",
            d,
            estimate.upper_bound,
            estimate.upper_bound as f64 / d as f64,
            estimate.ideal_statistical_failure_upper_bound,
            sketch.config.key_id,
        );
    }

    check_reference_vectors();
    println!(
        "STRICT_COMPACT_PRIVATE_API_PASS state_bytes={} table_bytes={} domain_cardinality={} useful_min={} q95_width={}",
        ROWS * 8,
        TABLE_ENTRIES * 8,
        DOMAIN_CARDINALITY,
        USEFUL_RANGE_MIN,
        Q95_WIDTH,
    );
}
