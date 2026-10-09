// Research-only B1-A physical TCP framing smoke. This is NOT B1-B p95 evidence.
#[allow(dead_code)]
mod lab {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_b2a.rs");
    include!("../research/deltaguard_b2b0.rs");
    include!("../research/deltaguard_b2b1_frames.rs");

    pub(super) fn entry(worker: usize) {
        b1_frame_lab(worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let worker: usize = args
        .next()
        .expect("worker index")
        .parse()
        .expect("integer worker");
    assert!(args.next().is_none());
    lab::entry(worker);
}
