# DeltaGuard B1-B1B1-B1B0 — incremental atomic receiver WAL evidence

Parent [issue #107](https://github.com/definitely-stable/deltameter/issues/107),
research [PR #108](https://github.com/definitely-stable/deltameter/pull/108).
[Frozen protocol](DELTAGUARD-B1B1B1-B1B0-PROTOCOL.md) was
committed **before** the first B1-B1B1-B1B0 measurement:
`48aafbbf15574430dfb6b36e16c90e3383fae4ba`.
No public API, production path or Snapshot v1 change.

## Source-bound outcome

**B1B1B1_B1B0_INCREMENTAL_RECEIVER_RESEARCH_ACCEPT_NO_PRODUCT_GO**.

First complete 5-hosted-worker
[CI #37985064187](https://github.com/definitely-stable/deltameter/actions/runs/37985064187)
at exact source `e3b92c9e606d219d465aa6199aa513ce698fc644`:
- Frozen 24-profile B2A exact cutoff regenerated, SHA256
  `c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`
  unchanged.
- All **5 independent GitHub-hosted workers** complete.
  3 fixtures x 2 N lanes x 100 successive source generations
  x 5 workers = **3000/3000 source-SHA-bound records**; independent
  Python aggregator checks exact counts, generation continuity,
  physical TCP frame byte counts, app WAL length, checkpoints,
  actual invoked sync-call counts and per-generation expected writes.
- Each owner retains its private fsync'd B1-B1B1-B1-A source WAL,
  chained hash, synced commit and post-ACK watermark. Each
  source generation alternates inserting and deleting one
  shared sentinel (50 INSERT + 50 DELETE per source).
  Exact canonical receiver state and both chain heads are
  compared to **two disk-reopened source inventories** and
  independent external oracle after each generation.
- **30 real receiver process SIGKILL** after physical ACK
  to owner1 but before ACK to owner2, all at generation51.
  A NEW receiver OS process reopens checkpoint/WAL from disk
  and physically receives/replies to both retransmitted
  identical owner events without double application.
  The accepted last 88B canonical record survives checkpoint.
- Negative corruption tests per fixture fail closed:
  checkpoint checksum corruption, committed marker mutation
  and rollback, uncommitted torn WAL tail (ignored, never
  accepted), owner source WAL changed inside an accepted
  event chain, and changed same-generation ABA receipt.
  Durable exact inventories unchanged on rejection.

## Receiver persistence change

Old [D63](DELTAGUARD-B1B1B1-B1A-EVIDENCE.md) persisted
the complete two-owner exact inventories **on every
generation**, O(N) logical file writes per change.

This slice instead writes exactly:
- **256B** complete two-owner receiver WAL transaction
  binding owner1/owner2 exact 88B event records, common
  generation, 32B prior transaction head and 32B
  transaction digest; physically appended, `File::sync_all`.
- **48B** receiver committed-generation marker with
  32B digest, atomically published via temp-file sync,
  rename and parent-directory fsync.
- At exact generations **51 and 101**, an atomic
  *full* two-owner receipt checkpoint, followed by
  atomic replacement of receiver WAL with empty.
  The loader confirms a surviving old WAL prefix is
  already covered by the durable checkpoint hash
  (positive recovery check before WAL replacement).
- Before either owner ACK, the receiver reloads the
  complete accepted prefix from checkpoint + WAL,
  verifies both source identities, chain predecessor,
  exact accepted last-record bytes and canonical
  insert/remove semantics. A half-pair cannot be
  ACKed as successful.

Per accepted non-checkpoint generation the code
performs **3 actual sync calls** (one WAL file sync,
commit marker file sync and directory sync) and writes
304 logical file payload bytes. At checkpoint
generations 51/101, **7 sync calls** and a second
full-source snapshot, after which current active
receiver WAL length is exactly 0. Max active WAL is
**49 * 256 = 12,544B** in this workload.

## Measured/verified application and logical disk bytes

Both lanes use source d=48, 100 event generations
per fixture per worker, 15 fixture/worker sessions
per lane.

| Metric | N256 | N65536 |
|---|---:|---:|
| Old D63 receiver logical bytes / 100-event run | 454,400B | 104,902,400B |
| New receiver logical bytes / 100-event run | **39,520B** | **2,128,480B** |
| New receiver logical bytes / all 15 runs | 592,800B | 31,927,200B |
| New two checkpoints / all 15 runs | 136,800B | 31,471,200B |
| Active receiver WAL maximum | 12,544B | 12,544B |
| Receiver sync calls / 100-event run | 308 | 308 |
| Two-owner physical first-attempt event TCP bytes / 15 runs | 533,760B | 533,760B |
| Physical identical retry TCP bytes / 15 runs | 5,340B | 5,340B |

Initial cold receiver source bootstrap/checkpoint is
EXCLUDED from the per-100-generation new vs old
comparison, exactly as in the old D63 evidence.
The new initial full checkpoint costs **4,560B**
for N256 or **1,049,040B** for N65536 and must be
charged for any complete lifetime comparison.

Old-to-new logical receipt write reduction is
approximately **11.5x** for N256 and **49.3x** for
N65536. These are *serialized file payload bytes*,
NOT actual SSD blocks/drive writes, device IOPS,
energy or clock latency. Source initial ingest,
source WAL+commit+ACK fsync, real device-sector
amplification and per-worker p95 are NOT included
in this receiver-write comparison.

## Critical audit / remaining correctness and product gaps

- **CONDITIONAL research ACCEPT** for the frozen
  *single-token alternating 100-generation* workload,
  not a general journal implementation. Uncommitted
  torn tails are ignored, but an actual physical
  SIGKILL/power-cut inserted at every intermediate
  WAL fsync, marker rename or checkpoint rename
  boundary has NOT been exhaustively tested. One
  checkpoint-after-write-before-WAL-rollover case is
  checked by the loader while the old WAL still
  exists; this is not a full power-failure proof.
- Checkpoint + WAL reconstruction currently reads
  the exact canonical two-owner checkpoint and
  replays WAL for **every receiver query** in the
  bounded harness. At large N this may impose
  substantial reads, allocations and hash/IO cost.
  O(1) received event *write size* does not imply
  O(1) service latency, memory or storage read load.
- The new receiver persistence layer is usable by
  BOTH DeltaGuard and **equally durable maintained
  exact**. This does NOT establish a DeltaGuard
  network, latency or economic win. For N256 frequent
  changes, D60 previously found warm maintained
  exact **strictly preferable** over guard in both
  app network bytes and observed n20 p95. The
  d57 near-T resolved guard STOP also remains.
- Public BLAKE3 integrity hashes are **NOT** peer
  authentication; adaptive repeated-query failure,
  colluding/malicious sources, epoch/key rotation,
  concurrent writers, many-key batches, checkpoint
  disk corruption during fsync, source WAL compaction
  and interrupted writes remain unproven.
  SIGKILL on hosted Linux is NOT a power-cut.
- No real shared-WAN link, true packet loss, source
  event ingestion CPU, per-process CPU ticks/RSS,
  file-system block I/O and latency p95 versus
  matched durable exact comparator were evaluated.

**Decision:** accept a bounded O(event) incremental
receiver WAL and 50-generation checkpoint research
foundation; remove D63's `STOP_FULL_RECEIVER_RECEIPT_
PER_EVENT_AS_PRODUCT_DESIGN` **for this candidate
implementation only**. Continue [#107](https://github.com/definitely-stable/deltameter/issues/107)
with new separately preregistered physical fsync/
crash/compaction boundaries and source+receiver
disk/CPU/RSS + same-lifecycle matched durable exact
comparator. Keep #105/#103/#101/#99/#97/#92 and
independent security #86 and public API #69 open.
**NO SYSTEM_PRODUCT_GO**.
