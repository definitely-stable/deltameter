// B2A RESEARCH ONLY: one dummy b-bit slot; exactly 2^b - 1 GF(2) rows.
// The public MASTER_KEY is a deterministic fixture, NOT a deployed secret.
// Do not conflate this with G1B unit or J52; D=m+1 exact chain only.
use std::collections::HashMap;

const B2A_CONTEXT: &str = "deltameter 2026-10-09 deltaguard b2a nearfull GF2 v1";
const B2A_G1B_UNIT_CONTEXT: &str = "deltameter 2026-10-09 deltaguard-g1b unit GF2 research v1";
const B2A_TS: [u64; 3] = [32, 64, 128];
const B2A_BS: [u8; 8] = [6, 7, 8, 9, 10, 11, 12, 13];
const B2A_RATIOS: [(u64, u64); 9] = [
    (0, 1),
    (1, 4),
    (1, 2),
    (3, 4),
    (9, 10),
    (99, 100),
    (1, 1),
    (101, 100),
    (11, 10),
];
const B2A_SEEDS: usize = 3;

fn b2a_d(t: u64, ratio: (u64, u64)) -> u64 {
    let (num, den) = ratio;
    assert!(den > 0);
    if num > den {
        (t * num).div_ceil(den)
    } else {
        t * num / den
    }
}

fn b2a_slot(key: &[u8; 32], b: u8, token: u64) -> usize {
    let hash = blake3::keyed_hash(key, &token.to_le_bytes());
    let word = u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap());
    (word & ((1_u64 << b) - 1)) as usize
}

struct NearFullGuard {
    b: u8,
    t: u64,
    key: [u8; 32],
    words: Box<[u64]>,
}

impl NearFullGuard {
    fn new(b: u8, t: u64, master: &[u8; 32]) -> Self {
        assert!(B2A_BS.contains(&b));
        assert!(B2A_TS.contains(&t));
        let m = (1_usize << b) - 1;
        Self {
            b,
            t,
            key: derive_key(B2A_CONTEXT, master),
            words: vec![0; m.div_ceil(64)].into_boxed_slice(),
        }
    }

    fn m(&self) -> usize {
        (1_usize << self.b) - 1
    }

    fn payload_bytes(&self) -> usize {
        self.words.len() * 8
    }

    fn toggle(&mut self, token: u64) {
        let slot = b2a_slot(&self.key, self.b, token);
        if slot == self.m() {
            return; // the single dummy/self-loop outcome
        }
        self.words[slot / 64] ^= 1_u64 << (slot % 64);
    }

    fn xor_assign(&mut self, other: &Self) -> Result<(), &'static str> {
        if self.b != other.b || self.t != other.t || self.key != other.key {
            return Err("incompatible-b-t-or-key");
        }
        if self.words.len() != other.words.len() {
            return Err("incompatible-state-size");
        }
        for (dst, &rhs) in self.words.iter_mut().zip(other.words.iter()) {
            *dst ^= rhs;
        }
        Ok(())
    }

    fn odd_count(&self) -> u32 {
        self.words.iter().map(|x| x.count_ones()).sum()
    }

    fn padding_is_zero(&self) -> bool {
        let m = self.m();
        let used = m % 64;
        assert_eq!(used, 63);
        self.words.last().unwrap() & (1_u64 << 63) == 0
    }
}

fn b2a_vectors() {
    let key = MASTER_KEY;
    let mut a = NearFullGuard::new(6, 32, &key);
    let b = NearFullGuard::new(7, 32, &key);
    let c = NearFullGuard::new(6, 64, &key);
    let mut modified = key;
    modified[0] ^= 1;
    let foreign = NearFullGuard::new(6, 32, &modified);
    assert_ne!(a.key, foreign.key);
    assert_ne!(a.key, derive_key(B2A_G1B_UNIT_CONTEXT, &key));
    assert_ne!(a.key, derive_key(ORACLE_CONTEXT, &key));
    assert_eq!(a.xor_assign(&b), Err("incompatible-b-t-or-key"));
    assert_eq!(a.xor_assign(&c), Err("incompatible-b-t-or-key"));
    assert_eq!(a.xor_assign(&foreign), Err("incompatible-b-t-or-key"));
    let pristine = a.words.clone();
    for token in 0..1_024 {
        a.toggle(token);
        a.toggle(token);
    }
    assert_eq!(a.words, pristine);
    assert!(a.padding_is_zero());
    // Genuine pairwise overlap: shared keys cancel after independent updates.
    let mut left = NearFullGuard::new(11, 64, &key);
    let mut right = NearFullGuard::new(11, 64, &key);
    for token in 0..128 {
        left.toggle(token);
    }
    for token in 64..192 {
        right.toggle(token);
    }
    left.xor_assign(&right).unwrap();
    let mut exact = NearFullGuard::new(11, 64, &key);
    for token in 0..64 {
        exact.toggle(token);
    }
    for token in 128..192 {
        exact.toggle(token);
    }
    assert_eq!(left.words, exact.words);
    assert!(left.padding_is_zero());
    // All 2^b slots are legal, but dummy slot must never touch padding.
    for size in B2A_BS {
        let mut s = NearFullGuard::new(size, 32, &key);
        for token in 0..500 {
            s.toggle(token);
        }
        assert!(s.padding_is_zero());
        assert_eq!(s.payload_bytes(), s.m().div_ceil(64) * 8);
    }
}

type B2ACutoffs = HashMap<(u64, u8), i32>;

fn b2a_load(path: &Path) -> B2ACutoffs {
    let contents = std::fs::read_to_string(path).expect("read exact integer certificate");
    let mut lines = contents.lines();
    assert_eq!(
        lines.next(),
        Some("# deltameter.guard-b2a-nearfull-cutoffs.v1")
    );
    let mut values = HashMap::new();
    for line in lines {
        if line.starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split_ascii_whitespace().collect();
        assert_eq!(cells.len(), 4, "truncated/malformed cutoff row");
        let t: u64 = cells[0].parse().expect("T integer");
        let b: u8 = cells[1].parse().expect("b integer");
        let m: usize = cells[2].parse().expect("m integer");
        let cutoff: i32 = cells[3].parse().expect("cutoff integer");
        assert!(B2A_TS.contains(&t) && B2A_BS.contains(&b));
        assert_eq!(m, (1_usize << b) - 1);
        assert!((-1..=(t as i32)).contains(&cutoff));
        assert!(
            values.insert((t, b), cutoff).is_none(),
            "duplicate cutoff profile"
        );
    }
    assert_eq!(values.len(), B2A_TS.len() * B2A_BS.len());
    values
}

struct B2ACase {
    worker: usize,
    scenario: usize,
    ratio: usize,
    seed: usize,
    t: u64,
    d: u64,
}

fn b2a_one(case: B2ACase, cuts: &B2ACutoffs) {
    let B2ACase {
        worker,
        scenario,
        ratio,
        seed,
        t,
        d,
    } = case;
    let start = (worker as u64 * 1000
        + scenario as u64 * 100
        + ratio as u64 * 10
        + seed as u64)
        * 2_000_000;
    assert!(start.checked_add(d).is_some());
    for b in B2A_BS {
        let mut left = NearFullGuard::new(b, t, &MASTER_KEY);
        let mut right = NearFullGuard::new(b, t, &MASTER_KEY);
        let ptr = left.words.as_ptr();
        let half = d / 2;
        for token in start..start + half {
            left.toggle(token);
        }
        for token in start + half..start + d {
            right.toggle(token);
        }
        assert_eq!(left.words.as_ptr(), ptr, "source bitmap allocation changed");
        let physical_bytes = left.payload_bytes();
        left.xor_assign(&right).expect("two independent compatible owner states");
        // Independent reference: direct per-token keyed row selection into
        // fresh bitmap. It does not call the candidate toggle/XOR methods.
        let mut oracle = vec![0_u64; physical_bytes / 8];
        let slot_mask = (1_u64 << b) - 1;
        for token in start..start + d {
            let digest = blake3::keyed_hash(&left.key, &token.to_le_bytes());
            let first = u64::from_le_bytes(digest.as_bytes()[..8].try_into().unwrap());
            let slot = (first & slot_mask) as usize;
            if slot < left.m() {
                oracle[slot / 64] ^= 1_u64 << (slot % 64);
            }
        }
        assert_eq!(left.words.as_ref(), oracle.as_slice(), "direct parity oracle");
        assert!(left.padding_is_zero());
        let odd = left.odd_count();
        let cutoff = *cuts.get(&(t, b)).expect("predeclared certificate exists");
        let safe = cutoff >= 0 && odd <= cutoff as u32;
        if cutoff >= 0 && d <= cutoff as u64 {
            assert!(safe, "deterministic TRUE guarantee when d<=cutoff");
        }
        println!(
            "B2A_SAMPLE worker={worker} scenario={scenario} ratio={ratio} seed={seed} T={t} d={d} b={b} m={} cutoff={cutoff} odd={odd} safe={} false_safe={} owner_bytes={} struct_bytes={} key_bytes=32 heap_allocations=1",
            left.m(),
            u8::from(safe),
            u8::from(safe && d > t),
            physical_bytes,
            std::mem::size_of::<NearFullGuard>(),
        );
    }
}

fn run_b2a(path: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    let cutoffs = b2a_load(path);
    b2a_vectors();
    assert_oracle_vectors();
    for (scenario, t) in B2A_TS.into_iter().enumerate() {
        for (ratio, fraction) in B2A_RATIOS.into_iter().enumerate() {
            let d = b2a_d(t, fraction);
            assert_eq!(d > t, fraction.0 > fraction.1);
            for seed in 0..B2A_SEEDS {
                b2a_one(
                    B2ACase {
                        worker,
                        scenario,
                        ratio,
                        seed,
                        t,
                        d,
                    },
                    &cutoffs,
                );
            }
        }
    }
    println!("DELTAGUARD_B2A_WORKER_PASS worker={worker}");
}
