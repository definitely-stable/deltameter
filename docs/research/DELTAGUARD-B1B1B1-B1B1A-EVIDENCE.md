# DeltaGuard B1-B1B1-B1B1-A — WAL crash-cut and read amplification evidence

Parent [#109](https://github.com/definitely-stable/deltameter/issues/109),
research [PR #110](https://github.com/definitely-stable/deltameter/pull/110).
Frozen hypothesis, five cutpoints and measured lanes were recorded
BEFORE any fault measurements in
[DELTAGUARD-B1B1B1-B1B1A-PROTOCOL.md](DELTAGUARD-B1B1B1-B1B1A-PROTOCOL.md),
initial commit `52679dc335929a12f2d33e169a745672421d9315`.
A **49-character SHA typo** in the 64-hex literal for the *unchanged*
B2A certificate blocked the initial proof job **before running
any fault samples**. It was corrected before sample collection
at protocol update `0ad0e01eabbfcb8f703f14246c631b881db10262`
and code/workflow updates `a93f084...`/`11af7ffa...`.
The unchanged original B2A certificate SHA256 is
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.

## Result and reproducible source

**B1B1B1_B1B1A_CRASHCUT_TAIL_REPAIR_RESEARCH_ACCEPT_NO_PRODUCT_GO**.

First complete hosted [GitHub CI #38022477456](https://github.com/definitely-stable/deltameter/actions/runs/38022477456)
on source `11af7ffa2ececea4af043a3e188a6a64035bbf69`:
all 5 GitHub-hosted workers, regenerated exact 24-profile
B2A cutoff and fail-closed independent SHA-bound aggregator
**SUCCESS**. Exactly **150/150 real receiver subprocess
SIGKILL** (5 workers ×2 N lanes×3 source fixtures×5
named crash-cut cases). Both source-owner processes load
their individually fsync-committed source event journals
and physically retransmit a valid two-owner B1-A framed
delta to a **new independent receiver OS subprocess**.
The receiver persists a durable two-owner transaction
BEFORE source ACK, reopens disk and matches both source
exact inventories and independent oracle. No unauthorized
partial receiver mutation or false ACK.

**90/90 canonical repairs** were required and completed:
- `partial_wal`: 80B WAL torn suffix after real write
  but before fsync/commit marker; SIGKILL; discard 80B.
- `full_wal_fsync`: full 256B WAL written and
  actually File::sync_all'd but without committed marker;
  SIGKILL; discard 256B.
- `commit_synced`: full WAL and atomically
  sync-published marker before kill, but NO ACK;
  persisted exact generation accepted, identical
  TCP replay ACKed without applying or rewriting.
- `commit_conflict`: same committed generation
  after SIGKILL; changed event payload rejected,
  then valid exact replay ACKed without reapplication.
  This is **not** malicious sender authentication.
- `checkpoint_synced`: after a physically maintained
  49-generation two-owner source+receiver history,
  generation51 receiver transaction, marker, full
  checkpoint written+fsync+rename+directory fsync,
  then SIGKILL **before** WAL rotation. Loader verifies
  old log's terminal checkpoint transaction digest,
  canonicalizes WAL to empty and recognizes the
  duplicate source event without reapplying it.

Per N, 75 actual cuts, 45 repairs, 300 read samples,
**26,700B physical source replay TCP**. The restart
requires a truly new receiver process, not a continuation
of the test coordinator. Missing/torn *uncommitted*
bytes are removed only AFTER a strict already
accepted checkpoint/WAL+marker reconstruct.
Mutated COMMITTED marker/transaction/digest is
still fatal, never silently discarded. Checkpoint
receipts preserve the exact 88B per-source
accepted events to prevent same-sequence ABA
confusion. No healthy canonical WAL gets rewritten.

The repair logic is integrated into the shared
research B1B1B1-B1B0 receiver `bf_accept`,
so it runs before either ACK and before any
subsequent WAL append, not just in a separate
fault-test function. Repair uses fsync of new
canonical WAL file plus its parent directory;
a source ACK occurs only after the canonical
prefix is verified. A checkpoint-in-progress
may temporarily have 50×256=**12,800B** WAL
before rotation; normal maximum active WAL
remains 49×256=**12,544B**, and checkpoint
recovery reduces it to exactly zero.

## Quantified read-path issue

600 frozen paired read-path observations, 20
repetitions each per worker, N lane and fixture:
`bf_load` reopens complete canonical two-source
checkpoint, 48B marker and WAL, validates/rebuilds
exact arrays; the other observation simply reads
an *already maintained* exact in-memory length.
They are NOT equal-work algorithms; the latter
is an intentionally trivial lower reference for
the rebuild cost, not a fair speedup ratio.

At checkpoint generation51 with empty active WAL,
`bf_load` reads the following logical file payload
on each nonmutating access:

| Source N | Checkpoint+marker read bytes / access | Hosted empirical n20 p95 across 15 worker×fixture runs |
|---|---:|---:|
| 256 | 4,608B | 0.023–0.035ms |
| 65,536 | 1,049,088B | 0.310–0.951ms |

**These p95 values are per run's nearest rank 19/20**;
the range is across independent hosted workers/fixture
groups. Storage is Linux runner-local with warmed
filesystem page cache, CPU and cache variability.
The 1,049,088B is logical file payload read by
each reconstruction, NOT measured SSD device
read bytes. No latency conclusion about shared
WAN, fully resolved guard or maintained-exact
product service follows from this audit.

Thus the old D63 O(N)-WRITE issue is addressed
by D64 on the frozen workload, but the B1B1B1-B1B0
**O(N) PER-QUERY READ/rehydration** still needs
removal in any long-lived hot receiver: cache the
materialized canonical state in memory, fsync
atomic two-owner event and commit marker, reload
only upon genuine receiver restart, then check
against an equally durable exact backend.
The same optimization is available to maintained
exact and DeltaGuard; it does **not** establish
a novel sketch advantage.

## Critical scope and next gate

Only these five preregistered cuts were tested:
there is **no** device power-cut/torn-fsync model,
adversarial authentication, multi-writer lock,
concurrent updates, source partial TCP body,
epoch rotation, multi-key batch or source/receiver
WAN jitter proof. Process SIGKILL does not
demonstrate physical power-loss persistence.
Guarded B2A algebra/probability remains frozen.
D60 **warm-N256 guard NO-GO** and near-T
resolved STOP remain. No public API/Snapshot
v1 changes; #86 security and #69 lifecycle
remain independent blockers.

**Next B1B1B1-B1B1-B** (still under #109) must
preregister and actually compare a correctly
maintained hot in-memory exact receiver with the
SAME atomic receiver journal as guard-only
SAFE/UNKNOWN and fully resolved exact fallback,
same two-source process lifecycle, source WAL
fsync/ingest, 1/10/100 Mbps and 0/10/50ms
application-emulated lanes, Q1/Q10/Q100 and
at least 20 paired samples/worker. Measure
true owner/receiver CPU, RSS, disk fsync wall,
logical+physical I/O and p95. Do NOT claim
SYSTEM_PRODUCT_GO from this crash-cut slice.
