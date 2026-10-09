// DeltaGuard B1-B1B1-B1-A research-only: fsync'd multigeneration WAL,
// bound event chain and receiver receipt. Public BLAKE3 != authentication.
use std::path::PathBuf;
const BE_REC: usize = 88;
const BE_MAGIC: &[u8;8] = b"DGEVT101";
const BE_COMMIT: &[u8;8] = b"DGCOM101";
const BE_ACK: &[u8;8] = b"DGACK101";
const BE_RECEIPT: &[u8;8] = b"DGREC101";
const BE_GENS:usize=100;
const BE_NS:[usize;2]=[256,65536];

fn be_digest(bytes:&[u8])->[u8;32] {
    *blake3::hash(bytes).as_bytes()
}
fn be_genesis(owner:u8,initial:&[u8])->[u8;32] {
    let mut h=blake3::Hasher::new();
    h.update(b"deltameter:B1B1B1-B1A:genesis:2026-10-09");
    h.update(&B1_EPOCH.to_le_bytes());
    h.update(&b1_key_id());
    h.update(&[owner,11]);
    h.update(initial);
    *h.finalize().as_bytes()
}
fn be_event(owner:u8,seq:usize,op:u8,token:u64,prev:&[u8;32])->[u8;BE_REC]{
    assert!([1,2].contains(&owner));
    assert!((2..=BE_GENS+1).contains(&seq));
    assert!([1,2].contains(&op));
    let mut ev=[0u8;BE_REC];
    ev[..8].copy_from_slice(BE_MAGIC);
    ev[8..16].copy_from_slice(&B1_EPOCH.to_le_bytes());
    ev[16..24].copy_from_slice(&(seq as u64).to_le_bytes());
    ev[24]=owner;
    ev[25]=op;
    ev[32..40].copy_from_slice(&token.to_le_bytes());
    ev[40..56].copy_from_slice(&prev[..16]);
    let mut h=blake3::Hasher::new();
    h.update(b"deltameter:B1B1B1-B1A:event:v1");
    h.update(&b1_key_id());
    h.update(&[11]);
    h.update(prev);
    h.update(&ev[..56]);
    ev[56..88].copy_from_slice(h.finalize().as_bytes());
    ev
}
fn be_check_event(ev:&[u8],owner:u8,seq:usize,prev:&[u8;32])
    ->Result<(u8,u64,[u8;32]),&'static str>{
    if ev.len()!=BE_REC || &ev[..8]!=BE_MAGIC {return Err("event-size-magic");}
    if u64::from_le_bytes(ev[8..16].try_into().unwrap())!=B1_EPOCH {return Err("event-epoch");}
    if u64::from_le_bytes(ev[16..24].try_into().unwrap())!=seq as u64 {return Err("event-generation");}
    if ev[24]!=owner || ![1,2].contains(&ev[25]) || ev[26..32]!=[0;6] {
        return Err("event-owner-op-padding");
    }
    if ev[40..56]!=prev[..16] {return Err("event-parent");}
    let token=u64::from_le_bytes(ev[32..40].try_into().unwrap());
    let expected=be_event(owner,seq,ev[25],token,prev);
    if expected!=ev {return Err("event-hash");}
    Ok((ev[25],token,ev[56..88].try_into().unwrap()))
}
fn be_apply(target:&mut Vec<u64>,op:u8,token:u64)->Result<(),&'static str>{
    match (op,target.binary_search(&token)){
        (1,Err(pos))=>target.insert(pos,token),
        (2,Ok(pos))=>{target.remove(pos);},
        (1,Ok(_))=>return Err("reinsert-without-remove"),
        (2,Err(_))=>return Err("delete-missing"),
        _=>return Err("unknown-event-op"),
    }
    Ok(())
}
fn be_mark(magic:&[u8;8],seq:usize,digest:&[u8;32])->[u8;56]{
    let mut out=[0_u8;56];
    out[..8].copy_from_slice(magic);
    out[8..16].copy_from_slice(&B1_EPOCH.to_le_bytes());
    out[16..24].copy_from_slice(&(seq as u64).to_le_bytes());
    out[24..56].copy_from_slice(digest);
    out
}
fn be_read_mark(dir:&Path,name:&str,magic:&[u8;8])->Result<(usize,[u8;32]),&'static str>{
    let m=std::fs::read(dir.join(name)).map_err(|_|"mark-missing")?;
    if m.len()!=56 || &m[..8]!=magic ||
        u64::from_le_bytes(m[8..16].try_into().unwrap())!=B1_EPOCH {
        return Err("mark-identity");
    }
    let seq=u64::from_le_bytes(m[16..24].try_into().unwrap()) as usize;
    let digest=m[24..56].try_into().unwrap();
    Ok((seq,digest))
}
struct BESource {
    tokens:Vec<u64>,
    seq:usize,
    head:[u8;32],
    ack:usize,
    ack_head:[u8;32],
    log:Vec<u8>,
}
fn be_source_load(dir:&Path,owner:u8)->Result<BESource,&'static str>{
    let base=std::fs::read(dir.join("base.snap")).map_err(|_|"base-missing")?;
    let initial=b1_decode(&base,owner,B1_FULL,B1_EPOCH,1)?;
    let mut tokens=b11_as_words(initial);
    let genesis=be_genesis(owner,&base);
    let (seq,mark_head)=be_read_mark(dir,"commit.mark",BE_COMMIT)?;
    if !(1..=BE_GENS+1).contains(&seq){return Err("commit-range");}
    let log=std::fs::read(dir.join("chain.wal")).map_err(|_|"wal-missing")?;
    if log.len()!=(seq-1)*BE_REC {return Err("wal-length");}
    let mut head=genesis;
    for (i,ev) in log.chunks_exact(BE_REC).enumerate(){
        let (op,token,next)=be_check_event(ev,owner,i+2,&head)?;
        be_apply(&mut tokens,op,token)?;
        head=next;
    }
    if head!=mark_head {return Err("commit-chain-head");}
    let (ack,ack_head)=be_read_mark(dir,"ack.mark",BE_ACK)?;
    if ack>seq || ack<1 {return Err("ack-ahead-or-before-base");}
    let canonical_ack=if ack==1 {genesis}else{
        log[(ack-2)*BE_REC+56..(ack-2)*BE_REC+88].try_into().unwrap()
    };
    if canonical_ack!=ack_head {return Err("ack-not-ancestor");}
    Ok(BESource{tokens,seq,head,ack,ack_head,log})
}
fn be_seed(args:&[String]){
    assert_eq!(args.len(),6);
    let worker:usize=args[1].parse().unwrap();
    let lane:usize=args[2].parse().unwrap();
    let rep:usize=args[3].parse().unwrap();
    let owner:u8=args[4].parse().unwrap();
    assert!((1..=5).contains(&worker)&&lane<2&&rep<3);
    let root=PathBuf::from(&args[5]);
    let dir=bd_dir(&root,owner);
    std::fs::create_dir_all(&dir).unwrap();
    let (initial,seed)=b11_owner_initial(worker,BE_NS[lane],48,rep,owner);
    let base=b1_encode(owner,B1_FULL,B1_EPOCH,1,&b1_full_bytes(&initial));
    bd_atomic(&dir,"base.snap",&base);
    bd_atomic(&dir,"seed.bin",&(seed+9_999).to_le_bytes());
    let file=std::fs::File::create(dir.join("chain.wal")).unwrap();
    file.sync_all().unwrap();
    std::fs::File::open(&dir).unwrap().sync_all().unwrap();
    let genesis=be_genesis(owner,&base);
    bd_atomic(&dir,"commit.mark",&be_mark(BE_COMMIT,1,&genesis));
    bd_atomic(&dir,"ack.mark",&be_mark(BE_ACK,1,&genesis));
}
fn be_writer(args:&[String]){
    use std::io::Write;
    assert_eq!(args.len(),4);
    let owner:u8=args[1].parse().unwrap();
    let seq:usize=args[2].parse().unwrap();
    let dir=PathBuf::from(&args[3]);
    let src=be_source_load(&dir,owner).expect("write must reopen source disk");
    assert_eq!(seq,src.seq+1);
    let bytes=std::fs::read(dir.join("seed.bin")).unwrap();
    assert_eq!(bytes.len(),8);
    let token=u64::from_le_bytes(bytes.try_into().unwrap());
    let op=if seq.is_multiple_of(2){1}else{2};
    let mut check=src.tokens.clone();
    be_apply(&mut check,op,token).expect("100-gen insert-delete ABA");
    let ev=be_event(owner,seq,op,token,&src.head);
    let mut wal=std::fs::OpenOptions::new().append(true).open(dir.join("chain.wal")).unwrap();
    wal.write_all(&ev).expect("write source WAL");
    wal.sync_all().expect("fsync source WAL before commit");
    drop(wal);
    bd_atomic(&dir,"commit.mark",&be_mark(BE_COMMIT,seq,&ev[56..88].try_into().unwrap()));
    let reloaded=be_source_load(&dir,owner).expect("replay after sync");
    assert_eq!(reloaded.seq,seq);
    assert_eq!(reloaded.tokens,check);
}
fn be_save_receipt(root:&Path,lists:&[Vec<u64>;2],seqs:[usize;2],heads:[[u8;32];2])
    ->usize
{
    let fa=b1_encode(1,B1_FULL,B1_EPOCH,seqs[0] as u64,&b1_full_bytes(&lists[0]));
    let fb=b1_encode(2,B1_FULL,B1_EPOCH,seqs[1] as u64,&b1_full_bytes(&lists[1]));
    let mut body=Vec::new();
    body.extend_from_slice(BE_RECEIPT);
    body.extend_from_slice(&B1_EPOCH.to_le_bytes());
    body.extend_from_slice(&(seqs[0] as u64).to_le_bytes());
    body.extend_from_slice(&(seqs[1] as u64).to_le_bytes());
    body.extend_from_slice(&heads[0]);
    body.extend_from_slice(&heads[1]);
    body.extend_from_slice(&(fa.len() as u32).to_le_bytes());
    body.extend_from_slice(&(fb.len() as u32).to_le_bytes());
    body.extend_from_slice(&fa);
    body.extend_from_slice(&fb);
    let hash=be_digest(&body);
    body.extend_from_slice(&hash);
    let bytes=body.len();
    bd_atomic(root,"receiver.chain",&body);
    bytes
}
struct BEReceipt {lists:[Vec<u64>;2],seq:[usize;2],heads:[[u8;32];2]}
fn be_load_receipt(root:&Path)->Result<BEReceipt,&'static str>{
    let bytes=std::fs::read(root.join("receiver.chain")).map_err(|_|"receipt-missing")?;
    if bytes.len()<136 || &bytes[..8]!=BE_RECEIPT ||
        u64::from_le_bytes(bytes[8..16].try_into().unwrap())!=B1_EPOCH {
        return Err("receipt-identity");
    }
    let g1=u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
    let g2=u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;
    let heads=[bytes[32..64].try_into().unwrap(),bytes[64..96].try_into().unwrap()];
    let n1=u32::from_le_bytes(bytes[96..100].try_into().unwrap()) as usize;
    let n2=u32::from_le_bytes(bytes[100..104].try_into().unwrap()) as usize;
    if n1>B1_MAX_PAYLOAD+B1_HEADER||n2>B1_MAX_PAYLOAD+B1_HEADER||
        bytes.len()!=104+n1+n2+32{return Err("receipt-size");}
    if bytes[104+n1+n2..]!=be_digest(&bytes[..104+n1+n2]){return Err("receipt-checksum");}
    if ![1,2].contains(&g1.min(2)) || ![1,2].contains(&g2.min(2)) ||
        g1>BE_GENS+1 || g2>BE_GENS+1{return Err("receipt-generation");}
    let fa=&bytes[104..104+n1];let fb=&bytes[104+n1..104+n1+n2];
    let a=b11_as_words(b1_decode(fa,1,B1_FULL,B1_EPOCH,g1 as u64)?);
    let b=b11_as_words(b1_decode(fb,2,B1_FULL,B1_EPOCH,g2 as u64)?);
    Ok(BEReceipt{lists:[a,b],seq:[g1,g2],heads})
}
fn be_init(worker:usize,lane:usize,rep:usize,root:&Path){
    use std::process::{Command,Stdio};
    std::fs::create_dir_all(root).unwrap();
    let exe=std::env::current_exe().unwrap();
    let mut children=Vec::new();
    for owner in 1..=2{
        children.push(Command::new(&exe).args(["--seed", &worker.to_string(),
            &lane.to_string(),&rep.to_string(),&owner.to_string(),
            root.to_str().unwrap()]).stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap());
    }
    for mut c in children{assert!(c.wait().unwrap().success());}
    let sources=[be_source_load(&bd_dir(root,1),1).unwrap(),
        be_source_load(&bd_dir(root,2),2).unwrap()];
    be_save_receipt(root,&[sources[0].tokens.clone(),sources[1].tokens.clone()],
        [1,1],[sources[0].head,sources[1].head]);
}
fn be_sender(args:&[String]){
    use std::io::{Read,Write};
    use std::net::TcpStream;
    use std::time::Duration;
    assert_eq!(args.len(),5);
    let owner:u8=args[1].parse().unwrap();
    let seq:usize=args[2].parse().unwrap();
    let dir=PathBuf::from(&args[3]);
    let addr=&args[4];
    let src=be_source_load(&dir,owner).expect("sender must reopen on-disk source");
    assert!(seq>=2 && seq<=src.seq);
    let offset=(seq-2)*BE_REC;
    let record=&src.log[offset..offset+BE_REC];
    let op=record[25];
    let token=u64::from_le_bytes(record[32..40].try_into().unwrap());
    let delta=if op==1{b1b_insert(token)}else{b1b_remove(token)};
    let frame=b1_encode(owner,B1_DELTA,B1_EPOCH,seq as u64,&delta);
    let mut sock=TcpStream::connect(addr).unwrap();
    sock.set_nodelay(true).unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    sock.set_write_timeout(Some(Duration::from_secs(4))).unwrap();
    sock.write_all(&[owner]).unwrap();
    sock.write_all(record).unwrap();
    sock.write_all(&frame).unwrap();
    let mut ack=[0_u8;BB_ACK];
    match sock.read_exact(&mut ack){
        Ok(())=>{
            assert_eq!(ack,bb_ack(true,seq));
            // Owner persists the delivery watermark AFTER receiver's receipt fsync and ACK.
            // Only a monotonically contiguous ack can be committed.
            if src.ack<seq {
                assert_eq!(src.ack+1,seq);
                bd_atomic(&dir,"ack.mark",&be_mark(BE_ACK,seq,
                    &record[56..88].try_into().unwrap()));
            }
        }
        Err(e) if matches!(e.kind(),std::io::ErrorKind::UnexpectedEof |
            std::io::ErrorKind::ConnectionReset |
            std::io::ErrorKind::TimedOut |
            std::io::ErrorKind::WouldBlock)=>{},
        Err(e)=>panic!("sender ACK IO: {e}"),
    }
}
fn be_read_pair(listener:&std::net::TcpListener)
    ->([std::net::TcpStream;2],[[u8;BE_REC];2],[Vec<u8>;2],usize)
{
    use std::io::Read;
    let mut sockets:[Option<std::net::TcpStream>;2]=[None,None];
    let mut records:[Option<[u8;BE_REC]>;2]=[None,None];
    let mut frames:[Option<Vec<u8>>;2]=[None,None];
    let mut bytes=0;
    for _ in 0..2 {
        let (mut stream,_)=listener.accept().unwrap();
        bc_sock_setup(&stream);
        let mut hello=[0_u8;1];
        stream.read_exact(&mut hello).unwrap();bytes+=1;
        let owner=hello[0];
        assert!((1..=2).contains(&owner));
        let ix=(owner-1) as usize;
        assert!(sockets[ix].is_none(),"duplicate source owner");
        let mut rec=[0_u8;BE_REC];
        stream.read_exact(&mut rec).unwrap();bytes+=BE_REC;
        let mut header=[0_u8;B1_HEADER];
        stream.read_exact(&mut header).unwrap();bytes+=B1_HEADER;
        let len=u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
        assert_eq!(len,9);
        let mut frame=header.to_vec();
        frame.resize(B1_HEADER+len,0);
        stream.read_exact(&mut frame[B1_HEADER..]).unwrap();bytes+=len;
        records[ix]=Some(rec);frames[ix]=Some(frame);sockets[ix]=Some(stream);
    }
    ([sockets[0].take().unwrap(),sockets[1].take().unwrap()],
        [records[0].take().unwrap(),records[1].take().unwrap()],
        [frames[0].take().unwrap(),frames[1].take().unwrap()],bytes)
}
fn be_validate_transition(receipt:&BEReceipt,recs:&[[u8;BE_REC];2],
    frames:&[Vec<u8>;2],seq:usize)->Result<(BEReceipt,bool),&'static str>{
    let mut next=BEReceipt{lists:receipt.lists.clone(),seq:receipt.seq,heads:receipt.heads};
    let mut fresh=[false;2];
    for i in 0..2 {
        let owner=(i+1) as u8;
        let this_g=receipt.seq[i];
        let decoded=b1_decode(&frames[i],owner,B1_DELTA,B1_EPOCH,seq as u64)?;
        if decoded.len()!=9 {return Err("delta-size");}
        let token=u64::from_le_bytes(decoded[1..9].try_into().unwrap());
        let op=decoded[0];
        if seq==this_g+1 {
            let (ev_op,ev_token,new_hash)=be_check_event(&recs[i],owner,seq,&receipt.heads[i])?;
            if ev_op!=op||ev_token!=token {return Err("event-frame-conflict");}
            be_apply(&mut next.lists[i],op,token)?;
            next.heads[i]=new_hash;
            next.seq[i]=seq;
            fresh[i]=true;
        } else if seq==this_g {
            // A duplicate is accepted based on EXACT persisted digest,
            // never by membership of the token (ABA vulnerability).
            if recs[i][56..88]!=receipt.heads[i] {return Err("conflicting-same-generation");}
            if &recs[i][..8]!=BE_MAGIC ||
                u64::from_le_bytes(recs[i][16..24].try_into().unwrap())!=seq as u64 ||
                recs[i][24]!=owner || recs[i][25]!=op ||
                recs[i][32..40]!=decoded[1..9] ||
                u64::from_le_bytes(recs[i][8..16].try_into().unwrap())!=B1_EPOCH {
                return Err("replay-frame-conflict");
            }
        } else {
            return Err("missing-stale-or-rolledback-generation");
        }
    }
    if fresh[0]!=fresh[1] {return Err("asymmetric-receiver-commit");}
    Ok((next,fresh[0]))
}
fn be_accept_and_commit(root:&Path,listener:&std::net::TcpListener,seq:usize,
    crash_after_one_ack:bool)->usize
{
    use std::io::Write;
    let (mut socks,events,frames,mut bytes)=be_read_pair(listener);
    let prev=be_load_receipt(root).unwrap();
    let (next,fresh)=be_validate_transition(&prev,&events,&frames,seq)
        .expect("strict B1B1B1A receipt chain + exact source oracle");
    if fresh {
        be_save_receipt(root,&next.lists,next.seq,next.heads);
    }
    let reopened=be_load_receipt(root).unwrap();
    assert_eq!(reopened.seq,[seq,seq]);
    assert_eq!(reopened.heads,next.heads);
    assert_eq!(reopened.lists,next.lists);
    socks[0].write_all(&bb_ack(true,seq)).unwrap();bytes+=BB_ACK;
    if crash_after_one_ack {
        // The independent receiver child tells controller ACK#1 crossed
        // the socket before we SIGKILL the child, BEFORE ACK#2.
        println!("BE_ACK1_READY generation={seq}");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(15));
        panic!("receiver was not killed at first-ACK point");
    }
    socks[1].write_all(&bb_ack(true,seq)).unwrap();bytes+=BB_ACK;
    bytes
}
fn be_child_send(seq:usize,root:&Path,address:&str)->[std::process::Child;2]{
    use std::process::{Command,Stdio};
    let exe=std::env::current_exe().unwrap();
    [1_u8,2_u8].map(|owner|
        Command::new(&exe).args(["--send",&owner.to_string(),
            &seq.to_string(),bd_dir(root,owner).to_str().unwrap(),address])
        .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap())
}
fn be_parent_round(root:&Path,seq:usize)->usize {
    let listener=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap();
    let addr=listener.local_addr().unwrap().to_string();
    let mut children=be_child_send(seq,root,&addr);
    let bytes=be_accept_and_commit(root,&listener,seq,false);
    for child in &mut children {assert!(child.wait().unwrap().success());}
    bytes
}
fn be_receiver_child(args:&[String]){
    assert_eq!(args.len(),3);
    let seq:usize=args[1].parse().unwrap();
    let root=Path::new(&args[2]);
    let listener=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap();
    println!("BE_PORT {}",listener.local_addr().unwrap());
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    be_accept_and_commit(root,&listener,seq,true);
}
fn be_crash_first_ack(root:&Path,seq:usize)->(usize,usize) {
    use std::io::{BufRead,BufReader};
    use std::process::{Command,Stdio};
    let mut receiver=Command::new(std::env::current_exe().unwrap())
        .args(["--receiver",&seq.to_string(),root.to_str().unwrap()])
        .stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap();
    let stdout=receiver.stdout.take().unwrap();
    let mut lines=BufReader::new(stdout);
    let mut port=String::new(); lines.read_line(&mut port).unwrap();
    assert!(port.starts_with("BE_PORT "));
    let addr=port.trim().strip_prefix("BE_PORT ").unwrap();
    let mut owners=be_child_send(seq,root,addr);
    let mut signal=String::new();
    lines.read_line(&mut signal).unwrap();
    assert_eq!(signal.trim(),format!("BE_ACK1_READY generation={seq}"));
    receiver.kill().expect("real OS receiver SIGKILL after ACK1");
    assert!(!receiver.wait().unwrap().success());
    for owner in &mut owners {assert!(owner.wait().unwrap().success());}
    let persisted=be_load_receipt(root).unwrap();
    assert_eq!(persisted.seq,[seq,seq]);
    assert_eq!(be_source_load(&bd_dir(root,1),1).unwrap().ack,seq);
    assert_eq!(be_source_load(&bd_dir(root,2),2).unwrap().ack,seq-1);
    // A fresh receiver can identify a precisely identical frame after crash.
    let retry=be_parent_round(root,seq);
    let reread=be_load_receipt(root).unwrap();
    assert_eq!(reread.seq,[seq,seq]);
    for owner in 1..=2 {
        assert_eq!(be_source_load(&bd_dir(root,owner),owner).unwrap().ack,seq);
    }
    (BE_REC*2+2*(1+B1_HEADER+9)+BB_ACK,retry)
}
fn be_writer_pair(root:&Path,seq:usize){
    use std::process::{Command,Stdio};
    let exe=std::env::current_exe().unwrap();
    let mut children=Vec::new();
    for owner in 1..=2{
        children.push(Command::new(&exe).args(["--write",&owner.to_string(),
            &seq.to_string(),bd_dir(root,owner).to_str().unwrap()])
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap());
    }
    for mut c in children {assert!(c.wait().unwrap().success());}
}
fn be_oracle(worker:usize,lane:usize,rep:usize,seq:usize,receiver:&BEReceipt){
    assert_eq!(receiver.seq,[seq,seq]);
    let mut expected=[Vec::new(),Vec::new()];
    for owner in 1..=2{
        let (mut initial,seed)=b11_owner_initial(worker,BE_NS[lane],48,rep,owner);
        if seq%2==0 {initial.push(seed+9_999);}
        expected[(owner-1) as usize]=initial;
    }
    assert_eq!(receiver.lists,expected);
    assert_eq!(b0_diff(&receiver.lists[0],&receiver.lists[1]).len(),48);
    for owner in 1..=2{
        let src=be_source_load(&bd_dir(Path::new(""),owner),owner);
        let _=src; // Oracle never supplies source state after restart.
    }
}
fn be_negative(worker:usize,lane:usize,rep:usize,root:&Path){
    let receipt=be_load_receipt(root).unwrap();
    let seq=receipt.seq[0];
    assert_eq!(seq,BE_GENS+1);
    let src1=be_source_load(&bd_dir(root,1),1).unwrap();
    let src2=be_source_load(&bd_dir(root,2),2).unwrap();
    let ev1: [u8;BE_REC]=src1.log[(seq-2)*BE_REC..(seq-1)*BE_REC].try_into().unwrap();
    let ev2: [u8;BE_REC]=src2.log[(seq-2)*BE_REC..(seq-1)*BE_REC].try_into().unwrap();
    let token=u64::from_le_bytes(ev1[32..40].try_into().unwrap());
    let b1=b1_encode(1,B1_DELTA,B1_EPOCH,seq as u64,&b1b_remove(token));
    let b2=b1_encode(2,B1_DELTA,B1_EPOCH,seq as u64,
        &b1b_remove(u64::from_le_bytes(ev2[32..40].try_into().unwrap())));
    // Real persisted digest MUST reject changed payload even though an
    // ABA insert/remove history may leave the same membership outcome.
    let mut changed=ev1;changed[32..40].copy_from_slice(&(token+1).to_le_bytes());
    assert!(be_validate_transition(&receipt,&[changed,ev2],&[b1.clone(),b2.clone()],seq).is_err());
    let mut changed_digest=ev1;changed_digest[65]^=1;
    assert!(be_validate_transition(&receipt,&[changed_digest,ev2],&[b1.clone(),b2.clone()],seq).is_err());
    assert!(be_validate_transition(&receipt,&[ev1,ev2],&[b1.clone(),b2.clone()],seq-1).is_err());
    assert!(be_validate_transition(&receipt,&[ev1,ev2],&[b1.clone(),b2.clone()],seq+2).is_err());
    let wrong=b1_encode(2,B1_DELTA,B1_EPOCH,seq as u64,&b1b_remove(token));
    assert!(be_validate_transition(&receipt,&[ev1,ev2],&[wrong,b2.clone()],seq).is_err());
    assert!(be_apply(&mut receipt.lists[0].clone(),2,token+555).is_err());
    // Mutate private committed WAL and verify source refuses it, then restore.
    let dir=bd_dir(root,1);
    let wal=std::fs::read(dir.join("chain.wal")).unwrap();
    let mut truncated=wal.clone();truncated.pop();
    std::fs::write(dir.join("chain.wal"),&truncated).unwrap();
    assert!(be_source_load(&dir,1).is_err());
    std::fs::write(dir.join("chain.wal"),&wal).unwrap();
    let original=std::fs::read(dir.join("ack.mark")).unwrap();
    bd_atomic(&dir,"ack.mark",&be_mark(BE_ACK,seq+1,&src1.head));
    assert!(be_source_load(&dir,1).is_err());
    bd_atomic(&dir,"ack.mark",&original);
    let receipt_bytes=std::fs::read(root.join("receiver.chain")).unwrap();
    let mut bad=receipt_bytes.clone();bad[32]^=1;
    std::fs::write(root.join("receiver.chain"),&bad).unwrap();
    assert!(be_load_receipt(root).is_err());
    bd_atomic(root,"receiver.chain",&receipt_bytes);
    assert_eq!(be_load_receipt(root).unwrap().seq,[seq,seq]);
    let _=(worker,lane,rep);
}
fn be_worker(worker:usize){
    assert!((1..=5).contains(&worker));
    let mut cases=0;let mut interruptions=0;
    for lane in 0..2{
        for rep in 0..3{
            let root=std::env::temp_dir().join(format!(
                "deltameter-b1b1b1a-w{worker}-l{lane}-r{rep}-pid{}",std::process::id()));
            if root.exists(){std::fs::remove_dir_all(&root).unwrap();}
            be_init(worker,lane,rep,&root);
            for seq in 2..=BE_GENS+1{
                be_writer_pair(&root,seq);
                let (bytes,retry)=if seq==51 {
                    interruptions+=1;
                    let (b1,b2)=be_crash_first_ack(&root,seq);
                    (b1,b2)
                }else{(be_parent_round(&root,seq),0)};
                let receiver=be_load_receipt(&root).unwrap();
                be_oracle(worker,lane,rep,seq,&receiver);
                for owner in 1..=2{
                    let src=be_source_load(&bd_dir(&root,owner),owner).unwrap();
                    assert_eq!(src.seq,seq);
                    assert_eq!(src.ack,seq);
                    assert_eq!(src.tokens,receiver.lists[(owner-1) as usize]);
                    assert_eq!(src.head,receiver.heads[(owner-1) as usize]);
                }
                println!("B1B1B1A_SAMPLE worker={worker} lane={lane} rep={rep} generation={seq} N={} op={} source_wal_bytes={} wire_bytes={bytes} retry_bytes={retry} receiver_crash={} exact=1",
                    BE_NS[lane],if seq%2==0{"insert"}else{"delete"},
                    2*(seq-1)*BE_REC,u8::from(seq==51));
                cases+=1;
            }
            be_negative(worker,lane,rep,&root);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    assert_eq!(cases,600);
    assert_eq!(interruptions,6);
    println!("DELTAGUARD_B1B1B1A_WORKER_PASS worker={worker} records={cases} actual_receiver_sigkill={interruptions} negative_groups=6");
}
