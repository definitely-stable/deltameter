// DeltaGuard research-only B1-B1B0 persistent source processes + same-lifecycle
// hot/hot queries. Fixed-fixture checksum is NOT malicious-peer authentication.
const BB_LANES:[(usize,usize,usize,u64);3]=[
    (256,48,10,0),(256,57,10,10),(65536,48,100,10),
];
const BB_SAMPLES:usize=20;
const BB_CMD:usize=32;
const BB_ACK:usize=16;
const BB_MAGIC:&[u8;8]=b"DGBPERS1";
const BB_GOOD:&[u8;8]=b"DGPACK01";
const BB_BAD:&[u8;8]=b"DGPNACK1";
const BB_ADV:u8=4;
const BB_FAULT:u8=5;
const BB_QUIT:u8=6;
#[derive(Clone,Copy,PartialEq,Eq)]
enum BBMode {Guard,Delta,Full,Resolved}
impl BBMode {
    fn name(self)->&'static str {
        match self{Self::Guard=>"guard",Self::Delta=>"retained_delta",
            Self::Full=>"direct_full",Self::Resolved=>"resolved_guard"}
    }
}
fn bb_cmd(kind:u8,seq:usize)->[u8;BB_CMD] {
    let mut out=[0_u8;BB_CMD];
    out[..8].copy_from_slice(BB_MAGIC);
    out[8]=kind;
    out[16..24].copy_from_slice(&B1_EPOCH.to_le_bytes());
    out[24..32].copy_from_slice(&(seq as u64).to_le_bytes());
    out
}
fn bb_ack(ok:bool,seq:usize)->[u8;BB_ACK] {
    let mut out=[0_u8;BB_ACK];
    out[..8].copy_from_slice(if ok{BB_GOOD}else{BB_BAD});
    out[8..16].copy_from_slice(&(seq as u64).to_le_bytes());
    out
}
fn bb_owner(args:&[String]) {
    use std::io::{Read,Write};
    use std::net::TcpStream;
    use std::time::Duration;
    assert_eq!(args.len(),5,"--owner worker lane owner addr");
    let worker:usize=args[1].parse().unwrap();
    let lane:usize=args[2].parse().unwrap();
    let owner:u8=args[3].parse().unwrap();
    assert!((1..=5).contains(&worker) && lane<BB_LANES.len() && (1..=2).contains(&owner));
    let (n,d,mbps,delay)=BB_LANES[lane];
    let (mut source,next)=b11_owner_initial(worker,n,d,0,owner);
    let mut guard=NearFullGuard::new(11,64,&MASTER_KEY);
    for &token in &source {guard.toggle(token);}
    let mut generation=1_usize;
    let mut cursor=1_usize;
    let mut socket=TcpStream::connect(&args[4]).expect("owner connects");
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
    socket.write_all(&[owner]).unwrap();
    loop {
        let mut command=[0_u8;BB_CMD];
        socket.read_exact(&mut command).expect("complete receiver command");
        assert_eq!(&command[..8],BB_MAGIC);
        assert!(command[9..16].iter().all(|b|*b==0));
        assert_eq!(u64::from_le_bytes(command[16..24].try_into().unwrap()),B1_EPOCH);
        let seq=u64::from_le_bytes(command[24..32].try_into().unwrap()) as usize;
        let kind=command[8];
        if kind==BB_ADV {
            assert_eq!(seq,generation+1,"strict monotone generation");
            let token=next+seq as u64;
            source.push(token);
            guard.toggle(token);
            generation=seq;
            socket.write_all(&bb_ack(true,seq)).unwrap();
            continue;
        }
        if kind==BB_QUIT {
            assert_eq!(seq,generation);
            socket.write_all(&bb_ack(true,seq)).unwrap();
            socket.write_all(&b11_rss().to_le_bytes()).unwrap();
            break;
        }
        assert_eq!(seq,generation,"query must bind current maintained source generation");
        let (frame_kind,body)=match kind {
            B1_GUARD=>(B1_GUARD,b1_words_bytes(&guard.words)),
            B1_FULL=>(B1_FULL,b1_full_bytes(&source)),
            B1_DELTA|BB_FAULT=>{
                let mut bytes=Vec::new();
                for current in cursor+1..=generation {
                    bytes.extend_from_slice(&b1b_insert(next+current as u64));
                }
                (B1_DELTA,bytes)
            }
            _=>panic!("unknown command"),
        };
        let mut frame=b1_encode(owner,frame_kind,B1_EPOCH,generation as u64,&body);
        if kind==BB_FAULT {frame[48]^=0x01;}
        std::thread::sleep(Duration::from_millis(delay));
        b11_paced_write(&mut socket,&frame,mbps);
        let mut ack=[0_u8;BB_ACK];
        socket.read_exact(&mut ack).expect("real physical receiver ACK/NACK");
        assert_eq!(u64::from_le_bytes(ack[8..16].try_into().unwrap()),generation as u64);
        if ack[..8]==BB_GOOD[..] {
            assert_ne!(kind,BB_FAULT,"bad fixture was accepted");
            if kind==B1_DELTA {cursor=generation;}
        } else {
            assert_eq!(&ack[..8],BB_BAD);
            assert_eq!(kind,BB_FAULT,"legitimate frame NACK");
            // Failed frame MUST NOT commit cursor / source generation.
        }
    }
}
struct BBPeer {
    socket:std::net::TcpStream,
    child:std::process::Child,
}
fn bb_start(worker:usize,lane:usize)->[BBPeer;2] {
    use std::io::Read;
    use std::net::TcpListener;
    use std::process::{Command,Stdio};
    use std::time::Duration;
    let listener=TcpListener::bind(("127.0.0.1",0)).unwrap();
    let address=listener.local_addr().unwrap().to_string();
    let executable=std::env::current_exe().unwrap();
    let mut children:[Option<std::process::Child>;2]=[None,None];
    for owner in 1..=2 {
        let child=Command::new(&executable)
            .args(["--owner".to_string(),worker.to_string(),
                lane.to_string(),owner.to_string(),address.clone()])
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
        children[(owner-1) as usize]=Some(child);
    }
    let mut got:[Option<BBPeer>;2]=[None,None];
    for _ in 0..2 {
        let (mut socket,_)=listener.accept().unwrap();
        socket.set_nodelay(true).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
        socket.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
        let mut owner=[0_u8;1];
        socket.read_exact(&mut owner).unwrap();
        assert!((1..=2).contains(&owner[0]));
        let idx=(owner[0]-1) as usize;
        assert!(got[idx].is_none(),"duplicate sender");
        got[idx]=Some(BBPeer{socket,child:children[idx].take().unwrap()});
    }
    [got[0].take().unwrap(),got[1].take().unwrap()]
}
fn bb_request(peer:&mut BBPeer,kind:u8,seq:usize) {
    use std::io::Write;
    peer.socket.write_all(&bb_cmd(kind,seq)).unwrap();
}
fn bb_feedback(peer:&mut BBPeer,ok:bool,seq:usize) {
    use std::io::Write;
    peer.socket.write_all(&bb_ack(ok,seq)).unwrap();
}
fn bb_packet(peer:&mut BBPeer,owner:u8,kind:u8,seq:usize)->(Vec<u8>,usize) {
    use std::io::Read;
    let mut header=[0_u8;B1_HEADER];
    peer.socket.read_exact(&mut header).expect("physical TCP header");
    let len=u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
    assert!(len<=B1_MAX_PAYLOAD,"physical oversized body");
    let mut frame=header.to_vec();
    frame.resize(B1_HEADER+len,0);
    peer.socket.read_exact(&mut frame[B1_HEADER..]).expect("physical TCP body");
    b1_decode(&frame,owner,kind,B1_EPOCH,seq as u64)
        .expect("strict physical frame validation before ACK");
    let size=frame.len();
    (frame,size)
}
fn bb_round(peers:&mut [BBPeer;2],kind:u8,seq:usize)
    ->([Vec<u8>;2],usize,u128)
{
    use std::time::Instant;
    let start=Instant::now();
    for p in peers.iter_mut(){bb_request(p,kind,seq);}
    let (one,n1)=bb_packet(&mut peers[0],1,kind,seq);
    let (two,n2)=bb_packet(&mut peers[1],2,kind,seq);
    // Both frames were checked BEFORE either sender commits its delta cursor.
    for p in peers.iter_mut(){bb_feedback(p,true,seq);}
    let elapsed=start.elapsed().as_nanos();
    ([one,two],2*(BB_CMD+BB_ACK)+n1+n2,elapsed)
}
fn bb_advance(peers:&mut [BBPeer;2],seq:usize)->usize {
    use std::io::Read;
    for p in peers.iter_mut(){bb_request(p,BB_ADV,seq);}
    for p in peers.iter_mut() {
        let mut ack=[0_u8;BB_ACK];
        p.socket.read_exact(&mut ack).unwrap();
        assert_eq!(ack,bb_ack(true,seq));
    }
    2*(BB_CMD+BB_ACK)
}
fn bb_verify_guard(packets:&[Vec<u8>;2],a:&[u64],b:&[u64],seq:usize)->(u32,bool) {
    let wa=b11_as_words(b1_decode(&packets[0],1,B1_GUARD,B1_EPOCH,seq as u64).unwrap());
    let wb=b11_as_words(b1_decode(&packets[1],2,B1_GUARD,B1_EPOCH,seq as u64).unwrap());
    let xor:Vec<u64>=wa.iter().zip(&wb).map(|(x,y)|x^y).collect();
    let key=NearFullGuard::new(11,64,&MASTER_KEY).key;
    assert_eq!(xor,b1b_reference_diff(a,b,&key),"independent source keyed XOR oracle");
    let odd=xor.iter().map(|x|x.count_ones()).sum();
    (odd,odd<=48)
}
fn bb_fixture(worker:usize,lane:usize)->(Vec<u64>,Vec<u64>,u64) {
    let (n,d,_,_)=BB_LANES[lane];
    b1b_inputs(worker,if n==256{0}else{1},
        B1B_DS.iter().position(|x|*x==d).unwrap(),0)
}
fn bb_fail_retry(peers:&mut [BBPeer;2],receiver:&mut [Vec<u64>;2],
    oracle:&[Vec<u64>;2],seq:usize)->usize
{
    use std::io::Read;
    // Source-1 corrupts only the fixture checksum in an actual sent frame.
    bb_request(&mut peers[0],BB_FAULT,seq);
    let mut header=[0_u8;B1_HEADER];
    peers[0].socket.read_exact(&mut header).unwrap();
    let length=u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
    assert_eq!(length,9);
    let mut bad=header.to_vec();
    bad.resize(B1_HEADER+length,0);
    peers[0].socket.read_exact(&mut bad[B1_HEADER..]).unwrap();
    assert_eq!(b1_decode(&bad,1,B1_DELTA,B1_EPOCH,seq as u64),Err("checksum"));
    let old=receiver.clone();
    bb_feedback(&mut peers[0],false,seq);
    assert_eq!(*receiver,old,"failed checksum mutated exact receiver");
    let (good,bytes,_)=bb_round(peers,B1_DELTA,seq);
    for (idx,frame) in good.iter().enumerate() {
        let body=b1_decode(frame,(idx+1) as u8,B1_DELTA,B1_EPOCH,seq as u64).unwrap();
        b1b_apply(&mut receiver[idx],body).unwrap();
        assert_eq!(receiver[idx],oracle[idx],"valid replay after NACK restores exact");
    }
    BB_CMD+B1_HEADER+length+BB_ACK+bytes
}
fn bb_end(peers:&mut [BBPeer;2],seq:usize)->(usize,[u64;2]) {
    use std::io::Read;
    for p in peers.iter_mut(){bb_request(p,BB_QUIT,seq);}
    let mut rss=[0_u64;2];
    for (idx,p) in peers.iter_mut().enumerate(){
        let mut ack=[0_u8;BB_ACK];
        p.socket.read_exact(&mut ack).unwrap();
        assert_eq!(ack,bb_ack(true,seq));
        let mut bytes=[0_u8;8];
        p.socket.read_exact(&mut bytes).unwrap();
        rss[idx]=u64::from_le_bytes(bytes);
        assert!(rss[idx]>0);
    }
    for p in peers.iter_mut(){assert!(p.child.wait().unwrap().success());}
    (2*(BB_CMD+BB_ACK+8),rss)
}
fn bb_worker(worker:usize) {
    assert!((1..=5).contains(&worker));
    bb_negative_unit();
    let mut total=0_usize;
    for (lane,&(n,d,mbps,delay)) in BB_LANES.iter().enumerate(){
        let (mut a,mut b,next)=bb_fixture(worker,lane);
        let mut peers=bb_start(worker,lane);
        let (init,boot_bytes,_)=bb_round(&mut peers,B1_FULL,1);
        let receiver_a=b11_as_words(b1_decode(&init[0],1,B1_FULL,B1_EPOCH,1).unwrap());
        let receiver_b=b11_as_words(b1_decode(&init[1],2,B1_FULL,B1_EPOCH,1).unwrap());
        let mut receiver=[receiver_a,receiver_b];
        assert_eq!(receiver[0],a);
        assert_eq!(receiver[1],b);
        assert_eq!(boot_bytes,2*(BB_CMD+BB_ACK+B1_HEADER)+8*(a.len()+b.len()));
        println!("B1B1B0_BOOT worker={worker} lane={lane} N={n} d={d} bytes={}",boot_bytes+2);
        let mut adv_bytes=0;
        let mut records=0;
        for seq in 2..=BB_SAMPLES+1 {
            a.push(next+seq as u64);
            b.push(next+seq as u64);
            adv_bytes+=bb_advance(&mut peers,seq);
            let mut modes=vec![BBMode::Guard,BBMode::Delta,BBMode::Full];
            if d==57 {modes.push(BBMode::Resolved);}
            let rot=(worker+lane+seq)%modes.len();
            modes.rotate_left(rot);
            for mode in modes {
                let (kind,expected_kind)=match mode {
                    BBMode::Guard|BBMode::Resolved=>(B1_GUARD,B1_GUARD),
                    BBMode::Delta=>(B1_DELTA,B1_DELTA),
                    BBMode::Full=>(B1_FULL,B1_FULL),
                };
                let query_started=std::time::Instant::now();
                let (packets,mut bytes,_wire_ns)=bb_round(&mut peers,kind,seq);
                let mut odd=0_u32;
                let mut safe=2_u8;
                let mut fallback=0_u8;
                if expected_kind==B1_GUARD {
                    let (observed,is_safe)=bb_verify_guard(&packets,&a,&b,seq);
                    odd=observed;
                    safe=u8::from(is_safe);
                    if d<=48 {assert_eq!(safe,1);}
                    if mode==BBMode::Resolved && !is_safe {
                        let (lists,more,_wire_ns)=bb_round(&mut peers,B1_FULL,seq);
                        bytes+=more;
                        for (idx,frame) in lists.iter().enumerate(){
                            let found=b11_as_words(b1_decode(frame,(idx+1) as u8,
                                B1_FULL,B1_EPOCH,seq as u64).unwrap());
                            let expected=if idx==0{a.as_slice()}else{b.as_slice()};
                            assert_eq!(found,expected);
                        }
                        fallback=1;
                    }
                }else if mode==BBMode::Full {
                    for (idx,frame) in packets.iter().enumerate() {
                        let found=b11_as_words(b1_decode(frame,(idx+1) as u8,
                            B1_FULL,B1_EPOCH,seq as u64).unwrap());
                        let expected=if idx==0{a.as_slice()}else{b.as_slice()};
                        assert_eq!(found,expected);
                    }
                }else {
                    for (idx,frame) in packets.iter().enumerate(){
                        let body=b1_decode(frame,(idx+1) as u8,B1_DELTA,B1_EPOCH,seq as u64).unwrap();
                        b1b_apply(&mut receiver[idx],body).expect("atomic receiver update");
                    }
                    assert_eq!(receiver[0],a);
                    assert_eq!(receiver[1],b);
                }
                if mode==BBMode::Guard||mode==BBMode::Resolved {
                    assert!(odd<=d as u32);
                }
                let elapsed=query_started.elapsed().as_nanos();
                println!("B1B1B0_SAMPLE worker={worker} lane={lane} generation={seq} N={n} d={d} mbps={mbps} delay={delay} mode={} bytes={bytes} wall_ns={elapsed} odd={odd} safe={safe} fallback={fallback} exact=1",mode.name());
                records+=1;
                total+=1;
            }
            assert_eq!(receiver[0],a,"receiver A drift");
            assert_eq!(receiver[1],b,"receiver B drift");
        }
        assert_eq!(adv_bytes,BB_SAMPLES*2*(BB_CMD+BB_ACK));
        let last=BB_SAMPLES+2;
        a.push(next+last as u64);
        b.push(next+last as u64);
        adv_bytes+=bb_advance(&mut peers,last);
        let failure=bb_fail_retry(&mut peers,&mut receiver,&[a,b],last);
        assert_eq!(failure,BB_CMD+B1_HEADER+9+BB_ACK+2*(BB_CMD+BB_ACK+B1_HEADER+9));
        let (quit,rss)=bb_end(&mut peers,last);
        println!("B1B1B0_FINISH worker={worker} lane={lane} records={records} advance_bytes={adv_bytes} failure_bytes={failure} quit_bytes={quit} child_rss1={} child_rss2={} parent_rss={}",rss[0],rss[1],b11_rss());
        assert_eq!(records,if d==57{80}else{60});
    }
    assert_eq!(total,200);
    println!("DELTAGUARD_B1B1B0_WORKER_PASS worker={worker} records={total} retry_tests=3");
}
fn bb_negative_unit() {
    let p=b1_encode(1,B1_DELTA,B1_EPOCH,2,&b1b_insert(44));
    for (owner,epoch,seq) in [(2,B1_EPOCH,2),(1,B1_EPOCH+1,2),
        (1,B1_EPOCH,1),(1,B1_EPOCH,3)] {
        assert!(b1_decode(&p,owner,B1_DELTA,epoch,seq).is_err());
    }
    let mut corrupt=p.clone();
    corrupt[48]^=1;
    assert_eq!(b1_decode(&corrupt,1,B1_DELTA,B1_EPOCH,2),Err("checksum"));
    let mut incomplete=p.clone();
    incomplete.pop();
    assert_eq!(b1_decode(&incomplete,1,B1_DELTA,B1_EPOCH,2),Err("payload-length"));
    assert_eq!(bb_cmd(BB_ADV,2)[9..16],[0;7]);
}
