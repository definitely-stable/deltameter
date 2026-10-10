# DeltaGuard B1-B1B1-B1B1-A — frozen WAL crash-cut, tail repair and read-path audit

Parent [#109](https://github.com/definitely-stable/deltameter/issues/109).
Preregistered **before new runs**. Research only; Snapshot v1/public
API frozen. D60 warm N256 and near-T resolved NO-GO remain binding.

## Falsifiable bug hypothesis

The B1B1B1-B1B0 receiver loader deliberately ignores WAL bytes after
a valid committed watermark, but `bf_commit` currently appends directly
without removing them. If an OS receiver SIGKILL leaves a partial or
complete **uncommitted** WAL event and the replacement process accepts
new source data, a later committed watermark may point past the tail
and cause a committed-prefix decode error or false ACK. A stale WAL
containing the checkpointed prefix can likewise survive a crash after
checkpoint atomic rename but before empty WAL atomic rename. These
are *continuation/recovery bugs* independent of DeltaGuard math.

## Recovery invariant and exact permitted repair

Before ANY new WAL append or positive ACK, under one exclusive
research-owner writer (NO concurrent writers), load and independently
verify the committed two-owner checkpoint and committed generation/
digest marker, then find the exact committed WAL span. Any complete
stale checkpoint-covered WAL prefix must be verified as terminating at
the checkpoint generation/digest; incomplete old-prefix metadata is
fatal. Any suffix after the committed marker is uncommitted, whether
complete or torn. Replace WAL with EXACT committed suffix through
temp-file write+File::sync_all+atomic rename+directory fsync. Ensure
reload gives **identical exact vectors, two source digests, last
accepted 88B event identities and committed receiver transaction
digest** before proceeding. Under this scope an already canonical
WAL needs no rewrite or extra fsync. No in-memory source fixture
regeneration on restart. A damaged COMMITTED prefix / marker must
fail closed; NEVER silently discard it.

SIGKILL after appending an uncommitted event never makes that event
accepted. A source with separately durable WAL may retransmit its
precise original event, and receiver may accept/recommit it
exactly once after canonical tail repair. If a fully synced receiver
commit was published before any ACK, a new OS receiver must recognize
the duplicate and ACK without reapplying or writing another txn.

## Freeze-before-measurement fault matrix

5 independently hosted GitHub workers x N={256,65536}, d48
x 3 fixed source fixtures x 4 generation-2 crash cuts PLUS
one generation-51 checkpoint crash cut = **150 actual SIGKILL**
process cases, 30 cases/worker:

1. `partial_wal`: a NEW receiver OS child writes the first
   80 bytes of a 256B two-source txn into receiver.wal and
   is SIGKILL'ed BEFORE fsync/marker. Rebooted receiver
   rejects the uncommitted tail and physically resends
   both source events. WAL repaired exactly once.
2. `full_wal_fsync`: append entire 256B transaction,
   File::sync_all, SIGKILL BEFORE marker. Same required
   result; no later append behind the uncommitted frame.
3. `commit_synced`: append/fsync txn, atomically sync
   committed 48B digest watermark; SIGKILL BEFORE ACK.
   A new receiver loads gen2 and ACKs identical
   physically resent events without mutation or new WAL.
4. `commit_conflict`: same durable accepted txn as 3,
   attempt changed same-generation event replay at
   receiver and fail closed, then genuine bit-identical
   replay succeeds. Do not count tamper as authenticated
   attack prevention; public BLAKE3 fixture integrity only.
5. `checkpoint_synced`: first physically commit 49
   generations (to gen50) using two independent persisted
   source owners. At generation51 receiver fsyncs a complete
   pair txn+marker and publishes complete gen51 checkpoint
   via temp+sync+rename+parent sync, then is SIGKILL'ed
   BEFORE WAL reset. New receiver checks saved checkpoint
   vs old WAL last generation/digest, repairs stale WAL
   atomically without changing exact state, and physically
   ACKs the *same* gen51 source pair. Anti-ABA last accepted
   exact event receipt must survive the checkpoint.

These fault windows use actual OS process boundaries,
actual File::sync_all and directory sync, and physical
localhost TCP replay from two independently persisted
source WALs. A readiness line must be printed AFTER the
last intended file-sync boundary; only then the controller
sends SIGKILL. The receiver restart is a new OS process.
Do NOT call SIGKILL a power-loss or partial-fsync model.

For normal crash-cut gen2, source gen1 is built by
independent owner source OS subprocesses, then generation2
source events are fsync committed by distinct source
writer subprocesses. For checkpoint gen51, writer and
physical receiver commit step-by-step to generation50,
with every source exact state validated; cut event
and post-cut retry at gen51. No free receiver source state.

## Read amplification audit (separate from correctness)

After recovery, for each worker/lane/fixture run **20
paired** nonmutating source-state reads:
(A) `bf_load`: opens checkpoint, receiver commit and WAL,
rebuilds exact vector and validates chain; (B) already
materialized hot exact vectors/head in memory, with
same externally validated source state. Record full
rebuild vs hot path elapsed ns per pair and size of
logical checkpoint+marker+WAL files read, but do NOT
gate ratios or claim cold storage vs OS page-cache
difference. An exact maintained service could keep
the hot state; simply avoiding repeated BF disk
rehydration is not unique to sketches.

Output all 150 named cut cases and 600 (5 x 2 x 3 x 20)
paired read observations. Strict independent SHA-pinned
aggregator must validate 150 unique crash classes,
each outcome, source/receiver exact oracle, no source
gen changes without valid event, WAL repair bytes and
positive read-path sample counts. 24-profile B2A
proof SHA256 fixed at
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59d9e935`.

## Exit

Only **B1B1B1_B1B1A_CRASHCUT_TAIL_REPAIR_RESEARCH_ACCEPT**
if 150/150 actual SIGKILL cut experiments and strict
read-path evidence PASS in 5 hosted workers, and repaired
WAL remains bounded (<=12,544B before checkpoint;
0 immediately after completed repair of checkpoint).
Do not close #109. The separately preregistered
system-comparison B1-B1B1-B1B1-B still must evaluate
equally durable maintained exact against guard-only
SAFE/UNKNOWN and fully resolved fallback in identical
hot source/query lifetimes and count physical network,
CPU, RSS and fsync/disk latency at Q=1/10/100.
No SYSTEM_PRODUCT_GO, no public API change and
#86 adaptive trust/#69 public lifecycle remain
independent blockers.
