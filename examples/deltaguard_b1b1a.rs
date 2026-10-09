// Research-only two independent OS processes physical TCP benchmark.
#[allow(dead_code)]
mod lab {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");
    include!("../research/deltaguard_b2b1_frames.rs");
    include!("../research/deltaguard_b2b1_retained.rs");
    include!("../research/deltaguard_b1b1a.rs");

    pub(super) fn sender(args: &[String]) {
        b11_sender(args);
    }

    pub(super) fn worker(index: usize) {
        b11_worker(index);
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--sender") {
        lab::sender(&args);
    } else {
        assert_eq!(args.len(), 1);
        let worker: usize = args[0].parse().expect("worker index");
        lab::worker(worker);
    }
}
