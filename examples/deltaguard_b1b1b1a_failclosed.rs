// Research-only two real OS source sender reset after faults, no product API.
#[allow(dead_code)]
mod lab {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");
    include!("../research/deltaguard_b2b1_frames.rs");
    include!("../research/deltaguard_b2b1_retained.rs");
    include!("../research/deltaguard_b1b1a.rs");
    include!("../research/deltaguard_b1b1b0_persistent.rs");
    include!("../research/deltaguard_b1b1b1a_failclosed.rs");

    pub(super) fn run(args: &[String]) {
        match args.first().map(String::as_str) {
            Some("--full") => bc_full_child(args),
            Some("--fault") => bc_fault_child(args),
            Some(worker) => {
                assert_eq!(args.len(), 1);
                bc_worker(worker.parse().expect("worker index 1..5"));
            }
            None => panic!("worker index or child role required"),
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    lab::run(&args);
}
