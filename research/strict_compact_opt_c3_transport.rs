// C3-A: real TCP loopback IO + application-layer rate and RTT pacing.
// No self-hosted runner, no public networking API, no snapshot mutation.
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const C3_SEEDS: usize = 8;
const C3_MBIT: u64 = 10;
const C3_RTTS: [u64; 3] = [0, 10, 50];
const C3_MODES: [&str; 4] = ["scalar", "complete", "bounded", "interactive4"];

fn c3_send(stream: &mut TcpStream, bytes: &[u8], count: &mut usize) {
    // Sleep BEFORE sending to simulate a 10Mbit/s application-layer link.
    // Not kernel shaping or TCP/IP packet-wire accounting.
    let micros = (bytes.len() as u64 * 8 + C3_MBIT - 1) / C3_MBIT;
    if micros > 0 {
        thread::sleep(Duration::from_micros(micros));
    }
    stream.write_all(bytes).expect("TCP write_all failed");
    *count += bytes.len();
}

fn c3_read(stream: &mut TcpStream, bytes: &mut [u8], count: &mut usize) {
    stream.read_exact(bytes).expect("short/failed TCP frame");
    *count += bytes.len();
}

fn c3_scan_bound(sketch: &Frozen, table: &[u64]) -> u128 {
    let mut counts = [0_u16; J as usize];
    for (j, count) in counts.iter_mut().enumerate() {
        let index = j * WORDS_PER_LEVEL;
        let ones = sketch.words[index..index + WORDS_PER_LEVEL]
            .iter().map(|w| w.count_ones()).sum::<u32>();
        *count = u16::try_from(ones).unwrap();
    }
    strict_upper_bound(&counts, table)
}

struct C3Wire {
    send: usize,
    read: usize,
    bound: u128,
    requests: usize,
}

fn c3_server(
    listener: TcpListener, snapshot: Frozen, table: Arc<[u64]>,
    mode: &'static str, threshold: u64, rtt_ms: u64,
) -> C3Wire {
    let (mut socket, _) = listener.accept().expect("TCP accept");
    socket.set_nodelay(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut received = 0;
    let mut sent = 0;
    let mut requested_t = [0_u8; 8];
    c3_read(&mut socket, &mut requested_t, &mut received);
    assert_eq!(u64::from_le_bytes(requested_t), threshold);
    let bound = c3_scan_bound(&snapshot, &table);
    // Application-layer emulation of request -> initial response RTT.
    if rtt_ms > 0 { thread::sleep(Duration::from_millis(rtt_ms)); }
    if mode == "scalar" {
        c3_send(&mut socket, &bound.to_le_bytes(), &mut sent);
        return C3Wire { send:sent, read:received, bound, requests:1 };
    }
    let header = serialize_session(&snapshot.session);
    c3_send(&mut socket, &header, &mut sent);
    let order = levels_for_order("center", threshold);
    let (code, plan) = match mode {
        "complete" => (0_u8, plan_bytes(0, (1_u64 << J) - 1, 0, 0)),
        "bounded" => {
            let take = stop_prefix(&snapshot, &table, &order, threshold);
            let mask = subset_mask(&order[..take]);
            (2, plan_bytes(2, mask, 0, take))
        }
        "interactive4" => (4, plan_bytes(4, 0, 0, 0)),
        _ => unreachable!(),
    };
    assert_eq!(plan[0], code);
    c3_send(&mut socket, &plan, &mut sent);
    match mode {
        "complete" => {
            let mut bytes = Vec::with_capacity(ROWS * J as usize / 8 + 4);
            for word in snapshot.words.iter() {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            let checksum = crc32(&bytes);
            bytes.extend_from_slice(&checksum.to_le_bytes());
            c3_send(&mut socket, &bytes, &mut sent);
        }
        "bounded" => {
            let count = usize::from(plan[1]);
            for &level in &order[..count] {
                c3_send(&mut socket, &snapshot.frame(level), &mut sent);
            }
        }
        "interactive4" => {
            loop {
                let mut request = [0_u8; EXTRA_REQUEST_BYTES];
                match socket.read_exact(&mut request) {
                    Ok(()) => received += EXTRA_REQUEST_BYTES,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => panic!("bad request: {e}"),
                }
                assert_eq!(request[0], 1);
                assert_eq!(&request[1..9], &snapshot.session.binding_tag()[..8]);
                let mask = u64::from_le_bytes(request[9..17].try_into().unwrap());
                assert!(mask != 0 && (mask >> J) == 0 && mask.count_ones() <= 4);
                if rtt_ms > 0 { thread::sleep(Duration::from_millis(rtt_ms)); }
                for &level in &order {
                    if mask & (1_u64 << (level - 1)) != 0 {
                        c3_send(&mut socket, &snapshot.frame(level), &mut sent);
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    C3Wire {
        send:sent, read:received, bound,
        requests: if mode == "interactive4" {
            1 + (received - 8) / EXTRA_REQUEST_BYTES
        } else { 1 },
    }
}

struct C3Observation {
    bytes: usize,
    requests: usize,
    bound: u128,
    useful: bool,
    elapsed_ns: u128,
    copy_ns: u128,
}

fn c3_tcp_session(
    state: &Frozen, table: &Arc<[u64]>, mode: &'static str,
    t: u64, rtt_ms: u64,
) -> C3Observation {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let start = Instant::now();
    let freeze = Instant::now();
    let snapshot = state.clone();
    let copy_ns = freeze.elapsed().as_nanos();
    let owned_table = Arc::clone(table);
    let server = thread::spawn(move || c3_server(listener, snapshot, owned_table, mode, t, rtt_ms));
    let mut client = TcpStream::connect(address).expect("TCP connect");
    client.set_nodelay(true).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    client.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut client_send = 0;
    let mut client_read = 0;
    c3_send(&mut client, &t.to_le_bytes(), &mut client_send);
    let mut requests = 1;
    let mut bound = DOMAIN_CARDINALITY;
    if mode == "scalar" {
        let mut result = [0_u8; 16];
        c3_read(&mut client, &mut result, &mut client_read);
        bound = u128::from_le_bytes(result);
    } else {
        let mut header = [0_u8; 96];
        c3_read(&mut client, &mut header, &mut client_read);
        validate_session(&header, &state.session);
        let mut plan = [0_u8; 18];
        c3_read(&mut client, &mut plan, &mut client_read);
        let mut rx = Receiver::new(&state.session, state.session.config_binding,
                                   state.session.table_binding).unwrap();
        if mode == "complete" {
            assert_eq!(plan[0], 0);
            let mut payload = vec![0_u8; ROWS * J as usize / 8 + 4];
            c3_read(&mut client, &mut payload, &mut client_read);
            let length = payload.len();
            assert_eq!(crc32(&payload[..length-4]),
                       u32::from_le_bytes(payload[length-4..].try_into().unwrap()));
            let decoded: Vec<u64> = payload[..length-4].as_chunks::<8>().0
                .iter().map(|bytes| u64::from_le_bytes(*bytes)).collect();
            assert_eq!(decoded.as_slice(), state.words.as_ref());
            bound = c3_scan_bound(state, table);
        } else if mode == "bounded" {
            let count = usize::from(plan[1]);
            let order = levels_for_order("center", t);
            assert_eq!(validate_plan(&plan, 2, count).0, subset_mask(&order[..count]));
            for _ in 0..count {
                let mut frame = [0_u8; FRAME_BYTES];
                c3_read(&mut client, &mut frame, &mut client_read);
                assert!(rx.ingest(&frame).unwrap());
            }
            bound = rx.bound(table);
        } else if mode == "interactive4" {
            assert_eq!(validate_plan(&plan, 4, 0), (0,0));
            let order = levels_for_order("center", t);
            for chunk in order.chunks(4) {
                let mut request = [0_u8; EXTRA_REQUEST_BYTES];
                request[0] = 1;
                request[1..9].copy_from_slice(&state.session.binding_tag()[..8]);
                request[9..17].copy_from_slice(&subset_mask(chunk).to_le_bytes());
                c3_send(&mut client, &request, &mut client_send);
                requests += 1;
                for _ in chunk {
                    let mut frame = [0_u8; FRAME_BYTES];
                    c3_read(&mut client, &mut frame, &mut client_read);
                    assert!(rx.ingest(&frame).unwrap());
                }
                bound = rx.bound(table);
                if bound <= u128::from(t) { break; }
            }
            client.shutdown(Shutdown::Write).unwrap();
        } else { unreachable!() }
        if mode != "complete" {
            for (i, part) in rx.parts.iter().enumerate() {
                if let Some(bytes) = part {
                    let source = &state.words[i*WORDS_PER_LEVEL..(i+1)*WORDS_PER_LEVEL];
                    for (chunk, word) in bytes.as_chunks::<8>().0.iter().zip(source) {
                        assert_eq!(*chunk, word.to_le_bytes());
                    }
                }
            }
        }
    }
    let wire = server.join().expect("server panic");
    assert_eq!(wire.read, client_send, "request bytes mismatch");
    assert_eq!(wire.send, client_read, "response bytes mismatch");
    assert_eq!(requests, wire.requests);
    assert!(bound >= wire.bound, "received prefix cannot beat full bound");
    let bytes = client_send + client_read;
    let expected = match mode {
        "scalar" => 24,
        "complete" => FULL_BYTES,
        "bounded" => 122 + 533 * ((bytes - 122) / 533),
        _ => bytes,
    };
    assert_eq!(bytes, expected);
    C3Observation {
        bytes, requests, bound, useful: bound <= u128::from(t),
        elapsed_ns: start.elapsed().as_nanos(), copy_ns,
    }
}

fn c3_reusable_two_source_oracle(table: &[u64], worker: usize, scenario: usize, seed: usize, d: u64, t: u64) {
    let mut left = PackedSketch::new(J, Layout::LevelMajor);
    let mut right = PackedSketch::new(J, Layout::LevelMajor);
    let begin = (worker as u64 * 1000 + scenario as u64 * 100 + seed as u64) * 2_000_000;
    let half = d / 2;
    for token in begin..begin + half { left.toggle(token); }
    for token in begin + half..begin + d { right.toggle(token); }
    let epoch_left = [0x11;16];
    let epoch_right = [0x22;16];
    let table_id = [0x63;32];
    let l = Frozen::from_sketch(&left, epoch_left, [0x39;32], table_id);
    let r = Frozen::from_sketch(&right, epoch_right, [0x39;32], table_id);
    let mut full = left;
    full.xor_assign(&right);
    let expected = full.estimate(table);
    let order = levels_for_order("center", t);
    let mut pl = Receiver::new(&l.session,[0x39;32],table_id).unwrap();
    let mut pr = Receiver::new(&r.session,[0x39;32],table_id).unwrap();
    let mut partial = DOMAIN_CARDINALITY;
    for (round, &j) in order.iter().enumerate() {
        assert!(pl.ingest(&l.frame(j)).unwrap());
        assert!(pr.ingest(&r.frame(j)).unwrap());
        let index = j as usize - 1;
        let la = pl.parts[index].as_ref().unwrap();
        let rb = pr.parts[index].as_ref().unwrap();
        let odd = la.iter().zip(rb).map(|(a,b)| (a^b).count_ones()).sum::<u32>() as usize;
        let level_upper = if odd >= TABLE_ENTRIES || table[odd] == SENTINEL {
            DOMAIN_CARDINALITY
        } else { q32_level_upper(table[odd],j as u32) };
        partial = partial.min(level_upper);
        assert!(partial >= expected);
        if round == 0 || round == 3 || round == 51 {
            println!("C3_XOR_REUSE worker={} scenario={} seed={} levels={} bytes={} bound={} useful={} full_bound={}",
                     worker,scenario,seed,round+1,2*(96+8+18+(round+1)*FRAME_BYTES),partial,
                     u8::from(partial <= u128::from(t)),expected);
        }
    }
    assert_eq!(partial,expected);
    assert_eq!(pl.complete_words().unwrap(),l.words);
    assert_eq!(pr.complete_words().unwrap(),r.words);
}

fn run_c3a(path: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    assert_oracle_vectors();
    let table: Arc<[u64]> = Arc::from(load_table(path));
    let table_id = *blake3::hash(&std::fs::read(path).unwrap()).as_bytes();
    // TCP preflight/warmup is intentionally not counted in observed samples.
    let sanity = PackedSketch::new(J,Layout::LevelMajor);
    let warm = Frozen::from_sketch(&sanity,[0x66;16],[0x39;32],table_id);
    let _ = c3_tcp_session(&warm,&table,"scalar",4096,0);
    for (scenario,(d,t)) in [(4096_u64,8192_u64),(65536,131072),(1048576,2097152)]
        .into_iter().enumerate() {
        for seed in 0..C3_SEEDS {
            let mut sketch = PackedSketch::new(J,Layout::LevelMajor);
            let begin = (worker as u64 * 1000 + scenario as u64 * 100 + seed as u64) * 2_000_000;
            for token in begin..begin+d { sketch.toggle(token); }
            let epoch = ((worker as u128)<<64 | ((scenario as u128)<<32) | seed as u128).to_le_bytes();
            let frozen = Frozen::from_sketch(&sketch,epoch,[0x39;32],table_id);
            c3_reusable_two_source_oracle(&table,worker,scenario,seed,d,t);
            // Rotate protocol order per seed to reduce systematic CI drift.
            let mut modes = C3_MODES;
            modes.rotate_left((seed+worker-1)%modes.len());
            for rtt in C3_RTTS {
                for mode in modes {
                    let result = c3_tcp_session(&frozen,&table,mode,t,rtt);
                    println!("C3_TCP_SAMPLE worker={} scenario={} seed={} d={} T={} mode={} mbps=10 rtt_ms={} bytes={} requests={} bound={} useful={} elapsed_ns={} copy_ns={}",
                        worker,scenario,seed,d,t,mode,rtt,result.bytes,result.requests,
                        result.bound,u8::from(result.useful),result.elapsed_ns,result.copy_ns);
                }
            }
        }
    }
    println!("STRICT_COMPACT_OPT_C3A_WORKER_PASS worker={worker}");
}
