// B1B1B1-B1B1-B1A: physically exact-capable guard→FULL vs direct FULL.
// Research only. Guard S<=48 neither identifies exact tokens nor certifies d.
const BJ_LANES: [(usize, usize, usize, u64, usize); 6] = [
    (256, 48, 100, 0, 100),
    (256, 57, 10, 10, 100),
    (256, 48, 1, 50, 100),
    (65536, 48, 100, 0, 10),
    (65536, 57, 100, 10, 10),
    (65536, 57, 1, 50, 1),
];

fn bj_owner(args: &[String]) {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::{Duration, Instant};
    assert_eq!(args.len(), 6, "--bj-owner owner directory address Mbps delay");
    let owner: u8 = args[1].parse().unwrap();
    let dir = Path::new(&args[2]);
    let mbps: usize = args[4].parse().unwrap();
    let delay: u64 = args[5].parse().unwrap();
    assert!((1..=2).contains(&owner));
    assert!([1,10,100].contains(&mbps) && [0,10,50].contains(&delay));
    let src = be_source_load(dir, owner).expect("source identity MUST reopen private fsync WAL");
    assert_eq!(src.seq, 2);
    let base = bd_full_state(&dir.join("base.snap"), owner, 1).unwrap();
    assert_eq!(src.tokens.len(), base.len() + 1);
    let now = Instant::now();
    let mut guard = NearFullGuard::new(11, 64, &MASTER_KEY);
    for &token in &src.tokens { guard.toggle(token); }
    let build_ns = now.elapsed().as_nanos() as u64;
    let mut sock = TcpStream::connect(&args[3]).unwrap();
    sock.set_nodelay(true).unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    sock.set_write_timeout(Some(Duration::from_secs(60))).unwrap();
    let mut hello = [0_u8; 25];
    hello[0] = owner;
    hello[1..9].copy_from_slice(&build_ns.to_le_bytes());
    hello[9..17].copy_from_slice(&b11_rss().to_le_bytes());
    hello[17..25].copy_from_slice(&bc_cpu_ticks().to_le_bytes());
    sock.write_all(&hello).unwrap();
    loop {
        let mut request = [0_u8; BB_CMD];
        sock.read_exact(&mut request).expect("physical source request");
        assert_eq!(&request[..8], BB_MAGIC);
        assert!(request[9..16].iter().all(|v| *v == 0));
        assert_eq!(u64::from_le_bytes(request[16..24].try_into().unwrap()), B1_EPOCH);
        let seq = u64::from_le_bytes(request[24..32].try_into().unwrap()) as usize;
        let kind = request[8];
        if kind == BB_QUIT {
            assert_eq!(seq, 2);
            sock.write_all(&bb_ack(true, seq)).unwrap();
            sock.write_all(&b11_rss().to_le_bytes()).unwrap();
            sock.write_all(&bc_cpu_ticks().to_le_bytes()).unwrap();
            break;
        }
        let body = match (kind, seq) {
            (B1_FULL, 1) => b1_full_bytes(&base),
            (B1_FULL, 2) => b1_full_bytes(&src.tokens),
            (B1_GUARD, 2) => b1_words_bytes(&guard.words),
            _ => panic!("wrong owner generation/frame kind"),
        };
        let frame = b1_encode(owner, kind, B1_EPOCH, seq as u64, &body);
        std::thread::sleep(Duration::from_millis(delay));
        b11_paced_write(&mut sock, &frame, mbps);
        let mut ack = [0_u8; BB_ACK];
        sock.read_exact(&mut ack).unwrap();
        assert_eq!(ack, bb_ack(true, seq));
    }
}
fn bj_peers(root: &Path, mbps: usize, delay: u64) -> BHPeers {
    use std::io::Read;
    use std::net::TcpListener;
    use std::process::{Command, Stdio};
    use std::time::Duration;
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let exe = std::env::current_exe().unwrap();
    let mut children: [Option<std::process::Child>; 2] = [None,None];
    for owner in 1..=2 {
        children[owner - 1] = Some(Command::new(&exe)
            .args(["--bj-owner".to_string(), owner.to_string(),
                bd_dir(root,owner as u8).to_str().unwrap().to_string(),
                addr.clone(), mbps.to_string(), delay.to_string()])
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap());
    }
    let mut peers: [Option<BBPeer>; 2] = [None,None];
    let mut build = [0_u64;2];
    let mut rss = [0_u64;2];
    let mut ticks = [0_u64;2];
    for _ in 0..2 {
        let (mut sock, _) = listener.accept().unwrap();
        sock.set_nodelay(true).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
        sock.set_write_timeout(Some(Duration::from_secs(60))).unwrap();
        let mut hello = [0_u8;25];
        sock.read_exact(&mut hello).unwrap();
        let owner = hello[0];
        assert!((1..=2).contains(&owner));
        let i = (owner - 1) as usize;
        assert!(peers[i].is_none());
        build[i] = u64::from_le_bytes(hello[1..9].try_into().unwrap());
        rss[i] = u64::from_le_bytes(hello[9..17].try_into().unwrap());
        ticks[i] = u64::from_le_bytes(hello[17..25].try_into().unwrap());
        assert!(rss[i] > 0);
        peers[i] = Some(BBPeer{socket:sock,child:children[i].take().unwrap()});
    }
    BHPeers {
        sources:[peers[0].take().unwrap(),peers[1].take().unwrap()],
        build,initial_rss:rss,initial_ticks:ticks,
    }
}
fn bj_exact_frame(frames: &[Vec<u8>;2], exact: &[Vec<u64>;2], seq: usize) {
    for (i, p) in frames.iter().enumerate() {
        let found = b11_as_words(b1_decode(
            p,(i+1) as u8,B1_FULL,B1_EPOCH,seq as u64).unwrap());
        assert_eq!(found,exact[i],"actual FULL TCP body != durable source oracle");
    }
}
fn bj_p95(values: &[u128]) -> u128 {
    assert!(!values.is_empty());
    let mut v = values.to_vec();
    v.sort_unstable();
    v[(v.len()*95).div_ceil(100)-1]
}
fn bj_fixture(worker: usize, lane: usize, rep: usize) {
    use std::time::Instant;
    let (n,d,mbps,delay,qmax) = BJ_LANES[lane];
    let root=std::env::temp_dir().join(format!(
        "deltameter-bj-w{worker}-lane{lane}-r{rep}-pid{}",std::process::id()));
    if root.exists(){std::fs::remove_dir_all(&root).unwrap();}
    bh_seed_pair(worker,n,d,rep,&root);
    be_writer_pair(&root,2);
    let mut peer = bj_peers(&root,mbps,delay);
    let (cold,event,write,sync,exact)=bh_setup(worker,n,d,rep,&root,&mut peer);
    let source_d = b0_diff(&exact[0],&exact[1]).len();
    assert_eq!(source_d,d);
    let key=NearFullGuard::new(11,64,&MASTER_KEY).key;
    let independent_odd: u32=b1b_reference_diff(&exact[0],&exact[1],&key)
        .iter().map(|w|w.count_ones()).sum();
    let mut prefix_full=0_usize;
    let mut prefix_resolved=0_usize;
    let mut under=0_usize;
    let mut direct_times=Vec::with_capacity(qmax);
    let mut resolved_times=Vec::with_capacity(qmax);
    let mut local_times=Vec::with_capacity(qmax);
    for q in 1..=qmax {
        let direct_first=(worker+lane+rep+q).is_multiple_of(2);
        let mut direct_result=(0_usize,0_u128);
        let mut resolved_result=(0_usize,0_u128,0_u32);
        for do_direct in if direct_first {[true,false]}else{[false,true]} {
            let start=Instant::now();
            if do_direct {
                let (frames,bytes,_)=bb_round(&mut peer.sources,B1_FULL,2);
                bj_exact_frame(&frames,&exact,2);
                direct_result=(bytes,start.elapsed().as_nanos());
            }else{
                let (guard,gb,_)=bb_round(&mut peer.sources,B1_GUARD,2);
                let odd=bh_guard_odd(&guard,2,independent_odd);
                // The boolean guard result can NEVER supply the exact tokens.
                // Physically fetch BOTH full lists irrespective of cutoff.
                let (full,fb,_)=bb_round(&mut peer.sources,B1_FULL,2);
                bj_exact_frame(&full,&exact,2);
                resolved_result=(gb+fb,start.elapsed().as_nanos(),odd);
            }
        }
        let hot_start=Instant::now();
        let actual_diff=b0_diff(&exact[0],&exact[1]);
        let hot_ns=hot_start.elapsed().as_nanos();
        assert_eq!(actual_diff.len(),d);
        let (db,d_ns)=direct_result;
        let (rb,r_ns,odd)=resolved_result;
        assert_eq!(db,240+16*n+8*(d%2));
        assert_eq!(rb,db+736);
        assert_eq!(odd,independent_odd);
        under += usize::from(odd<=48);
        prefix_full+=db;
        prefix_resolved+=rb;
        assert_eq!(prefix_resolved-prefix_full,q*736);
        direct_times.push(d_ns);
        resolved_times.push(r_ns);
        local_times.push(hot_ns);
        println!("B1B1B1_B1B1B1A_SAMPLE worker={worker} lane={lane} rep={rep} q={q} N={n} d={d} mbps={mbps} delay={delay} direct_bytes={db} resolved_bytes={rb} direct_ns={d_ns} resolved_ns={r_ns} exact_local_ns={hot_ns} S={odd} under={} exact=1",u8::from(odd<=48));
        if q==1||q==10||q==100 {
            println!("B1B1B1_B1B1B1A_PREFIX worker={worker} lane={lane} rep={rep} q={q} N={n} d={d} mbps={mbps} delay={delay} direct_bytes={prefix_full} resolved_bytes={prefix_resolved} exact_hot_bytes=0 cold_exact_bytes={cold} source_event_bytes={event} receiver_write={write} receiver_sync={sync} exact=1");
        }
    }
    let (rss,ticks,quit)=bh_finish(&mut peer.sources);
    for (now, before) in ticks.iter().zip(&peer.initial_ticks) {
        assert!(now>=before);
    }
    let tickdelta = ticks[0]+ticks[1]-peer.initial_ticks[0]-peer.initial_ticks[1];
    println!("B1B1B1_B1B1B1A_SESSION worker={worker} lane={lane} rep={rep} N={n} d={d} mbps={mbps} delay={delay} Q={qmax} direct_bytes={prefix_full} resolved_bytes={prefix_resolved} under={under} direct_p95_ns={} resolved_p95_ns={} local_p95_ns={} source_build_ns={} source_ticks={tickdelta} source_rss={} receiver_rss={} cold_exact_bytes={cold} source_event_bytes={event} receiver_write={write} receiver_sync={sync} quit_bytes={quit} exact=1",
        bj_p95(&direct_times[..qmax.min(20)]),
        bj_p95(&resolved_times[..qmax.min(20)]),
        bj_p95(&local_times[..qmax.min(20)]),
        peer.build[0]+peer.build[1],rss[0].max(rss[1]),b11_rss());
    assert_eq!(under,if d==48 {qmax} else {0},
        "named fixtures d57 must stay UNKNOWN for this exact test");
    assert!(peer.initial_rss.iter().all(|v|*v>0));
    std::fs::remove_dir_all(root).unwrap();
}
fn bj_worker(worker: usize) {
    assert!((1..=5).contains(&worker));
    let mut queries=0;
    for (lane, config) in BJ_LANES.iter().enumerate() {
        for rep in 0..3 {
            bj_fixture(worker,lane,rep);
            queries+=config.4;
        }
    }
    assert_eq!(queries,963);
    println!("DELTAGUARD_B1B1B1_B1B1B1A_WORKER_PASS worker={worker} sessions=18 paired_queries={queries} full_reconstruction=1");
}
