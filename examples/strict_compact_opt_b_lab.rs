// OPT-B research-only consumer of the exact OPT-A oracle, bitmap and Q32 inference.
// Keep the original OPT-A lab frozen: include it inside an isolated module.
#[allow(dead_code)]
mod opt_b {
    include!("strict_compact_opt_a_lab.rs");

    const PROFILES: [u32; 3] = [24, 52, 64];
    const MIXED_UPDATES: u64 = 8_192;
    const UPDATE_START: u64 = 32_768;
    const QUERY_REPEAT: usize = 128;
    const MERGE_REPEAT: usize = 128;
    const LANES: [&str; 7] = [
        "update",
        "query",
        "merge",
        "mixed_1",
        "mixed_10",
        "mixed_100",
        "mixed_1000",
    ];

    struct CachedSketch {
        canonical: PackedSketch,
        odd_counts: Box<[u16]>,
    }

    impl CachedSketch {
        fn new(j: u32) -> Self {
            let canonical = PackedSketch::new(j, Layout::LevelMajor);
            let odd_counts = vec![0_u16; j as usize].into_boxed_slice();
            Self {
                canonical,
                odd_counts,
            }
        }

        #[inline]
        fn toggle(&mut self, token: u64) {
            // Identical oracle/layout to the independent OPT-A control. Only one hash.
            let hash = blake3::keyed_hash(&self.canonical.oracle_key, &token.to_le_bytes());
            let b = hash.as_bytes();
            let row_word = u64::from_le_bytes(b[0..8].try_into().unwrap());
            let level_word = u64::from_le_bytes(b[8..16].try_into().unwrap());
            let coefficient_word = u64::from_le_bytes(b[16..24].try_into().unwrap());
            if coefficient_word & 1 == 0 || level_word == 0 {
                return;
            }
            let level = level_word.trailing_zeros() + 1;
            if level > self.canonical.levels {
                return;
            }
            let row = (row_word & (ROWS as u64 - 1)) as usize;
            let offset = (level as usize - 1) * ROWS + row;
            let word = &mut self.canonical.words[offset / 64];
            let mask = 1_u64 << (offset % 64);
            let old = *word & mask != 0;
            *word ^= mask;
            let count = &mut self.odd_counts[level as usize - 1];
            if old {
                *count = count.checked_sub(1).expect("count underflow");
            } else {
                *count = count.checked_add(1).expect("count overflow");
                assert!(*count <= ROWS as u16);
            }
        }

        fn rebuild(&mut self) {
            for (j, count) in self.odd_counts.iter_mut().enumerate() {
                let start = j * ROWS / 64;
                let sum: u32 = self.canonical.words[start..start + ROWS / 64]
                    .iter()
                    .map(|w| w.count_ones())
                    .sum();
                *count = u16::try_from(sum).unwrap();
            }
        }

        fn merge(&mut self, other: &Self) {
            self.canonical.xor_assign(&other.canonical);
            self.rebuild();
        }

        fn estimate(&self, table: &[u64]) -> u128 {
            strict_upper_bound(&self.odd_counts, table)
        }

        fn check(&self, table: &[u64]) {
            assert_eq!(self.odd_counts.as_ref(), self.canonical.level_counts());
            assert_eq!(self.estimate(table), scan_estimate(&self.canonical, table));
            assert_eq!(self.estimate(table), self.canonical.estimate(table));
        }
    }

    // Strong SCAN control: fixed stack counts, no per-query allocation.
    fn scan_estimate(core: &PackedSketch, table: &[u64]) -> u128 {
        let mut counts = [0_u16; 64];
        for j in 0..core.levels as usize {
            let start = j * ROWS / 64;
            let sum: u32 = core.words[start..start + ROWS / 64]
                .iter()
                .map(|w| w.count_ones())
                .sum();
            counts[j] = u16::try_from(sum).unwrap();
        }
        strict_upper_bound(&counts[..core.levels as usize], table)
    }

    fn populate_pair(j: u32) -> (PackedSketch, CachedSketch) {
        let mut scan = PackedSketch::new(j, Layout::LevelMajor);
        let mut cached = CachedSketch::new(j);
        for token in 0_u64..16_384 {
            scan.toggle(token);
            cached.toggle(token);
        }
        assert_eq!(scan.words, cached.canonical.words);
        (scan, cached)
    }

    fn verify(table: &[u64]) {
        assert_oracle_vectors();
        for j in PROFILES {
            let (mut scan, mut cache) = populate_pair(j);
            cache.check(table);
            let ptr = cache.canonical.words.as_ptr();
            let cptr = cache.odd_counts.as_ptr();
            let mut state = 0x91e1_0da5_8f7c_3443_u64;
            for index in 0_u64..18_000 {
                // Fresh deterministic hashed tokens, plus explicit cancellations.
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let token = if index % 97 == 0 { 42 } else { state };
                scan.toggle(token);
                cache.toggle(token);
                if index % 131 == 0 {
                    assert_eq!(scan.words, cache.canonical.words);
                    cache.check(table);
                }
            }
            assert_eq!(scan.words, cache.canonical.words);
            cache.check(table);
            let before = cache.canonical.words.to_vec();
            for token in [0, 1, 42, u64::MAX] {
                cache.toggle(token);
                cache.toggle(token);
            }
            assert_eq!(before, cache.canonical.words.to_vec());
            assert_eq!(ptr, cache.canonical.words.as_ptr());
            assert_eq!(cptr, cache.odd_counts.as_ptr());

            let (_, right) = populate_pair(j);
            let mut ctrl = PackedSketch::new(j, Layout::LevelMajor);
            ctrl.words.copy_from_slice(&cache.canonical.words);
            ctrl.xor_assign(&right.canonical);
            cache.merge(&right);
            assert_eq!(ctrl.words, cache.canonical.words);
            cache.check(table);
            cache.merge(&right);
            assert_eq!(before.as_slice(), cache.canonical.words.as_ref());
            cache.check(table);
        }
        println!("STRICT_COMPACT_OPT_B_CORRECTNESS_PASS");
    }

    fn once(j: u32, mode: &str, lane: &str, table: &[u64]) -> f64 {
        let (mut scan, mut cached) = populate_pair(j);
        let start = Instant::now();
        let count = match lane {
            "update" => {
                for token in UPDATE_START..UPDATE_START + 32_768 {
                    match mode {
                        "scan" => scan.toggle(black_box(token)),
                        "cache" => cached.toggle(black_box(token)),
                        _ => unreachable!(),
                    }
                }
                32_768_u64
            }
            "query" => {
                let mut sink = 0_u128;
                for _ in 0..QUERY_REPEAT {
                    sink ^= if mode == "scan" {
                        scan_estimate(black_box(&scan), black_box(table))
                    } else {
                        black_box(&cached).estimate(black_box(table))
                    };
                }
                black_box(sink);
                QUERY_REPEAT as u64
            }
            "merge" => {
                let (_, right) = populate_pair(j);
                for _ in 0..MERGE_REPEAT {
                    if mode == "scan" {
                        scan.xor_assign(black_box(&right.canonical));
                    } else {
                        cached.merge(black_box(&right));
                    }
                }
                MERGE_REPEAT as u64
            }
            _ => {
                let period: u64 = lane.strip_prefix("mixed_").unwrap().parse().unwrap();
                let mut sink = 0_u128;
                for offset in 0..MIXED_UPDATES {
                    let token = black_box(UPDATE_START + offset);
                    if mode == "scan" {
                        scan.toggle(token);
                    } else {
                        cached.toggle(token);
                    }
                    if (offset + 1) % period == 0 {
                        sink ^= if mode == "scan" {
                            scan_estimate(black_box(&scan), black_box(table))
                        } else {
                            black_box(&cached).estimate(black_box(table))
                        };
                    }
                }
                black_box(sink);
                MIXED_UPDATES
            }
        };
        if mode == "scan" {
            black_box(&scan.words);
        } else {
            black_box(&cached.canonical.words);
            black_box(&cached.odd_counts);
        }
        let nanos = start.elapsed().as_nanos() as f64 / count as f64;
        assert!(nanos.is_finite() && nanos > 0.0);
        nanos
    }

    pub(super) fn run(table_path: &Path, worker: usize) {
        assert!((1..=5).contains(&worker));
        let table = load_table(table_path);
        verify(&table);
        // Keep bench order paired, rotated by worker/round. Both modes share
        // the same corpus. QUERY and MERGE are charged per operation, others
        // per update.
        for round in 0..(WARMUPS + SAMPLES) {
            let mut profiles = PROFILES;
            let plen = profiles.len();
            profiles.rotate_left((worker - 1 + round) % plen);
            let mut lanes = LANES;
            let llen = lanes.len();
            lanes.rotate_left((worker - 1 + round) % llen);
            for j in profiles {
                for lane in lanes {
                    let modes = if (worker + round) % 2 == 0 {
                        ["scan", "cache"]
                    } else {
                        ["cache", "scan"]
                    };
                    for mode in modes {
                        let value = once(j, mode, lane, &table);
                        if round >= WARMUPS {
                            println!(
                                "STRICT_COMPACT_OPT_B_SAMPLE worker={} round={} J={} mode={} lane={} state_bytes={} cache_bytes={} ns={:.6}",
                                worker,
                                round - WARMUPS,
                                j,
                                mode,
                                lane,
                                ROWS * j as usize / 8,
                                if mode == "cache" { 2 * j as usize } else { 0 },
                                value,
                            );
                        }
                    }
                }
            }
        }
        println!("STRICT_COMPACT_OPT_B_WORKER_PASS worker={}", worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let table = args.next().expect("q32-table path");
    let worker: usize = args
        .next()
        .expect("worker")
        .parse()
        .expect("worker integer");
    assert!(args.next().is_none(), "unexpected arguments");
    opt_b::run(std::path::Path::new(&table), worker);
}
