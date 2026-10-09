// Research-only persistent source owner WAL + two-source receiver cache restart.
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

    pub(super) fn durable_entry(args: &[String]) {
        match args.first().map(String::as_str) {
            Some("--boot") => bd_boot_child(args),
            Some("--stage") => bd_stage_child(args),
            Some("--reboot") => bd_reboot_child(args),
            Some("--recover") => {
                assert_eq!(args.len(), 5);
                let worker: usize = args[1].parse().unwrap();
                let lane: usize = args[2].parse().unwrap();
                let rep: usize = args[3].parse().unwrap();
                bd_receiver_recovery(worker, lane, rep, std::path::Path::new(&args[4]));
            }
            Some(s) => {
                assert_eq!(args.len(), 1);
                bd_worker(s.parse().unwrap());
            }
            None => panic!("worker or source/receiver role required"),
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    lab::durable_entry(&args);
}
