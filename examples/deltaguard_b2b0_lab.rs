// RESEARCH ONLY: retained independent source maintenance vs direct exact.
#[allow(dead_code)]
mod b2b0 {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");

    pub(super) fn entry(path: &std::path::Path, worker: usize) {
        b0_run(path, worker);
    }
}
fn main() {
    let mut args = std::env::args().skip(1);
    let cutoff_path = args.next().expect("certified B2A cutoff TSV path");
    let worker: usize = args.next().expect("worker 1..5").parse().expect("integer worker");
    assert!(args.next().is_none());
    b2b0::entry(std::path::Path::new(&cutoff_path), worker);
}
