// DeltaGuard B1B1B1-B1B1-B0: research-only matched hot contract
// comparison. Public BLAKE3 bitmap != authentication or exact recovery.
const BH_NS: [usize; 2] = [256, 65536];
const BH_DS: [usize; 2] = [48, 57];
const BH_Q: usize = 20;
const BH_RATE: usize = 100;
const BH_APP_DELAY_MS: u64 = 0;

fn bh_seed(args: &[String]) {
    assert_eq!(args.len(), 7);
    let worker: usize = args[1].parse().unwrap();
    let n: usize = args[2].parse().unwrap();
    let d: usize = args[3].parse().unwrap();
    let rep: usize = args[4].parse().unwrap();
    let owner: u8 = args[5].parse().unwrap();
    let root = Path::new(&args[6]);
    assert!((1..=5).contains(&worker));
    assert!(BH_NS.contains(&n) && BH_DS.contains(&d) && rep < 3);
    let dir = bd_dir(root, owner);
    std::fs::create_dir_all(&dir).unwrap();
    let (tokens, base) = b11_owner_initial(worker, n, d, rep, owner);
    let frame = b1_encode(owner, B1_FULL, B1_EPOCH, 1, &b1_full_bytes(&tokens));
    bd_atomic(&dir, "base.snap", &frame);
    bd_atomic(&dir, "seed.bin", &(base + 9999).to_le_bytes());
    let file = std::fs::File::create(dir.join("chain.wal")).unwrap();
    file.sync_all().unwrap();
    std::fs::File::open(&dir).unwrap().sync_all().unwrap();
    let genesis = be_genesis(owner, &frame);
    bd_atomic(&dir, "commit.mark", &be_mark(BE_COMMIT, 1, &genesis));
    bd_atomic(&dir, "ack.mark", &be_mark(BE_ACK, 1, &genesis));
    assert_eq!(be_source_load(&dir, owner).unwrap().tokens, tokens);
}
fn bh_seed_pair(worker: usize, n: usize, d: usize, rep: usize, root: &Path) {
    use std::process::{Command, Stdio};
    std::fs::create_dir_all(root).unwrap();
    let exe = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for owner in 1..=2 {
        children.push(Command::new(&exe).args([
            "--bh-seed", &worker.to_string(), &n.to_string(),
            &d.to_string(), &rep.to_string(), &owner.to_string(),
            root.to_str().unwrap(),
        ]).stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap());
    }
    for mut child in children { assert!(child.wait().unwrap().success()); }
}
fn bh_owner(args: &[String]) {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;
    assert_eq!(args.len(), 4);
    let owner: u8 = args[1].parse().unwrap();
    let dir = Path::new(&args[2]);
    let src = be_source_load(dir, owner).expect("owner must reopen durable private WAL");
    assert_eq!(src.seq, 2);
    let init = bd_full_state(&dir.join("base.snap"), owner, 1).unwrap();
    assert_eq!(src.tokens.len(), init.len() + 1);
    let start = std::time::Instant::now();
    let mut guard = NearFullGuard::new(11, 64, &MASTER_KEY);
    for &token in &src.tokens { guard.toggle(token); }
    let build_ns = start.elapsed().as_nanos() as u64;
    let mut sock = TcpStream::connect(&args[3]).unwrap();
    sock.set_nodelay(true).unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    sock.set_write_timeout(Some(Duration::from_secs(20))).unwrap();
    // 1B owner hello + actual source build / pre-transfer VmHWM / proc ticks.
    sock.write_all(&[owner]).unwrap();
    sock.write_all(&build_ns.to_le_bytes()).unwrap();
    sock.write_all(&b11_rss().to_le_bytes()).unwrap();
    sock.write_all(&bc_cpu_ticks().to_le_bytes()).unwrap();
    loop {
        let mut request = [0_u8; BB_CMD];
        sock.read_exact(&mut request).unwrap();
        assert_eq!(&request[..8], BB_MAGIC);
        assert!(request[9..16].iter().all(|v| *v == 0));
        assert_eq!(u64::from_le_bytes(request[16..24].try_into().unwrap()), B1_EPOCH);
        let generation = u64::from_le_bytes(request[24..32].try_into().unwrap()) as usize;
        let kind = request[8];
        if kind == BB_QUIT {
            assert_eq!(generation, 2);
            sock.write_all(&bb_ack(true, generation)).unwrap();
            sock.write_all(&b11_rss().to_le_bytes()).unwrap();
            sock.write_all(&bc_cpu_ticks().to_le_bytes()).unwrap();
            break;
        }
        let payload = match (kind, generation) {
            (B1_FULL, 1) => b1_full_bytes(&init),
            (B1_FULL, 2) => b1_full_bytes(&src.tokens),
            (B1_GUARD, 2) => b1_words_bytes(&guard.words),
            _ => panic!("invalid source request version/kind"),
        };
        let frame = b1_encode(owner, kind, B1_EPOCH, generation as u64, &payload);
        std::thread::sleep(Duration::from_millis(BH_APP_DELAY_MS));
        b11_paced_write(&mut sock, &frame, BH_RATE);
        let mut ack = [0_u8; BB_ACK];
        sock.read_exact(&mut ack).unwrap();
        assert_eq!(ack, bb_ack(true, generation));
    }
}
struct BHPeers {
    sources: [BBPeer; 2],
    build: [u64; 2],
    initial_rss: [u64; 2],
    initial_ticks: [u64; 2],
}
fn bh_start(root: &Path) -> BHPeers {
    use std::io::Read;
    use std::net::TcpListener;
    use std::process::{Command, Stdio};
    use std::time::Duration;
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(false).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let exe = std::env::current_exe().unwrap();
    let mut children: [Option<std::process::Child>; 2] = [None, None];
    for owner in 1_u8..=2 {
        children[(owner - 1) as usize] = Some(Command::new(&exe)
            .args(["--bh-owner".to_string(), owner.to_string(),
                bd_dir(root, owner).to_str().unwrap().to_string(), addr.clone()])
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap());
    }
    let mut peers: [Option<BBPeer>; 2] = [None, None];
    let mut build = [0; 2];
    let mut rss = [0; 2];
    let mut ticks = [0; 2];
    for _ in 0..2 {
        let (mut sock, _) = listener.accept().unwrap();
        sock.set_nodelay(true).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
        sock.set_write_timeout(Some(Duration::from_secs(20))).unwrap();
        let mut hello = [0_u8; 25];
        sock.read_exact(&mut hello).unwrap();
        let owner = hello[0];
        assert!((1..=2).contains(&owner));
        let i = (owner - 1) as usize;
        assert!(peers[i].is_none());
        build[i] = u64::from_le_bytes(hello[1..9].try_into().unwrap());
        rss[i] = u64::from_le_bytes(hello[9..17].try_into().unwrap());
        ticks[i] = u64::from_le_bytes(hello[17..25].try_into().unwrap());
        assert!(rss[i] > 0);
        peers[i] = Some(BBPeer {socket: sock, child: children[i].take().unwrap()});
    }
    BHPeers {
        sources: [peers[0].take().unwrap(), peers[1].take().unwrap()],
        build, initial_rss: rss, initial_ticks: ticks,
    }
}
fn bh_finish(peers: &mut [BBPeer; 2]) -> ([u64; 2], [u64; 2], usize) {
    use std::io::Read;
    let mut rss = [0; 2];
    let mut ticks = [0; 2];
    for peer in peers.iter_mut() {bb_request(peer, BB_QUIT, 2);}
    for (idx, peer) in peers.iter_mut().enumerate() {
        let mut ack = [0_u8; BB_ACK];
        peer.socket.read_exact(&mut ack).unwrap();
        assert_eq!(ack, bb_ack(true, 2));
        let mut data = [0_u8; 16];
        peer.socket.read_exact(&mut data).unwrap();
        rss[idx] = u64::from_le_bytes(data[..8].try_into().unwrap());
        ticks[idx] = u64::from_le_bytes(data[8..].try_into().unwrap());
        assert!(rss[idx] > 0);
    }
    for p in peers.iter_mut() { assert!(p.child.wait().unwrap().success()); }
    (rss, ticks, 2 * (BB_CMD + BB_ACK + 16))
}
fn bh_guard_odd(packets: &[Vec<u8>; 2], seq: usize, exact: &[Vec<u64>; 2]) -> u32 {
    let words_a = b11_as_words(b1_decode(
        &packets[0], 1, B1_GUARD, B1_EPOCH, seq as u64).unwrap());
    let words_b = b11_as_words(b1_decode(
        &packets[1], 2, B1_GUARD, B1_EPOCH, seq as u64).unwrap());
    let xor: Vec<u64> = words_a.iter().zip(&words_b)
        .map(|(a, b)| a ^ b).collect();
    let k = NearFullGuard::new(11, 64, &MASTER_KEY).key;
    assert_eq!(xor, b1b_reference_diff(&exact[0], &exact[1], &k));
    xor.iter().map(|v| v.count_ones()).sum()
}
fn bh_setup(worker: usize, n: usize, d: usize, rep: usize,
    root: &Path, peers: &mut BHPeers) -> (usize, usize, usize, usize, [Vec<u64>;2])
{
    // Source base is on private fsync'd disk. Fetch the FULL gen1 bootstrap
    // physically, rather than giving the exact receiver a free fixture.
    let (frames, cold_wire, _) = bb_round(&mut peers.sources, B1_FULL, 1);
    let a = b11_as_words(b1_decode(&frames[0], 1, B1_FULL, B1_EPOCH, 1).unwrap());
    let b = b11_as_words(b1_decode(&frames[1], 2, B1_FULL, B1_EPOCH, 1).unwrap());
    assert_eq!(a.len(), n);
    assert_eq!(b.len(), n);
    let (truth_a, _) = b11_owner_initial(worker, n, d, rep, 1);
    let (truth_b, _) = b11_owner_initial(worker, n, d, rep, 2);
    assert_eq!(a, truth_a);
    assert_eq!(b, truth_b);
    assert_eq!(cold_wire, 224 + 16 * n);
    be_save_receipt(root, &[a.clone(), b.clone()], [1, 1], [
        be_source_load(&bd_dir(root,1),1).unwrap().log
            .get(..0).map(|_|be_genesis(1,&std::fs::read(bd_dir(root,1).join("base.snap")).unwrap())).unwrap(),
        be_genesis(2,&std::fs::read(bd_dir(root,2).join("base.snap")).unwrap()),
    ], [[0_u8; BE_REC];2]);
    let initial_checkpoint = bf_init(root);
    // Gen2 source state was already durably committed by two separate writer
    // OS processes; transmit/replay it with actual TCP and receiver fsync.
    let (event_wire, event_cost) = bf_normal(root, 2);
    assert_eq!(event_wire, 356);
    assert_eq!(event_cost.data_bytes, 304);
    assert_eq!(event_cost.sync_calls, 3);
    let current = bf_load(root).unwrap();
    assert_eq!(current.receipt.seq, [2,2]);
    assert_eq!(b0_diff(&current.receipt.lists[0],&current.receipt.lists[1]).len(), d);
    assert_eq!(current.receipt.lists[0],
        be_source_load(&bd_dir(root,1),1).unwrap().tokens);
    assert_eq!(current.receipt.lists[1],
        be_source_load(&bd_dir(root,2),2).unwrap().tokens);
    let raw_receipt = 440 + 16 * n;
    let logical_exact_write = raw_receipt + initial_checkpoint + 48 + 304;
    (cold_wire + 2, event_wire, logical_exact_write,
        2 + 6 + event_cost.sync_calls, current.receipt.lists)
}
fn bh_fixture(worker: usize, ni: usize, di: usize, rep: usize) {
    use std::time::Instant;
    let n = BH_NS[ni];
    let d = BH_DS[di];
    let root = std::env::temp_dir().join(format!(
        "deltameter-bh-w{worker}-n{n}-d{d}-r{rep}-pid{}",std::process::id()));
    if root.exists() {std::fs::remove_dir_all(&root).unwrap();}
    bh_seed_pair(worker, n, d, rep, &root);
    be_writer_pair(&root, 2);
    let mut peers = bh_start(&root);
    let (bootstrap, shared_event, receiver_write, receiver_syncs, exact) =
        bh_setup(worker,n,d,rep,&root,&mut peers);
    let true_d = b0_diff(&exact[0],&exact[1]).len();
    assert_eq!(true_d,d);
    let mut guard_total = 0_usize;
    let mut full_total = 0_usize;
    let mut guard_times = Vec::new();
    let mut full_times = Vec::new();
    let mut local_times = Vec::new();
    let mut under = 0;
    for q in 0..BH_Q {
        // Rotation keeps source processes identical between both modes
        // and avoids a systematic 'always measure guard first' bias.
        let guard_first = (worker + ni + di + rep + q).is_multiple_of(2);
        let mut measured_guard = (0_u32, 0_usize, 0_u128);
        let mut measured_full = (0_usize, 0_u128);
        for kind in if guard_first {[B1_GUARD,B1_FULL]} else {[B1_FULL,B1_GUARD]} {
            let start = Instant::now();
            let (packets, wire, _round_ns) = bb_round(&mut peers.sources, kind, 2);
            if kind == B1_GUARD {
                let odd = bh_guard_odd(&packets,2,&exact);
                measured_guard = (odd,wire,start.elapsed().as_nanos());
            } else {
                let a = b11_as_words(b1_decode(&packets[0],1,B1_FULL,B1_EPOCH,2).unwrap());
                let b = b11_as_words(b1_decode(&packets[1],2,B1_FULL,B1_EPOCH,2).unwrap());
                assert_eq!([a,b], exact);
                measured_full=(wire,start.elapsed().as_nanos());
            }
        }
        let hot = Instant::now();
        let hot_d = b0_diff(&exact[0], &exact[1]).len();
        let local_ns = hot.elapsed().as_nanos();
        assert_eq!(hot_d,d);
        let (odd, guard_bytes, guard_ns)=measured_guard;
        let (full_bytes, full_ns)=measured_full;
        assert!(odd<=d as u32);
        assert_eq!(guard_bytes,736);
        assert_eq!(full_bytes,240+16*n);
        guard_total+=guard_bytes;
        full_total+=full_bytes;
        under+=usize::from(odd<=48);
        guard_times.push(guard_ns);
        full_times.push(full_ns);
        local_times.push(local_ns);
        println!("B1B1B1_B1B1B0_SAMPLE worker={worker} ni={ni} di={di} rep={rep} q={} N={n} d={d} mbps={BH_RATE} delay={BH_APP_DELAY_MS} guard_bytes={guard_bytes} full_bytes={full_bytes} hot_exact_bytes=0 guard_ns={guard_ns} full_ns={full_ns} hot_exact_ns={local_ns} odd={odd} under={} exact=1",
            q+1,u8::from(odd<=48));
    }
    assert_eq!(guard_total,20*736);
    assert_eq!(full_total,20*(240+16*n));
    let (rss,ticks,quit)=bh_finish(&mut peers.sources);
    for i in 0..2 {assert!(ticks[i]>=peers.initial_ticks[i]);}
    let p95 = |vals:&mut Vec<u128>|->u128 {
        vals.sort_unstable();
        vals[18]
    };
    println!("B1B1B1_B1B1B0_SESSION worker={worker} ni={ni} di={di} rep={rep} N={n} d={d} cold_exact_bytes={bootstrap} shared_source_event_bytes={shared_event} exact_receiver_write={receiver_write} exact_receiver_sync={receiver_syncs} guard_total={guard_total} full_total={full_total} under_cutoff={under} guard_p95_ns={} full_p95_ns={} hot_exact_p95_ns={} owner1_build_ns={} owner2_build_ns={} owner1_rss={} owner2_rss={} owner1_ticks={} owner2_ticks={} quit_bytes={quit} receiver_rss={} exact=1",
        p95(&mut guard_times),p95(&mut full_times),p95(&mut local_times),
        peers.build[0],peers.build[1],rss[0],rss[1],
        ticks[0]-peers.initial_ticks[0],ticks[1]-peers.initial_ticks[1],
        b11_rss());
    assert!(peers.initial_rss.iter().all(|v|*v>0));
    std::fs::remove_dir_all(root).unwrap();
}
fn bh_worker(worker: usize) {
    assert!((1..=5).contains(&worker));
    for ni in 0..BH_NS.len() {
        for di in 0..BH_DS.len() {
            for rep in 0..3 {bh_fixture(worker,ni,di,rep);}
        }
    }
    println!("DELTAGUARD_B1B1B1_B1B1B0_WORKER_PASS worker={worker} sessions=12 queries=240");
}
