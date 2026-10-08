// C3-C research-only T-informed eager first parity level (one RTT).
// Historical request-first C3-B remains unmodified and independently frozen.
fn c3c_server(listener: TcpListener, frozen: Frozen, t: u64, rtt: u64) -> C3Wire {
    let (mut socket, _) = listener.accept().expect("push-first accept");
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut sent=0_usize;
    let mut received=0_usize;
    let mut threshold=[0_u8;8];
    c3_read(&mut socket,&mut threshold,&mut received);
    assert_eq!(u64::from_le_bytes(threshold),t);
    if rtt != 0 { thread::sleep(Duration::from_millis(rtt)); }
    let first=levels_for_order("center",t)[0];
    let first_mask=1_u64 << (first-1);
    c3_send(&mut socket,&serialize_session(&frozen.session),&mut sent);
    c3_send(&mut socket,&plan_bytes(6,first_mask,0,1),&mut sent);
    c3_send(&mut socket,&frozen.frame(first),&mut sent);
    let mut past=first_mask;
    let mut requests=1_usize;
    loop {
        let mut req=[0_u8;EXTRA_REQUEST_BYTES];
        match socket.read_exact(&mut req) {
            Ok(())=>received+=EXTRA_REQUEST_BYTES,
            Err(e) if e.kind()==std::io::ErrorKind::UnexpectedEof=>break,
            Err(e)=>panic!("bad sender-first request: {e}"),
        }
        assert_eq!(req[0],1);
        assert_eq!(&req[1..9],&frozen.session.binding_tag()[..8]);
        let mask=u64::from_le_bytes(req[9..17].try_into().unwrap());
        assert!(mask!=0 && mask>>J==0 && mask & past==0);
        past|=mask;
        requests+=1;
        if rtt!=0 {thread::sleep(Duration::from_millis(rtt));}
        for &j in &levels_for_order("center",t) {
            if mask & (1_u64 << (j-1))!=0 {
                c3_send(&mut socket,&frozen.frame(j),&mut sent);
            }
        }
    }
    C3Wire {send:sent,read:received,bound:DOMAIN_CARDINALITY,requests}
}

fn c3c_checkpoint(meta:&C3BMeta,mode:&str,levels:usize,bytes:usize,
                  requests:usize,upper:u128,elapsed_ns:u128) {
    println!(
      "C3C_SAMPLE worker={} scenario={} seed={} d={} T={} mode={} levels={} mbps=10 rtt_ms={} bytes={} requests={} bound={} useful={} elapsed_ns={}",
      meta.worker,meta.scenario,meta.seed,meta.d,meta.t,mode,levels,
      meta.rtt,bytes,requests,upper,u8::from(upper<=u128::from(meta.t)),elapsed_ns
    );
}

fn c3c_pair_session(left:&Frozen,right:&Frozen,table:&[u64],meta:&C3BMeta,full:bool) {
    assert_eq!(left.session.config_binding,right.session.config_binding);
    assert_eq!(left.session.table_binding,right.session.table_binding);
    assert_ne!(left.session.epoch,right.session.epoch);
    let expected=c3b_pair_expected(left,right,table);
    let listener_l=TcpListener::bind("127.0.0.1:0").unwrap();
    let listener_r=TcpListener::bind("127.0.0.1:0").unwrap();
    let addr_l=listener_l.local_addr().unwrap();
    let addr_r=listener_r.local_addr().unwrap();
    let timer=Instant::now();
    let copy_l=left.clone();
    let copy_r=right.clone();
    let t=meta.t;
    let rtt=meta.rtt;
    let th_l=thread::spawn(move||{
        if full {c3b_server(listener_l,copy_l,t,rtt,true)}
        else {c3c_server(listener_l,copy_l,t,rtt)}
    });
    let th_r=thread::spawn(move||{
        if full {c3b_server(listener_r,copy_r,t,rtt,true)}
        else {c3c_server(listener_r,copy_r,t,rtt)}
    });
    let mut a=TcpStream::connect(addr_l).unwrap();
    let mut b=TcpStream::connect(addr_r).unwrap();
    for s in [&mut a,&mut b] {
        s.set_nodelay(true).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        s.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    }
    let (mut sent_a,mut sent_b,mut recv_a,mut recv_b)=(0_usize,0_usize,0_usize,0_usize);
    c3_send(&mut a,&t.to_le_bytes(),&mut sent_a);
    c3_send(&mut b,&t.to_le_bytes(),&mut sent_b);
    let first=levels_for_order("center",t)[0];
    let mut rxa=Receiver::new(&left.session,left.session.config_binding,left.session.table_binding).unwrap();
    let mut rxb=Receiver::new(&right.session,right.session.config_binding,right.session.table_binding).unwrap();
    for (sock,recv,state,rx) in [(&mut a,&mut recv_a,left,&mut rxa),(&mut b,&mut recv_b,right,&mut rxb)] {
        let mut header=[0_u8;96];
        c3_read(sock,&mut header,recv);
        validate_session(&header,&state.session);
        let mut plan=[0_u8;18];
        c3_read(sock,&mut plan,recv);
        if full {
            assert_eq!(plan[0],0);
        } else {
            assert_eq!(validate_plan(&plan,6,1),(1_u64<<(first-1),0));
            c3b_recv_frames(sock,recv,rx,&[first]);
        }
    }
    if full {
        let l=c3b_recv_full(&mut a,&mut recv_a,left);
        let r=c3b_recv_full(&mut b,&mut recv_b,right);
        for (i,(x,y)) in l.iter().zip(&r).enumerate() {
            assert_eq!(x^y,left.words[i]^right.words[i]);
        }
        let bytes=sent_a+recv_a+sent_b+recv_b;
        assert_eq!(bytes,53500);
        c3c_checkpoint(meta,"dual_full",52,bytes,2,expected,timer.elapsed().as_nanos());
    } else {
        let order=levels_for_order("center",t);
        let mut bound=c3b_merged_bound(&rxa,&rxb,table);
        assert!(bound>=expected);
        c3c_checkpoint(meta,"push_first",1,sent_a+recv_a+sent_b+recv_b,
                       2,bound,timer.elapsed().as_nanos());
        for (end,begin) in [(4_usize,1_usize),(J as usize,4)] {
            let more=&order[begin..end];
            c3b_request(&mut a,&mut sent_a,left,more);
            c3b_request(&mut b,&mut sent_b,right,more);
            c3b_recv_frames(&mut a,&mut recv_a,&mut rxa,more);
            c3b_recv_frames(&mut b,&mut recv_b,&mut rxb,more);
            let next=c3b_merged_bound(&rxa,&rxb,table);
            assert!(next>=expected && next<=bound);
            bound=next;
            let bytes=sent_a+recv_a+sent_b+recv_b;
            let expected_bytes=match end {4=>4542,_=>55744};
            assert_eq!(bytes,expected_bytes);
            c3c_checkpoint(meta,"push_first",end,bytes,if end==4 {4}else{6},
                           bound,timer.elapsed().as_nanos());
        }
        assert_eq!(bound,expected);
        assert_eq!(rxa.complete_words().unwrap(),left.words);
        assert_eq!(rxb.complete_words().unwrap(),right.words);
        let mut obsolete=left.clone();
        obsolete.session.epoch[0]^=1;
        assert_eq!(rxa.ingest(&obsolete.frame(first)),Err("wrong-session"));
    }
    a.shutdown(Shutdown::Write).unwrap();
    b.shutdown(Shutdown::Write).unwrap();
    let sla=th_l.join().unwrap();
    let slb=th_r.join().unwrap();
    assert_eq!(sla.read,sent_a);
    assert_eq!(slb.read,sent_b);
    assert_eq!(sla.send,recv_a);
    assert_eq!(slb.send,recv_b);
}

fn run_c3c(path:&Path,worker:usize) {
    assert!((1..=5).contains(&worker));
    assert_oracle_vectors();
    let table=load_table(path);
    let table_id=*blake3::hash(&std::fs::read(path).unwrap()).as_bytes();
    for (scenario,(d,t)) in [(4096_u64,8192_u64),(65536,131072),(1048576,2097152)]
        .into_iter().enumerate() {
        for seed in 0..C3_SEEDS {
            let begin=(worker as u64*1000+scenario as u64*100+seed as u64)*2_000_000;
            let mut a=PackedSketch::new(J,Layout::LevelMajor);
            let mut b=PackedSketch::new(J,Layout::LevelMajor);
            for token in begin..begin+d/2 {a.toggle(token);}
            for token in begin+d/2..begin+d {b.toggle(token);}
            let l=Frozen::from_sketch(&a,[0x11;16],[0x39;32],table_id);
            let r=Frozen::from_sketch(&b,[0x22;16],[0x39;32],table_id);
            for rtt in C3_RTTS {
                let meta=C3BMeta{worker,scenario,seed,d,t,rtt};
                if (seed+worker).is_multiple_of(2) {
                    c3c_pair_session(&l,&r,&table,&meta,true);
                    c3c_pair_session(&l,&r,&table,&meta,false);
                } else {
                    c3c_pair_session(&l,&r,&table,&meta,false);
                    c3c_pair_session(&l,&r,&table,&meta,true);
                }
            }
        }
    }
    println!("STRICT_COMPACT_OPT_C3C_WORKER_PASS worker={worker}");
}
