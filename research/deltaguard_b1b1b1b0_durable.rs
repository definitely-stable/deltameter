// B1-B1B1-B0: research-only actual synced owner snapshot/WAL/commit/ACK
// across separate child OS SIGKILL+reconnect. Public fixture != authentication.
use std::path::{Path,PathBuf};
const BD_LANES:[usize;2]=[256,65536];
const BD_CASES:[&str;7]=[
    "before_wal","after_write_before_sync","after_wal_sync_before_commit",
    "after_commit_before_send","after_send_before_ack",
    "after_receiver_sync_before_ack","after_ack_before_owner_sync",
];
const BD_REC_MAGIC:&[u8;8]=b"DGREC2!!";
const BD_COMMIT:&[u8;8]=b"DGCMIT02";
const BD_ACK_MARK:&[u8;8]=b"DGAACK02";
fn bd_dir(root:&Path,owner:u8)->PathBuf {root.join(format!("owner-{owner}"))}
fn bd_atomic(dir:&Path,file:&str,content:&[u8]){
    use std::io::Write;
    let tmp=dir.join(format!("{file}.tmp"));
    let mut f=std::fs::File::create(&tmp).expect("create durable temp");
    f.write_all(content).expect("write durable file");
    f.sync_all().expect("fsync durable file BEFORE publication");
    drop(f);
    std::fs::rename(&tmp,dir.join(file)).expect("atomic durable publish");
    std::fs::File::open(dir).unwrap().sync_all().expect("fsync parent directory");
}
fn bd_digest(body:&[u8])->[u8;16]{
    blake3::hash(body).as_bytes()[..16].try_into().unwrap()
}
fn bd_marker(magic:&[u8;8],seq:u64,content:&[u8])->Vec<u8>{
    let mut mark=Vec::with_capacity(40);
    mark.extend_from_slice(magic);
    mark.extend_from_slice(&B1_EPOCH.to_le_bytes());
    mark.extend_from_slice(&seq.to_le_bytes());
    mark.extend_from_slice(&bd_digest(content));
    mark
}
fn bd_verify_marker(data:&[u8],magic:&[u8;8],seq:u64,content:&[u8])
    ->Result<(),&'static str>
{
    if data!=bd_marker(magic,seq,content){return Err("invalid-durable-marker");}
    Ok(())
}
fn bd_full_state(path:&Path,owner:u8,seq:usize)->Result<Vec<u64>,&'static str>{
    let frame=std::fs::read(path).map_err(|_|"source-snapshot-unavailable")?;
    let body=b1_decode(&frame,owner,B1_FULL,B1_EPOCH,seq as u64)?;
    Ok(b11_as_words(body))
}
struct BDSource {tokens:Vec<u64>,seq:usize,ack:usize,wal:Option<Vec<u8>>}
fn bd_owner_load(dir:&Path,owner:u8)->Result<BDSource,&'static str>{
    let mut source=bd_full_state(&dir.join("base.snap"),owner,1)?;
    let mut seq=1_usize;
    let mut wal=None;
    if dir.join("commit.mark").exists(){
        let event=std::fs::read(dir.join("events.wal")).map_err(|_|"committed-wal-missing")?;
        let mark=std::fs::read(dir.join("commit.mark")).map_err(|_|"commit-unreadable")?;
        bd_verify_marker(&mark,BD_COMMIT,2,&event)?;
        let body=b1_decode(&event,owner,B1_DELTA,B1_EPOCH,2)?;
        b1b_apply(&mut source,body)?;
        seq=2;
        wal=Some(event);
    } // An uncommitted WAL tail is ignored by policy, regardless of OS page cache.
    let mut ack=1_usize;
    if dir.join("ack.mark").exists(){
        let mark=std::fs::read(dir.join("ack.mark")).map_err(|_|"ack-unreadable")?;
        let content=wal.as_ref().ok_or("ack-without-committed-wal")?;
        bd_verify_marker(&mark,BD_ACK_MARK,2,content)?;
        ack=2;
    }
    if ack>seq {return Err("ack-ahead-of-durable-commit");}
    Ok(BDSource{tokens:source,seq,ack,wal})
}
fn bd_store_receiver(root:&Path,a:&[u64],b:&[u64],ga:usize,gb:usize) {
    let fa=b1_encode(1,B1_FULL,B1_EPOCH,ga as u64,&b1_full_bytes(a));
    let fb=b1_encode(2,B1_FULL,B1_EPOCH,gb as u64,&b1_full_bytes(b));
    let mut bytes=Vec::with_capacity(16+fa.len()+fb.len()+16);
    bytes.extend_from_slice(BD_REC_MAGIC);
    bytes.extend_from_slice(&(fa.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(fb.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&fa);
    bytes.extend_from_slice(&fb);
    let tag=bd_digest(&bytes);
    bytes.extend_from_slice(&tag);
    bd_atomic(root,"receiver.bin",&bytes);
}
fn bd_load_receiver(root:&Path)->Result<([Vec<u64>;2],[usize;2]),&'static str>{
    let data=std::fs::read(root.join("receiver.bin")).map_err(|_|"receiver-file-missing")?;
    if data.len()<32||&data[..8]!=BD_REC_MAGIC{return Err("receiver-magic-size");}
    let na=u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    let nb=u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
    if na>B1_MAX_PAYLOAD+B1_HEADER||nb>B1_MAX_PAYLOAD+B1_HEADER||
        data.len()!=16+na+nb+16 {return Err("receiver-length");}
    if data[16+na+nb..]!=bd_digest(&data[..16+na+nb]){return Err("receiver-digest");}
    let f1=&data[16..16+na];
    let f2=&data[16+na..16+na+nb];
    if f1.len()<B1_HEADER||f2.len()<B1_HEADER{return Err("receiver-header");}
    let ga=u64::from_le_bytes(f1[20..28].try_into().unwrap()) as usize;
    let gb=u64::from_le_bytes(f2[20..28].try_into().unwrap()) as usize;
    if ![1,2].contains(&ga)||![1,2].contains(&gb){return Err("receiver-unknown-generation");}
    let a=b11_as_words(b1_decode(f1,1,B1_FULL,B1_EPOCH,ga as u64)?);
    let b=b11_as_words(b1_decode(f2,2,B1_FULL,B1_EPOCH,gb as u64)?);
    Ok(([a,b],[ga,gb]))
}
fn bd_new_socket(addr:&str,owner:u8)->std::net::TcpStream {
    use std::io::Write;
    use std::net::TcpStream;
    use std::time::Duration;
    let mut s=TcpStream::connect(addr).expect("physical child connects");
    s.set_nodelay(true).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
    s.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
    s.write_all(&[owner]).unwrap();
    s
}
fn bd_telemetry(sock:&mut std::net::TcpStream) {
    use std::io::Write;
    sock.write_all(&b11_rss().to_le_bytes()).unwrap();
    sock.write_all(&bc_cpu_ticks().to_le_bytes()).unwrap();
}
fn bd_stage_event(dir:&Path,owner:u8,token:u64,case:&str) {
    use std::io::Write;
    let existing=bd_owner_load(dir,owner).expect("stage must read source from disk");
    assert_eq!(existing.seq,1);
    if case=="before_wal" {return;}
    let ev=b1_encode(owner,B1_DELTA,B1_EPOCH,2,&b1b_insert(token));
    let mut f=std::fs::File::create(dir.join("events.wal")).unwrap();
    f.write_all(&ev).unwrap();
    if case=="after_write_before_sync" {return;}
    f.sync_all().unwrap();
    if case=="after_wal_sync_before_commit" {return;}
    drop(f);
    bd_atomic(dir,"commit.mark",&bd_marker(BD_COMMIT,2,&ev));
}
fn bd_boot_child(args:&[String]) {
    use std::io::{Read,Write};
    assert_eq!(args.len(),7);
    let worker:usize=args[1].parse().unwrap();
    let lane:usize=args[2].parse().unwrap();
    let rep:usize=args[3].parse().unwrap();
    let owner:u8=args[4].parse().unwrap();
    let root=PathBuf::from(&args[5]);
    let dir=bd_dir(&root,owner);
    std::fs::create_dir_all(&dir).unwrap();
    let (initial,_)=b11_owner_initial(worker,BD_LANES[lane],48,rep,owner);
    bd_atomic(&dir,"base.snap",&b1_encode(owner,B1_FULL,B1_EPOCH,1,&b1_full_bytes(&initial)));
    let mut sock=bd_new_socket(&args[6],owner);
    let mut req=[0_u8;B11_REQ];
    sock.read_exact(&mut req).unwrap();
    assert_eq!(req,b11_req(1));
    sock.write_all(&b1_encode(owner,B1_FULL,B1_EPOCH,1,&b1_full_bytes(&initial))).unwrap();
    let mut ack=[0_u8;BB_ACK];
    sock.read_exact(&mut ack).unwrap();
    assert_eq!(ack,bb_ack(true,1));
    bd_telemetry(&mut sock);
}
fn bd_stage_child(args:&[String]){
    use std::io::{Read,Write};
    assert_eq!(args.len(),8);
    let worker:usize=args[1].parse().unwrap();
    let lane:usize=args[2].parse().unwrap();
    let rep:usize=args[3].parse().unwrap();
    let owner:u8=args[4].parse().unwrap();
    let case=&args[5];
    let root=PathBuf::from(&args[6]);
    let dir=bd_dir(&root,owner);
    assert!(BD_CASES.contains(&case.as_str()));
    let (_,token_seed)=b11_owner_initial(worker,BD_LANES[lane],48,rep,owner);
    bd_stage_event(&dir,owner,token_seed+2,case);
    let src=bd_owner_load(&dir,owner).unwrap();
    let mut sock=bd_new_socket(&args[7],owner);
    let sent=case.contains("send")||case.contains("receiver")||case.contains("ack_before");
    // after_send_before_ack, after_receiver_sync_before_ack and
    // after_ack_before_owner_sync have a physically sent delta.
    sock.write_all(&[u8::from(sent)]).unwrap();
    if sent {
        let frame=src.wal.as_ref().unwrap();
        sock.write_all(frame).unwrap();
        if *case=="after_ack_before_owner_sync"{
            let mut ack=[0_u8;BB_ACK];
            sock.read_exact(&mut ack).unwrap();
            assert_eq!(ack,bb_ack(true,2));
            sock.write_all(b"A").unwrap();
        }
    }
    // Controller kills this REAL OS process before any unrequested cursor
    // fsync. Crash is SIGKILL, never a clean child exit.
    let mut unused=[0_u8;1];
    let _=sock.read_exact(&mut unused);
}
fn bd_reboot_child(args:&[String]){
    use std::io::{Read,Write};
    assert_eq!(args.len(),5);
    let owner:u8=args[1].parse().unwrap();
    let dir=PathBuf::from(&args[2]);
    let src=bd_owner_load(&dir,owner).expect("only durable owner source can reconnect");
    let mut sock=bd_new_socket(&args[3],owner);
    // The source sends persisted source generation and ACK watermark,
    // not a recomputed fixture truth.
    sock.write_all(&(src.seq as u64).to_le_bytes()).unwrap();
    sock.write_all(&(src.ack as u64).to_le_bytes()).unwrap();
    let mut cmd=[0_u8;BB_CMD];
    sock.read_exact(&mut cmd).unwrap();
    assert_eq!(&cmd[..8],BB_MAGIC);
    assert!(cmd[9..16].iter().all(|v|*v==0));
    assert_eq!(u64::from_le_bytes(cmd[16..24].try_into().unwrap()),B1_EPOCH);
    let requested=u64::from_le_bytes(cmd[24..32].try_into().unwrap()) as usize;
    let payload=match cmd[8] {
        B1_FULL=>{assert_eq!(src.seq,1);assert_eq!(requested,1);
            b1_encode(owner,B1_FULL,B1_EPOCH,1,&b1_full_bytes(&src.tokens))}
        B1_DELTA=>{assert_eq!(src.seq,2);assert_eq!(requested,2);
            src.wal.as_ref().unwrap().clone()}
        _=>panic!("invalid persisted-source recovery request"),
    };
    sock.write_all(&payload).unwrap();
    let mut ack=[0_u8;BB_ACK];
    sock.read_exact(&mut ack).unwrap();
    assert_eq!(ack,bb_ack(true,requested));
    if requested==2 && src.ack==1 {
        bd_atomic(&dir,"ack.mark",
            &bd_marker(BD_ACK_MARK,2,src.wal.as_ref().unwrap()));
    }
    bd_telemetry(&mut sock);
}
fn bd_child(role:&str,worker:usize,lane:usize,rep:usize,owner:u8,
    case:&str,root:&Path,address:&str)->std::process::Child
{
    use std::process::{Command,Stdio};
    let mut cmd=Command::new(std::env::current_exe().unwrap());
    match role {
        "--boot"=>{cmd.args(["--boot".to_string(),worker.to_string(),
            lane.to_string(),rep.to_string(),owner.to_string(),
            root.to_str().unwrap().to_string(),address.to_string()]);}
        "--stage"=>{cmd.args(["--stage".to_string(),worker.to_string(),lane.to_string(),
            rep.to_string(),owner.to_string(),case.to_string(),
            root.to_str().unwrap().to_string(),address.to_string()]);}
        "--reboot"=>{cmd.args(["--reboot".to_string(),owner.to_string(),
            bd_dir(root,owner).to_str().unwrap().to_string(),
            address.to_string()]);}
        _=>panic!("wrong source role")
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap()
}
fn bd_listen()->(std::net::TcpListener,String){
    let l=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap();
    let a=l.local_addr().unwrap().to_string();
    (l,a)
}
fn bd_accept_pair(listener:&std::net::TcpListener)->[std::net::TcpStream;2]{
    use std::io::Read;
    let mut paired:[Option<std::net::TcpStream>;2]=[None,None];
    for _ in 0..2 {
        let (mut sock,_)=listener.accept().unwrap();
        bc_sock_setup(&sock);
        let mut owner=[0_u8;1];
        sock.read_exact(&mut owner).unwrap();
        assert!((1..=2).contains(&owner[0]));
        let idx=(owner[0]-1) as usize;
        assert!(paired[idx].is_none(),"duplicate persisted source identity");
        paired[idx]=Some(sock);
    }
    [paired[0].take().unwrap(),paired[1].take().unwrap()]
}
fn bd_frame(sock:&mut std::net::TcpStream,owner:u8,kind:u8,seq:usize)->Vec<u8>{
    use std::io::Read;
    let mut h=[0_u8;B1_HEADER];
    sock.read_exact(&mut h).unwrap();
    let len=u32::from_le_bytes(h[28..32].try_into().unwrap()) as usize;
    assert!(len<=B1_MAX_PAYLOAD);
    let mut frame=h.to_vec();
    frame.resize(B1_HEADER+len,0);
    sock.read_exact(&mut frame[B1_HEADER..]).unwrap();
    b1_decode(&frame,owner,kind,B1_EPOCH,seq as u64)
        .expect("durable source bad TCP identity/framing");
    frame
}
fn bd_wait_children(children:&mut [std::process::Child;2]){
    for child in children {
        assert!(child.wait().unwrap().success(),"durable source did not exit cleanly");
    }
}
fn bd_init(worker:usize,lane:usize,rep:usize,root:&Path)->usize{
    use std::io::{Read,Write};
    std::fs::create_dir_all(root).unwrap();
    let (listener,address)=bd_listen();
    let mut children=[
        bd_child("--boot",worker,lane,rep,1,"",root,&address),
        bd_child("--boot",worker,lane,rep,2,"",root,&address),
    ];
    let mut socks=bd_accept_pair(&listener);
    let mut frames=Vec::new();
    let mut traffic=2; // hello owners
    for (idx,sock) in socks.iter_mut().enumerate(){
        sock.write_all(&b11_req(1)).unwrap(); traffic+=B11_REQ;
        let frame=bd_frame(sock,(idx+1) as u8,B1_FULL,1);
        traffic+=frame.len();
        frames.push(frame);
    }
    let a=b11_as_words(b1_decode(&frames[0],1,B1_FULL,B1_EPOCH,1).unwrap());
    let b=b11_as_words(b1_decode(&frames[1],2,B1_FULL,B1_EPOCH,1).unwrap());
    bd_store_receiver(root,&a,&b,1,1); // sync before ACK.
    for sock in &mut socks {
        sock.write_all(&bb_ack(true,1)).unwrap();traffic+=BB_ACK;
        let mut telemetry=[0_u8;16];
        sock.read_exact(&mut telemetry).unwrap();traffic+=16;
        assert!(u64::from_le_bytes(telemetry[..8].try_into().unwrap())>0);
    }
    bd_wait_children(&mut children);
    traffic
}
fn bd_stage(worker:usize,lane:usize,rep:usize,case:&str,root:&Path)->(usize,bool){
    use std::io::{Read,Write};
    let (listener,address)=bd_listen();
    let mut children=[
        bd_child("--stage",worker,lane,rep,1,case,root,&address),
        bd_child("--stage",worker,lane,rep,2,case,root,&address),
    ];
    let mut socks=bd_accept_pair(&listener);
    let mut bytes=2_usize;
    let sent=case.contains("send")||case.contains("receiver")||case.contains("ack_before");
    let mut delivered=Vec::new();
    for (idx,sock) in socks.iter_mut().enumerate(){
        let mut report=[0_u8;1];
        sock.read_exact(&mut report).unwrap();
        assert_eq!(report[0],u8::from(sent));
        bytes+=1;
        if sent {
            let frame=bd_frame(sock,(idx+1) as u8,B1_DELTA,2);
            bytes+=frame.len();
            delivered.push(frame);
        }
    }
    let receiver_synced=case=="after_receiver_sync_before_ack"||
        case=="after_ack_before_owner_sync";
    if receiver_synced {
        let (old,seq)=bd_load_receiver(root).unwrap();
        assert_eq!(seq,[1,1]);
        let mut tentative=old;
        for (idx,frame) in delivered.iter().enumerate(){
            let body=b1_decode(frame,(idx+1) as u8,B1_DELTA,B1_EPOCH,2).unwrap();
            b1b_apply(&mut tentative[idx],body).unwrap();
        }
        bd_store_receiver(root,&tentative[0],&tentative[1],2,2);
    }
    if case=="after_ack_before_owner_sync" {
        for sock in &mut socks {
            sock.write_all(&bb_ack(true,2)).unwrap(); bytes+=BB_ACK;
        }
        for sock in &mut socks {
            let mut ready=[0_u8;1];
            sock.read_exact(&mut ready).unwrap();
            assert_eq!(&ready,b"A");
            bytes+=1;
        }
    }
    for child in &mut children {
        child.kill().expect("physical source SIGKILL");
        assert!(!child.wait().unwrap().success(),"source was not actually killed");
    }
    (bytes,receiver_synced)
}
fn bd_receiver_recovery(worker:usize,lane:usize,rep:usize,root:&Path) {
    use std::io::{Read,Write};
    let (initial,gs)=bd_load_receiver(root).expect("receiver must reopen fsync'd cache");
    let (listener,address)=bd_listen();
    let mut children=[
        bd_child("--reboot",worker,lane,rep,1,"",root,&address),
        bd_child("--reboot",worker,lane,rep,2,"",root,&address),
    ];
    let mut socks=bd_accept_pair(&listener);
    let mut source_g=[0_usize;2];
    let mut source_ack=[0_usize;2];
    let mut bytes=2_usize; // owner hellos
    for (idx,sock) in socks.iter_mut().enumerate() {
        let mut metadata=[0_u8;16];
        sock.read_exact(&mut metadata).unwrap();
        bytes+=16;
        source_g[idx]=u64::from_le_bytes(metadata[..8].try_into().unwrap()) as usize;
        source_ack[idx]=u64::from_le_bytes(metadata[8..].try_into().unwrap()) as usize;
        assert!([1,2].contains(&source_g[idx]));
        assert!(source_ack[idx]<=source_g[idx]);
        assert!(source_g[idx]>=gs[idx],"source rollback against durable receiver cache");
        let kind=if source_g[idx]==2{B1_DELTA}else{B1_FULL};
        sock.write_all(&bb_cmd(kind,source_g[idx])).unwrap();
        bytes+=BB_CMD;
    }
    let mut updated=initial;
    for (idx,sock) in socks.iter_mut().enumerate() {
        let owner=(idx+1) as u8;
        let kind=if source_g[idx]==2{B1_DELTA}else{B1_FULL};
        let frame=bd_frame(sock,owner,kind,source_g[idx]);
        bytes+=frame.len();
        let body=b1_decode(&frame,owner,kind,B1_EPOCH,source_g[idx] as u64).unwrap();
        if kind==B1_FULL {
            assert_eq!(gs[idx],1);
            let found=b11_as_words(body);
            assert_eq!(found,updated[idx]);
        }else if gs[idx]==1 {
            b1b_apply(&mut updated[idx],body).expect("durable replay delta");
        }else {
            // Valid identical repeated generation is not applied twice.
            assert_eq!(gs[idx],2);
            assert_eq!(body.len(),9);
            assert_eq!(body[0],1);
            let token=u64::from_le_bytes(body[1..9].try_into().unwrap());
            assert!(updated[idx].binary_search(&token).is_ok(),
                "same generation payload not present in saved receiver exact");
        }
    }
    // ONE atomic two-owner durable receiver transaction,
    // before any source ACK is physically sent.
    if source_g!=gs {
        bd_store_receiver(root,&updated[0],&updated[1],source_g[0],source_g[1]);
    }
    for sock in &mut socks {
        sock.write_all(&bb_ack(true,source_g[if std::ptr::eq(sock,&socks[0]){0}else{1}])).unwrap();
        bytes+=BB_ACK;
    }
    for sock in &mut socks {
        let mut telem=[0_u8;16];
        sock.read_exact(&mut telem).unwrap();
        bytes+=16;
        assert!(u64::from_le_bytes(telem[..8].try_into().unwrap())>0);
    }
    bd_wait_children(&mut children);
    let (reopened,g)=bd_load_receiver(root).expect("reopen receiver AFTER commit");
    assert_eq!(g,source_g);
    assert_eq!(reopened,updated);
    // Fixture is ONLY post-transport external oracle, NEVER owner recovery.
    for owner in 1..=2 {
        let (mut expected,next)=b11_owner_initial(worker,BD_LANES[lane],48,rep,owner);
        if g[(owner-1) as usize]==2 {expected.push(next+2);}
        assert_eq!(reopened[(owner-1) as usize],expected,
            "physically replayed disk source differs from external fixture truth");
    }
    println!("B1B1B1B0_REC worker={worker} lane={lane} rep={rep} generation={} sender_ack_before={} bytes={bytes} exact=1",
        source_g[0],source_ack[0]);
}
fn bd_one(worker:usize,lane:usize,rep:usize,case_idx:usize) {
    use std::process::{Command,Stdio};
    let case=BD_CASES[case_idx];
    let root=std::env::temp_dir().join(format!(
        "deltameter-b1b1b1b0-w{worker}-l{lane}-r{rep}-c{case_idx}-pid{}",std::process::id()));
    if root.exists(){std::fs::remove_dir_all(&root).unwrap();}
    let bootstrap=bd_init(worker,lane,rep,&root);
    let (traffic,recv_sync)=bd_stage(worker,lane,rep,case,&root);
    let (persisted,g_before)=bd_load_receiver(&root).unwrap();
    assert_eq!(g_before,if recv_sync{[2,2]}else{[1,1]});
    assert_eq!(persisted[0].len(),BD_LANES[lane]+usize::from(recv_sync));
    for owner in 1..=2 {
        let src=bd_owner_load(&bd_dir(&root,owner),owner).unwrap();
        let committed=case_idx>=3;
        assert_eq!(src.seq,if committed{2}else{1});
        assert_eq!(src.ack,1,"owner ACK cursor cannot be advanced before the killed stage child");
    }
    // Crash the parent receiver process boundary too: the recovery
    // is a genuinely distinct OS child that reloads receiver.bin from disk.
    let output=Command::new(std::env::current_exe().unwrap())
        .args(["--recover", &worker.to_string(), &lane.to_string(),
            &rep.to_string(),root.to_str().unwrap()])
        .stdout(Stdio::piped()).stderr(Stdio::inherit())
        .output().unwrap();
    assert!(output.status.success(),"fresh receiver subprocess failed");
    let trace=String::from_utf8(output.stdout).unwrap();
    assert!(trace.contains("B1B1B1B0_REC worker="));
    let mut reopened=[0_usize;2];
    for owner in 1..=2{
        let src=bd_owner_load(&bd_dir(&root,owner),owner).unwrap();
        reopened[(owner-1) as usize]=src.seq;
        assert_eq!(src.ack,src.seq,"persistent source ACK marker must commit on reconnect");
    }
    let (receiver,gens)=bd_load_receiver(&root).unwrap();
    assert_eq!(gens,reopened);
    assert_eq!(receiver[0].len(),BD_LANES[lane]+usize::from(case_idx>=3));
    assert_eq!(receiver[1].len(),BD_LANES[lane]+usize::from(case_idx>=3));
    let committed=u8::from(case_idx>=3);
    let replayed=u8::from(case_idx>=3 && !recv_sync);
    let duplicate=u8::from(case_idx>=3 && recv_sync);
    let (_,_,_,_)=(committed,replayed,duplicate,traffic);
    let mut actual_bytes=0_usize;
    for token in trace.split_ascii_whitespace(){
        if let Some(v)=token.strip_prefix("bytes=") {
            actual_bytes=v.parse().unwrap();
        }
    }
    assert!(actual_bytes>0);
    println!("B1B1B1B0_SAMPLE worker={worker} lane={lane} rep={rep} case_idx={case_idx} case={case} N={} initial_bytes={bootstrap} stage_bytes={traffic} recovery_bytes={actual_bytes} source_gen={} ack_gen={} receiver_gen={} replayed={replayed} dedup={duplicate} exact=1",
        BD_LANES[lane],gens[0],gens[0],gens[0]);
    std::fs::remove_dir_all(root).unwrap();
}
fn bd_worker(worker:usize){
    assert!((1..=5).contains(&worker));
    bd_negative();
    let mut total=0;
    for lane in 0..2 {
        for rep in 0..3 {
            for case in 0..BD_CASES.len(){
                bd_one(worker,lane,rep,case);
                total+=1;
            }
        }
    }
    assert_eq!(total,42);
    println!("DELTAGUARD_B1B1B1B0_WORKER_PASS worker={worker} records={total} negative=5");
}
fn bd_negative(){
    let root=std::env::temp_dir().join(format!("bd-negative-{}",std::process::id()));
    if root.exists(){std::fs::remove_dir_all(&root).unwrap();}
    std::fs::create_dir_all(bd_dir(&root,1)).unwrap();
    std::fs::create_dir_all(bd_dir(&root,2)).unwrap();
    let owner=bd_dir(&root,1);
    bd_atomic(&owner,"base.snap",&b1_encode(1,B1_FULL,B1_EPOCH,1,&b1_full_bytes(&[10,20])));
    // Uncommitted journal is never treated as committed.
    let event=b1_encode(1,B1_DELTA,B1_EPOCH,2,&b1b_insert(30));
    std::fs::write(owner.join("events.wal"),&event).unwrap();
    assert_eq!(bd_owner_load(&owner,1).unwrap().seq,1);
    bd_atomic(&owner,"commit.mark",&bd_marker(BD_COMMIT,2,&event));
    assert_eq!(bd_owner_load(&owner,1).unwrap().seq,2);
    // Committed journal tampering is fatal.
    let mut corrupt=event.clone();
    corrupt[48]^=1;
    std::fs::write(owner.join("events.wal"),&corrupt).unwrap();
    assert!(bd_owner_load(&owner,1).is_err());
    std::fs::write(owner.join("events.wal"),&event).unwrap();
    // ACK cursor after committed generation is rejected.
    bd_atomic(&owner,"ack.mark",&bd_marker(BD_ACK_MARK,3,&event));
    assert!(bd_owner_load(&owner,1).is_err());
    std::fs::remove_file(owner.join("ack.mark")).unwrap();
    // Base owner identity mismatch is fatal.
    let mut b=std::fs::read(owner.join("base.snap")).unwrap();
    b[9]=2;
    std::fs::write(owner.join("base.snap"),&b).unwrap();
    assert!(bd_owner_load(&owner,1).is_err());
    b[9]=1;
    std::fs::write(owner.join("base.snap"),&b).unwrap();
    bd_store_receiver(&root,&[10,20],&[11,21],1,1);
    // Receiver epoch and checksum violation cannot be accepted.
    let mut r=std::fs::read(root.join("receiver.bin")).unwrap();
    r[16+12]^=1;
    std::fs::write(root.join("receiver.bin"),&r).unwrap();
    assert!(bd_load_receiver(&root).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
