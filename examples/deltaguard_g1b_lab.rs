// Research-only exact fixed-d single-level DeltaGuard G1-B1.
#[allow(dead_code)]
mod g1b {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_g1b.rs");

    pub(super) fn entry(path: &std::path::Path, worker: usize) {
        run_g1b(path, worker);
    }
}
fn main() {
    let mut args = std::env::args().skip(1);
    let cutoff_file = args
        .next()
        .expect("independently certified G1B cutoffs TSV");
    let worker: usize = args
        .next()
        .expect("worker 1..5")
        .parse()
        .expect("worker number");
    assert!(args.next().is_none());
    g1b::entry(std::path::Path::new(&cutoff_file), worker);
}
