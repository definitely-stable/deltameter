// Research-only sustained dual OS source process, physically ACKed hot/hot test.
#[allow(dead_code)]
mod lab {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");
    include!("../research/deltaguard_b2b1_frames.rs");
    include!("../research/deltaguard_b2b1_retained.rs");
    include!("../research/deltaguard_b1b1a.rs");
    include!("../research/deltaguard_b1b1b0_persistent.rs");

    pub(super) fn owner(args: &[String]) {
        bb_owner(args);
    }
    pub(super) fn worker(index: usize) {
        bb_worker(index);
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--owner") {
        lab::owner(&args);
    } else {
        assert_eq!(args.len(), 1);
        let worker: usize = args[0].parse().expect("worker index 1..5");
        lab::worker(worker);
    }
}
