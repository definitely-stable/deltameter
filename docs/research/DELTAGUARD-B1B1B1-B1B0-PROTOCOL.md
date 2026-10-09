# DeltaGuard B1-B1B1-B1B0 — premeasurement bounded atomic incremental receiver WAL

Parent [#107](https://github.com/definitely-stable/deltameter/issues/107), #105/#103/#101/#99.
This protocol is committed **BEFORE any B1-B1B1-B1B0 measurements**.
D60 warm N256 guard NO-GO, D63 full-receipt-write STOP, and ideal
nonadaptive B2A c48 proof remain unchanged. **Research only**; no
Snapshot v1, production API, hostile-peer authentication or product GO.

## One narrow correctness/cost foundation

Replace the B1-B1B1-B1A receiver's full two-source inventory rewrite
*per event* with:
1. One fsync'd canonical two-source CHECKPOINT (both sorted source
   lists, generations, two 32B chain heads, exact 88B last accepted
   record per owner, 32B transaction-chain digest, checksum).
2. A fixed **256B receiver transaction**: 8B domain/magic,
   8B common generation, 2 * 88B full source event records,
   32B previous receiver transaction digest and 32B digest of
   the entire header+predecessor bound to key/profile/domain.
   Exactly ONE transaction updates BOTH owners atomically; a
   source never gets a positive ACK for a half-applied pair.
3. Append complete 256B txn to receiver.wal and File::sync_all,
   THEN atomically publish a 48B commit watermark
   (generation+32B digest) via temporary file sync + rename +
   parent-directory sync. Only the prefix named by a valid
   watermark is accepted; trailing uncommitted/torn bytes are
   ignored, NOT physically applied. The generation-bound
   watermark must match the actual hash-chained WAL prefix.
4. Receiver loader reads checkpoint + verified committed WAL;
   validates both source events through existing
   `be_validate_transition`, including exactly stored previous
   event bytes, owner/epoch/key/profile, predecessor chain,
   canonical membership, and then verifies receiver transaction
   digest. Mismatched committed record, digest, missing sequence,
   truncation or unexpected rollback rejects completely.
5. At exact generations **51** and **101**, after fsync'd commit,
   publish a NEW full two-owner checkpoint via temp+fsync+rename+
   parent-directory sync. Then atomically replace WAL with empty
   file+fsync+rename+directory fsync. Crash between checkpoint
   publish and WAL replacement must recover from checkpoint
   and recognize the older WAL prefix as already included.
   No concurrent writes during compaction; file names stable.
   The last exact accepted source event identity survives
   checkpoint and is used to reject conflicting ABA retransmit.
6. Only after all persistence above, send ACK to owner1, then
   owner2. If receiver child is SIGKILL'ed after ACK1 before
   ACK2 at generation 51, a new OS receiver process must reopen
   disk checkpoint/WAL, accept both **bit-identical** repeated
   events without applying twice, and ACK. No in-memory
   borrowed receiver state, no fixture-derived reset on restart.

## Frozen workload, fault coverage, accounting

Five separately hosted GitHub workers × two N lanes
N256,d48 and N65536,d48 × 3 fixed fixtures ×
100 generations (2..101) = **3,000 complete paired-source
physical TCP observations** and 30 post-ACK1 SIGKILL events.
Source writers/senders remain independent subprocesses with
private fsync'd B1B1B1-A hash-chain event WALs.
Each source alternates one INSERT, one DELETE of the same
sentinel (ABA); independently verify exact receiver list and
chain against both disk-reopened sources and exact external
oracle **after each generation**. Re-use original 24-profile
B2A exact certificate anchored at
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.
Physical pair-event bytes 356B, at receiver ACK1 fault 340B +
356B retry. No WAN/p95 claim from these correctness probes.

Every worker must reject these **real disk mutations** with
receiver unchanged: mutated checkpoint payload/digest,
wrong receiver watermark generation/hash, truncated
COMMITTED WAL, spliced committed event/body, altered
source last-event identity on duplicate, stale and
skipped sequences, invalid delete, and torn
UNCOMMITTED WAL tail. The final case is accepted only
as a noncommitted tail, never changes exact state.
Fail-closed Python evidence checker verifies all 3000
records, 5 workers, 30 real receiver SIGKILL, both
two-owner checkpoints, fixed byte counts and SHA.

Account **logical file payload writes and number of actual
File::sync_all/directory sync calls** separately for
receiver WAL append, 48B marker publication,
checkpoint and WAL compaction; initial ingestion/bootstrap
charged separately. File serialization bytes are NOT
physical disk-sector write amplification, CPU time or
network throughput. Source WAL/ACK journal cost is
identical in baseline and proposed receiver variant
and must NOT disappear from subsequent comparisons.

No generic comparator or PRODUCT_GO from this slice:
the **equally durable maintained exact** already provides
the same inserted/deleted event streams; this receiver
WAL is a common storage fix, NOT a new DeltaGuard
network advantage. Subsequent B1-B1B1-B1B1 must compare
guard-only (SAFE/UNKNOWN), resolved fallback and both
durable exact/full with identical active source process,
cold bootstrap, 20+ same-lifecycle paired samples/worker,
source+receiver disk, CPU/RSS and predeclared 1/10/100Mbps
and app-delays 0/10/50ms. Security #86 and API #69 remain
independent blockers.

## Exit criterion

Only `B1B1B1_B1B0_INCREMENTAL_RECEIVER_RESEARCH_ACCEPT`
if all source-bound records and fault classes PASS and
O(256B) steady per-generation receiver WAL+marker writes
hold; checkpoint O(N) permitted only **once per 50
generations**, with bounded WAL size. Must explicitly
flag any full snapshot per query, out-of-range generation,
power-cut inference or lost-record false ACK as NO-GO.
