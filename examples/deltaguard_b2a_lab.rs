// DeltaGuard B2A independent oracle and two-source XOR laboratory. Research only.
#[allow(dead_code)]
mod b2a {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");

    pub(super) fn entry(path: &std::path::Path, worker: usize) {
        run_b2a(path, worker);
    }
}
fn main() {
    let mut args = std::env::args().skip(1);
    let cutoff_file = args
        .next()
        .expect("independently certified B2A cutoff TSV");
    let worker: usize = args
        .next()
        .expect("hosted worker 1..5")
        .parse()
        .expect("worker index");
    assert!(args.next().is_none());
    b2a::entry(std::path::Path::new(&cutoff_file), worker);
}
