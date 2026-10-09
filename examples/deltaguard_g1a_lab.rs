// Research-only G1-A per-k alpha calibration comparison; never public.
#[allow(dead_code)]
mod g1a {
    include!("../research/strict_compact_opt_c1_core.rs");
    include!("../research/deltaguard_g0.rs");
    include!("../research/deltaguard_g1a.rs");

    pub(super) fn entry(path: &std::path::Path, worker: usize) {
        run_g1a(path, worker);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("G1A table certificate directory");
    let worker: usize = args
        .next()
        .expect("worker 1..5")
        .parse()
        .expect("worker integer");
    assert!(args.next().is_none(), "unexpected argument");
    g1a::entry(std::path::Path::new(&dir), worker);
}
