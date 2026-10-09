// Research-only receiver retained-exact, measured TCP wire and true fallback.
// No public API, no claim of authenticated parties or product p95.
#[allow(dead_code)]
mod lab {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");
    include!("../research/deltaguard_b2b1_frames.rs");
    include!("../research/deltaguard_b2b1_retained.rs");

    pub(super) fn entry(worker: usize) {
        b1b_worker(worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let worker: usize = args
        .next()
        .expect("worker index 1..5")
        .parse()
        .expect("integer worker");
    assert!(args.next().is_none());
    lab::entry(worker);
}
