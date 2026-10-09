# DeltaGuard B1-B1B1-B1-A — persisted receipt-chain ABA and partial-ACK crash evidence

Parent [#105](https://github.com/definitely-stable/deltameter/issues/105)
and #103/#101/#99/#97/#92, implementation
[PR #106](https://github.com/definitely-stable/deltameter/pull/106).
Frozen prior to measurement: [B1A protocol](DELTAGUARD-B1B1B1-B1A-PROTOCOL.md)
at commit `934ff11ea393f16487619001c7911ab8d50e1339`.
Additional premeasurement clarification at `a39da0b677e7aabf256b9bec5136e9ff45bac678`:
accepted receiver receipt stores **exact event bytes** in addition
to the owner chain digest, needed for fail-closed ABA replay.

## Results, source and verdict

**B1B1B1_B1A_ABA_RECEIPT_CHAIN_RESEARCH_ACCEPT_NO_PRODUCT_GO**.

First complete run at source `996b86f3c4d7e66a558d6801911353153514973d`:
[GitHub CI #37942522822](https://github.com/definitely-stable/deltameter/actions/runs/37942522822)
5 GitHub-hosted workers, 3000/3000 source-bound two-owner
TCP event observations, B2A original cutoff proof regenerated
and fail-closed independent aggregator **SUCCESS**. Source at
`9ad9411081525f075ceb7f59a57a47c86c68cfe7`:
[GitHub CI #37942833576](https://github.com/definitely-stable/deltameter/actions/runs/37942833576)
the same 3000/3000 **SUCCESS after the receiver restart was
strengthened to a genuinely NEW OS receiver process**.
All workers use 2 independent owner subprocesses and private
fsync'd WAL/commit/ACK files, 100 generations each
(gen2..101), 3 input fixture repetitions, N256 and N65536,
d48, five independent hosted machines.

For each owner, exact canonical source state is loaded
from its private disk base snapshot + **all** hash-chained
88B events. Every generation alternates INSERT/DELETE of
the same shared sentinel, i.e. **50 inserts and 50 deletes**
per owner per run, with genuine ABA history. WAL event
integrity binds prior 32B hash, 16B recorded parent
prefix, owner ID, epoch, generation, operation, token,
B1 key identity/profile and domain label. Source WAL is
File::sync_all committed, then atomically published
chain commit marker and parent-directory sync. Source
ACK watermark sync occurs AFTER physical ACK.

Receiver separately loads its existing on-disk canonical
two-owner inventory, owner generations, two chain heads
and the exact **88B last accepted event per owner**.
Both physical two-owner B1-A DELTA frames are validated,
and the receiver atomically writes both updated inventories,
chains and event identities to the same synced file
BEFORE sending either ACK. An identical same-sequence
replay must match the persisted last-event bytes
**not merely show that the token exists**. Changed same
generation record, digest or token is rejected before
mutation. Gap/rollback or owner/epoch/frame violations
are fail closed.

At generation51, a separate receiver OS child physically
receives two owner frames, writes its durable two-source
receipt, sends ACK to owner1 ONLY, and is **truly SIGKILL'ed**
before ACK to owner2. The owner with missing ACK retains
its older durable cursor. Both fresh owner sender child
processes then reconnect to a **new independently spawned
receiver OS process** which reloads receipt from disk;
bit-identical duplicate events are acknowledged without
reapplication. These are **30 real OS receiver kills**
across the five hosted workers, three fixtures and two
N lanes. All 3000 generations were subsequently checked
against independently reconstructed exact source oracle
and *disk-reopened* owner/receiver states.

Nine negative categories are exercised per fixture:
changed same-generation body; changed digest;
stale/gap sequence; wrong owner/frame; remove-nonmember;
committed WAL truncation; owner ACK watermark ahead;
receiver receipt corruption; exact-state rollback checks.
These are checksummed public fixtures, not adversarial
Byzantine signature authentication.

## Actual application TCP bytes

The physical two-owner per-generation transfer consists
of 1B hello + 88B WAL provenance + 64B B1-A DELTA
frame + 9B event + 16B ACK **per owner**:
**356B** per normal two-source accepted generation.
At the ACK#1 crash cut, the first session transmits
**340B** (only one of two ACKs), then genuine receiver
and sender process reconnection retransmits **356B**.
Each lane N has 1500 generations and 15 such faults,
so first-attempt physical event bytes **533,760B**,
retransmissions **5,340B**. Across both N lanes,
total 1,067,520B event bytes + 10,680B retry bytes.
This is RESEARCH-ONLY locality/transport cost,
not a fair product bandwidth or latency claim.

## Major engineering cost discovered

This foundation currently persists **both complete
exact receiver source inventories on EVERY generation**
to achieve a straightforward atomic two-source fsync
boundary. Exact encoded durable receipt lengths are
derived directly from the frozen file layout, not
kernel physical block counters:

- N256: each pair-generation receiver receipt
  4,536B when the shared sentinel is absent,
  4,552B when present. The 100-generation
  sequence writes **454,400 logical bytes** per
  fixture/worker; all 15 runs of N256:
  **6,816,000B** receiver receipt writes.
- N65536: receipt 1,049,016/1,049,032B per
  generation. The 100-generation sequence writes
  **104,902,400 logical bytes** per fixture/worker;
  all 15 runs of N65536: **1,573,536,000B**
  logical full receiver receipt writes.
- Source fixed WAL is **8,800B per owner for 100
  generations**, PLUS separate 56B commit and
  56B ACK marker rewrites per generation
  and initial source snapshot/metadata. Fsync
  calls and directory sync occur, but actual
  device block/IO accounting, latency and
  page write amplification are not measured.

**STOP_FULL_RECEIVER_RECEIPT_PER_EVENT_AS_PRODUCT_DESIGN**:
do not extrapolate a 356B TCP message into efficient
durable distributed reconciliation without replacing
the O(N) per-generation receiver snapshot fsync.
This is a concrete design problem, not a weakness
resolved by optimizing the parity decoder.

## Strict scope/remaining work

SIGKILL and sync_all on GitHub Linux is NOT an
unplugged power cut or formal crash consistency
over a partial fsync. BLAKE3 chain is unkeyed/public
and is NOT peer authentication. Source first snapshot
is seeded from public fixture BEFORE the crash;
subsequent owner restart and sender event loading
are disk based, not silently regenerated fixture
sources. The 100-generation stream uses one
shared token toggled insert/delete; no generic
mixed sorted batches, compaction, concurrent writers,
source conflict resolution, delete-reinsert of
multiple keys, different session epochs or WAL
growth beyond 100 events. Hash-chain receipt
uses exact last-record identity for replay, but
does not prove adversarial authorization.
No WAN network, paired fair *equally durable
exact vs guard* p95, link fault-rate break-even,
or fully resolved near-T product GO follows.

D60 **WARM_N256_GENERAL_B2A_NO_GO** and
**STOP_RESOLVED_NEAR_T** remain. Next #105 B1-B1B1-B1B
requires bounded receiver WAL/atomic multi-owner
checkpoint (without full O(N) receipt per event),
source WAL compaction/rollover, multi-token deletes,
source/sink crash at each commit boundary,
actual disk device bytes/IO/CPU/RSS, and fair
cold sparse-N65536 system cost comparison. #86
adaptive/key security and #69 public lifecycle
remain independent gates.
