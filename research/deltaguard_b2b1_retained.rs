// Research-only B1-B0: real socket transfers, physical receiver-retained exact.
// No authenticated wire, real WAN, paced RTT/p95, or public API claims.
const B1B_NS: [usize; 2] = [256, 65536];
const B1B_DS: [usize; 4] = [16, 48, 57, 65];
const B1B_CHECKPOINTS: [usize; 3] = [1, 10, 100];
const B1B_REPEATS: usize = 3;
const B1B_REQUEST: usize = 24;

fn b1b_inputs(worker: usize, ni: usize, di: usize, repeat: usize) -> (Vec<u64>, Vec<u64>, u64) {
    let n = B1B_NS[ni];
    let d = B1B_DS[di];
    let base = worker as u64 * 10_000_000_000
        + ni as u64 * 2_000_000_000
        + di as u64 * 300_000_000
        + repeat as u64 * 20_000_000;
    let a: Vec<u64> = (0..n as u64).map(|x| base + x).collect();
    let mut b: Vec<u64> = (0..(n - d / 2) as u64).map(|x| base + x).collect();
    b.extend((0..d.div_ceil(2) as u64).map(|x| base + n as u64 + x));
    assert_eq!(b0_diff(&a, &b).len(), d);
    (a, b, base + 1_000_000)
}

fn b1b_list(frame: &[u8], owner: u8, seq: u64) -> Vec<u64> {
    let payload = b1_decode(frame, owner, B1_FULL, B1_EPOCH, seq).unwrap();
    payload
        .as_chunks::<8>()
        .0
        .iter()
        .map(|word| u64::from_le_bytes(*word))
        .collect()
}

fn b1b_insert(token: u64) -> [u8; 9] {
    let mut bytes = [0; 9];
    bytes[0] = 1;
    bytes[1..].copy_from_slice(&token.to_le_bytes());
    bytes
}

fn b1b_remove(token: u64) -> [u8; 9] {
    let mut bytes = b1b_insert(token);
    bytes[0] = 2;
    bytes
}

fn b1b_apply(target: &mut Vec<u64>, event: &[u8]) -> Result<(), &'static str> {
    b1_canonical_delta(event)?;
    // Phase 1: validate ALL operations before any mutation.
    // Canonical batches contain each token at most once, so membership against
    // the original target remains sufficient for the entire transaction.
    for record in event.as_chunks::<9>().0 {
        let token = u64::from_le_bytes(record[1..].try_into().unwrap());
        match (record[0], target.binary_search(&token)) {
            (1, Err(_)) | (2, Ok(_)) => (),
            (1, Ok(_)) => return Err("repeated-insert"),
            (2, Err(_)) => return Err("delete-missing"),
            _ => return Err("delta-op"),
        }
    }
    // Phase 2 cannot fail after complete prevalidation, even for batched
    // ordered inserts/deletes. No partially committed malformed frame.
    for record in event.as_chunks::<9>().0 {
        let token = u64::from_le_bytes(record[1..].try_into().unwrap());
        match record[0] {
            1 => {
                let pos = target.binary_search(&token).unwrap_err();
                target.insert(pos, token);
            }
            2 => {
                let pos = target.binary_search(&token).unwrap();
                target.remove(pos);
            }
            _ => unreachable!("prevalidated operation"),
        }
    }
    Ok(())
}

fn b1b_guard_transfer(a: &NearFullGuard, b: &NearFullGuard, generation: u64)
    -> (usize, u32)
{
    let (left, right, bytes) = b1_actual_pair(
        b1_encode(1, B1_GUARD, B1_EPOCH, generation, &b1_words_bytes(&a.words)),
        b1_encode(2, B1_GUARD, B1_EPOCH, generation, &b1_words_bytes(&b.words)),
        B1_GUARD, generation,
    ).expect("two independent guard TCP senders");
    let pa = b1_decode(&left, 1, B1_GUARD, B1_EPOCH, generation).unwrap();
    let pb = b1_decode(&right, 2, B1_GUARD, B1_EPOCH, generation).unwrap();
    let odd: u32 = pa.as_chunks::<8>().0.iter()
        .zip(pb.as_chunks::<8>().0.iter())
        .map(|(a, b)| (u64::from_le_bytes(*a) ^ u64::from_le_bytes(*b)).count_ones())
        .sum();
    assert_eq!(bytes, 640);
    (bytes, odd)
}

fn b1b_full_transfer(a: &[u64], b: &[u64], generation: u64)
    -> (Vec<u64>, Vec<u64>, usize)
{
    let (left, right, bytes) = b1_actual_pair(
        b1_encode(1, B1_FULL, B1_EPOCH, generation, &b1_full_bytes(a)),
        b1_encode(2, B1_FULL, B1_EPOCH, generation, &b1_full_bytes(b)),
        B1_FULL, generation,
    ).expect("two physical canonical list senders");
    (b1b_list(&left, 1, generation), b1b_list(&right, 2, generation), bytes)
}

fn b1b_fallback(a: &[u64], b: &[u64], generation: u64)
    -> (Vec<u64>, Vec<u64>, usize)
{
    // Distinct sockets, one per owner, including a physically written request.
    // 1B owner hello + 24B receiver request + 64B full response frame each.
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let mut threads = Vec::new();
    for (owner, source) in [(1_u8, a), (2_u8, b)] {
        let source = source.to_vec();
        threads.push(std::thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_nodelay(true).unwrap();
            stream.write_all(&[owner]).unwrap();
            let mut req = [0_u8; B1B_REQUEST];
            stream.read_exact(&mut req).unwrap();
            assert_eq!(&req[..8], b"DGFBRQ01");
            assert_eq!(u64::from_le_bytes(req[8..16].try_into().unwrap()), B1_EPOCH);
            assert_eq!(u64::from_le_bytes(req[16..24].try_into().unwrap()), generation);
            let packet = b1_encode(owner, B1_FULL, B1_EPOCH, generation, &b1_full_bytes(&source));
            stream.write_all(&packet).unwrap();
        }));
    }
    let mut results: [Option<Vec<u64>>; 2] = [None, None];
    let mut count = 0_usize;
    for _ in 0..2 {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut owner = [0_u8; 1];
        stream.read_exact(&mut owner).unwrap();
        assert!((1..=2).contains(&owner[0]));
        assert!(results[(owner[0]-1) as usize].is_none());
        let mut req = [0; B1B_REQUEST];
        req[..8].copy_from_slice(b"DGFBRQ01");
        req[8..16].copy_from_slice(&B1_EPOCH.to_le_bytes());
        req[16..24].copy_from_slice(&generation.to_le_bytes());
        stream.write_all(&req).unwrap();
        let mut header = [0; B1_HEADER];
        stream.read_exact(&mut header).unwrap();
        let n = u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
        assert!(n <= B1_MAX_PAYLOAD);
        let mut packet = header.to_vec();
        packet.resize(B1_HEADER+n, 0);
        stream.read_exact(&mut packet[B1_HEADER..]).unwrap();
        results[(owner[0]-1) as usize] = Some(b1b_list(&packet, owner[0], generation));
        count += 1+B1B_REQUEST+packet.len();
    }
    for thread in threads { thread.join().unwrap(); }
    (results[0].take().unwrap(), results[1].take().unwrap(), count)
}

fn b1b_reference_diff(a: &[u64], b: &[u64], key: &[u8; 32]) -> Vec<u64> {
    let mut words = vec![0_u64; 32];
    for token in b0_diff(a,b) {
        let digest = blake3::keyed_hash(key, &token.to_le_bytes());
        let slot = (u64::from_le_bytes(digest.as_bytes()[..8].try_into().unwrap()) & 2047) as usize;
        if slot < 2047 { words[slot/64] ^= 1_u64 << (slot % 64); }
    }
    words
}

fn b1b_negative_tests() {
    let mut source=vec![1,3,5];
    assert!(b1b_apply(&mut source, &b1b_insert(7)).is_ok());
    assert_eq!(source,vec![1,3,5,7]);
    assert_eq!(b1b_apply(&mut source, &b1b_insert(7)),Err("repeated-insert"));
    assert_eq!(b1b_apply(&mut source, &b1b_remove(9)),Err("delete-missing"));
    assert!(b1b_apply(&mut source, &b1b_remove(3)).is_ok());
    assert_eq!(source,vec![1,5,7]);
    let before = source.clone();
    let mut invalid_batch = Vec::new();
    invalid_batch.extend_from_slice(&b1b_insert(8));
    invalid_batch.extend_from_slice(&b1b_remove(9));
    assert_eq!(b1b_apply(&mut source, &invalid_batch), Err("delete-missing"));
    assert_eq!(source, before, "partial malformed batch mutated source");
    // Difference preservation and cancellation under legitimate asymmetric churn.
    let mut a=vec![1,3,5];
    let mut b=vec![1,5,7];
    assert_eq!(b0_diff(&a,&b),vec![3,7]);
    b1b_apply(&mut a,&b1b_insert(7)).unwrap();
    assert_eq!(b0_diff(&a,&b),vec![3]);
    b1b_apply(&mut b,&b1b_insert(3)).unwrap();
    assert!(b0_diff(&a,&b).is_empty());
    b1b_apply(&mut b,&b1b_remove(7)).unwrap();
    assert_eq!(b0_diff(&a,&b),vec![7]);
    b1b_apply(&mut a,&b1b_remove(3)).unwrap();
    assert_eq!(b0_diff(&a,&b),vec![3,7]);
}

fn b1b_worker(worker:usize) {
    use std::time::Instant;
    assert!((1..=5).contains(&worker));
    b1b_negative_tests();
    let mut samples=0_usize;
    for (ni,&n) in B1B_NS.iter().enumerate() {
        for (di,&d) in B1B_DS.iter().enumerate() {
            for rep in 0..B1B_REPEATS {
                let (mut a, mut b, next) = b1b_inputs(worker,ni,di,rep);
                let t0=Instant::now();
                let mut ga=NearFullGuard::new(11,64,&MASTER_KEY);
                let mut gb=NearFullGuard::new(11,64,&MASTER_KEY);
                for &t in &a { ga.toggle(t); }
                for &t in &b { gb.toggle(t); }
                let build_ns=t0.elapsed().as_nanos();
                let t0=Instant::now();
                let (mut recv_a, mut recv_b, bootstrap_bytes) = b1b_full_transfer(&a,&b,1);
                let bootstrap_ns=t0.elapsed().as_nanos();
                assert_eq!(recv_a,a);
                assert_eq!(recv_b,b);
                let mut exact_bytes=bootstrap_bytes;
                let mut batched_a=recv_a.clone();
                let mut batched_b=recv_b.clone();
                let mut pending_a=Vec::<u8>::new();
                let mut pending_b=Vec::<u8>::new();
                let mut batched_exact_bytes=bootstrap_bytes;
                let mut updates_ns=0_u128;
                let mut dense_bytes=0_usize;
                let mut sparse_bytes=0_usize;
                let mut index=0_usize;
                for generation in 1_usize..=100 {
                    if generation > 1 {
                        let token=next+generation as u64;
                        a.push(token);
                        b.push(token);
                        ga.toggle(token);
                        gb.toggle(token);
                        pending_a.extend_from_slice(&b1b_insert(token));
                        pending_b.extend_from_slice(&b1b_insert(token));
                        let t0=Instant::now();
                        let (pa,pb,wire)=b1_actual_pair(
                            b1_encode(1,B1_DELTA,B1_EPOCH,generation as u64,&b1b_insert(token)),
                            b1_encode(2,B1_DELTA,B1_EPOCH,generation as u64,&b1b_insert(token)),
                            B1_DELTA,generation as u64,
                        ).expect("retained physical per-owner exact event pair");
                        let ea=b1_decode(&pa,1,B1_DELTA,B1_EPOCH,generation as u64).unwrap();
                        let eb=b1_decode(&pb,2,B1_DELTA,B1_EPOCH,generation as u64).unwrap();
                        b1b_apply(&mut recv_a,ea).unwrap();
                        b1b_apply(&mut recv_b,eb).unwrap();
                        updates_ns+=t0.elapsed().as_nanos();
                        exact_bytes+=wire;
                        assert_eq!(wire,146);
                        assert_eq!(recv_a,a,"owner A receiver exact drift");
                        assert_eq!(recv_b,b,"owner B receiver exact drift");
                    }
                    let (gb_wire, odd)=b1b_guard_transfer(&ga,&gb,generation as u64);
                    dense_bytes+=gb_wire;
                    if B1B_CHECKPOINTS.contains(&generation) {
                        index+=1;
                        sparse_bytes+=gb_wire;
                        if generation>1 {
                            let (ba,bb,bytes)=b1_actual_pair(
                                b1_encode(1,B1_DELTA,B1_EPOCH,generation as u64,&pending_a),
                                b1_encode(2,B1_DELTA,B1_EPOCH,generation as u64,&pending_b),
                                B1_DELTA,generation as u64,
                            ).expect("physical canonical batched sparse updates");
                            b1b_apply(&mut batched_a,b1_decode(&ba,1,B1_DELTA,B1_EPOCH,generation as u64).unwrap()).unwrap();
                            b1b_apply(&mut batched_b,b1_decode(&bb,2,B1_DELTA,B1_EPOCH,generation as u64).unwrap()).unwrap();
                            batched_exact_bytes+=bytes;
                            pending_a.clear();
                            pending_b.clear();
                        }
                        assert_eq!(batched_a,a,"batched owner A divergence");
                        assert_eq!(batched_b,b,"batched owner B divergence");
                        assert_eq!(batched_exact_bytes,bootstrap_bytes+128*(index-1)+18*(generation-1));
                        let source_diff=b0_diff(&a,&b);
                        assert_eq!(source_diff.len(),d);
                        let independent=b1b_reference_diff(&a,&b,&ga.key);
                        let xor_words:Vec<u64>=ga.words.iter().zip(gb.words.iter())
                            .map(|(x,y)|x^y).collect();
                        assert_eq!(xor_words,independent,"true oracle XOR mismatch");
                        assert_eq!(odd, independent.iter().map(|w|w.count_ones()).sum::<u32>());
                        let safe=odd<=48;
                        if d<=48 {assert!(safe,"B2A pointwise S<=d guarantee");}
                        let (da,db,direct_bytes)=b1b_full_transfer(&a,&b,generation as u64);
                        assert_eq!(da,a);
                        assert_eq!(db,b);
                        assert_eq!(b0_diff(&da,&db).len(),d);
                        let fallback_bytes=if safe {0} else {
                            let (fa,fb,bytes)=b1b_fallback(&a,&b,generation as u64);
                            assert_eq!(fa,a);
                            assert_eq!(fb,b);
                            assert_eq!(b0_diff(&fa,&fb).len(),d);
                            assert_eq!(bytes,direct_bytes+50);
                            bytes
                        };
                        assert_eq!(exact_bytes,bootstrap_bytes+146*(generation-1));
                        assert_eq!(dense_bytes,640*generation);
                        assert_eq!(sparse_bytes,640*index);
                        assert_eq!(direct_bytes,128+8*(a.len()+b.len()));
                        if fallback_bytes!=0 {assert!(640+fallback_bytes>direct_bytes);}
                        println!("B1B0_SAMPLE worker={worker} ni={ni} di={di} rep={rep} N={n} d={d} S={generation} odd={odd} safe={} source_a={} source_b={} build_ns={build_ns} bootstrap_ns={bootstrap_ns} delta_ns={updates_ns} bootstrap_bytes={bootstrap_bytes} retained_exact_bytes={exact_bytes} retained_batched_bytes={batched_exact_bytes} guard_sparse_bytes={sparse_bytes} guard_dense_bytes={dense_bytes} direct_bytes={direct_bytes} fallback_bytes={fallback_bytes} resolved_bytes={} generations={} queries_sparse={index}",
                            u8::from(safe),a.len(),b.len(),640+fallback_bytes,generation-1);
                        samples+=1;
                    }
                }
                assert_eq!(index,3);
            }
        }
    }
    assert_eq!(samples,72);
    println!("DELTAGUARD_B1B0_WORKER_PASS worker={worker} samples={samples}");
}
