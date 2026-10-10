// Research-only source-WAL-backed hot retained exact versus stateless guard.
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
    include!("../research/deltaguard_b1b1b1b0_durable.rs");
    include!("../research/deltaguard_b1b1b1a_chain.rs");
    include!("../research/deltaguard_b1b1b1_b1b0_receiver.rs");
    include!("../research/deltaguard_b1b1b1_b1b1b0_hot.rs");

    pub(super) fn entry(args: &[String]) {
        match args.first().map(String::as_str) {
            Some("--bh-seed") => bh_seed(args),
            Some("--write") => be_writer(args),
            Some("--send") => be_sender(args),
            Some("--bh-owner") => bh_owner(args),
            Some(worker) => {
                assert_eq!(args.len(), 1);
                bh_worker(worker.parse().expect("worker index 1..5"));
            }
            None => panic!("research worker or private-owner role required"),
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    lab::entry(&args);
}
