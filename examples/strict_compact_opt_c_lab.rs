// STRICT-COMPACT OPT-C: research-only immutable level-transfer correctness.
// Embed the already accepted OPT-A reference without changing its oracle/math.
#[allow(dead_code)]
mod opt_c {
    include!("strict_compact_opt_a_lab.rs");

    const J: u32 = 52;
    const WORDS_PER_LEVEL: usize = ROWS / 64;
    const LEVEL_BYTES: usize = ROWS / 8;
    const EPOCH_BYTES: usize = 16;
    const FRAME_BYTES: usize = EPOCH_BYTES + 1 + LEVEL_BYTES + 4;
    const MAGIC: [u8; 8] = *b"DMPCRES0";

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct Session {
        magic: [u8; 8],
        version: u8,
        rows: u16,
        levels: u8,
        epoch: [u8; 16],
        config_binding: [u8; 32],
        table_binding: [u8; 32],
    }

    #[derive(Clone)]
    struct Frozen {
        session: Session,
        words: Box<[u64]>,
    }

    impl Frozen {
        fn from_sketch(sketch: &PackedSketch, epoch: [u8; 16], config: [u8; 32]) -> Self {
            assert_eq!(sketch.levels, J);
            assert_eq!(sketch.layout, Layout::LevelMajor);
            let mut table_binding = [0_u8; 32];
            // Fixed *research fixture* binding. This is NOT an authenticated
            // identifier of actual Q32 contents (workflow separately hashes Q32).
            table_binding.copy_from_slice(&[0x5a; 32]);
            let session = Session {
                magic: MAGIC,
                version: 0,
                rows: ROWS as u16,
                levels: J as u8,
                epoch,
                config_binding: config,
                table_binding,
            };
            Self {
                session,
                words: sketch.words.clone(),
            }
        }

        fn frame(&self, level_one_based: u8) -> [u8; FRAME_BYTES] {
            assert!((1..=J as u8).contains(&level_one_based));
            let mut frame = [0_u8; FRAME_BYTES];
            frame[..EPOCH_BYTES].copy_from_slice(&self.session.epoch);
            frame[EPOCH_BYTES] = level_one_based;
            let start = (level_one_based as usize - 1) * WORDS_PER_LEVEL;
            let payload = &mut frame[EPOCH_BYTES + 1..EPOCH_BYTES + 1 + LEVEL_BYTES];
            for (out, word) in payload
                .chunks_exact_mut(8)
                .zip(&self.words[start..start + WORDS_PER_LEVEL])
            {
                out.copy_from_slice(&word.to_le_bytes());
            }
            let checksum = crc32(&frame[..FRAME_BYTES - 4]);
            frame[FRAME_BYTES - 4..].copy_from_slice(&checksum.to_le_bytes());
            frame
        }
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = !0_u32;
        for &b in data {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (if crc & 1 != 0 { 0xedb8_8320 } else { 0 });
            }
        }
        !crc
    }

    struct Receiver {
        session: Session,
        parts: Vec<Option<[u8; LEVEL_BYTES]>>,
    }

    impl Receiver {
        fn new(
            session: &Session,
            expected_config: [u8; 32],
            expected_table: [u8; 32],
        ) -> Result<Self, &'static str> {
            if session.magic != MAGIC
                || session.version != 0
                || session.rows != ROWS as u16
                || session.levels != J as u8
            {
                return Err("bad-session-profile");
            }
            if session.config_binding != expected_config || session.table_binding != expected_table
            {
                return Err("wrong-key-or-table-config");
            }
            Ok(Self {
                session: session.clone(),
                parts: vec![None; J as usize],
            })
        }

        fn ingest(&mut self, bytes: &[u8]) -> Result<bool, &'static str> {
            if bytes.len() != FRAME_BYTES {
                return Err("bad-frame-length");
            }
            if bytes[..EPOCH_BYTES] != self.session.epoch {
                return Err("wrong-epoch");
            }
            let level = bytes[EPOCH_BYTES];
            if !(1..=J as u8).contains(&level) {
                return Err("bad-level-index");
            }
            let recorded = u32::from_le_bytes(bytes[FRAME_BYTES - 4..].try_into().unwrap());
            if recorded != crc32(&bytes[..FRAME_BYTES - 4]) {
                return Err("corrupt-frame");
            }
            let mut payload = [0_u8; LEVEL_BYTES];
            payload.copy_from_slice(&bytes[EPOCH_BYTES + 1..FRAME_BYTES - 4]);
            let index = level as usize - 1;
            if let Some(old) = &self.parts[index] {
                return if *old == payload {
                    Ok(false)
                } else {
                    Err("conflicting-level-duplicate")
                };
            }
            self.parts[index] = Some(payload);
            Ok(true)
        }

        fn received(&self) -> usize {
            self.parts.iter().filter(|part| part.is_some()).count()
        }

        fn bound(&self, table: &[u64]) -> u128 {
            let mut best = DOMAIN_CARDINALITY;
            for (j, part) in self.parts.iter().enumerate() {
                let Some(data) = part else { continue };
                let odd = data.iter().map(|v| v.count_ones()).sum::<u32>() as usize;
                if odd >= TABLE_ENTRIES {
                    continue;
                }
                let q = table[odd];
                if q != SENTINEL {
                    best = best.min(q32_level_upper(q, (j + 1) as u32));
                }
            }
            best
        }

        fn complete_words(&self) -> Result<Box<[u64]>, &'static str> {
            if self.received() != J as usize {
                return Err("missing-levels");
            }
            let mut words = Vec::with_capacity(J as usize * WORDS_PER_LEVEL);
            for part in &self.parts {
                let data = part.as_ref().ok_or("missing-levels")?;
                for chunk in data.chunks_exact(8) {
                    words.push(u64::from_le_bytes(chunk.try_into().unwrap()));
                }
            }
            Ok(words.into_boxed_slice())
        }
    }

    fn verify_frame_correctness(table: &[u64]) {
        let mut live = PackedSketch::new(J, Layout::LevelMajor);
        for token in 0_u64..65_536 {
            live.toggle(token);
        }
        let epoch = [0x51; 16];
        let cfg = [0x39; 32];
        let frozen = Frozen::from_sketch(&live, epoch, cfg);
        assert_eq!(frozen.words.len(), J as usize * WORDS_PER_LEVEL);
        let full_estimate = live.estimate(table);
        // Snapshot immutability is independent of later updates.
        for token in 100_000_u64..105_000 {
            live.toggle(token);
        }
        assert_ne!(frozen.words.as_ref(), live.words.as_ref());
        let mut rx = Receiver::new(&frozen.session, cfg, frozen.session.table_binding).unwrap();
        assert_eq!(rx.bound(table), DOMAIN_CARDINALITY);
        assert_eq!(rx.received(), 0);
        assert!(rx.complete_words().is_err());

        let mut order: Vec<u8> = (1..=J as u8).collect();
        order.reverse();
        let mut last = DOMAIN_CARDINALITY;
        for level in order {
            let frame = frozen.frame(level);
            assert_eq!(frame.len(), 533);
            assert!(rx.ingest(&frame).unwrap());
            assert!(!rx.ingest(&frame).unwrap()); // equal duplicate idempotent
            let new_bound = rx.bound(table);
            assert!(new_bound <= last, "adding a level raised bound");
            last = new_bound;
        }
        assert_eq!(rx.received(), J as usize);
        assert_eq!(rx.complete_words().unwrap(), frozen.words);
        assert_eq!(rx.bound(table), full_estimate);
        let mut rx_forward =
            Receiver::new(&frozen.session, cfg, frozen.session.table_binding).unwrap();
        for j in 1..=J as u8 {
            rx_forward.ingest(&frozen.frame(j)).unwrap();
        }
        assert_eq!(rx_forward.bound(table), full_estimate);
        assert_eq!(
            rx_forward.complete_words().unwrap(),
            rx.complete_words().unwrap()
        );

        // Missing level is not the same as a received all-zero level.
        let empty = PackedSketch::new(J, Layout::LevelMajor);
        let blank = Frozen::from_sketch(&empty, [0x52; 16], cfg);
        let mut zero_rx = Receiver::new(&blank.session, cfg, blank.session.table_binding).unwrap();
        assert_eq!(zero_rx.received(), 0);
        assert!(zero_rx.ingest(&blank.frame(1)).unwrap());
        assert_eq!(zero_rx.received(), 1);

        // Fail closed on wrong-session config, corrupt/short frame, bad index,
        // wrong epoch and a conflicting duplicate even if CRC is recomputed.
        assert!(Receiver::new(&frozen.session, [0x37; 32], frozen.session.table_binding).is_err());
        assert!(Receiver::new(&frozen.session, cfg, [0x37; 32]).is_err());
        let mut corrupt = frozen.frame(1);
        corrupt[EPOCH_BYTES + 1] ^= 0x80;
        assert_eq!(rx.ingest(&corrupt), Err("corrupt-frame"));
        assert_eq!(
            rx.ingest(&corrupt[..FRAME_BYTES - 1]),
            Err("bad-frame-length")
        );
        let mut wrong_epoch = frozen.frame(1);
        wrong_epoch[0] ^= 1;
        assert_eq!(rx.ingest(&wrong_epoch), Err("wrong-epoch"));
        let mut wrong_index = frozen.frame(1);
        wrong_index[EPOCH_BYTES] = 0;
        assert_eq!(rx.ingest(&wrong_index), Err("bad-level-index"));
        let mut conflict = frozen.frame(1);
        conflict[EPOCH_BYTES + 1] ^= 1;
        let checksum = crc32(&conflict[..FRAME_BYTES - 4]);
        conflict[FRAME_BYTES - 4..].copy_from_slice(&checksum.to_le_bytes());
        assert_eq!(rx.ingest(&conflict), Err("conflicting-level-duplicate"));
        println!(
            "STRICT_COMPACT_OPT_C_FOUNDATION_PASS state_bytes={} level_bytes={} frame_bytes={} levels={}",
            frozen.words.len() * 8,
            LEVEL_BYTES,
            FRAME_BYTES,
            J
        );
    }

    pub(super) fn run(path: &Path) {
        let table = load_table(path);
        assert_oracle_vectors();
        verify_frame_correctness(&table);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let table_path = args
        .next()
        .expect("usage: strict_compact_opt_c_lab q32.txt");
    assert!(args.next().is_none(), "unexpected argument");
    opt_c::run(std::path::Path::new(&table_path));
}
