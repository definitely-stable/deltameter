# DeltaGuard B1-B1B1-B0 — real fsync'd two-owner WAL crash/replay evidence

Parent [#103](https://github.com/definitely-stable/deltameter/issues/103),
[PR #104](https://github.com/definitely-stable/deltameter/pull/104).
[Frozen protocol](DELTAGUARD-B1B1B1B0-PROTOCOL.md)
preregistered **before** any B1-B1B1-B0 measurement at commit
`fbada643ee1ee0d2ea557acd4371775b63a0ecfe`.
Prior nonadaptive B2A proof SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`
unchanged. D60 and D61 remain binding.

## Actual source-bound PASS, not a product release

**B1B1B1B0_DURABLE_PREFIX_REPLAY_RESEARCH_ACCEPT_NO_PRODUCT_GO**.
First complete [five hosted-worker GitHub Actions #37928262462](https://github.com/definitely-stable/deltameter/actions/runs/37928262462)
at code SHA `64641d81db97bbead9eedaf8a4e936a173ed6326`:
certificate regeneration, all 5 distinct hosted workers, source-pinned
fail-closed Python aggregate **SUCCESS; 210/210 complete real
owner restart trials**, 42 per worker. Each trial kills TWO
source-owner OS processes via SIGKILL, spawns fresh successors
which read source state from their own private persisted
`base.snap`, `events.wal`, `commit.mark` and `ack.mark`,
and restarts a separate receiver OS process which reopens
the atomically persisted two-owner `receiver.bin`.
No source fixture reconstruction is invoked in the sender
restart path. Fixture reconstruction exists only in the
independent ORACLE following physically transmitted recovery.

All source snapshots, WAL, commit and ACK marker publication,
and the complete receiver state use real Linux
`File::sync_all` followed as appropriate by atomic
rename and parent directory `sync_all`. The source
uncommitted WAL, even if its bytes survive SIGKILL in
kernel page cache, is deliberately *not* an accepted
state without the independent synced commit marker.
After physical ACK, an owner syncs its durable
delivery watermark. The two-owner receiver exact
update is atomically persisted BEFORE any positive
ACK. A restarted receiver accepts a duplicate
identical persisted-generation insertion WITHOUT
double application, then ACKs. Identity includes
owner ID, epoch, generation, profile/key and canonical
B1-A framed payload, with integrity checksums.

## Seven independently preregistered crash boundaries

| Cutpoint | Persisted source on reboot | Persisted receiver before replay | Required action |
|---|---|---|---|
| Before WAL | Generation 1 | Generation 1 | Full generation1 source verification |
| WAL write, before fsync/commit | Generation 1 | Generation 1 | Ignore WAL tail, physically send full gen1 |
| WAL fsync, before commit marker | Generation 1 | Generation 1 | Reject unaccepted update, physically send full gen1 |
| After committed WAL marker, before send | Generation 2 | Generation 1 | Source disk delta replay |
| After sending delta, before ACK | Generation 2 | Generation 1 | Replay physical delta, durable receiver commit |
| Receiver fsync, before ACK | Generation 2 | Generation 2 | Duplicate acknowledged without reapply |
| ACK sent, before owner ACK watermark fsync | Generation 2 | Generation 2 | Reopen stale owner cursor, idempotent replay and fsync cursor |

The first three cutpoints intentionally permit losing
an **unacknowledged / uncommitted** update. A complete
WAL record that was fsync'd but never acquired its
atomically published commit marker is intentionally
ignored, which is a conservative *application-level*
commit policy, not data loss after a successful ACK.
The last four retain the committed source update
and reconstruct both exact receiver inventories
from the durable source WAL without transferring
full canonical source contents.

## Real TCP application bytes measured

Two source-child and receiver-child identities, physically
transmitted source initial FULL snapshots, requests/headers,
generation/cursor handshake, 16B ACK and 16B post-transfer
RSS+CPU telemetry were counted. The independent aggregator
rejects any missing case or incorrect generation/bytes.

| Source size N | Initial physical two-owner bootstrap *per trial* | Reconnect with uncommitted WAL (full gen1 both) | Reconnect with durable generation2 (two small deltas) |
|---|---:|---:|---:|
| 256 | 4,338B | 4,386B | **308B** |
| 65,536 | 1,048,818B | 1,048,866B | **308B** |

The stage SIGKILL handshake costs 4B for a cut before
payload delivery; 150B after two on-wire DELTA frames;
184B when actual ACKs are sent before killing source
processes. These bytes are separately accounted.

Across all five workers and three fixtures per N
(**105 crash trials per N**):
- N256 physical bootstrap **455,490B**;
  recovery physical **215,850B**.
- N65536 bootstrap **110,125,890B**;
  recovery physical **47,217,450B**.
- Each N lane contains 45 intentionally uncommitted
  generation1 full source reconnects, 30 committed
  source generation2 replays and 30 receiver-durable
  exact deduplications.
- Across both N lanes **420 real source SIGKILL**,
  **60 committed state replays**, **60 already-durable
  receiver idempotent ACK sequences**.

Source ACK markers are re-opened from disk after
receiving ACK and source restart. Five independent
negative disk tests per worker verify committed-WAL
checksum corruption rejection, owner mismatch,
receiver epoch/integrity corruption rejection,
ACK watermark ahead of committed WAL rejection,
and ignored uncommitted WAL tail.

## Critical limitations — what this does NOT prove

- SIGKILL + Linux File::sync_all + directory fsync
  tests a **process crash protocol** on hosted Linux.
  It is not simulated power failure, disk cache
  capacitor loss, filesystem corruption under I/O
  errors, power-cut model checking or end-to-end
  transactional replication under hostile clients.
- Only ONE inserted token per owner at generation2
  was tested. Multi-generation append, removal/
  conflicting inserts, log rollover and checkpoint
  compaction, concurrent unrelated writers,
  socket disconnect or receiver crash BETWEEN
  per-owner ACKs, and arbitrary reordered frames
  need additional predeclared tests.
- Source event origin for the initial single update
  is fixed-fixture during the initial autonomous
  writer process; restart does NOT regenerate fixture
  truth and the receiver recovery does NOT bypass
  physical TCP. This does NOT yet establish trusted
  autonomous remote data authority.
- **Unkeyed public BLAKE3 checksums are NOT
  sender authentication**. Adaptive security #86
  and public product/API #69 remain blocked.
- Real disk-sync calls were made but latency,
  durable journal disk write amplification/CPU,
  large source construction and compaction,
  tail p95 under failure, and broad 1/10/100Mbps
  + 0/10/50ms application-emulated matrix
  were not compared with equally durable
  retained exact or fully resolved guard.
  Therefore 308B reconnect is **transport-only
  evidence**, not an algorithm/service throughput
  win or release justification.

Decision is a **narrow durable-prefix replay research
accept**. D60's warm-N256 DeltaGuard NO-GO and
d57 fully resolved STOP remain; cold N65536 sparse
guard niche is still an unproven product candidate.
Next B1-B1B1-B1 needs multi-generation delete/restart,
receiver crash between two ACKs, malicious changes and
journal corrupt tail model, source+receiver CPU/RSS/disk
write amplification, fault-frequency and matched
retained-exact vs DeltaGuard total cost before
closing parent #103/#101/#99/#97/#92. No public API
or Snapshot v1 changes.
