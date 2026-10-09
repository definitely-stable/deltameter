// G1-B1 research-only bit parity guard, two distinct ORACLE versions.
// Not public; the MASTER_KEY in the inherited test harness is a public fixture.
// Certificates are produced by exact integer Python independent of Rust.
use std::collections::HashMap;
const G1B_TS: [u64; 3] = [32, 64, 256];
const G1B_NUM: [u64; 5] = [0, 2, 6, 8, 9];
const G1B_LEVELS: [u32; 4] = [1, 2, 3, 4];
const G1B_SEEDS: usize = 3;
const UNIT_ORACLE_CONTEXT: &str = "deltameter 2026-10-09 deltaguard-g1b unit GF2 research v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GuardProfile {
    Original,
    Unit,
}
impl GuardProfile {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Unit => "unit",
        }
    }
}

struct ExactBitGuard {
    level: u32,
    profile: GuardProfile,
    key: [u8; 32],
    words: Box<[u64; WORDS_PER_LEVEL]>,
}
impl ExactBitGuard {
    fn new(level: u32, profile: GuardProfile) -> Self {
        assert!(G1B_LEVELS.contains(&level));
        let context = match profile {
            GuardProfile::Original => ORACLE_CONTEXT,
            GuardProfile::Unit => UNIT_ORACLE_CONTEXT,
        };
        Self {
            level,
            profile,
            key: derive_key(context, &MASTER_KEY),
            words: Box::new([0; WORDS_PER_LEVEL]),
        }
    }

    fn toggle(&mut self, token: u64) {
        let hash = blake3::keyed_hash(&self.key, &token.to_le_bytes());
        let bytes = hash.as_bytes();
        let level_word = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if level_word == 0 || level_word.trailing_zeros() + 1 != self.level {
            return;
        }
        if self.profile == GuardProfile::Original {
            let coeff = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
            if coeff & 1 == 0 {
                return;
            }
        }
        let row = (u64::from_le_bytes(bytes[0..8].try_into().unwrap()) as usize) & (ROWS - 1);
        self.words[row / 64] ^= 1_u64 << (row % 64);
    }

    fn xor_assign(&mut self, rhs: &Self) -> Result<(), &'static str> {
        if self.profile != rhs.profile || self.level != rhs.level || self.key != rhs.key {
            return Err("incompatible-profile-level-or-key");
        }
        for (dst, source) in self.words.iter_mut().zip(rhs.words.iter()) {
            *dst ^= source;
        }
        Ok(())
    }

    fn odd_count(&self) -> u32 {
        self.words.iter().map(|x| x.count_ones()).sum()
    }

    fn assert_original_matches(&self, reference: &PackedSketch) {
        assert_eq!(self.profile, GuardProfile::Original);
        assert_eq!(reference.layout, Layout::LevelMajor);
        assert_eq!(reference.oracle_key, self.key);
        let start = (self.level as usize - 1) * WORDS_PER_LEVEL;
        assert_eq!(
            self.words.as_slice(),
            &reference.words[start..start + WORDS_PER_LEVEL]
        );
    }
}

type GuardCutoffs = HashMap<(u64, String, u32), i32>;

fn load_exact_cutoffs(path: &Path) -> GuardCutoffs {
    let content = std::fs::read_to_string(path).expect("read independently certified cutoff file");
    let mut result = HashMap::new();
    assert!(content.starts_with("# format deltameter.guard-g1b-cutoff.v1\n"));
    for line in content.lines().filter(|s| !s.starts_with('#')) {
        let items: Vec<&str> = line.split_ascii_whitespace().collect();
        assert_eq!(items.len(), 4);
        let t: u64 = items[0].parse().expect("positive T");
        let profile = items[1].to_owned();
        let level: u32 = items[2].parse().expect("level integer");
        let cutoff: i32 = items[3].parse().expect("cutoff integer");
        assert!(G1B_TS.contains(&t));
        assert!(["original", "unit"].contains(&profile.as_str()));
        assert!(G1B_LEVELS.contains(&level));
        assert!((-1..=(t as i32 + 1)).contains(&cutoff));
        assert!(result.insert((t, profile, level), cutoff).is_none());
    }
    assert_eq!(result.len(), 24);
    result
}

struct G1BCase {
    worker: usize,
    scenario: usize,
    ratio: usize,
    seed: usize,
    t: u64,
    d: u64,
}

fn run_case(case: G1BCase, cutoffs: &GuardCutoffs) {
    let G1BCase {
        worker,
        scenario,
        ratio,
        seed,
        t,
        d,
    } = case;
    let begin = (worker as u64 * 1000
        + scenario as u64 * 100
        + ratio as u64 * 10
        + seed as u64)
        * 2_000_000;
    assert!(begin.checked_add(d).is_some());
    let half = d / 2;

    // Full J52 reference is strictly a test oracle, NOT candidate maintained state.
    let mut full = PackedSketch::new(J, Layout::LevelMajor);
    let profiles = [GuardProfile::Original, GuardProfile::Unit];
    for profile in profiles {
        let mut left: Vec<ExactBitGuard> = G1B_LEVELS
            .into_iter()
            .map(|j| ExactBitGuard::new(j, profile))
            .collect();
        let mut right: Vec<ExactBitGuard> = G1B_LEVELS
            .into_iter()
            .map(|j| ExactBitGuard::new(j, profile))
            .collect();
        let pointers: Vec<*const u64> = left.iter().map(|g| g.words.as_ptr()).collect();
        for token in begin..begin + half {
            for sketch in &mut left {
                sketch.toggle(token);
            }
            if profile == GuardProfile::Original {
                full.toggle(token);
            }
        }
        for token in begin + half..begin + d {
            for sketch in &mut right {
                sketch.toggle(token);
            }
            if profile == GuardProfile::Original {
                full.toggle(token);
            }
        }
        for (i, mut first) in left.into_iter().enumerate() {
            assert_eq!(first.words.as_ptr(), pointers[i], "bitmap reallocated");
            assert_eq!(std::mem::size_of_val(first.words.as_ref()), 512);
            first.xor_assign(&right[i]).expect("compatible two-owner XOR");
            if profile == GuardProfile::Original {
                first.assert_original_matches(&full);
            }
            let cutoff = *cutoffs.get(&(t, profile.as_str().to_owned(), first.level))
                .expect("fixed predeclared profile cutoff missing");
            let odd = first.odd_count();
            let safe = cutoff >= 0 && odd <= cutoff as u32;
            println!(
                "G1B_SAMPLE worker={worker} scenario={scenario} ratio={ratio} seed={seed} T={t} d={d} profile={} level={} cutoff={cutoff} odd={odd} safe={} false_safe={} bitmap_bytes=512",
                profile.as_str(),
                first.level,
                u8::from(safe),
                u8::from(safe && d > t),
            );
        }
    }
}

fn g1b_vectors() {
    let mut orig = ExactBitGuard::new(1, GuardProfile::Original);
    let unit = ExactBitGuard::new(1, GuardProfile::Unit);
    let different_level = ExactBitGuard::new(2, GuardProfile::Original);
    assert_ne!(orig.key, unit.key);
    assert_eq!(orig.xor_assign(&unit), Err("incompatible-profile-level-or-key"));
    assert_eq!(
        orig.xor_assign(&different_level),
        Err("incompatible-profile-level-or-key")
    );
    let initial = *orig.words;
    for token in 0..1000 {
        orig.toggle(token);
        orig.toggle(token);
    }
    assert_eq!(*orig.words, initial, "repeat toggle cancellation failed");
}

fn run_g1b(path: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    let cuts = load_exact_cutoffs(path);
    g1b_vectors();
    assert_oracle_vectors();
    for (scenario, t) in G1B_TS.into_iter().enumerate() {
        for (ratio, n) in G1B_NUM.into_iter().enumerate() {
            let d = t * n / 8;
            for seed in 0..G1B_SEEDS {
                run_case(
                    G1BCase {
                        worker,
                        scenario,
                        ratio,
                        seed,
                        t,
                        d,
                    },
                    &cuts,
                );
            }
        }
    }
    println!("DELTAGUARD_G1B_WORKER_PASS worker={worker}");
}
