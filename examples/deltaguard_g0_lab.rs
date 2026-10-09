// Private fixed-m DeltaGuard G0 proof/implementation lab; no crate/API surface.
#[allow(dead_code)]
mod guard {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_g0.rs");

    pub(super) fn run(path: &std::path::Path, worker: usize) {
        run_guard_g0(path, worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let table = args.next().expect("Q32 table");
    let worker: usize = args
        .next()
        .expect("worker 1..5")
        .parse()
        .expect("worker integer");
    assert!(args.next().is_none(), "unexpected argument");
    guard::run(std::path::Path::new(&table), worker);
}
