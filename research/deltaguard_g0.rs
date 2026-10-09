// Research-only DeltaGuard G0; inclusion under the private C1/J52 laboratory.
// WARNING: the deterministic MASTER_KEY in the reference harness is public.
// It is for reproducible testing, NOT deployment and NOT a PRF security proof.
const GUARD_TS: [u64; 4] = [64, 4096, 65_536, 1_048_576];
const GUARD_NUMERATORS: [u64; 5] = [0, 2, 6, 8, 9];
const GUARD_KS: [usize; 4] = [1, 2, 4, 8];
const GUARD_SEEDS: usize = 3;

struct TinyGuard {
    levels: Box<[u8]>,
    words: Box<[[u64; WORDS_PER_LEVEL]]>,
    oracle_key: [u8; 32],
}

impl TinyGuard {
    fn new(t: u64, k: usize) -> Self {
        assert!(GUARD_KS.contains(&k));
        let levels = levels_for_order("center", t);
        assert_eq!(levels.len(), J as usize);
        let levels = levels[..k].to_vec().into_boxed_slice();
        let words = vec![[0_u64; WORDS_PER_LEVEL]; k].into_boxed_slice();
        Self {
            levels,
            words,
            oracle_key: derive_key(ORACLE_CONTEXT, &MASTER_KEY),
        }
    }

    fn state_payload_bytes(&self) -> usize {
        self.words.len() * LEVEL_BYTES
    }

    fn toggle(&mut self, token: u64) {
        let hash = blake3::keyed_hash(&self.oracle_key, &token.to_le_bytes());
        let bytes = hash.as_bytes();
        let coefficient_word = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if coefficient_word & 1 == 0 || level_word == 0 {
            return;
        }
        let level = level_word.trailing_zeros() + 1;
        if let Some(i) = self.levels.iter().position(|&j| u32::from(j) == level) {
            let row_word = u64::from_le_bytes(bytes[..8].try_into().unwrap());
            let row = (row_word as usize) & (ROWS - 1);
            self.words[i][row / 64] ^= 1_u64 << (row % 64);
        }
    }

    fn xor_assign(&mut self, other: &Self) {
        assert_eq!(self.levels, other.levels);
        assert_eq!(self.oracle_key, other.oracle_key);
        assert_eq!(self.words.len(), other.words.len());
        for (dst, source) in self.words.iter_mut().zip(other.words.iter()) {
            for (a, &b) in dst.iter_mut().zip(source.iter()) {
                *a ^= b;
            }
        }
    }

    fn upper(&self, table: &[u64]) -> u128 {
        let mut best = DOMAIN_CARDINALITY;
        for (&level, words) in self.levels.iter().zip(self.words.iter()) {
            let count: usize = words.iter().map(|x| x.count_ones() as usize).sum();
            if count < TABLE_ENTRIES && table[count] != SENTINEL {
                best = best.min(q32_level_upper(table[count], u32::from(level)));
            }
        }
        best
    }

    fn compare_to_full(&self, full: &PackedSketch, table: &[u64]) {
        assert_eq!(full.levels, J);
        assert_eq!(full.layout, Layout::LevelMajor);
        assert_eq!(self.oracle_key, full.oracle_key);
        let mut expected = DOMAIN_CARDINALITY;
        for (&level, words) in self.levels.iter().zip(self.words.iter()) {
            let start = usize::from(level - 1) * WORDS_PER_LEVEL;
            assert_eq!(&full.words[start..start + WORDS_PER_LEVEL], words);
            let frozen_count: usize = full.words[start..start + WORDS_PER_LEVEL]
                .iter()
                .map(|w| w.count_ones() as usize)
                .sum();
            let from_full = if frozen_count >= TABLE_ENTRIES || table[frozen_count] == SENTINEL {
                DOMAIN_CARDINALITY
            } else {
                q32_level_upper(table[frozen_count], u32::from(level))
            };
            expected = expected.min(from_full);
        }
        assert_eq!(self.upper(table), expected);
        assert!(self.upper(table) >= full.estimate(table));
    }
}

fn guard_vectors(table: &[u64]) {
    let mut a = TinyGuard::new(65_536, 4);
    let mut b = TinyGuard::new(65_536, 4);
    let original = a.words.clone();
    assert_eq!(a.state_payload_bytes(), 2_048);
    assert_eq!(a.levels.len(), 4);
    assert_eq!(a.levels.as_ref(), &levels_for_order("center", 65_536)[..4]);
    assert_eq!(a.upper(table), DOMAIN_CARDINALITY.min(a.upper(table)));
    let p = a.words.as_ptr();
    for token in 0..4_096 {
        a.toggle(token);
        b.toggle(token);
    }
    assert_eq!(a.words.as_ptr(), p, "per-token allocation changed backing state");
    a.xor_assign(&b);
    assert_eq!(a.words, original, "XOR/cancellation law");
    assert_eq!(a.upper(table), TinyGuard::new(65_536, 4).upper(table));

    // Check profile incompatibility cannot be silently XOR-ed.
    let changed = TinyGuard::new(4_096, 4);
    assert_ne!(a.levels, changed.levels);
    assert_oracle_vectors();
}

fn guard_one(t: u64, d: u64, worker: usize, scenario: usize, ratio: usize,
             seed: usize, table: &[u64]) {
    let mut left = PackedSketch::new(J, Layout::LevelMajor);
    let mut right = PackedSketch::new(J, Layout::LevelMajor);
    let mut l: Vec<TinyGuard> = GUARD_KS.iter().map(|&k| TinyGuard::new(t,k)).collect();
    let mut r: Vec<TinyGuard> = GUARD_KS.iter().map(|&k| TinyGuard::new(t,k)).collect();
    let begin = (worker as u64 * 1000 + scenario as u64 * 100 + ratio as u64 * 10
                 + seed as u64) * 2_000_000;
    assert!(begin.checked_add(d).is_some());
    let half = d / 2;
    for token in begin..begin+half {
        left.toggle(token);
        for tiny in &mut l { tiny.toggle(token); }
    }
    for token in begin+half..begin+d {
        right.toggle(token);
        for tiny in &mut r { tiny.toggle(token); }
    }
    left.xor_assign(&right);
    let full = left.estimate(table);
    for ((mut tiny, other), &k) in l.into_iter().zip(r.iter()).zip(&GUARD_KS) {
        tiny.xor_assign(other);
        tiny.compare_to_full(&left, table);
        let value = tiny.upper(table);
        let bytes = tiny.state_payload_bytes();
        assert_eq!(bytes, k * 512);
        let safe = value <= u128::from(t);
        let violation = safe && d > t;
        println!(
            "GUARD_G0_SAMPLE worker={} scenario={} ratio={} seed={} T={} d={} k={} bytes={} pair_bytes={} full_bound={} guard_bound={} safe={} false_safe={}",
            worker,scenario,ratio,seed,t,d,k,bytes,2*bytes,full,value,
            u8::from(safe),u8::from(violation)
        );
    }
}

fn run_guard_g0(path: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    let table = load_table(path);
    assert_eq!(table.len(), TABLE_ENTRIES);
    guard_vectors(&table);
    for (scenario,t) in GUARD_TS.into_iter().enumerate() {
        for (ratio,numerator) in GUARD_NUMERATORS.into_iter().enumerate() {
            let d = t * numerator / 8;
            for seed in 0..GUARD_SEEDS {
                guard_one(t,d,worker,scenario,ratio,seed,&table);
            }
        }
    }
    println!("DELTAGUARD_G0_WORKER_PASS worker={worker}");
}
