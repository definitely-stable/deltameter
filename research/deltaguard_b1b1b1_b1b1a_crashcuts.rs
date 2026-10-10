// DeltaGuard B1-B1B1-B1B1-A: actual receiver process SIGKILL at
// precise fsync/watermark/checkpoint cuts; no public API.
const BG_CASES: [&str; 5] = [
    "partial_wal", "full_wal_fsync", "commit_synced",
    "commit_conflict", "checkpoint_synced",
];
fn bg_records(root: &Path, seq: usize) -> [[u8; BE_REC]; 2] {
    [1_u8, 2_u8].map(|owner| {
        let src = be_source_load(&bd_dir(root, owner), owner)
            .expect("source restart reopens fsync committed WAL only");
        assert!(src.seq >= seq);
        src.log[(seq - 2) * BE_REC..(seq - 1) * BE_REC]
            .try_into().unwrap()
    })
}
fn bg_stage_child(args: &[String]) {
    use std::io::Write;
    assert_eq!(args.len(), 4);
    let case = &args[1];
    assert!(BG_CASES.contains(&case.as_str()));
    let seq: usize = args[2].parse().unwrap();
    let root = Path::new(&args[3]);
    assert_eq!(seq, if case == "checkpoint_synced" {51} else {2});
    let before = bf_load(root).expect("receiver stage loads committed disk prefix");
    assert_eq!(before.receipt.seq, [seq - 1, seq - 1]);
    let records = bg_records(root, seq);
    let frames = [bf_event_frame(1, seq, &records[0]),
        bf_event_frame(2, seq, &records[1])];
    let (next, fresh) = be_validate_transition(
        &before.receipt, &records, &frames, seq).unwrap();
    assert!(fresh);
    let tx = bf_tx(seq, &records, &before.tx_head);
    let mut file = std::fs::OpenOptions::new().append(true)
        .open(root.join("receiver.wal")).unwrap();
    if case == "partial_wal" {
        file.write_all(&tx[..80]).unwrap();
        // Cut deliberately BEFORE File::sync_all and BEFORE commit marker.
    } else {
        file.write_all(&tx).unwrap();
        file.sync_all().unwrap();
        drop(file);
        if case != "full_wal_fsync" {
            bd_atomic(root, "receiver.commit", &bf_mark(seq, &bf_tx_head(&tx)));
            if case == "checkpoint_synced" {
                assert_eq!(seq, 51);
                let checkpoint = BFState {
                    receipt: next, tx_head: bf_tx_head(&tx),
                    checkpoint_gen: seq,
                };
                bd_atomic(root, "receiver.ckpt", &bf_encode_checkpoint(&checkpoint));
                // NO WAL truncate yet. The old committed WAL is fully present.
                assert_eq!(bf_load(root).unwrap().receipt.seq, [seq, seq]);
            }
        }
    }
    println!("BG_STAGE_REACHED case={case} seq={seq}");
    std::io::stdout().flush().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
    panic!("controller failed to deliver required SIGKILL");
}
fn bg_kill_at_cut(root: &Path, case: &str, seq: usize) {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--bg-stage", case, &seq.to_string(), root.to_str().unwrap()])
        .stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut out = BufReader::new(stdout);
    let mut line = String::new();
    out.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), format!("BG_STAGE_REACHED case={case} seq={seq}"));
    child.kill().expect("SIGKILL exact receiver child after synced cut");
    assert!(!child.wait().unwrap().success());
}
fn bg_disk_negative(root: &Path) {
    let before = bf_load(root).unwrap();
    let m = std::fs::read(root.join("receiver.commit")).unwrap();
    let mut bad = m.clone();
    bad[28] ^= 1;
    bd_atomic(root, "receiver.commit", &bad);
    assert!(bf_load(root).is_err(), "forged committed digest accepted");
    bd_atomic(root, "receiver.commit", &m);

    let wal = std::fs::read(root.join("receiver.wal")).unwrap();
    assert_eq!(wal.len(), BF_TX);
    let mut bad = wal.clone();
    bad[40] ^= 1;
    bd_atomic(root, "receiver.wal", &bad);
    assert!(bf_load(root).is_err(), "mutated COMMITTED WAL accepted");
    bd_atomic(root, "receiver.wal", &wal);
    assert_eq!(bf_load(root).unwrap().receipt.seq, before.receipt.seq);
    assert_eq!(bf_load(root).unwrap().receipt.lists, before.receipt.lists);
}
fn bg_case(worker: usize, lane: usize, rep: usize, case_idx: usize) -> (String, usize) {
    let case = BG_CASES[case_idx];
    let checkpoint = case == "checkpoint_synced";
    let seq = if checkpoint {51} else {2};
    let root = std::env::temp_dir().join(format!(
        "deltameter-bg-w{worker}-lane{lane}-rep{rep}-case{case_idx}-pid{}",
        std::process::id()));
    if root.exists() {std::fs::remove_dir_all(&root).unwrap();}
    be_init(worker, lane, rep, &root);
    bf_init(&root);
    if checkpoint {
        for g in 2..=50 {
            be_writer_pair(&root, g);
            let (bytes, cost) = bf_normal(&root, g);
            assert_eq!(bytes, 356);
            assert_eq!(cost.checkpoint_bytes, 0);
            assert_eq!(bf_load(&root).unwrap().receipt.seq, [g, g]);
        }
    }
    be_writer_pair(&root, seq);
    bg_kill_at_cut(&root, case, seq);
    let initial = bf_load(&root).expect("SIGKILL cannot fabricate committed receiver event");
    let before = initial.receipt.seq[0];
    let expected = if case_idx <= 1 {seq - 1} else {seq};
    assert_eq!(before, expected, "cutpoint changed accepted generation");
    assert_eq!(initial.receipt.seq, [expected, expected]);
    let size_before = std::fs::metadata(root.join("receiver.wal")).unwrap().len() as usize;
    let wanted = (before - initial.checkpoint_gen) * BF_TX;
    let repairs = usize::from(size_before != wanted);
    let expected_repair = usize::from(case_idx <= 1 || checkpoint);
    assert_eq!(repairs, expected_repair);
    if case == "commit_conflict" {
        let orig = bg_records(&root, seq);
        let mut altered = orig;
        altered[0][32] ^= 1;
        let frames = [bf_event_frame(1, seq, &orig[0]),
            bf_event_frame(2, seq, &orig[1])];
        assert!(be_validate_transition(&initial.receipt, &altered, &frames, seq).is_err(),
            "different payload replay at durable generation accepted");
        bg_disk_negative(&root);
    }
    let (wire, cost) = bf_process_round(&root, seq);
    assert_eq!(wire, 356, "two real owner child TCP frames and ACK");
    assert_eq!(cost.data_bytes, if before == seq {0} else {BF_TX + 48});
    let after = bf_load(&root).unwrap();
    assert_eq!(after.receipt.seq, [seq, seq]);
    be_oracle(worker, lane, rep, seq, &after.receipt);
    for owner in 1..=2 {
        let source = be_source_load(&bd_dir(&root, owner), owner).unwrap();
        assert_eq!(source.seq, seq);
        assert_eq!(source.ack, seq);
        assert_eq!(source.tokens, after.receipt.lists[(owner-1) as usize]);
        assert_eq!(source.head, after.receipt.heads[(owner-1) as usize]);
    }
    let length = std::fs::metadata(root.join("receiver.wal")).unwrap().len() as usize;
    assert_eq!(length, if checkpoint {0} else {BF_TX});
    let report = format!(
        "B1B1B1_B1B1A_CASE worker={worker} lane={lane} rep={rep} case_idx={case_idx} case={case} N={} gen={seq} before={before} after={seq} repair={repairs} wire={wire} wal={length} exact=1",
        BE_NS[lane]);
    // Physically read the EXACT disk state in an unmodified, nonmutating
    // read-path audit. No p95 win claim from a materialized hot pointer.
    if checkpoint {
        let hot = bf_load(&root).unwrap();
        for sample in 0..20 {
            let cold_start = std::time::Instant::now();
            let recovered = bf_load(&root).unwrap();
            let cold_ns = cold_start.elapsed().as_nanos();
            let hot_start = std::time::Instant::now();
            let check = std::hint::black_box(
                hot.receipt.lists[0].len() + hot.receipt.lists[1].len());
            let hot_ns = hot_start.elapsed().as_nanos();
            assert_eq!(recovered.receipt.lists, hot.receipt.lists);
            assert_eq!(check, recovered.receipt.lists[0].len()
                + recovered.receipt.lists[1].len());
            let logical_read = std::fs::metadata(root.join("receiver.ckpt")).unwrap().len()
                + std::fs::metadata(root.join("receiver.commit")).unwrap().len()
                + std::fs::metadata(root.join("receiver.wal")).unwrap().len();
            assert_eq!(logical_read as usize,
                std::fs::metadata(root.join("receiver.ckpt")).unwrap().len() as usize + 48);
            println!("B1B1B1_B1B1A_READ worker={worker} lane={lane} rep={rep} sample={sample} N={} cold_ns={cold_ns} hot_ns={hot_ns} logical_read={logical_read} exact=1",
                BE_NS[lane]);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
    (report, repairs)
}
fn bg_worker(worker: usize) {
    assert!((1..=5).contains(&worker));
    let mut cases = 0;
    let mut repaired = 0;
    for (lane, _) in BE_NS.iter().enumerate() {
        for rep in 0..3 {
            for case_idx in 0..BG_CASES.len() {
                let (log, did_repair) = bg_case(worker, lane, rep, case_idx);
                println!("{log}");
                cases += 1;
                repaired += did_repair;
            }
        }
    }
    assert_eq!(cases, 30);
    assert_eq!(repaired, 18);
    println!("DELTAGUARD_B1B1B1_B1B1A_WORKER_PASS worker={worker} cases={cases} repairs={repaired} reads=120 real_sigkill={cases}");
}
