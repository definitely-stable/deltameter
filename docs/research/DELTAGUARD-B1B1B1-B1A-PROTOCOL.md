# DeltaGuard B1-B1B1-B1-A — precommitted multigeneration ABA/receipt-chain foundation

Parent [#105](https://github.com/definitely-stable/deltameter/issues/105), downstream #103/#101/#99.
Frozen **before** B1-B1B1-B1-A execution or observation. This is one deliberately
bounded correctness slice, **NOT** the complete #105 physical crash/reconnect,
WAL compaction, durability, performance or security/product acceptance.

## New research-only state contract

Two **different** source-owner OS processes each own their private durable
`base.snap`, append-only `chain.wal`, fsync'd `commit.mark` and eventual
`ack.mark`. Both source processes must derive canonical current exact tokens
and committed SHA-256-sized (BLAKE3-256, unkeyed) hash-chain heads by reopening
their OWN DISK files on every restart. Fixture values are allowed at initial
snapshot ingestion and as an independent post-transport oracle ONLY, never
to synthesize missing source events on restart.

A WAL event is fixed-width 88B:
8B magic, 8B epoch, 8B generation, 1B owner, 1B operation,
6B canonical zero padding, 8B token, 16B previous digest,
32B chained digest. Hash domain binds previous digest, owner,
B1 epoch/key ID/profile, generation, operation, exact token
and header. Genesis digest is independently bound to the full
canonical initial B1_FULL snapshot and owner identity. The
digest is a PUBLIC integrity commitment, NOT authentication.
Each complete event is appended+File::sync_all and followed
by a `commit.mark` atomically published with fsync and directory
fsync containing latest accepted generation and chained digest.
No valid commit mark means all tail events beyond the last
committed sequence are unaccepted. Reopen verifies EVERY
predecessor digest, exact monotone generation and canonical
membership in sequence; missing, repeated, reordered or
changed records MUST NOT be silently accepted.
Bounded max WAL bytes and count; no compaction in this slice.

The receiver durably stores both source exact inventories,
their generation and **respective last accepted chain digest**
inside ONE synced+atomically renamed two-owner receipt.
On each physical two-owner TCP event request, independently
verify owner/epoch/key/profile/frame checksum, monotone
predecessor sequence, previous digest AND next digest before
any mutation. Validate BOTH source changes, commit BOTH
receiver lists and digests atomically with file+directory
fsync BEFORE any source ACK, then send ACK.
A duplicate **same generation** is idempotent ONLY if
its persisted digest matches the exact received event hash;
different payload/digest at same generation is a fatal
conflict, EVEN IF its token is present. An older or skipped
generation is not idempotent. ACK cursor is advisory,
never a proof of receiver durable state. Every source
must sync ACK marker after receipt of physical ACK.

## Frozen workload / negative controls

Five independent GitHub-hosted workers × N=256 and N=65536,
d=48, 3 deterministic fixture repeats × **100 successive
generations (2..101) per source** = **3000 paired-source
generation observations** (100 per lane/repeat/worker).
Both sources alternate INSERT/DELETE of the SAME distinct
per-owner token on every adjacent generation (ABA true
insert→delete→insert→delete). 50 inserts, 50 deletes per
owner. The receiver is independently checked against
actual two-owner disk states and fixed external exact oracle
at EVERY generation. All generations commit via physical
localhost TCP source events, bounded header and actual ACK.
Samples include all source WAL fsync and atomic receipt
sync operations, but this is NOT a p95 comparison with
equally durable retained exact and not a WAN model.

At generation51, one receiver subprocess must be killed
after the durable two-owner atomic receipt and *first*
owner ACK, but before second ACK. Both owners reconnect,
reload private WAL/ACK state, retransmit the identical
generation; restarted receiver reads only its synced
receipt and recognizes the exact digest replay without
second mutation. Mark lost ACK retry separately from
normal generation observations; no invented free
state. Preserve actual owner TCP framing, byte counts,
and per-child end VmHWM/OS CPU ticks when obtainable.

Mandatory fail-closed negative vectors after generation
101: (1) conflicting same-generation insert token while
already present; (2) changed digest despite identical
payload; (3) missing WAL record / predecessor splice;
(4) stale/repeated or rolled-back source generation;
(5) wrong owner/epoch/key/profile; (6) truncated committed
WAL; (7) invalid remove-nonmember; (8) ACK watermark
ahead of committed generation; (9) receiver receipt
corruption. Every negative MUST retain last valid durable
receiver state and fail closed, not silently resume.

## Release gate

- Exact 3000 source SHA bound queries and 30 physical
  mid-two-ACK receiver process kills, no missing or
  duplicate observations in independent 5-worker
  aggregator, 24-profile original B2A cutoff certificate
  unchanged.
- Entire receiver chain is persisted and reopened, not
  just seen in live vectors. Compare source+receiver
  exact lists to independent oracle after 100 insert
  and delete steps, including ABA history and negatives.
- All WAL fsync operations executed and credited, plus
  receiver receipt file/dir sync; report actual disk bytes,
  physical TCP bytes, source/receiver process RSS and CPU
  counter semantics without inventing a steady-state win.
- No Snapshot v1/public API edits. D60 warm N256 general
  DeltaGuard NO-GO and near-T resolved STOP remain.

NEXT separately frozen B1-B1B1-B1-B will need truly
uncontrolled process kill at every WAL/commit/fsync
boundary, autonomous event source conflicts, multi-owner
receiver crash between ACKs under varied sequences,
compaction/checkpoint, full error-rate/disk-write and
same-lifecycle p95 vs equally durable exact/direct full,
and security #86 and public API lifecycle #69.
