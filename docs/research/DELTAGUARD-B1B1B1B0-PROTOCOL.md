# DeltaGuard B1-B1B1-B0 — frozen durable two-owner WAL and restart protocol

Research parent [#103](https://github.com/definitely-stable/deltameter/issues/103);
D60 and D61 remain binding. **This protocol is committed before
B1-B1B1-B0 timings or fault CI.** No public API, Snapshot v1 or product GO.

## Narrow falsifiable goal

Unlike B1B1B1-A (owner fixtures regenerated on restart), **TWO
independent OS source-owner processes must load their canonical
initial inventories and at-most-one committed update solely from
their own private disk files after SIGKILL**. The receiver's two-owner
exact cache is also atomically fsync-persisted and REOPENED from
disk across subprocess restarts; oracle fixture reconstruction
must never be used as the source in recovery mode. The parent
test oracle may independently calculate expected truth only
after the receiving children have physically transmitted data.

Epoch B1_EPOCH, key/profile owner ids and monotone generation
must be verified on disk AND on B1-A TCP frames. The integrity
tags are public/unkeyed BLAKE3 and **NOT peer authentication**.

Source durability model (Linux GitHub-hosted POSIX):
- A canonical complete generation1 B1_FULL snapshot written via
  temp-file create+write_all+File::sync_all+rename+sync parent dir.
- A complete generation2 B1_DELTA record in an owner-exclusive
  `events.wal` written and `sync_all`ed. An uncommitted complete
  WAL tail is **NOT** treated as accepted state, even if SIGKILL
  happens to leave dirty cached bytes present.
- A separate `commit.mark` contains B1 epoch, sequence2 and
  BLAKE3 digest of the *exact complete* canonical WAL bytes.
  It is published via sync temp+rename+directory fsync
  **after** WAL fsync. Only both verified files constitute a
  committed generation2 update.
- `ack.mark` is separately published AFTER a real receiver
  16B ACK; it is an advisory owner delivery watermark, NOT
  evidence of a remote disk fsync by itself.
- On restart refuse missing/malformed/changed committed WAL,
  mismatched epoch/owner/key/gen, ACK ahead of commit, or
  conflicting generation. If uncommitted WAL exists, ignore
  its tail, report generation1 and NEVER claim it had
  reached stable storage. Separate fsync from real power loss:
  SIGKILL **is not a power-cut / durable hardware flush test**.

Receiver durability model: persist BOTH owner exact canonical
B1_FULL frames and their generation in ONE atomic synced
`receiver.bin`. ACK is sent only AFTER the new two-owner
receiver file is synced/renamed and parent directory fsynced.
On receiver restart, reopen verified snapshots from DISK,
not vectors retained by benchmark controller. Valid replay
against already persisted generation2 is idempotently
acknowledged without double insertion; different same-seq
payload or rollbacks fail closed.

## PREDECLARED CI experiment

5 GitHub-hosted runners × two source profiles
(N256,d48 and N65536,d48) × 3 fixed fixture repeats ×
7 named cutpoints **= 210 real two-owner source restart
records**, with 2 independently SIGKILL'ed OS sender
processes for each record:

1. `before_wal`: kill owner before WAL write;
   restart both sources at gen1; receiver remains gen1.
2. `after_write_before_sync`: write complete WAL but
   no fsync/commit marker; kill; restart at gen1,
   ignoring uncommitted WAL tail (no power-cut claim).
3. `after_wal_sync_before_commit`: fsync WAL but NO
   commit mark; kill; restart at gen1 (no accepted update).
4. `after_commit_before_send`: fsync WAL+commit marker,
   kill before sending; restart at gen2 and physically
   replay two source deltas into receiver gen1.
5. `after_send_before_ack`: physically send both
   committed delta packets, kill owners BEFORE receiver
   commits new state and before ACK; restart gen2,
   replay and commit exact receiver once.
6. `after_receiver_sync_before_ack`: physically send
   two valid committed delta packets, receiver fsyncs
   its two-owner gen2 cache BEFORE ACK, then kills
   senders; restart owners with stale ack cursor1,
   verify identical idempotent gen2 replay.
7. `after_ack_before_owner_sync`: receiver fsyncs
   both g2 inventories, sends physical ACK and sender
   acknowledges receipt before its local ack.mark
   is written, kill; restart with stale source ack1
   and receiver gen2, replay idempotently and fsync
   ACK cursor2.

Each reboot must create genuinely fresh child OS
processes and reread their private source snapshot
and WAL. Each receiver state reload physically reads
`receiver.bin`, verifying strict frame identity
and canonical inventory; the process hosting the
receiver is a fresh OS process on recovery, not
the orchestrator carrying an in-memory cache.
An early fault is allowed to lose an UNACKED update;
an accepted durable source commit MUST NOT be lost.
No pending data may be fabricated from fixture after
restart (fixtures permitted solely as an external
positive/negative oracle).

Physical TCP bytes must include source 1B hello,
generation/cursor identity, 32B request, full/delta
B1-A response frame, 16B ACK, and child end telemetry.
Charge separately disk writes + sync count/bytes,
source initial complete snapshots, receiver initial
sync, and retransmission bytes. FSYNC calls are
actually invoked, not mocked. Record source child
post-serializing RSS and real process CPU ticks, but
do not pretend Linux tick resolution establishes
microbenchmark CPU costs. No WAN RTT or artificial
latency p95 is measured in this correctness sub-slice.

Negative disk probes per worker (separate from 210):
committed WAL checksum tamper, invalid owner on snapshot,
bad epoch in receiver snapshot, ACK generation ahead of
commit, and reintroduced uncommitted WAL tail. All
must fail closed without accepting fabricated state.

## End of slice

Accept only 210/210 source-bound physical cases,
5/5 hosted workers and complete negative disk
tests, independent exact oracle and fail-closed
aggregation. Attest original B2A SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.

**B1B1B1B0_DURABLE_PREFIX_REPLAY_RESEARCH_ACCEPT**
does NOT prove power-cut hardware durability, system
product superiority, malicious/adaptive security or
robust reestablishment of remote autonomous source
authority. Next B1-B1B1-B1 must add process-independent
receiver restarts injected between individual owner
ACKs, true body truncation over reconnect, token
deletions and multiple generations, fault rates,
CPU/RSS and cold N65536 product comparison against
retained exact, with #86 and #69 unresolved.
