// G1-A research-only paired alpha=delta/k Q32 experiment using G0 physical TinyGuard.
// The fixture key is public; this is NOT an adaptive-token/security or Monte Carlo proof.
const G1_TS: [u64; 4] = [32, 64, 256, 4096];
const G1_NUM: [u64; 5] = [0, 2, 6, 8, 9];
const G1_KS: [usize; 3] = [1, 2, 4];
const G1_SEEDS: usize = 3;

struct G1Case {
    t: u64,
    d: u64,
    worker: usize,
    scenario: usize,
    ratio: usize,
    seed: usize,
}

fn g1_one(case: G1Case, old: &[u64], new: &[Vec<u64>; 3]) {
    let G1Case {
        t, d, worker, scenario, ratio, seed,
    } = case;
    let begin = (worker as u64 * 1000
        + scenario as u64 * 100
        + ratio as u64 * 10
        + seed as u64)
        * 2_000_000;
    assert!(begin.checked_add(d).is_some());
    let half = d / 2;
    let mut full_left = PackedSketch::new(J, Layout::LevelMajor);
    let mut full_right = PackedSketch::new(J, Layout::LevelMajor);
    let mut tiny_left: Vec<TinyGuard> =
        G1_KS.iter().map(|&k| TinyGuard::new(t, k)).collect();
    let mut tiny_right: Vec<TinyGuard> =
        G1_KS.iter().map(|&k| TinyGuard::new(t, k)).collect();

    for token in begin..begin + half {
        full_left.toggle(token);
        for sk in &mut tiny_left {
            sk.toggle(token);
        }
    }
    for token in begin + half..begin + d {
        full_right.toggle(token);
        for sk in &mut tiny_right {
            sk.toggle(token);
        }
    }
    full_left.xor_assign(&full_right);
    let full_old = full_left.estimate(old);
    for ((mut left, right), (&k, table)) in tiny_left
        .into_iter()
        .zip(tiny_right.iter())
        .zip(G1_KS.iter().zip(new.iter()))
    {
        assert_eq!(left.levels.as_ref(), right.levels.as_ref());
        left.xor_assign(right);
        left.compare_to_full(&full_left, old);
        let old_upper = left.upper(old);
        let new_upper = left.upper(table);
        assert!(new_upper <= old_upper, "the relaxed-alpha bound became worse");
        assert!(old_upper >= full_old);
        assert_eq!(left.state_payload_bytes(), k * 512);
        let safe_old = old_upper <= u128::from(t);
        let safe_new = new_upper <= u128::from(t);
        assert!(!safe_old || safe_new, "new tighter bound lost old safe reply");
        let bad_new = safe_new && d > t;
        println!(
            "G1A_SAMPLE worker={} scenario={} ratio={} seed={} T={} d={} k={} owner_bytes={} old_bound={} new_bound={} old_safe={} new_safe={} false_safe={}",
            worker,
            scenario,
            ratio,
            seed,
            t,
            d,
            k,
            left.state_payload_bytes(),
            old_upper,
            new_upper,
            u8::from(safe_old),
            u8::from(safe_new),
            u8::from(bad_new),
        );
    }
}

fn run_g1a(data_dir: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    let old = load_table(&data_dir.join("q32-base.txt"));
    let new: [Vec<u64>; 3] = [
        load_table(&data_dir.join("q32-k1.txt")),
        load_table(&data_dir.join("q32-k2.txt")),
        load_table(&data_dir.join("q32-k4.txt")),
    ];
    guard_vectors(&old);
    for (i, t) in G1_TS.into_iter().enumerate() {
        for (ratio, number) in G1_NUM.into_iter().enumerate() {
            let d = t * number / 8;
            for seed in 0..G1_SEEDS {
                g1_one(G1Case { t, d, worker, scenario: i, ratio, seed }, &old, &new);
            }
        }
    }
    println!("DELTAGUARD_G1A_WORKER_PASS worker={worker}");
}
