// B1-B1B1-A research: physical TCP error injection + safe FULL reset only.
// Public fixture and unkeyed checksum are NOT malicious-peer authentication.
const BC_LANES: [(usize, usize, usize); 2] = [(256, 10, 48), (65536, 100, 48)];
const BC_CASES: [&str; 13] = [
    "disconnect", "short_header", "short_body", "checksum", "owner",
    "epoch", "generation", "key", "noncanonical", "membership",
    "oversize", "ackloss", "sigkill",
];

fn bc_cpu_ticks() -> u64 {
    let s = std::fs::read_to_string("/proc/self/stat").expect("Linux proc stat");
    let end = s.rfind(')').expect("comm closing bracket");
    let fields: Vec<&str> = s[end + 1..].split_whitespace().collect();
    // After comm ends: field3 = fields[0], utime=field14 -> 11,
    // stime=field15 -> 12. Linux process CPU ticks, NOT nanoseconds.
    fields[11].parse::<u64>().unwrap() + fields[12].parse::<u64>().unwrap()
}
fn bc_source(worker: usize, lane: usize, rep: usize, owner: u8, seq: usize) -> Vec<u64> {
    let (n, _, d) = BC_LANES[lane];
    let (mut source, next) = b11_owner_initial(worker, n, d, rep, owner);
    for generation in 2..=seq {
        source.push(next + generation as u64);
    }
    source
}
fn bc_token(worker: usize, lane: usize, rep: usize) -> u64 {
    let (n, _, d) = BC_LANES[lane];
    b11_owner_initial(worker, n, d, rep, 1).1 + 2
}
fn bc_full_child(args: &[String]) {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;
    assert_eq!(args.len(), 7);
    let worker: usize = args[1].parse().unwrap();
    let lane: usize = args[2].parse().unwrap();
    let rep: usize = args[3].parse().unwrap();
    let owner: u8 = args[4].parse().unwrap();
    let seq: usize = args[5].parse().unwrap();
    assert!((1..=5).contains(&worker) && lane < 2 && rep < 3 && [1, 2].contains(&seq));
    let (_, mbps, _) = BC_LANES[lane];
    let source = bc_source(worker, lane, rep, owner, seq);
    let mut socket = TcpStream::connect(&args[6]).unwrap();
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.write_all(&[owner]).unwrap();
    let mut request = [0; B11_REQ];
    socket.read_exact(&mut request).unwrap();
    assert_eq!(request, b11_req(seq));
    let frame = b1_encode(owner, B1_FULL, B1_EPOCH, seq as u64, &b1_full_bytes(&source));
    b11_paced_write(&mut socket, &frame, mbps);
    let mut ack = [0_u8; BB_ACK];
    socket.read_exact(&mut ack).unwrap();
    assert_eq!(ack, bb_ack(true, seq));
    let mut telemetry = [0_u8; 16];
    telemetry[..8].copy_from_slice(&b11_rss().to_le_bytes());
    telemetry[8..].copy_from_slice(&bc_cpu_ticks().to_le_bytes());
    socket.write_all(&telemetry).unwrap();
}

fn bc_fault_child(args: &[String]) {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;
    assert_eq!(args.len(), 7);
    let worker: usize = args[1].parse().unwrap();
    let lane: usize = args[2].parse().unwrap();
    let rep: usize = args[3].parse().unwrap();
    let case = &args[4];
    let seq: usize = args[5].parse().unwrap();
    assert_eq!(seq, 2);
    assert!(BC_CASES.contains(&case.as_str()));
    let token = bc_token(worker, lane, rep);
    let good = b1_encode(1, B1_DELTA, B1_EPOCH, 2, &b1b_insert(token));
    let mut frame = good.clone();
    match case.as_str() {
        "checksum" => frame[48] ^= 1,
        "owner" => frame = b1_encode(2, B1_DELTA, B1_EPOCH, 2, &b1b_insert(token)),
        "epoch" => frame = b1_encode(1, B1_DELTA, B1_EPOCH + 1, 2, &b1b_insert(token)),
        "generation" => frame = b1_encode(1, B1_DELTA, B1_EPOCH, 3, &b1b_insert(token)),
        "key" => frame[32] ^= 1,
        "noncanonical" => {
            let mut body = Vec::new();
            body.extend_from_slice(&b1b_insert(token + 1));
            body.extend_from_slice(&b1b_insert(token));
            frame = b1_encode(1, B1_DELTA, B1_EPOCH, 2, &body);
        }
        "membership" => {
            let existing = bc_source(worker, lane, rep, 1, 1)[0];
            frame = b1_encode(1, B1_DELTA, B1_EPOCH, 2, &b1b_insert(existing));
        }
        "oversize" => frame[28..32].copy_from_slice(&((B1_MAX_PAYLOAD as u32) + 1).to_le_bytes()),
        _ => (),
    }
    let mut socket = TcpStream::connect(&args[6]).unwrap();
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_millis(40))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.write_all(&[1]).unwrap();
    match case.as_str() {
        "disconnect" => (),
        "short_header" => socket.write_all(&frame[..16]).unwrap(),
        "short_body" => socket.write_all(&frame[..B1_HEADER + 3]).unwrap(),
        "oversize" => socket.write_all(&frame[..B1_HEADER]).unwrap(),
        "sigkill" => {
            socket.write_all(&frame[..B1_HEADER + 4]).unwrap();
            std::thread::sleep(Duration::from_secs(5));
            panic!("SIGKILL was not delivered to partial-body child");
        }
        "ackloss" => {
            socket.write_all(&good).unwrap();
            let mut ack = [0_u8; BB_ACK];
            let e = socket.read_exact(&mut ack).unwrap_err();
            assert!(matches!(e.kind(), std::io::ErrorKind::TimedOut |
                std::io::ErrorKind::WouldBlock));
            socket.write_all(&good).unwrap();
            socket.read_exact(&mut ack).unwrap();
            assert_eq!(ack, bb_ack(true, 2));
        }
        _ => {
            socket.write_all(&frame).unwrap();
            let mut nack = [0_u8; BB_ACK];
            socket.read_exact(&mut nack).unwrap();
            assert_eq!(nack, bb_ack(false, 2));
        }
    }
}
fn bc_child_cmd(role: &str, worker: usize, lane: usize, rep: usize,
    detail: &str, seq: usize, addr: &str) -> std::process::Child
{
    use std::process::{Command, Stdio};
    Command::new(std::env::current_exe().unwrap())
        .args([role.to_string(), worker.to_string(), lane.to_string(),
            rep.to_string(), detail.to_string(), seq.to_string(), addr.to_string()])
        .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap()
}
fn bc_sock_setup(stream: &std::net::TcpStream) {
    use std::time::Duration;
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
}
fn bc_full_pair(worker: usize, lane: usize, rep: usize, seq: usize)
    -> ([Vec<u64>;2], usize, [u64;2], [u64;2])
{
    use std::io::{Read, Write};
    use std::net::TcpListener;
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let mut children: [Option<std::process::Child>;2] = [None, None];
    for owner in 1..=2 {
        children[(owner - 1) as usize] =
            Some(bc_child_cmd("--full", worker, lane, rep, &owner.to_string(), seq, &addr));
    }
    let mut decoded: [Option<Vec<u64>>;2] = [None,None];
    let mut rss = [0_u64;2];
    let mut cpu = [0_u64;2];
    let mut wire = 0_usize;
    for _ in 0..2 {
        let (mut stream, _) = listener.accept().unwrap();
        bc_sock_setup(&stream);
        let mut hello = [0_u8;1];
        stream.read_exact(&mut hello).unwrap();
        let owner = hello[0];
        assert!((1..=2).contains(&owner));
        let idx = (owner - 1) as usize;
        assert!(decoded[idx].is_none(), "duplicate physical owner");
        wire += 1;
        stream.write_all(&b11_req(seq)).unwrap();
        wire += B11_REQ;
        let mut header = [0_u8;B1_HEADER];
        stream.read_exact(&mut header).unwrap();
        let size = u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
        assert!(size <= B1_MAX_PAYLOAD);
        let mut packet=header.to_vec();
        packet.resize(B1_HEADER+size,0);
        stream.read_exact(&mut packet[B1_HEADER..]).unwrap();
        wire += packet.len();
        let body = b1_decode(&packet,owner,B1_FULL,B1_EPOCH,seq as u64).unwrap();
        let list = b11_as_words(body);
        assert_eq!(list,bc_source(worker,lane,rep,owner,seq),
            "only actual source full TCP frame can restore oracle");
        decoded[idx] = Some(list);
        stream.write_all(&bb_ack(true,seq)).unwrap();
        wire += BB_ACK;
        let mut report = [0_u8;16];
        stream.read_exact(&mut report).unwrap();
        wire += report.len();
        rss[idx] = u64::from_le_bytes(report[..8].try_into().unwrap());
        cpu[idx] = u64::from_le_bytes(report[8..].try_into().unwrap());
        assert!(rss[idx] > 0);
    }
    for child in children.iter_mut().flatten() {
        assert!(child.wait().unwrap().success(),"full source child");
    }
    ([decoded[0].take().unwrap(),decoded[1].take().unwrap()],wire,rss,cpu)
}

fn bc_read_bytes(socket: &mut std::net::TcpStream, target: usize) -> (Vec<u8>,bool) {
    use std::io::Read;
    let mut got=Vec::new();
    while got.len()<target {
        let mut buf=[0_u8;4096];
        let max=(target-got.len()).min(buf.len());
        match socket.read(&mut buf[..max]) {
            Ok(0)=>break,
            Ok(n)=>got.extend_from_slice(&buf[..n]),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::UnexpectedEof |
                std::io::ErrorKind::ConnectionReset)=>break,
            Err(e)=>panic!("physical TCP read failed: {e}"),
        }
    }
    let complete=got.len()==target;
    (got,complete)
}
fn bc_fault(worker: usize, lane: usize, rep: usize, case: &str,
    receiver: &mut [Vec<u64>;2]) -> (usize, bool)
{
    use std::io::{Read,Write};
    use std::net::TcpListener;
    assert!(BC_CASES.contains(&case));
    let listener = TcpListener::bind(("127.0.0.1",0)).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let mut child=bc_child_cmd("--fault",worker,lane,rep,case,2,&addr);
    let (mut stream,_)=listener.accept().unwrap();
    bc_sock_setup(&stream);
    let mut hello=[0_u8;1];
    stream.read_exact(&mut hello).unwrap();
    assert_eq!(hello[0],1);
    let mut wire=1_usize;
    let before=receiver.clone();
    let (header,ready)=bc_read_bytes(&mut stream,B1_HEADER);
    wire+=header.len();
    let mut accepted_once=false;
    if ready {
        let claimed=u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
        if claimed<=B1_MAX_PAYLOAD {
            let mut payload=Vec::new();
            let complete;
            if case=="sigkill" {
                let (prefix,ok)=bc_read_bytes(&mut stream,4);
                wire+=prefix.len();
                assert!(ok && prefix.len()==4);
                child.kill().expect("kill real blocked sender child");
                payload.extend(prefix);
                let (tail,done)=bc_read_bytes(&mut stream,claimed-4);
                wire+=tail.len();
                payload.extend(tail);
                complete=done;
            } else {
                let (body,done)=bc_read_bytes(&mut stream,claimed);
                wire+=body.len();
                payload=body;
                complete=done;
            }
            if complete {
                let mut packet=header;
                packet.extend_from_slice(&payload);
                match b1_decode(&packet,1,B1_DELTA,B1_EPOCH,2) {
                    Ok(body) => {
                        let mut updated=receiver[0].clone();
                        if b1b_apply(&mut updated,body).is_ok() {
                            assert_eq!(case,"ackloss","only ACK-loss case can succeed");
                            assert_eq!(updated,bc_source(worker,lane,rep,1,2));
                            receiver[0]=updated;
                            accepted_once=true;
                            // ACK is intentionally not transmitted. Physical sender
                            // times out then replays identical sequence/payload.
                            let (header2,got)=bc_read_bytes(&mut stream,B1_HEADER);
                            assert!(got);
                            wire+=header2.len();
                            let n=u32::from_le_bytes(header2[28..32].try_into().unwrap()) as usize;
                            assert!(n<=B1_MAX_PAYLOAD);
                            let (body2,got)=bc_read_bytes(&mut stream,n);
                            assert!(got);
                            wire+=body2.len();
                            let mut replay=header2;
                            replay.extend(body2);
                            assert_eq!(replay,packet,"lost ACK must retransmit identical full frame");
                            assert_eq!(receiver[0],bc_source(worker,lane,rep,1,2));
                            stream.write_all(&bb_ack(true,2)).unwrap();
                            wire+=BB_ACK;
                        } else {
                            assert_eq!(case,"membership");
                            stream.write_all(&bb_ack(false,2)).unwrap();
                            wire+=BB_ACK;
                        }
                    }
                    Err(_) => {
                        assert_ne!(case,"ackloss");
                        stream.write_all(&bb_ack(false,2)).unwrap();
                        wire+=BB_ACK;
                    }
                }
            }
        }
    }
    let status=child.wait().unwrap();
    if case=="sigkill" {
        assert!(!status.success(),"SIGKILL was not executed");
    } else {
        assert!(status.success(),"fault sender did not complete expected protocol");
    }
    if !accepted_once {assert_eq!(*receiver,before,"failed frame partially mutated receiver");}
    (wire,accepted_once)
}
fn bc_negative_test() {
    let v=b1_encode(1,B1_DELTA,B1_EPOCH,2,&b1b_insert(8));
    assert_eq!(b1_decode(&v,1,B1_DELTA,B1_EPOCH,3),Err("generation"));
    let mut corrupt=v.clone();
    corrupt[48]^=1;
    assert_eq!(b1_decode(&corrupt,1,B1_DELTA,B1_EPOCH,2),Err("checksum"));
    let mut state=vec![2,4,6];
    let original=state.clone();
    let mut bad=Vec::new();
    bad.extend_from_slice(&b1b_insert(9));
    bad.extend_from_slice(&b1b_remove(11));
    assert_eq!(b1b_apply(&mut state,&bad),Err("delete-missing"));
    assert_eq!(state,original,"invalid tail of batch partially mutated receiver");
}
fn bc_worker(worker: usize) {
    assert!((1..=5).contains(&worker));
    bc_negative_test();
    let mut count=0;
    for (lane,&(n,_,d)) in BC_LANES.iter().enumerate(){
        for rep in 0..3 {
            for (fault_idx,case) in BC_CASES.iter().enumerate(){
                let (mut receiver,bootstrap,rss0,cpu0)=bc_full_pair(worker,lane,rep,1);
                let base=receiver.clone();
                let (fault_bytes,acked)=bc_fault(worker,lane,rep,case,&mut receiver);
                if !acked {assert_eq!(receiver,base);}
                else {
                    assert_eq!(receiver[0],bc_source(worker,lane,rep,1,2));
                    assert_eq!(receiver[1],base[1]);
                }
                // Conservative fail-closed policy: never trust the old
                // incremental cursor after disconnect or NACK.
                let (fresh,recovery,rss1,cpu1)=bc_full_pair(worker,lane,rep,2);
                assert_eq!(fresh[0],bc_source(worker,lane,rep,1,2));
                assert_eq!(fresh[1],bc_source(worker,lane,rep,2,2));
                assert_eq!(b0_diff(&fresh[0],&fresh[1]).len(),d);
                receiver=fresh;
                assert_eq!(receiver[0],bc_source(worker,lane,rep,1,2));
                assert_eq!(receiver[1],bc_source(worker,lane,rep,2,2));
                assert_eq!(bootstrap,242+8*(2*n+d%2));
                assert_eq!(recovery,bootstrap+16);
                println!("B1B1B1A_PROBE worker={worker} lane={lane} rep={rep} fault_idx={fault_idx} case={case} N={n} d={d} bootstrap_bytes={bootstrap} fault_bytes={fault_bytes} recovery_bytes={recovery} ackloss_once={} rss_healthy_max={} cpu_healthy_ticks={} exact=1",
                    u8::from(acked),
                    rss0.into_iter().chain(rss1).max().unwrap(),
                    cpu0.into_iter().chain(cpu1).sum::<u64>());
                count+=1;
            }
        }
    }
    assert_eq!(count,78);
    println!("DELTAGUARD_B1B1B1A_WORKER_PASS worker={worker} probes={count} resets={count} ackloss=6 sigkill=6");
}
