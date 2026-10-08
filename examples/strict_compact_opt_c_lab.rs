// STRICT-COMPACT OPT-C: research-only immutable level-transfer correctness.
// Embed the already accepted OPT-A reference without changing its oracle/math.
#[allow(dead_code)]
mod opt_c {
    include!("../research/strict_compact_opt_c1_core.rs");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let table_path = args
        .next()
        .expect("usage: strict_compact_opt_c_lab q32.txt");
    match args.next().as_deref() {
        None => opt_c::run(std::path::Path::new(&table_path)),
        Some("transfer") => {
            let worker: usize = args
                .next()
                .expect("worker 1..5")
                .parse()
                .expect("worker id");
            assert!(args.next().is_none(), "unexpected argument");
            opt_c::transfer(std::path::Path::new(&table_path), worker);
        }
        _ => panic!("unknown laboratory mode"),
    }
}
