// Research-only B1-A: two independent localhost TCP senders and fail-closed frames.
// All test keys are public fixtures. The BLAKE3 tag is NOT sender authentication.
// No latency, throughput, WAN, or system-product conclusion is made in B1-A.

const B1_MAGIC: &[u8; 8] = b"DGB1TCP1";
const B1_HEADER: usize = 64;
const B1_MAX_PAYLOAD: usize = 1_100_000;
const B1_EPOCH: u64 = 0x2026_1009_0000_0001;
const B1_KEY_CONTEXT: &str = "deltameter 2026-10-09 deltaguard b1 transport fixture v1";
const B1_GUARD: u8 = 1;
const B1_FULL: u8 = 2;
const B1_DELTA: u8 = 3;

fn b1_key_id() -> [u8; 16] {
    let derived = derive_key(B1_KEY_CONTEXT, &MASTER_KEY);
    blake3::hash(&derived).as_bytes()[..16].try_into().unwrap()
}

fn b1_tag(header_prefix: &[u8], body: &[u8]) -> [u8; 16] {
    let mut hash = blake3::Hasher::new();
    hash.update(header_prefix);
    hash.update(body);
    hash.finalize().as_bytes()[..16].try_into().unwrap()
}

fn b1_encode(owner: u8, kind: u8, epoch: u64, seq: u64, body: &[u8]) -> Vec<u8> {
    assert!((1..=2).contains(&owner));
    assert!([B1_GUARD, B1_FULL, B1_DELTA].contains(&kind));
    assert!(body.len() <= B1_MAX_PAYLOAD);
    let mut result = vec![0_u8; B1_HEADER + body.len()];
    result[..8].copy_from_slice(B1_MAGIC);
    result[8] = 1;
    result[9] = owner;
    result[10] = kind;
    result[11] = if kind == B1_GUARD { 11 } else { 0 };
    result[12..20].copy_from_slice(&epoch.to_le_bytes());
    result[20..28].copy_from_slice(&seq.to_le_bytes());
    result[28..32].copy_from_slice(&(body.len() as u32).to_le_bytes());
    result[32..48].copy_from_slice(&b1_key_id());
    result[B1_HEADER..].copy_from_slice(body);
    let tag = b1_tag(&result[..48], body);
    result[48..64].copy_from_slice(&tag);
    result
}

fn b1_canonical_full(body: &[u8]) -> Result<(), &'static str> {
    if !body.len().is_multiple_of(8) {
        return Err("full-width");
    }
    let mut previous = None;
    for chunk in body.as_chunks::<8>().0.iter() {
        let token = u64::from_le_bytes(chunk.try_into().unwrap());
        if previous.is_some_and(|value| token <= value) {
            return Err("full-not-canonical");
        }
        previous = Some(token);
    }
    Ok(())
}

fn b1_canonical_delta(body: &[u8]) -> Result<(), &'static str> {
    if !body.len().is_multiple_of(9) {
        return Err("delta-width");
    }
    let mut previous = None;
    for chunk in body.as_chunks::<9>().0.iter() {
        if chunk[0] != 1 && chunk[0] != 2 {
            return Err("delta-op");
        }
        let token = u64::from_le_bytes(chunk[1..9].try_into().unwrap());
        if previous.is_some_and(|value| token <= value) {
            return Err("delta-not-canonical");
        }
        previous = Some(token);
    }
    Ok(())
}

fn b1_decode(
    packet: &[u8],
    expected_owner: u8,
    expected_kind: u8,
    epoch: u64,
    seq: u64,
) -> Result<&[u8], &'static str> {
    if packet.len() < B1_HEADER {
        return Err("short-header");
    }
    if &packet[..8] != B1_MAGIC || packet[8] != 1 {
        return Err("magic-version");
    }
    if packet[9] != expected_owner || !(1..=2).contains(&packet[9]) {
        return Err("sender");
    }
    if packet[10] != expected_kind || ![B1_GUARD, B1_FULL, B1_DELTA].contains(&packet[10]) {
        return Err("kind");
    }
    if packet[11] != if expected_kind == B1_GUARD { 11 } else { 0 } {
        return Err("profile");
    }
    if u64::from_le_bytes(packet[12..20].try_into().unwrap()) != epoch {
        return Err("epoch");
    }
    if u64::from_le_bytes(packet[20..28].try_into().unwrap()) != seq {
        return Err("generation");
    }
    let len = u32::from_le_bytes(packet[28..32].try_into().unwrap()) as usize;
    if len > B1_MAX_PAYLOAD || packet.len() != B1_HEADER + len {
        return Err("payload-length");
    }
    if packet[32..48] != b1_key_id() {
        return Err("key-identity");
    }
    if packet[48..64] != b1_tag(&packet[..48], &packet[64..]) {
        return Err("checksum");
    }
    let body = &packet[B1_HEADER..];
    match expected_kind {
        B1_GUARD => {
            if body.len() != 256 || body[255] & 0x80 != 0 {
                return Err("bitmap-shape-padding");
            }
        }
        B1_FULL => b1_canonical_full(body)?,
        B1_DELTA => b1_canonical_delta(body)?,
        _ => return Err("kind"),
    }
    Ok(body)
}

fn b1_full_bytes(tokens: &[u64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(tokens.len() * 8);
    for token in tokens {
        output.extend_from_slice(&token.to_le_bytes());
    }
    output
}

fn b1_words_bytes(words: &[u64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(words.len() * 8);
    for word in words {
        output.extend_from_slice(&word.to_le_bytes());
    }
    output
}

fn b1_actual_pair(
    left: Vec<u8>,
    right: Vec<u8>,
    kind: u8,
    seq: u64,
) -> Result<(Vec<u8>, Vec<u8>, usize), &'static str> {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| "bind")?;
    let address = listener.local_addr().map_err(|_| "address")?;
    let original_bytes = left.len() + right.len();
    let mut workers = Vec::new();
    for packet in [left, right] {
        workers.push(std::thread::spawn(move || {
            let mut socket = TcpStream::connect(address).expect("sender connects");
            socket.set_nodelay(true).unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            socket.write_all(&packet).expect("send complete frame");
        }));
    }

    let mut received: [Option<Vec<u8>>; 2] = [None, None];
    let mut error = None;
    let mut receiver_bytes = 0;
    for _ in 0..2 {
        let (mut socket, _) = listener.accept().map_err(|_| "accept")?;
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|_| "read-timeout")?;
        let mut header = [0_u8; B1_HEADER];
        socket
            .read_exact(&mut header)
            .map_err(|_| "short-header-over-tcp")?;
        let len = u32::from_le_bytes(header[28..32].try_into().unwrap()) as usize;
        if len > B1_MAX_PAYLOAD {
            error = Some("oversized-over-tcp");
            break;
        }
        let mut packet = header.to_vec();
        packet.resize(B1_HEADER + len, 0);
        if socket.read_exact(&mut packet[B1_HEADER..]).is_err() {
            error = Some("short-payload-over-tcp");
            break;
        }
        receiver_bytes += packet.len();
        let owner = packet[9];
        if !(1..=2).contains(&owner) || received[(owner - 1) as usize].is_some() {
            error = Some("duplicate-or-invalid-owner");
            break;
        }
        match b1_decode(&packet, owner, kind, B1_EPOCH, seq) {
            Ok(_) => received[(owner - 1) as usize] = Some(packet),
            Err(e) => error = Some(e),
        }
    }
    for worker in workers {
        worker.join().map_err(|_| "sender-panicked")?;
    }
    if let Some(error) = error {
        return Err(error);
    }
    if receiver_bytes != original_bytes {
        return Err("wire-byte-mismatch");
    }
    Ok((
        received[0].take().ok_or("missing-owner-1")?,
        received[1].take().ok_or("missing-owner-2")?,
        receiver_bytes,
    ))
}

fn b1_assert_failure(packet: &[u8], owner: u8, kind: u8, epoch: u64, seq: u64) {
    assert!(
        b1_decode(packet, owner, kind, epoch, seq).is_err(),
        "malformed frame was accepted"
    );
}

fn b1_corruption_vectors(good: &[u8]) {
    for idx in [0, 8, 9, 10, 11, 12, 20, 28, 32, 48, 63, 65] {
        let mut tampered = good.to_vec();
        tampered[idx] ^= 1;
        b1_assert_failure(&tampered, 1, B1_GUARD, B1_EPOCH, 1);
    }
    b1_assert_failure(&good[..63], 1, B1_GUARD, B1_EPOCH, 1);
    b1_assert_failure(&good[..good.len() - 1], 1, B1_GUARD, B1_EPOCH, 1);
    b1_assert_failure(good, 2, B1_GUARD, B1_EPOCH, 1);
    b1_assert_failure(good, 1, B1_GUARD, B1_EPOCH + 1, 1);
    b1_assert_failure(good, 1, B1_GUARD, B1_EPOCH, 2);
    let mut bad_bitmap = good[B1_HEADER..].to_vec();
    bad_bitmap[255] |= 0x80;
    let bad = b1_encode(1, B1_GUARD, B1_EPOCH, 1, &bad_bitmap);
    b1_assert_failure(&bad, 1, B1_GUARD, B1_EPOCH, 1);
    let full = b1_encode(1, B1_FULL, B1_EPOCH, 1, &b1_full_bytes(&[1, 2, 3]));
    assert!(b1_decode(&full, 1, B1_FULL, B1_EPOCH, 1).is_ok());
    for bad_tokens in [&[1, 1][..], &[3, 2][..]] {
        let packet = b1_encode(1, B1_FULL, B1_EPOCH, 1, &b1_full_bytes(bad_tokens));
        b1_assert_failure(&packet, 1, B1_FULL, B1_EPOCH, 1);
    }
    let mut valid_delta = Vec::new();
    valid_delta.push(1);
    valid_delta.extend_from_slice(&10_u64.to_le_bytes());
    valid_delta.push(2);
    valid_delta.extend_from_slice(&11_u64.to_le_bytes());
    let packet = b1_encode(1, B1_DELTA, B1_EPOCH, 10, &valid_delta);
    assert!(b1_decode(&packet, 1, B1_DELTA, B1_EPOCH, 10).is_ok());
    valid_delta[9] = 3;
    let packet = b1_encode(1, B1_DELTA, B1_EPOCH, 10, &valid_delta);
    b1_assert_failure(&packet, 1, B1_DELTA, B1_EPOCH, 10);
}

fn b1_frame_lab(worker: usize) {
    assert!((1..=5).contains(&worker));
    for &d in &[48_usize, 57, 65] {
        let n = 256;
        let start = worker as u64 * 1_000_000;
        let a: Vec<u64> = (0..n as u64).map(|x| start + x).collect();
        let mut b: Vec<u64> = (0..(n - d / 2) as u64).map(|x| start + x).collect();
        b.extend((0..d.div_ceil(2) as u64).map(|x| start + n as u64 + x));
        assert_eq!(b0_diff(&a, &b).len(), d);

        let mut ga = NearFullGuard::new(11, 64, &MASTER_KEY);
        let mut gb = NearFullGuard::new(11, 64, &MASTER_KEY);
        for token in &a {
            ga.toggle(*token);
        }
        for token in &b {
            gb.toggle(*token);
        }

        let left = b1_encode(1, B1_GUARD, B1_EPOCH, 1, &b1_words_bytes(&ga.words));
        let right = b1_encode(2, B1_GUARD, B1_EPOCH, 1, &b1_words_bytes(&gb.words));
        b1_corruption_vectors(&left);
        let (p1, p2, measured_bytes) =
            b1_actual_pair(left, right, B1_GUARD, 1).expect("two genuine guarded senders");
        assert_eq!(measured_bytes, 640);
        let one = b1_decode(&p1, 1, B1_GUARD, B1_EPOCH, 1).unwrap();
        let two = b1_decode(&p2, 2, B1_GUARD, B1_EPOCH, 1).unwrap();
        let odd: u32 = one
            .as_chunks::<8>().0.iter()
            .zip(two.as_chunks::<8>().0.iter())
            .map(|(l, r)| {
                (u64::from_le_bytes(l.try_into().unwrap())
                    ^ u64::from_le_bytes(r.try_into().unwrap()))
                .count_ones()
            })
            .sum();
        let safe = odd <= 48;
        if d <= 48 {
            assert!(safe, "S <= d gives deterministic SAFE");
        }

        let (f1, f2, full_bytes) = b1_actual_pair(
            b1_encode(1, B1_FULL, B1_EPOCH, 1, &b1_full_bytes(&a)),
            b1_encode(2, B1_FULL, B1_EPOCH, 1, &b1_full_bytes(&b)),
            B1_FULL,
            1,
        )
        .expect("two genuine exact-list senders");
        assert_eq!(full_bytes, 128 + 8 * (a.len() + b.len()));
        let read_tokens = |frame: &[u8], owner: u8| {
            b1_decode(frame, owner, B1_FULL, B1_EPOCH, 1)
                .unwrap()
                .as_chunks::<8>().0.iter()
                .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
                .collect::<Vec<u64>>()
        };
        assert_eq!(b0_diff(&read_tokens(&f1, 1), &read_tokens(&f2, 2)).len(), d);

        let mut corrupt = p1;
        corrupt[64] ^= 1;
        let clean = p2;
        assert!(b1_actual_pair(corrupt, clean, B1_GUARD, 1).is_err());
        println!(
            "B1A_TCP_SAMPLE worker={worker} d={d} guard_wire={measured_bytes} exact_wire={full_bytes} odd={odd} safe={}",
            u8::from(safe)
        );
    }
    println!("DELTAGUARD_B1A_FRAME_PASS worker={worker} physical_pairs=9");
}
