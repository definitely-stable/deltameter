// Research-only C3-A comparison; the C0/C1/C2 oracle and frames stay frozen.
#[allow(dead_code)]
mod c3 {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/strict_compact_opt_c3_transport.rs");
    include!("../research/strict_compact_opt_c3_two_party.rs");
    include!("../research/strict_compact_opt_c3_sender_first.rs");

    pub(super) fn run_c3c_entry(path: &std::path::Path, worker: usize) {
        run_c3c(path, worker);
    }

    pub(super) fn run_c3b_entry(path: &std::path::Path, worker: usize) {
        run_c3b(path, worker);
    }

    pub(super) fn run_c3_entry(path: &std::path::Path, worker: usize) {
        run_c3a(path, worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let q32_path = args.next().expect("Q32 table path");
    let worker: usize = args
        .next()
        .expect("worker 1..5")
        .parse()
        .expect("worker ID");
    let mode = args.next();
    assert!(args.next().is_none(), "unexpected argument");
    match mode.as_deref() {
        None => c3::run_c3_entry(std::path::Path::new(&q32_path), worker),
        Some("twoparty") => c3::run_c3b_entry(std::path::Path::new(&q32_path), worker),
        Some("senderfirst") => c3::run_c3c_entry(std::path::Path::new(&q32_path), worker),
        _ => panic!("unknown C3 research mode"),
    }
}
