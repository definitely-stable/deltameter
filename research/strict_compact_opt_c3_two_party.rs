// C3-B research-only two independent TCP senders, actual multi-round retained XOR.
fn c3b_server(
    listener: TcpListener, frozen: Frozen, threshold: u64, rtt: u64, full: bool,
) -> C3Wire {
    let (mut socket, _) = listener.accept().expect("dual TCP accept");
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut send = 0_usize;
    let mut read = 0_usize;
    let mut tbytes = [0_u8; 8];
    c3_read(&mut socket, &mut tbytes, &mut read);
    assert_eq!(u64::from_le_bytes(tbytes), threshold);
    if rtt != 0 { thread::sleep(Duration::from_millis(rtt)); }
    c3_send(&mut socket, &serialize_session(&frozen.session), &mut send);
    if full {
        let mask = (1_u64 << J) - 1;
        c3_send(&mut socket, &plan_bytes(0, mask, 0, 0), &mut send);
        let mut payload = Vec::with_capacity(ROWS * J as usize / 8 + 4);
        for w in frozen.words.iter() { payload.extend_from_slice(&w.to_le_bytes()); }
        let crc = crc32(&payload);
        payload.extend_from_slice(&crc.to_le_bytes());
        c3_send(&mut socket, &payload, &mut send);
        return C3Wire {send,read,bound:DOMAIN_CARDINALITY,requests:1};
    }
    c3_send(&mut socket, &plan_bytes(5, 0, 0, 0), &mut send);
    let mut previously_sent = 0_u64;
    let mut requests = 1_usize;
    loop {
        let mut req = [0_u8; EXTRA_REQUEST_BYTES];
        match socket.read_exact(&mut req) {
            Ok(()) => read += EXTRA_REQUEST_BYTES,
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => panic!("bad two-party request {err}"),
        }
        assert_eq!(req[0], 1);
        assert_eq!(&req[1..9], &frozen.session.binding_tag()[..8]);
        let mask = u64::from_le_bytes(req[9..17].try_into().unwrap());
        assert!(mask != 0 && mask >> J == 0 && mask & previously_sent == 0);
        previously_sent |= mask;
        requests += 1;
        if rtt != 0 { thread::sleep(Duration::from_millis(rtt)); }
        // The receiver predeclared T-centered order determines request framing.
        for &level in &levels_for_order("center", threshold) {
            if mask & (1_u64 << (level - 1)) != 0 {
                c3_send(&mut socket, &frozen.frame(level), &mut send);
            }
        }
    }
    C3Wire {send,read,bound:DOMAIN_CARDINALITY,requests}
}

fn c3b_merged_bound(left: &Receiver, right: &Receiver, table: &[u64]) -> u128 {
    assert_eq!(left.parts.len(),right.parts.len());
    let mut upper = DOMAIN_CARDINALITY;
    for (j,(a,b)) in left.parts.iter().zip(&right.parts).enumerate() {
        match (a,b) {
            (None,None) => {},
            (Some(lhs),Some(rhs)) => {
                let odd: usize = lhs.iter().zip(rhs).map(|(x,y)| (x^y).count_ones() as usize).sum();
                if odd < TABLE_ENTRIES && table[odd] != SENTINEL {
                    upper = upper.min(q32_level_upper(table[odd],(j+1) as u32));
                }
            }
            _ => panic!("unpaired received level"),
        }
    }
    upper
}

fn c3b_pair_expected(l: &Frozen, r: &Frozen, table: &[u64]) -> u128 {
    let mut counts = [0_u16; J as usize];
    for (j,count) in counts.iter_mut().enumerate() {
        let start = j * WORDS_PER_LEVEL;
        let odd: u32 = l.words[start..start+WORDS_PER_LEVEL]
            .iter().zip(&r.words[start..start+WORDS_PER_LEVEL])
            .map(|(a,b)| (a^b).count_ones()).sum();
        *count = u16::try_from(odd).unwrap();
    }
    strict_upper_bound(&counts,table)
}

struct C3BMeta {
    worker:usize, scenario:usize, seed:usize, d:u64, t:u64, rtt:u64,
}

fn c3b_checkpoint(meta:&C3BMeta, mode:&str, levels:usize, bytes:usize,
                  requests:usize, upper:u128, elapsed_ns:u128, expected:u128) {
    assert!(upper >= expected);
    let useful=u8::from(upper <= u128::from(meta.t));
    println!(
        "C3B_SAMPLE worker={} scenario={} seed={} d={} T={} mode={} levels={} mbps=10 rtt_ms={} bytes={} requests={} bound={} useful={} elapsed_ns={}",
        meta.worker,meta.scenario,meta.seed,meta.d,meta.t,mode,levels,meta.rtt,
        bytes,requests,upper,useful,elapsed_ns
    );
}

fn c3b_recv_full(stream: &mut TcpStream, count: &mut usize, frozen: &Frozen) -> Vec<u64> {
    let mut payload = vec![0_u8; ROWS * J as usize / 8 + 4];
    c3_read(stream,&mut payload,count);
    let end = payload.len();
    let stored=u32::from_le_bytes(payload[end-4..].try_into().unwrap());
    assert_eq!(stored,crc32(&payload[..end-4]));
    let decoded: Vec<u64> = payload[..end-4].as_chunks::<8>().0
        .iter().map(|word|u64::from_le_bytes(*word)).collect();
    assert_eq!(decoded.as_slice(),frozen.words.as_ref());
    decoded
}

fn c3b_recv_frames(
    stream: &mut TcpStream, count: &mut usize, receiver: &mut Receiver,
    levels: &[u8],
) {
    for &level in levels {
        let mut frame=[0_u8; FRAME_BYTES];
        c3_read(stream,&mut frame,count);
        assert_eq!(frame[EPOCH_BYTES],level);
        assert!(receiver.ingest(&frame).unwrap());
    }
}

fn c3b_request(stream: &mut TcpStream, count: &mut usize,
               frozen: &Frozen, levels: &[u8]) {
    let mut request=[0_u8; EXTRA_REQUEST_BYTES];
    request[0]=1;
    request[1..9].copy_from_slice(&frozen.session.binding_tag()[..8]);
    request[9..17].copy_from_slice(&subset_mask(levels).to_le_bytes());
    c3_send(stream,&request,count);
}

fn c3b_pair_session(
    left: &Frozen, right: &Frozen, table:&[u64], meta:&C3BMeta,
    full:bool,
) {
    assert_eq!(left.session.config_binding,right.session.config_binding);
    assert_eq!(left.session.table_binding,right.session.table_binding);
    assert_ne!(left.session.epoch,right.session.epoch);
    let oracle=c3b_pair_expected(left,right,table);
    let listener_l=TcpListener::bind("127.0.0.1:0").unwrap();
    let listener_r=TcpListener::bind("127.0.0.1:0").unwrap();
    let addr_l=listener_l.local_addr().unwrap();
    let addr_r=listener_r.local_addr().unwrap();
    let start=Instant::now();
    let sender_l=left.clone();
    let sender_r=right.clone();
    let t=meta.t;
    let rtt=meta.rtt;
    let thread_l=thread::spawn(move||c3b_server(listener_l,sender_l,t,rtt,full));
    let thread_r=thread::spawn(move||c3b_server(listener_r,sender_r,t,rtt,full));
    let mut a=TcpStream::connect(addr_l).unwrap();
    let mut b=TcpStream::connect(addr_r).unwrap();
    for sock in [&mut a,&mut b] {
        sock.set_nodelay(true).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        sock.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    }
    let mut sent_a=0_usize;
    let mut sent_b=0_usize;
    let mut recv_a=0_usize;
    let mut recv_b=0_usize;
    c3_send(&mut a,&t.to_le_bytes(),&mut sent_a);
    c3_send(&mut b,&t.to_le_bytes(),&mut sent_b);
    for (sock,recv,frozen) in [(&mut a,&mut recv_a,left),(&mut b,&mut recv_b,right)] {
        let mut hdr=[0_u8;96];
        c3_read(sock,&mut hdr,recv);
        validate_session(&hdr,&frozen.session);
        let mut plan=[0_u8;18];
        c3_read(sock,&mut plan,recv);
        if full { assert_eq!(plan[0],0); }
        else { assert_eq!(validate_plan(&plan,5,0),(0,0)); }
    }
    if full {
        let decoded_l=c3b_recv_full(&mut a,&mut recv_a,left);
        let decoded_r=c3b_recv_full(&mut b,&mut recv_b,right);
        let paired:Vec<u64>=decoded_l.iter().zip(&decoded_r).map(|(x,y)|x^y).collect();
        for (i,value) in paired.iter().enumerate() {
            assert_eq!(*value,left.words[i]^right.words[i]);
        }
        c3b_checkpoint(meta,"dual_full",52,sent_a+recv_a+sent_b+recv_b,
                       2,oracle,start.elapsed().as_nanos(),oracle);
    } else {
        let mut rx_l=Receiver::new(&left.session,left.session.config_binding,
                                   left.session.table_binding).unwrap();
        let mut rx_r=Receiver::new(&right.session,right.session.config_binding,
                                   right.session.table_binding).unwrap();
        let order=levels_for_order("center",t);
        let mut last=DOMAIN_CARDINALITY;
        for (end,begin) in [(1_usize,0_usize),(4,1),(J as usize,4)] {
            let levels=&order[begin..end];
            c3b_request(&mut a,&mut sent_a,left,levels);
            c3b_request(&mut b,&mut sent_b,right,levels);
            c3b_recv_frames(&mut a,&mut recv_a,&mut rx_l,levels);
            c3b_recv_frames(&mut b,&mut recv_b,&mut rx_r,levels);
            let upper=c3b_merged_bound(&rx_l,&rx_r,table);
            assert!(upper<=last && upper>=oracle);
            last=upper;
            let total=sent_a+recv_a+sent_b+recv_b;
            let expected=2*(122+match end {1=>17,4=>34,_=>51}+end*FRAME_BYTES);
            assert_eq!(total,expected);
            c3b_checkpoint(meta,"dual_reuse",end,total,2*(1+match end {1=>1,4=>2,_=>3}),
                           upper,start.elapsed().as_nanos(),oracle);
        }
        assert_eq!(last,oracle);
        assert_eq!(rx_l.complete_words().unwrap(),left.words);
        assert_eq!(rx_r.complete_words().unwrap(),right.words);
        // New generations must never be silently spliced into retained states.
        let mut changed=left.clone();
        changed.session.epoch[0]^=1;
        assert_eq!(rx_l.ingest(&changed.frame(1)),Err("wrong-session"));
    }
    a.shutdown(Shutdown::Write).unwrap();
    b.shutdown(Shutdown::Write).unwrap();
    let sa=thread_l.join().unwrap();
    let sb=thread_r.join().unwrap();
    assert_eq!(sa.read,sent_a);
    assert_eq!(sb.read,sent_b);
    assert_eq!(sa.send,recv_a);
    assert_eq!(sb.send,recv_b);
}

fn run_c3b(path: &Path, worker:usize) {
    assert!((1..=5).contains(&worker));
    let table=load_table(path);
    let tid=*blake3::hash(&std::fs::read(path).unwrap()).as_bytes();
    assert_oracle_vectors();
    for (scenario,(d,t)) in [(4096_u64,8192_u64),(65536,131072),(1048576,2097152)]
        .into_iter().enumerate() {
        for seed in 0..C3_SEEDS {
            let begin=(worker as u64*1000+scenario as u64*100+seed as u64)*2_000_000;
            let mid=d/2;
            let mut l=PackedSketch::new(J,Layout::LevelMajor);
            let mut r=PackedSketch::new(J,Layout::LevelMajor);
            for token in begin..begin+mid {l.toggle(token);}
            for token in begin+mid..begin+d {r.toggle(token);}
            let state_l=Frozen::from_sketch(&l,[0x11;16],[0x39;32],tid);
            let state_r=Frozen::from_sketch(&r,[0x22;16],[0x39;32],tid);
            for rtt in C3_RTTS {
                let meta=C3BMeta{worker,scenario,seed,d,t,rtt};
                if (worker+seed)%2==0 {
                    c3b_pair_session(&state_l,&state_r,&table,&meta,true);
                    c3b_pair_session(&state_l,&state_r,&table,&meta,false);
                } else {
                    c3b_pair_session(&state_l,&state_r,&table,&meta,false);
                    c3b_pair_session(&state_l,&state_r,&table,&meta,true);
                }
            }
        }
    }
    println!("STRICT_COMPACT_OPT_C3B_WORKER_PASS worker={worker}");
}
