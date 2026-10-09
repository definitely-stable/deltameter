// DeltaGuard B1-B1B1-B1B0 research-only: one atomic two-owner WAL txn,
// bounded 50-generation checkpoint and exact durable anti-ABA receipt.
// This is localhost process/crash research, NOT a production protocol.
const BF_TX: usize = 256;
const BF_TX_MAGIC: &[u8; 8] = b"DGRTX101";
const BF_CP_MAGIC: &[u8; 8] = b"DGRCP101";
const BF_MARK_MAGIC: &[u8; 8] = b"DGRMK101";
const BF_CHECKPOINT_INTERVAL: usize = 50;

struct BFState {
    receipt: BEReceipt,
    tx_head: [u8; 32],
    checkpoint_gen: usize,
}
struct BFCost {
    data_bytes: usize,
    checkpoint_bytes: usize,
    sync_calls: usize,
}
fn bf_base_tx_head(receipt: &BEReceipt) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"deltameter:B1B1B1-B1B0:txn-genesis:v1");
    h.update(&B1_EPOCH.to_le_bytes());
    h.update(&b1_key_id());
    h.update(&[11]);
    h.update(&receipt.heads[0]);
    h.update(&receipt.heads[1]);
    *h.finalize().as_bytes()
}
fn bf_tx(seq: usize, records: &[[u8; BE_REC]; 2], prev: &[u8; 32]) -> [u8; BF_TX] {
    let mut out = [0_u8; BF_TX];
    out[..8].copy_from_slice(BF_TX_MAGIC);
    out[8..16].copy_from_slice(&(seq as u64).to_le_bytes());
    out[16..104].copy_from_slice(&records[0]);
    out[104..192].copy_from_slice(&records[1]);
    out[192..224].copy_from_slice(prev);
    let mut h = blake3::Hasher::new();
    h.update(b"deltameter:B1B1B1-B1B0:txn:v1");
    h.update(&B1_EPOCH.to_le_bytes());
    h.update(&b1_key_id());
    h.update(&[11]);
    h.update(&out[..224]);
    out[224..].copy_from_slice(h.finalize().as_bytes());
    out
}
fn bf_check_tx(bytes: &[u8], seq: usize, head: &[u8; 32])
    -> Result<[[u8; BE_REC]; 2], &'static str>
{
    if bytes.len() != BF_TX || &bytes[..8] != BF_TX_MAGIC {
        return Err("receiver-txn-magic-length");
    }
    if u64::from_le_bytes(bytes[8..16].try_into().unwrap()) != seq as u64 ||
        bytes[192..224] != head[..] {
        return Err("receiver-txn-generation-parent");
    }
    let a: [u8; BE_REC] = bytes[16..104].try_into().unwrap();
    let b: [u8; BE_REC] = bytes[104..192].try_into().unwrap();
    if bf_tx(seq, &[a, b], head) != bytes {
        return Err("receiver-txn-hash");
    }
    Ok([a, b])
}
fn bf_tx_head(bytes: &[u8]) -> [u8; 32] {
    bytes[224..256].try_into().unwrap()
}
fn bf_mark(seq: usize, digest: &[u8; 32]) -> [u8; 48] {
    let mut v = [0_u8; 48];
    v[..8].copy_from_slice(BF_MARK_MAGIC);
    v[8..16].copy_from_slice(&(seq as u64).to_le_bytes());
    v[16..48].copy_from_slice(digest);
    v
}
fn bf_load_marker(root: &Path) -> Result<(usize, [u8; 32]), &'static str> {
    let m = std::fs::read(root.join("receiver.commit"))
        .map_err(|_| "receiver-commit-missing")?;
    if m.len() != 48 || &m[..8] != BF_MARK_MAGIC {
        return Err("receiver-commit-magic");
    }
    let seq = u64::from_le_bytes(m[8..16].try_into().unwrap()) as usize;
    if !(1..=BE_GENS + 1).contains(&seq) {
        return Err("receiver-commit-range");
    }
    Ok((seq, m[16..48].try_into().unwrap()))
}
fn bf_encode_checkpoint(state: &BFState) -> Vec<u8> {
    let receipt = &state.receipt;
    assert_eq!(receipt.seq[0], receipt.seq[1]);
    let gen = receipt.seq[0];
    let a = b1_encode(1, B1_FULL, B1_EPOCH, gen as u64,
        &b1_full_bytes(&receipt.lists[0]));
    let b = b1_encode(2, B1_FULL, B1_EPOCH, gen as u64,
        &b1_full_bytes(&receipt.lists[1]));
    let mut v = Vec::with_capacity(336 + a.len() + b.len());
    v.extend_from_slice(BF_CP_MAGIC);
    v.extend_from_slice(&B1_EPOCH.to_le_bytes());
    v.extend_from_slice(&(gen as u64).to_le_bytes());
    v.extend_from_slice(&state.tx_head);
    for h in &receipt.heads { v.extend_from_slice(h); }
    for e in &receipt.last { v.extend_from_slice(e); }
    v.extend_from_slice(&(a.len() as u32).to_le_bytes());
    v.extend_from_slice(&(b.len() as u32).to_le_bytes());
    v.extend_from_slice(&a);
    v.extend_from_slice(&b);
    let digest = be_digest(&v);
    v.extend_from_slice(&digest);
    v
}
fn bf_read_checkpoint(root: &Path) -> Result<BFState, &'static str> {
    let v = std::fs::read(root.join("receiver.ckpt"))
        .map_err(|_| "checkpoint-missing")?;
    if v.len() < 336 || &v[..8] != BF_CP_MAGIC ||
        u64::from_le_bytes(v[8..16].try_into().unwrap()) != B1_EPOCH {
        return Err("checkpoint-identity");
    }
    let seq = u64::from_le_bytes(v[16..24].try_into().unwrap()) as usize;
    if !(1..=BE_GENS + 1).contains(&seq) { return Err("checkpoint-generation"); }
    let tx_head = v[24..56].try_into().unwrap();
    let heads = [v[56..88].try_into().unwrap(), v[88..120].try_into().unwrap()];
    let last = [v[120..208].try_into().unwrap(), v[208..296].try_into().unwrap()];
    let n1 = u32::from_le_bytes(v[296..300].try_into().unwrap()) as usize;
    let n2 = u32::from_le_bytes(v[300..304].try_into().unwrap()) as usize;
    if n1 > B1_HEADER + B1_MAX_PAYLOAD ||
        n2 > B1_HEADER + B1_MAX_PAYLOAD ||
        v.len() != 304 + n1 + n2 + 32 {
        return Err("checkpoint-bounded-length");
    }
    if v[304 + n1 + n2..] != be_digest(&v[..304 + n1 + n2]) {
        return Err("checkpoint-checksum");
    }
    let a = b11_as_words(b1_decode(&v[304..304 + n1], 1, B1_FULL, B1_EPOCH, seq as u64)?);
    let b = b11_as_words(b1_decode(&v[304 + n1..304 + n1 + n2],
        2, B1_FULL, B1_EPOCH, seq as u64)?);
    if seq == 1 {
        if last != [[0_u8; BE_REC]; 2] { return Err("checkpoint-genesis-event"); }
    } else {
        for i in 0..2 {
            if &last[i][..8] != BE_MAGIC ||
                u64::from_le_bytes(last[i][16..24].try_into().unwrap()) != seq as u64 ||
                last[i][24] != (i + 1) as u8 ||
                last[i][56..88] != heads[i] {
                return Err("checkpoint-last-identity");
            }
        }
    }
    Ok(BFState {
        receipt: BEReceipt {lists: [a, b], seq: [seq, seq], heads, last},
        tx_head, checkpoint_gen: seq,
    })
}
fn bf_event_frame(owner: u8, seq: usize, record: &[u8; BE_REC]) -> Vec<u8> {
    let token = u64::from_le_bytes(record[32..40].try_into().unwrap());
    let ev = match record[25] {
        1 => b1b_insert(token),
        2 => b1b_remove(token),
        _ => panic!("noncanonical source event operation"),
    };
    b1_encode(owner, B1_DELTA, B1_EPOCH, seq as u64, &ev)
}
fn bf_replay_tx(state: &mut BFState, bytes: &[u8], seq: usize)
    -> Result<(), &'static str>
{
    let records = bf_check_tx(bytes, seq, &state.tx_head)?;
    let frames = [bf_event_frame(1, seq, &records[0]),
        bf_event_frame(2, seq, &records[1])];
    let (next, fresh) = be_validate_transition(&state.receipt, &records, &frames, seq)?;
    if !fresh { return Err("committed-txn-not-new"); }
    state.receipt = next;
    state.tx_head = bf_tx_head(bytes);
    Ok(())
}
fn bf_load(root: &Path) -> Result<BFState, &'static str> {
    let mut state = bf_read_checkpoint(root)?;
    let (commit_seq, commit_head) = bf_load_marker(root)?;
    if commit_seq < state.checkpoint_gen { return Err("checkpoint-ahead-of-commit"); }
    if commit_seq == state.checkpoint_gen && commit_head != state.tx_head {
        return Err("checkpoint-commit-hash");
    }
    let wal = std::fs::read(root.join("receiver.wal")).map_err(|_| "receiver-wal-missing")?;
    // Old complete transactions may remain after checkpoint publication but
    // before atomic WAL replacement; confirm the stale prefix closes at
    // exactly this durable checkpoint, then ignore it.
    let mut last_old = 0_usize;
    let mut last_old_head = [0_u8; 32];
    for entry in wal.chunks_exact(BF_TX) {
        let seq = u64::from_le_bytes(entry[8..16].try_into().unwrap()) as usize;
        if seq <= state.checkpoint_gen {
            if &entry[..8] != BF_TX_MAGIC { return Err("stale-wal-magic"); }
            if last_old != 0 && seq != last_old + 1 { return Err("stale-wal-gap"); }
            last_old = seq;
            last_old_head = bf_tx_head(entry);
            continue;
        }
        if seq > commit_seq { break; } // uncommitted complete tail is ignored.
        if seq != state.receipt.seq[0] + 1 { return Err("committed-wal-gap"); }
        bf_replay_tx(&mut state, entry, seq)?;
    }
    if last_old > 0 &&
        (last_old != state.checkpoint_gen || last_old_head != bf_read_checkpoint(root)?.tx_head) {
        return Err("stale-wal-not-checkpoint-ancestor");
    }
    if state.receipt.seq != [commit_seq, commit_seq] ||
        state.tx_head != commit_head {
        return Err("receiver-committed-prefix-incomplete-or-different");
    }
    Ok(state)
}
fn bf_init(root: &Path) -> usize {
    let receipt = be_load_receipt(root).unwrap();
    assert_eq!(receipt.seq, [1, 1]);
    let state = BFState {tx_head: bf_base_tx_head(&receipt), receipt, checkpoint_gen: 1};
    let ckpt = bf_encode_checkpoint(&state);
    bd_atomic(root, "receiver.ckpt", &ckpt);
    bd_atomic(root, "receiver.commit", &bf_mark(1, &state.tx_head));
    bd_atomic(root, "receiver.wal", &[]);
    std::fs::remove_file(root.join("receiver.chain")).unwrap();
    std::fs::File::open(root).unwrap().sync_all().unwrap();
    assert_eq!(bf_load(root).unwrap().receipt.seq, [1, 1]);
    ckpt.len()
}
fn bf_commit(root: &Path, prev: &BFState, next: &BEReceipt,
    records: &[[u8; BE_REC]; 2], seq: usize) -> BFCost
{
    use std::io::Write;
    assert_eq!(prev.receipt.seq, [seq - 1, seq - 1]);
    assert_eq!(next.seq, [seq, seq]);
    let txn = bf_tx(seq, records, &prev.tx_head);
    let mut file = std::fs::OpenOptions::new().append(true)
        .open(root.join("receiver.wal")).unwrap();
    file.write_all(&txn).unwrap();
    file.sync_all().unwrap();
    drop(file);
    bd_atomic(root, "receiver.commit", &bf_mark(seq, &bf_tx_head(&txn)));
    let mut cost = BFCost {data_bytes: BF_TX + 48, checkpoint_bytes: 0, sync_calls: 3};
    if (seq - 1).is_multiple_of(BF_CHECKPOINT_INTERVAL) {
        let cp = BFState {receipt: BEReceipt {
            lists: next.lists.clone(), seq: next.seq, heads: next.heads,
            last: next.last,
        }, tx_head: bf_tx_head(&txn), checkpoint_gen: seq};
        let snapshot = bf_encode_checkpoint(&cp);
        bd_atomic(root, "receiver.ckpt", &snapshot);
        // Crash after checkpoint rename but before WAL replacement is
        // safe because bf_load can skip the verified old committed prefix.
        assert_eq!(bf_load(root).unwrap().receipt.seq, [seq, seq]);
        bd_atomic(root, "receiver.wal", &[]);
        cost.checkpoint_bytes = snapshot.len();
        cost.data_bytes += snapshot.len();
        cost.sync_calls += 4; // snapshot + empty-WAL rename/parent fsync.
    }
    cost
}
fn bf_accept(root: &Path, listener: &std::net::TcpListener,
    seq: usize, stop_after_ack1: bool) -> (usize, BFCost)
{
    use std::io::Write;
    let (mut sockets, events, frames, mut wire) = be_read_pair(listener);
    let before = bf_load(root).expect("must reconstruct canonical receiver from disk WAL");
    let (next, fresh) = be_validate_transition(&before.receipt, &events, &frames, seq)
        .expect("fully bound source event / receipt hash transition");
    let cost = if fresh {
        bf_commit(root, &before, &next, &events, seq)
    } else {
        BFCost {data_bytes: 0, checkpoint_bytes: 0, sync_calls: 0}
    };
    let after = bf_load(root).expect("durable prefix reloaded BEFORE ACK");
    assert_eq!(after.receipt.seq, [seq, seq]);
    assert_eq!(after.receipt.lists, next.lists);
    assert_eq!(after.receipt.heads, next.heads);
    assert_eq!(after.receipt.last, next.last);
    sockets[0].write_all(&bb_ack(true, seq)).unwrap();
    wire += BB_ACK;
    if stop_after_ack1 {
        println!("BF_ACK1_READY generation={seq}");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(15));
        panic!("controller must kill receiver after physical first ACK");
    }
    sockets[1].write_all(&bb_ack(true, seq)).unwrap();
    wire += BB_ACK;
    (wire, cost)
}
fn bf_normal(root: &Path, seq: usize) -> (usize, BFCost) {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let mut sources = be_child_send(seq, root, &addr);
    let result = bf_accept(root, &listener, seq, false);
    for child in &mut sources {assert!(child.wait().unwrap().success());}
    result
}
fn bf_child(args: &[String]) {
    use std::io::Write;
    assert_eq!(args.len(), 3);
    let seq: usize = args[1].parse().unwrap();
    let root = Path::new(&args[2]);
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    println!("BF_PORT {}", listener.local_addr().unwrap());
    std::io::stdout().flush().unwrap();
    let (bytes, cost) = bf_accept(root, &listener, seq, false);
    println!("BF_RESULT bytes={bytes} written={} fsync={} checkpoint={}",
        cost.data_bytes, cost.sync_calls, cost.checkpoint_bytes);
    std::io::stdout().flush().unwrap();
}
fn bf_crash_child(args: &[String]) {
    use std::io::Write;
    assert_eq!(args.len(), 3);
    let seq: usize = args[1].parse().unwrap();
    let root = Path::new(&args[2]);
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    println!("BF_PORT {}", listener.local_addr().unwrap());
    std::io::stdout().flush().unwrap();
    bf_accept(root, &listener, seq, true);
}
fn bf_spawn_receiver(root: &Path, seq: usize, crash: bool)
    -> (std::process::Child, std::io::BufReader<std::process::ChildStdout>, String)
{
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    let role = if crash {"--bf-crash"} else {"--bf-child"};
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([role, &seq.to_string(), root.to_str().unwrap()])
        .stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut greeting = String::new();
    out.read_line(&mut greeting).unwrap();
    let addr = greeting.trim().strip_prefix("BF_PORT ")
        .expect("new physical receiver child must publish TCP endpoint");
    (child, out, addr.to_string())
}
fn bf_process_round(root: &Path, seq: usize) -> (usize, BFCost) {
    use std::io::BufRead;
    let (mut receiver, mut out, addr) = bf_spawn_receiver(root, seq, false);
    let mut sources = be_child_send(seq, root, &addr);
    let mut result = String::new();
    out.read_line(&mut result).unwrap();
    let chunks: Vec<&str> = result.split_whitespace().collect();
    assert_eq!(chunks.len(), 5);
    assert_eq!(chunks[0], "BF_RESULT");
    let number = |index: usize, prefix: &str| -> usize {
        chunks[index].strip_prefix(prefix).unwrap().parse().unwrap()
    };
    let bytes = number(1, "bytes=");
    let cost = BFCost {
        data_bytes: number(2, "written="),
        sync_calls: number(3, "fsync="),
        checkpoint_bytes: number(4, "checkpoint="),
    };
    assert!(receiver.wait().unwrap().success());
    for source in &mut sources {assert!(source.wait().unwrap().success());}
    (bytes, cost)
}
fn bf_crash_and_retry(root: &Path, seq: usize) -> (usize, usize, BFCost) {
    use std::io::BufRead;
    let (mut receiver, mut out, addr) = bf_spawn_receiver(root, seq, true);
    let mut sources = be_child_send(seq, root, &addr);
    let mut ack = String::new();
    out.read_line(&mut ack).unwrap();
    assert_eq!(ack.trim(), format!("BF_ACK1_READY generation={seq}"));
    receiver.kill().unwrap();
    assert!(!receiver.wait().unwrap().success());
    for source in &mut sources {assert!(source.wait().unwrap().success());}
    assert_eq!(bf_load(root).unwrap().receipt.seq, [seq, seq]);
    assert_eq!(be_source_load(&bd_dir(root, 1), 1).unwrap().ack, seq);
    assert_eq!(be_source_load(&bd_dir(root, 2), 2).unwrap().ack, seq - 1);
    let (retry, cost) = bf_process_round(root, seq);
    assert_eq!(cost.data_bytes, 0, "duplicate cannot rewrite WAL");
    assert_eq!(cost.sync_calls, 0, "duplicate cannot re-fsync commit");
    assert_eq!(be_source_load(&bd_dir(root, 2), 2).unwrap().ack, seq);
    (340, retry, BFCost {
        data_bytes: BF_TX + 48 + bf_load_checkpoint_size(root, seq),
        sync_calls: if seq == 51 {7} else {3},
        checkpoint_bytes: bf_load_checkpoint_size(root, seq),
    })
}
fn bf_load_checkpoint_size(root: &Path, seq: usize) -> usize {
    if seq == 51 || seq == 101 {
        std::fs::metadata(root.join("receiver.ckpt")).unwrap().len() as usize
    } else {0}
}
fn bf_negative(root: &Path) {
    let receipt = bf_load(root).unwrap();
    assert_eq!(receipt.receipt.seq, [101, 101]);
    let before = receipt.receipt.lists.clone();
    let chk = std::fs::read(root.join("receiver.ckpt")).unwrap();
    let mut changed = chk.clone();
    changed[32] ^= 1;
    std::fs::write(root.join("receiver.ckpt"), &changed).unwrap();
    assert!(bf_load(root).is_err(), "checkpoint corruption accepted");
    bd_atomic(root, "receiver.ckpt", &chk);
    let mark = std::fs::read(root.join("receiver.commit")).unwrap();
    let mut forged = mark.clone();
    forged[16] ^= 1;
    bd_atomic(root, "receiver.commit", &forged);
    assert!(bf_load(root).is_err(), "commit digest mismatch accepted");
    bd_atomic(root, "receiver.commit", &mark);
    let mut rollback = mark.clone();
    rollback[8..16].copy_from_slice(&100_u64.to_le_bytes());
    bd_atomic(root, "receiver.commit", &rollback);
    assert!(bf_load(root).is_err(), "commit rollback below checkpoint accepted");
    bd_atomic(root, "receiver.commit", &mark);
    // Noncommitted/torn tail must not fabricate a new accepted generation.
    use std::io::Write;
    let mut log = std::fs::OpenOptions::new().append(true)
        .open(root.join("receiver.wal")).unwrap();
    log.write_all(&[0xaa; 41]).unwrap();
    log.sync_all().unwrap();
    assert_eq!(bf_load(root).unwrap().receipt.lists, before);
    bd_atomic(root, "receiver.wal", &[]);
    assert_eq!(bf_load(root).unwrap().receipt.seq, [101, 101]);
    // Mutate a source's bound WAL, preserve exact live receiver state.
    let path = bd_dir(root, 1).join("chain.wal");
    let wal = std::fs::read(&path).unwrap();
    let mut corrupted = wal.clone();
    corrupted[BE_REC * 65 + 68] ^= 1;
    std::fs::write(&path, &corrupted).unwrap();
    assert!(be_source_load(&bd_dir(root, 1), 1).is_err());
    bd_atomic(&bd_dir(root, 1), "chain.wal", &wal);
    assert_eq!(bf_load(root).unwrap().receipt.lists, before);
    // ABA duplicate MUST bind the exact persisted last accepted event,
    // not a coincidentally identical token-membership state.
    let stable = bf_load(root).unwrap();
    let first = stable.receipt.last[0];
    let second = stable.receipt.last[1];
    let mut altered = first;
    altered[32] ^= 1;
    let seq = stable.receipt.seq[0];
    let frames = [bf_event_frame(1, seq, &first), bf_event_frame(2, seq, &second)];
    assert!(be_validate_transition(&stable.receipt,
        &[altered, second], &frames, seq).is_err());
    assert_eq!(bf_load(root).unwrap().receipt.lists, before);
}
fn bf_worker(worker: usize) {
    assert!((1..=5).contains(&worker));
    let mut count = 0_usize;
    for (lane, &n) in BE_NS.iter().enumerate() {
        for rep in 0..3 {
            let root = std::env::temp_dir().join(format!(
                "deltameter-bf-w{worker}-n{n}-rep{rep}-pid{}", std::process::id()));
            if root.exists() {std::fs::remove_dir_all(&root).unwrap();}
            be_init(worker, lane, rep, &root);
            let initial = bf_init(&root);
            for seq in 2..=BE_GENS + 1 {
                be_writer_pair(&root, seq);
                let (wire, retry, cost) = if seq == 51 {
                    bf_crash_and_retry(&root, seq)
                } else {
                    let (bytes, write) = bf_normal(&root, seq);
                    (bytes, 0, write)
                };
                let state = bf_load(&root).unwrap();
                be_oracle(worker, lane, rep, seq, &state.receipt);
                assert_eq!(state.receipt.seq, [seq, seq]);
                for owner in 1..=2 {
                    let src = be_source_load(&bd_dir(&root, owner), owner).unwrap();
                    assert_eq!(src.seq, seq);
                    assert_eq!(src.ack, seq);
                    assert_eq!(src.tokens, state.receipt.lists[(owner-1) as usize]);
                    assert_eq!(src.head, state.receipt.heads[(owner-1) as usize]);
                }
                assert_eq!(wire, if seq == 51 {340} else {356});
                assert_eq!(retry, if seq == 51 {356} else {0});
                let ckpt = if seq == 51 || seq == 101 {initial} else {0};
                assert_eq!(cost.checkpoint_bytes, ckpt);
                assert_eq!(cost.data_bytes, BF_TX + 48 + ckpt);
                assert_eq!(cost.sync_calls, if ckpt > 0 {7} else {3});
                println!("B1B1B1B1B0_SAMPLE worker={worker} lane={lane} rep={rep} gen={seq} N={n} mode={} tcp={wire} retry={retry} wal_bytes={} written={} sync={} checkpoint={} crash={} exact=1",
                    if seq.is_multiple_of(2) {"insert"} else {"delete"},
                    std::fs::metadata(root.join("receiver.wal")).unwrap().len(),
                    cost.data_bytes, cost.sync_calls,
                    cost.checkpoint_bytes, u8::from(seq == 51));
                count += 1;
            }
            bf_negative(&root);
            assert_eq!(std::fs::metadata(root.join("receiver.wal")).unwrap().len(), 0);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    assert_eq!(count, 600);
    println!("DELTAGUARD_B1B1B1B1B0_WORKER_PASS worker={worker} records={count} actual_receiver_sigkill=6 negative_groups=6");
}
