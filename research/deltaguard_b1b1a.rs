// B1-B1-A RESEARCH ONLY. Two distinct sender OS processes, one receiver process.
// Source-public fixture, process TCP and app pacing; NOT WAN or authenticated peers.
const B11_LANES: [(usize,usize,usize,usize,u64);4] = [
    (256,48,10,10,0),
    (256,48,100,10,10),
    (256,57,10,10,50),
    (65536,48,10,100,10),
];
const B11_REPEATS:usize=3;
const B11_HELLO:usize=25;
const B11_REQ:usize=24;

#[derive(Clone,Copy,PartialEq,Eq)]
enum B11Mode {Guard,Full,ColdExact,WarmExact,Resolved}
impl B11Mode {
    fn label(self)->&'static str {
        match self {
            Self::Guard=>"guard",Self::Full=>"full",
            Self::ColdExact=>"cold_exact",Self::WarmExact=>"warm_exact",
            Self::Resolved=>"resolved",
        }
    }
    fn all()->[Self;5] {
        [Self::Guard,Self::Full,Self::ColdExact,Self::WarmExact,Self::Resolved]
    }
    fn parse(s:&str)->Self {
        Self::all().into_iter().find(|x| x.label()==s).expect("mode")
    }
}
fn b11_rss()->u64 {
    let s=std::fs::read_to_string("/proc/self/status").expect("Linux hosted /proc status");
    let line=s.lines().find(|l|l.starts_with("VmHWM:")).expect("VmHWM");
    line.split_ascii_whitespace().nth(1).unwrap().parse::<u64>().unwrap()*1024
}
fn b11_owner_initial(worker:usize,n:usize,d:usize,rep:usize,owner:u8)->(Vec<u64>,u64) {
    assert!((1..=2).contains(&owner));
    let ni=if n==256 {0}else{1};
    let di=B1B_DS.iter().position(|x|*x==d).expect("named d");
    let base=worker as u64*10_000_000_000
        +ni as u64*2_000_000_000
        +di as u64*300_000_000
        +rep as u64*20_000_000;
    let v=if owner==1 {
        (0..n as u64).map(|x|base+x).collect()
    }else{
        let mut v:Vec<u64>=(0..(n-d/2) as u64).map(|x|base+x).collect();
        v.extend((0..d.div_ceil(2) as u64).map(|x|base+n as u64+x));
        v
    };
    (v,base+1_000_000)
}
fn b11_paced_write(stream:&mut std::net::TcpStream,bytes:&[u8],mbps:usize) {
    use std::io::Write;
    let start=std::time::Instant::now();
    let mut written=0_usize;
    for chunk in bytes.chunks(1024) {
        let target=std::time::Duration::from_secs_f64(
            (written+chunk.len()) as f64*8.0/(mbps as f64*1_000_000.0));
        if let Some(left)=target.checked_sub(start.elapsed()) {
            std::thread::sleep(left);
        }
        stream.write_all(chunk).expect("write paced framed response");
        written+=chunk.len();
    }
}
fn b11_req(seq:usize)->[u8;B11_REQ] {
    let mut b=[0_u8;B11_REQ];
    b[..8].copy_from_slice(b"DGBPERF1");
    b[8..16].copy_from_slice(&B1_EPOCH.to_le_bytes());
    b[16..24].copy_from_slice(&(seq as u64).to_le_bytes());
    b
}
fn b11_sender(args:&[String]) {
    use std::io::{Read,Write};
    use std::net::TcpStream;
    use std::time::{Duration,Instant};
    assert_eq!(args.len(),11,"sender worker N d rep S mbps delay owner mode addr");
    let worker:usize=args[1].parse().unwrap();
    let n:usize=args[2].parse().unwrap();
    let d:usize=args[3].parse().unwrap();
    let rep:usize=args[4].parse().unwrap();
    let s:usize=args[5].parse().unwrap();
    let mbps:usize=args[6].parse().unwrap();
    let delay:u64=args[7].parse().unwrap();
    let owner:u8=args[8].parse().unwrap();
    let mode=B11Mode::parse(&args[9]);
    let addr=&args[10];
    let now=Instant::now();
    let (initial,next)=b11_owner_initial(worker,n,d,rep,owner);
    let build_ns=now.elapsed().as_nanos() as u64;
    let now=Instant::now();
    let mut current=initial.clone();
    let mut bitmap=if matches!(mode,B11Mode::Guard|B11Mode::Resolved) {
        let mut guard=NearFullGuard::new(11,64,&MASTER_KEY);
        for &v in &initial {guard.toggle(v);}
        Some(guard)
    }else{None};
    let mut events=Vec::new();
    for generation in 2..=s {
        let token=next+generation as u64;
        current.push(token);
        events.extend_from_slice(&b1b_insert(token));
        if let Some(ref mut sketch)=bitmap {sketch.toggle(token);}
    }
    let update_ns=now.elapsed().as_nanos() as u64;
    let mut stream=TcpStream::connect(addr).expect("sender connects localhost");
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
    let mut hello=[0_u8;B11_HELLO];
    hello[0]=owner;
    hello[1..9].copy_from_slice(&b11_rss().to_le_bytes());
    hello[9..17].copy_from_slice(&build_ns.to_le_bytes());
    hello[17..25].copy_from_slice(&update_ns.to_le_bytes());
    stream.write_all(&hello).unwrap();
    let mut req=[0_u8;B11_REQ];
    stream.read_exact(&mut req).expect("first receiver request");
    let first_seq=if matches!(mode,B11Mode::ColdExact|B11Mode::WarmExact){1}else{s};
    assert_eq!(req,b11_req(first_seq));
    std::thread::sleep(Duration::from_millis(delay));
    let (kind,seq,payload) = match mode {
        B11Mode::Guard|B11Mode::Resolved =>
            (B1_GUARD,s,b1_words_bytes(&bitmap.as_ref().unwrap().words)),
        B11Mode::Full=>(B1_FULL,s,b1_full_bytes(&current)),
        B11Mode::ColdExact|B11Mode::WarmExact =>
            (B1_FULL,1,b1_full_bytes(&initial)),
    };
    b11_paced_write(&mut stream,&b1_encode(owner,kind,B1_EPOCH,seq as u64,&payload),mbps);
    if matches!(mode,B11Mode::ColdExact|B11Mode::WarmExact) {
        stream.read_exact(&mut req).expect("delta request");
        assert_eq!(req,b11_req(s));
        std::thread::sleep(Duration::from_millis(delay));
        b11_paced_write(&mut stream,&b1_encode(owner,B1_DELTA,B1_EPOCH,s as u64,&events),mbps);
    }else if mode==B11Mode::Resolved {
        // SAFE requests are terminal; receiver closes both streams.
        match stream.read_exact(&mut req) {
            Ok(())=>{
                assert_eq!(req,b11_req(s));
                std::thread::sleep(Duration::from_millis(delay));
                b11_paced_write(&mut stream,
                    &b1_encode(owner,B1_FULL,B1_EPOCH,s as u64,&b1_full_bytes(&current)),mbps);
            }
            Err(e) if e.kind()==std::io::ErrorKind::UnexpectedEof=>(),
            Err(e)=>panic!("resolved request read: {e}"),
        }
    }
}
struct B11Peer {
    socket:std::net::TcpStream,
    child:std::process::Child,
    rss:u64,
    build_ns:u64,
    update_ns:u64,
}
fn b11_recv(peer:&mut B11Peer,owner:u8,kind:u8,seq:usize)->(Vec<u8>,usize) {
    use std::io::Read;
    let mut header=[0_u8;B1_HEADER];
    peer.socket.read_exact(&mut header).expect("complete physical header");
    let len=u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
    assert!(len<=B1_MAX_PAYLOAD,"oversized response");
    let mut packet=header.to_vec();
    packet.resize(B1_HEADER+len,0);
    peer.socket.read_exact(&mut packet[B1_HEADER..]).expect("complete physical payload");
    b1_decode(&packet,owner,kind,B1_EPOCH,seq as u64).expect("fully bound physical frame");
    let size=packet.len();
    (packet,size)
}
fn b11_send_request(peer:&mut B11Peer,seq:usize) {
    use std::io::Write;
    peer.socket.write_all(&b11_req(seq)).expect("real receiver request");
}
fn b11_peers(worker:usize,n:usize,d:usize,rep:usize,s:usize,mbps:usize,delay:u64,mode:B11Mode)
 -> [B11Peer;2] {
    use std::io::Read;
    use std::net::TcpListener;
    use std::process::{Command,Stdio};
    use std::time::Duration;
    let listener=TcpListener::bind(("127.0.0.1",0)).expect("listener");
    let addr=listener.local_addr().unwrap().to_string();
    let executable=std::env::current_exe().unwrap();
    let mut children:[Option<std::process::Child>;2]=[None,None];
    for owner in 1..=2 {
        let child=Command::new(&executable)
            .args(["--sender".to_string(),worker.to_string(),n.to_string(),
                d.to_string(),rep.to_string(),s.to_string(),mbps.to_string(),
                delay.to_string(),owner.to_string(),mode.label().to_string(),addr.clone()])
            .stdout(Stdio::null()).stderr(Stdio::inherit())
            .spawn().expect("spawn distinct sender process");
        children[(owner-1) as usize]=Some(child);
    }
    let mut peers:[Option<B11Peer>;2]=[None,None];
    for _ in 0..2 {
        let (mut stream,_)=listener.accept().expect("receive physical sender");
        stream.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
        stream.set_nodelay(true).unwrap();
        let mut hello=[0_u8;B11_HELLO];
        stream.read_exact(&mut hello).expect("complete process hello");
        let owner=hello[0];
        assert!((1..=2).contains(&owner));
        let slot=(owner-1) as usize;
        assert!(peers[slot].is_none(),"duplicate child owner");
        let rss=u64::from_le_bytes(hello[1..9].try_into().unwrap());
        let build_ns=u64::from_le_bytes(hello[9..17].try_into().unwrap());
        let update_ns=u64::from_le_bytes(hello[17..25].try_into().unwrap());
        // Children are indexed by owner, regardless of accept ordering.
        let child=children[slot].take().expect("child identity");
        peers[slot]=Some(B11Peer {socket:stream,child,rss,build_ns,update_ns});
    }
    [peers[0].take().unwrap(),peers[1].take().unwrap()]
}
fn b11_as_words(payload:&[u8])->Vec<u64> {
    payload.as_chunks::<8>().0.iter().map(|c|u64::from_le_bytes(*c)).collect()
}
fn b11_trial(worker:usize,lane:usize,rep:usize,mode:B11Mode) {
    use std::time::Instant;
    let (n,d,s,mbps,delay)=B11_LANES[lane];
    let (initial_a,initial_b,next)=b1b_inputs(worker,
        if n==256{0}else{1},
        B1B_DS.iter().position(|x|*x==d).unwrap(),rep);
    let mut final_a=initial_a.clone();
    let mut final_b=initial_b.clone();
    for gen in 2..=s {
        final_a.push(next+gen as u64);
        final_b.push(next+gen as u64);
    }
    assert_eq!(b0_diff(&final_a,&final_b).len(),d);
    let started=Instant::now();
    let mut peers=b11_peers(worker,n,d,rep,s,mbps,delay,mode);
    let mut bytes=2*B11_HELLO;
    let mut bootstrap_bytes=0_usize;
    let mut query_start=started;
    let first_seq=if matches!(mode,B11Mode::ColdExact|B11Mode::WarmExact){1}else{s};
    for p in &mut peers {b11_send_request(p,first_seq);bytes+=B11_REQ;}
    let kind=match mode {
        B11Mode::Guard|B11Mode::Resolved=>B1_GUARD,
        _=>B1_FULL,
    };
    let mut first=Vec::new();
    for (idx,p) in peers.iter_mut().enumerate() {
        let (packet,count)=b11_recv(p,(idx+1) as u8,kind,first_seq);
        first.push(packet);
        bytes+=count;
    }
    let mut odd=0_u32;
    let mut safe=2_u8; // exact-only modes have no probabilistic classification.
    let mut fallback=0_u8;
    if kind==B1_GUARD {
        let a=b1_decode(&first[0],1,B1_GUARD,B1_EPOCH,s as u64).unwrap();
        let b=b1_decode(&first[1],2,B1_GUARD,B1_EPOCH,s as u64).unwrap();
        let words_a=b11_as_words(a);
        let words_b=b11_as_words(b);
        let merged:Vec<u64>=words_a.iter().zip(words_b.iter()).map(|(a,b)|a^b).collect();
        assert_eq!(merged,b1b_reference_diff(&final_a,&final_b,&NearFullGuard::new(11,64,&MASTER_KEY).key));
        odd=merged.iter().map(|x|x.count_ones()).sum();
        safe=u8::from(odd<=48);
        if d<=48 {assert_eq!(safe,1);}
        if mode==B11Mode::Resolved && safe==0 {
            fallback=1;
            for p in &mut peers {b11_send_request(p,s);bytes+=B11_REQ;}
            for (idx,p) in peers.iter_mut().enumerate() {
                let (full,count)=b11_recv(p,(idx+1) as u8,B1_FULL,s);
                let tokens=b11_as_words(b1_decode(&full,(idx+1) as u8,B1_FULL,B1_EPOCH,s as u64).unwrap());
                let expected=if idx==0{final_a.as_slice()}else{final_b.as_slice()};
                assert_eq!(tokens,expected);
                bytes+=count;
            }
        }
    }else {
        let a=b11_as_words(b1_decode(&first[0],1,B1_FULL,B1_EPOCH,first_seq as u64).unwrap());
        let b=b11_as_words(b1_decode(&first[1],2,B1_FULL,B1_EPOCH,first_seq as u64).unwrap());
        if mode==B11Mode::Full {
            assert_eq!(a,final_a);
            assert_eq!(b,final_b);
        }else {
            assert_eq!(a,initial_a);
            assert_eq!(b,initial_b);
            bootstrap_bytes=bytes;
            if mode==B11Mode::WarmExact {
                // Warm exact receiver has actually received full source data,
                // but the clock and QUERY bytes begin only after bootstrapping.
                query_start=Instant::now();
                bytes=0;
            }
            for p in &mut peers {b11_send_request(p,s);bytes+=B11_REQ;}
            let mut receiver=[a,b];
            for (idx,p) in peers.iter_mut().enumerate() {
                let (event,count)=b11_recv(p,(idx+1) as u8,B1_DELTA,s);
                let body=b1_decode(&event,(idx+1) as u8,B1_DELTA,B1_EPOCH,s as u64).unwrap();
                b1b_apply(&mut receiver[idx],body).expect("atomic canonical replay");
                bytes+=count;
            }
            assert_eq!(receiver[0],final_a);
            assert_eq!(receiver[1],final_b);
        }
    }
    let query_ns=query_start.elapsed().as_nanos();
    let all_ns=started.elapsed().as_nanos();
    let child_rss_a=peers[0].rss;
    let child_rss_b=peers[1].rss;
    let child_init_ns=peers.iter().map(|p|p.build_ns as u128).sum::<u128>();
    let child_update_ns=peers.iter().map(|p|p.update_ns as u128).sum::<u128>();
    drop(first);
    for p in &mut peers {
        // Closing the receiver socket terminates SAFE/no-fallback child waits.
        let _=p.socket.shutdown(std::net::Shutdown::Both);
    }
    for p in &mut peers {
        assert!(p.child.wait().expect("wait child process").success(),"sender failed");
    }
    let parent_rss=b11_rss();
    assert!(bytes>0 && query_ns>0 && all_ns>0);
    if mode==B11Mode::Resolved && fallback==1 {
        assert!(bytes>2*B11_HELLO+2*B11_REQ+640+2*B11_REQ);
    }
    let prep=if mode==B11Mode::WarmExact{bootstrap_bytes}else{0};
    println!("B1B1A_SAMPLE worker={worker} lane={lane} rep={rep} N={n} d={d} S={s} mbps={mbps} delay={delay} mode={} bytes={bytes} warm_setup_bytes={prep} wall_ns={query_ns} total_ns={all_ns} rss_parent={parent_rss} rss_owner1={child_rss_a} rss_owner2={child_rss_b} source_build_ns={child_init_ns} source_update_ns={child_update_ns} odd={odd} safe={safe} fallback={fallback} exact_verified=1",mode.label());
}
fn b11_worker(worker:usize) {
    assert!((1..=5).contains(&worker));
    let mut records=0;
    for lane in 0..B11_LANES.len(){
        for rep in 0..B11_REPEATS{
            let kinds=B11Mode::all();
            let rotation=(worker+lane+rep)%kinds.len();
            for offset in 0..kinds.len(){
                b11_trial(worker,lane,rep,kinds[(rotation+offset)%kinds.len()]);
                records+=1;
            }
        }
    }
    assert_eq!(records,60);
    println!("DELTAGUARD_B1B1A_WORKER_PASS worker={worker} records={records}");
}
