// Included by the research-only OPT-C example inside mod opt_c.
// Frozen preregistered C2 transfer/cost model; not a public wire format.
const TRANSFER_SEEDS: usize = 32;
const COMMON_BYTES: usize = 96 + 8 + 18;
const FULL_BYTES: usize = COMMON_BYTES + ROWS * J as usize / 8 + 4;
const EXTRA_REQUEST_BYTES: usize = 17;

fn frozen_level_bound(snapshot: &Frozen, table: &[u64], level: u8) -> u128 {
    let first = usize::from(level - 1) * WORDS_PER_LEVEL;
    let odd = snapshot.words[first..first + WORDS_PER_LEVEL]
        .iter().map(|x| x.count_ones()).sum::<u32>() as usize;
    if odd >= TABLE_ENTRIES || table[odd] == SENTINEL {
        DOMAIN_CARDINALITY
    } else {
        q32_level_upper(table[odd], u32::from(level))
    }
}

fn levels_for_order(name: &str, target: u64) -> Vec<u8> {
    let mut levels: Vec<u8> = (1..=J as u8).collect();
    match name {
        "asc" => {}
        "desc" => levels.reverse(),
        "center" => {
            // T-only order: no use of the hidden synthetic ground-truth d.
            let pivot = (((target as f64 / ROWS as f64).log2().round() as i32) + 1)
                .clamp(1, J as i32);
            levels.sort_by_key(|j| ((i32::from(*j) - pivot).abs(), *j));
        }
        _ => panic!("unknown order"),
    }
    levels
}

fn stop_prefix(snapshot: &Frozen, table: &[u64], order: &[u8], t: u64) -> usize {
    let mut bound = DOMAIN_CARDINALITY;
    for (index, &level) in order.iter().enumerate() {
        bound = bound.min(frozen_level_bound(snapshot, table, level));
        if bound <= u128::from(t) {
            return index + 1;
        }
    }
    order.len() // fail closed: no useful bound, send all available levels
}

fn serialize_session(session: &Session) -> Vec<u8> {
    let mut out = Vec::with_capacity(96);
    out.extend_from_slice(&session.magic);
    out.push(session.version);
    out.extend_from_slice(&session.rows.to_le_bytes());
    out.push(session.levels);
    out.extend_from_slice(&session.epoch);
    out.extend_from_slice(&session.config_binding);
    out.extend_from_slice(&session.table_binding);
    assert_eq!(out.len(), 92);
    let crc = crc32(&out);
    out.extend_from_slice(&crc.to_le_bytes());
    assert_eq!(out.len(), 96);
    out
}

fn validate_session(bytes: &[u8], session: &Session) {
    assert_eq!(bytes.len(), 96);
    assert_eq!(u32::from_le_bytes(bytes[92..96].try_into().unwrap()), crc32(&bytes[..92]));
    assert_eq!(bytes, serialize_session(session));
}

fn plan_bytes(mode: u8, received_mask: u64, zero_mask: u64, frames: usize) -> [u8; 18] {
    assert_eq!(zero_mask & !received_mask, 0);
    assert!(frames <= J as usize);
    let mut bytes = [0_u8; 18];
    bytes[0] = mode;
    bytes[1] = frames as u8;
    bytes[2..10].copy_from_slice(&received_mask.to_le_bytes());
    bytes[10..18].copy_from_slice(&zero_mask.to_le_bytes());
    bytes
}

fn validate_plan(plan: &[u8; 18], mode: u8, frames: usize) -> (u64, u64) {
    assert_eq!(plan[0], mode);
    assert_eq!(usize::from(plan[1]), frames);
    let received = u64::from_le_bytes(plan[2..10].try_into().unwrap());
    let zero = u64::from_le_bytes(plan[10..18].try_into().unwrap());
    assert_eq!(received >> J, 0);
    assert_eq!(zero & !received, 0);
    assert_eq!(received.count_ones() as usize - zero.count_ones() as usize, frames);
    (received, zero)
}

fn subset_mask(levels: &[u8]) -> u64 {
    levels.iter().fold(0_u64, |acc, &j| acc | (1_u64 << (j - 1)))
}

fn is_zero_level(snapshot: &Frozen, level: u8) -> bool {
    let a = usize::from(level - 1) * WORDS_PER_LEVEL;
    snapshot.words[a..a + WORDS_PER_LEVEL].iter().all(|x| *x == 0)
}

struct TransferResult {
    bytes: usize,
    frames: usize,
    requests: usize,
    useful: bool,
    bound: u128,
    selected: usize,
    prep_ns: u128,
    process_ns: u128,
}

fn physical_complete(snapshot: &Frozen, table: &[u64], threshold: u64) -> TransferResult {
    let start = Instant::now();
    let header = serialize_session(&snapshot.session);
    validate_session(&header, &snapshot.session);
    let request = threshold.to_le_bytes();
    black_box(request);
    let received_mask = (1_u64 << J) - 1;
    let plan = plan_bytes(0, received_mask, 0, 0);
    // Full mode's payload is not a sequence of level frames; validate mask separately.
    assert_eq!(plan[0], 0);
    assert_eq!(u64::from_le_bytes(plan[2..10].try_into().unwrap()), received_mask);
    let mut payload = Vec::with_capacity(ROWS * J as usize / 8 + 4);
    for word in snapshot.words.iter() {
        payload.extend_from_slice(&word.to_le_bytes());
    }
    let crc = crc32(&payload);
    payload.extend_from_slice(&crc.to_le_bytes());
    assert_eq!(payload.len(), ROWS * J as usize / 8 + 4);
    assert_eq!(u32::from_le_bytes(payload[payload.len()-4..].try_into().unwrap()),
               crc32(&payload[..payload.len()-4]));
    let reconstructed: Vec<u64> = payload[..payload.len()-4].as_chunks::<8>().0
        .iter().map(|chunk| u64::from_le_bytes(*chunk)).collect();
    assert_eq!(reconstructed.as_slice(), snapshot.words.as_ref());
    let mut counts = [0_u16; J as usize];
    for (level, count) in counts.iter_mut().enumerate() {
        let a = level * WORDS_PER_LEVEL;
        *count = snapshot.words[a..a+WORDS_PER_LEVEL]
            .iter().map(|w| w.count_ones()).sum::<u32>() as u16;
    }
    let bound = strict_upper_bound(&counts, table);
    TransferResult {
        bytes: header.len() + request.len() + plan.len() + payload.len(),
        frames: 0, requests: 1, useful: bound <= u128::from(threshold),
        bound, selected: J as usize, prep_ns: 0, process_ns: start.elapsed().as_nanos(),
    }
}

fn physical_levels(snapshot: &Frozen, table: &[u64], order: &[u8], t: u64,
                   mode: &str, batch: usize) -> TransferResult {
    let begin_prep = Instant::now();
    let limit = match mode {
        "pushall" => J as usize,
        "bounded" | "elide" => stop_prefix(snapshot, table, order, t),
        "interactive" => J as usize, // request feedback determines actual stop
        _ => panic!("invalid mode"),
    };
    let preparation = begin_prep.elapsed().as_nanos();
    let start = Instant::now();
    let header = serialize_session(&snapshot.session);
    validate_session(&header, &snapshot.session);
    let request = t.to_le_bytes();
    black_box(request);
    let mut receiver = Receiver::new(&snapshot.session, snapshot.session.config_binding,
                                     snapshot.session.table_binding).unwrap();
    let mut total_frames = 0_usize;
    let mut selected = 0_usize;
    let mut requests = if mode == "interactive" { 0 } else { 1 };
    let mut level_mask = 0_u64;
    let mut zero_mask = 0_u64;
    if mode == "interactive" {
        while selected < J as usize {
            let end = (selected + batch).min(J as usize);
            let requested = subset_mask(&order[selected..end]);
            let mut control = [0_u8; EXTRA_REQUEST_BYTES];
            control[0] = 1;
            control[1..9].copy_from_slice(&snapshot.session.binding_tag()[..8]);
            control[9..17].copy_from_slice(&requested.to_le_bytes());
            black_box(control);
            requests += 1;
            for &level in &order[selected..end] {
                let frame = snapshot.frame(level);
                assert!(receiver.ingest(&frame).unwrap());
                total_frames += 1;
                level_mask |= 1_u64 << (level - 1);
            }
            selected = end;
            if receiver.bound(table) <= u128::from(t) { break; }
        }
    } else {
        for &level in &order[..limit] {
            level_mask |= 1_u64 << (level - 1);
            if mode == "elide" && is_zero_level(snapshot, level) {
                // Explicit sender-declared zero for a frozen state.
                zero_mask |= 1_u64 << (level - 1);
                let i = level as usize - 1;
                assert!(receiver.parts[i].is_none());
                receiver.parts[i] = Some([0_u8; LEVEL_BYTES]);
            } else {
                let frame = snapshot.frame(level);
                assert!(receiver.ingest(&frame).unwrap());
                total_frames += 1;
            }
        }
        selected = limit;
    }
    assert_eq!(receiver.received(), selected);
    let code = match mode { "pushall" => 1, "bounded" => 2,
                            "elide" => 3, "interactive" => 4, _ => unreachable!() };
    let plan = if mode == "interactive" {
        // Unknown prefix before feedback: actual requests carry level IDs.
        plan_bytes(code, 0, 0, 0)
    } else {
        plan_bytes(code, level_mask, zero_mask, total_frames)
    };
    if mode == "interactive" {
        assert_eq!(validate_plan(&plan, code, 0), (0,0));
    } else {
        assert_eq!(validate_plan(&plan, code, total_frames), (level_mask,zero_mask));
    }
    for (level, part) in receiver.parts.iter().enumerate() {
        if level_mask & (1_u64 << level) != 0 {
            let received = part.as_ref().unwrap();
            let a = level * WORDS_PER_LEVEL;
            for (out, input) in received.as_chunks::<8>().0.iter()
                .zip(snapshot.words[a..a + WORDS_PER_LEVEL].iter()) {
                assert_eq!(*out, input.to_le_bytes());
            }
        } else { assert!(part.is_none()); }
    }
    let bound = receiver.bound(table);
    let bytes = header.len() + request.len() + plan.len()
        + total_frames * FRAME_BYTES
        + if mode == "interactive" { requests * EXTRA_REQUEST_BYTES } else { 0 };
    TransferResult {
        bytes, frames: total_frames, requests, bound,
        useful: bound <= u128::from(t), selected,
        prep_ns: preparation, process_ns: start.elapsed().as_nanos(),
    }
}

fn c2_sample(worker: usize, scenario: usize, seed: usize, order: &str,
             mode: &str, batch: usize, d: u64, t: u64, frozen: &Frozen,
             table: &[u64], copy_ns: u128, result: TransferResult, full_bound: u128) {
    assert_eq!(frozen.words.len(), J as usize * WORDS_PER_LEVEL);
    assert_eq!(table.len(), TABLE_ENTRIES);
    assert_eq!(result.useful, result.bound <= u128::from(t));
    assert!(result.bound >= full_bound, "partial bound tighter than full minimum");
    assert!(result.bytes >= COMMON_BYTES);
    if mode == "complete" { assert_eq!(result.bytes, FULL_BYTES); }
    println!(
      "C2_SAMPLE worker={} scenario={} seed={} order={} mode={} batch={} d={} T={} bytes={} frames={} requests={} selected={} useful={} bound={} copy_ns={} prep_ns={} process_ns={}",
      worker,scenario,seed,order,mode,batch,d,t,result.bytes,result.frames,result.requests,
      result.selected,u8::from(result.useful),result.bound,copy_ns,result.prep_ns,result.process_ns
    );
}

fn run_transfer(path: &Path, worker: usize) {
    assert!((1..=5).contains(&worker));
    let table = load_table(path);
    let table_binding = *blake3::hash(&std::fs::read(path).unwrap()).as_bytes();
    assert_oracle_vectors();
    let workloads = [(4096_u64,8192_u64),(65536,131072),(1048576,2097152)];
    for (scenario, (d,t)) in workloads.into_iter().enumerate() {
        for seed in 0..TRANSFER_SEEDS {
            let mut live = PackedSketch::new(J, Layout::LevelMajor);
            let begin = (worker as u64 * 1000 + seed as u64 + scenario as u64 * 100) * 2_000_000;
            for token in begin..begin+d { live.toggle(token); }
            let freeze = Instant::now();
            let epoch = ((worker as u128) << 64 | ((scenario as u128) << 32) | seed as u128).to_le_bytes();
            let frozen = Frozen::from_sketch(&live, epoch, [0x39;32], table_binding);
            let copy_ns = freeze.elapsed().as_nanos();
            let full = physical_complete(&frozen,&table,t);
            assert_eq!(full.bound,live.estimate(&table));
            let full_bound=full.bound;
            c2_sample(worker,scenario,seed,"none","complete",0,d,t,&frozen,&table,copy_ns,full,full_bound);
            for order_name in ["asc","desc","center"] {
                let order = levels_for_order(order_name,t);
                assert_eq!(order.len(),J as usize);
                for (mode,batch) in [("pushall",0),("bounded",0),("elide",0),
                                     ("interactive",1),("interactive",2),
                                     ("interactive",4),("interactive",8)] {
                    let measured = physical_levels(&frozen,&table,&order,t,mode,batch);
                    if mode=="pushall" { assert_eq!(measured.bytes, COMMON_BYTES+J as usize*FRAME_BYTES); }
                    c2_sample(worker,scenario,seed,order_name,mode,batch,d,t,
                              &frozen,&table,copy_ns,measured,full_bound);
                }
            }
            black_box(frozen);
        }
    }
    println!("STRICT_COMPACT_OPT_C_C2_WORKER_PASS worker={}",worker);
}
