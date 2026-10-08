// Research-only C3-A comparison; the C0/C1/C2 oracle and frames stay frozen.
#[allow(dead_code)]
mod c3 {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/strict_compact_opt_c3_transport.rs");

    pub(super) fn run(path: &std::path::Path, worker: usize) {
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
    assert!(args.next().is_none(), "unexpected argument");
    c3::run(std::path::Path::new(&q32_path), worker);
}
