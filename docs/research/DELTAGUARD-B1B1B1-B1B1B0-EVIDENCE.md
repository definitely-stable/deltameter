# DeltaGuard B1-B1B1-B1B1-B0 — source-WAL-backed hot state contracts and fair byte accounting

Parent [#109](https://github.com/definitely-stable/deltameter/issues/109),
[PR #111](https://github.com/definitely-stable/deltameter/pull/111).
The [protocol](DELTAGUARD-B1B1B1-B1B1B0-PROTOCOL.md)
was frozen BEFORE measurements at commit
`72636c019f20de89e3fb3de6a49d64a156b58c50`.
Two pre-completion CI iterations revealed (1) a
Rust Clippy iterator warning and (2) a byte-accounting
mistake for odd d=57: second source has N+1 elements,
not N. The frozen lane/mode/decision gates were NOT
changed. Exact source-size erratum is explicitly
recorded in the updated protocol, and corrected
before first complete five-worker observation.

## Result and source

**B1B1B1_B1B1B0_HOT_CONTRACT_COMPARISON_RESEARCH_ACCEPT_NO_PRODUCT_GO**
under the explicitly different receiver-state contracts
below, **NOT a product release**.

First completed source-attested
[GitHub CI #38023917391](https://github.com/definitely-stable/deltameter/actions/runs/38023917391),
code SHA `3ffe9b81533bf11a35f5a54f8640884132a78448`:
all 5 distinct GitHub-hosted workers, 24-profile
unchanged B2A exact cutoff SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`,
**60/60 private source-WAL-backed OS sessions**
and **1,200/1,200 distinct matched paired two-owner
physical query rounds** PASS. Independent fail-closed
Python summarizer checks all source HEADs, session
and query identities, exact bytestream accounting,
nonadaptive odd count vs true oracle,
committed cold source ingest/write and per-run
n20 empirical nearest-rank p95.

Each of TWO distinct owner source OS processes
uses its private `base.snap`, `chain.wal`,
`commit.mark` and `ack.mark` written with real
File::sync_all plus atomically fsync'd markers.
Owner query processes reopen source snapshots
and accepted chained generation2 records FROM
DISK, compute the bitmap once and remain LIVE
for twenty matched GUARD and FULL query rounds.
Source generation2 physically delivered to exact
receiver via additional real owner TCP subprocesses,
then accepted with the same atomic receiver
256B two-owner WAL+48B commit watermark
from D64 and D65. Cold exact gen1 snapshots
are physically transmitted and validated first;
receipt is derived from those transmitted bytes,
NOT silently reconstructed from fixture.
Source fixture is used as post-transfer external
oracle only.

### Explicit state contracts

- **guard-only/stateless receiver:** does not cache
  full exact source lists; its local result is
  the B2A 2x256B keyed bitmap observation
  `S` with `S<=48` boolean. No exact token
  reconciliation is possible from this boolean.
  Full exact fallback is NOT free.
- **maintained exact:** receiver pays actual cold
  two-owner full transfer, full durable initial
  checkpoint/commit/WAL writes and one 256B
  two-owner event+48B commit sync before the
  next hot query. It keeps exact vectors
  materialized in RAM; a repeated exact
  difference query on that durable generation
  does not require another TCP exchange.
- **direct full:** physically transfers exact
  two-owner lists on EVERY query and obtains
  exact differences without a receiver-maintained
  cache. Q20 application emulation uses
  **100Mbps/0ms on localhost**, not WAN.
- **guard resolved:** when `S>48`, exact
  reconstruction would require FULL transfer
  of both owners. Full frames were physically
  obtained in the paired direct-FULL mode,
  but a separate serialized conditional
  `guard→fallback` pipeline was NOT benchmarked
  for latency. Derived total bytes are an
  accounting bound only, not resolved p95.

These are **NOT identical service contracts**:
a keyed parity observation and an exact
difference list do not answer the same query.
The public source key and BLAKE3 integrity tag
are fixture checks, not hostile authentication.

## Physical TCP bytes and cold exact receiver costs

Per SINGLE fixture session, with 20 hot requests
and one already committed generation2 insertion
on BOTH owners:

| Source size N, d | Guard-only queries | Maintained exact initial FULL | Maintained exact Q20 hot requests | Direct FULL Q20 |
|---|---:|---:|---:|---:|
| 256,48 | 14,720B | 4,322B | **0B** | 86,720B |
| 256,57 | 14,720B | 4,330B | **0B** | 86,880B |
| 65,536,48 | 14,720B | 1,048,802B | **0B** | 20,976,320B |
| 65,536,57 | 14,720B | 1,048,810B | **0B** | 20,976,480B |

Two source-children also physically transmit
a **356B** committed generation2 event to the
exact receiver. Per-setup 2x25B source owner
hello + VmHWM/CPU/build telemetry, and each
session's 128B shutdown/telemetry are measured
separately, not disguised as queries.
Owner source WAL fsync, snapshot ingest and
gen2 commit costs exist for BOTH contract
families, even though only maintained exact
pays the gen2 receiver event and its fsync.

Exact receiver initial receipt plus checkpoint/
commit plus gen2 atomic receiver WAL writes
and syncs:

| N, d | Exact receiver logical write bytes per setup | Actual invoked receiver sync calls |
|---|---:|---:|
| 256,48 | 9,448B | 11 |
| 256,57 | 9,464B | 11 |
| 65,536,48 | 2,098,408B | 11 |
| 65,536,57 | 2,098,424B | 11 |

These are source+receiver serialization BYTES,
not OS storage device sectors, SSD write
amplification, real IOPS or fsync elapsed latency.

For N256,d48, even **cold FULL+gen2 event**
total 4,678B is less than 14,720B for guard
Q20 alone; **warm maintained exact uses zero
network bytes for subsequent exact questions**.
For N65536,d48, cold FULL+event is 1,049,158B,
so guard-only 14,720B has a huge cold network
advantage **only when its partial capability
is sufficient**. Exact receiver pays about
2.1MB logical disk writes/initialization and
RAM for both inventory lists: do not hide this
when evaluating that cold niche.

For d57, every one of the 300 strict observations
per N had `S>48` (guard under-cutoff 0/300).
A resolved workflow must therefore transfer
FULL after guard for all those fixtures.
The resulting **derived** 20-query total is
101,600B N256 or **20,991,200B** N65536,
worse than direct full 86,880B /
20,976,480B respectively. This reinforces
D60 **STOP_RESOLVED_NEAR_T** but is NOT a
failure probability theorem at d57. By
contrast d48: `S<=48` 300/300 each N,
consistent with the pointwise S<=d relation.
`S<=48` MUST NOT be described as a
deterministic certificate that d<=48.

## Host empirical p95, not WAN or a guarantee

Each row is a range of FIFTEEN independently
calculated n=20 nearest-rank empirical p95
on fixed GH-hosted workers and three fixtures
per worker, paced 100Mbps/0ms app server:

| N,d | Guard two-source TCP query p95 | Full two-source TCP query p95 | Hot exact local symmetric-difference computation p95 |
|---|---:|---:|---:|
| 256,48 | 0.147–0.257ms | 0.304–0.409ms | 0.0013–0.0031ms |
| 256,57 | 0.145–0.244ms | 0.303–0.418ms | 0.0010–0.0027ms |
| 65,536,48 | 0.165–0.369ms | 43.033–44.308ms | 0.0562–0.1886ms |
| 65,536,57 | 0.156–0.310ms | 42.952–44.087ms | 0.0544–0.1576ms |

Local exact computes the actual set difference
on resident source lists; it is NOT mere
length read or a cached known d, but CPU
execution under a different no-network
receiver contract. Comparing its p95 to
a TCP guard is NOT an equal-service speedup
claim. CPU `/proc/self/stat` tick resolution
was too coarse to distinguish most N256
sender sessions (reported tick delta =0),
while N65536 source sender work registered
kernel ticks; do not report nanosecond CPU.
Source bitmap build wall ns, owner process
VmHWM and exact receiver VmHWM are separately
recorded in source-bound per-session artifacts.

## Critical product decision and next evidence gate

This system experiment rules out a *general*
claim that a stateless guard is better than
already maintained exact at Q20. They have
different capabilities and cold costs,
and equally durable exact can use D64 WAL.
D60 warm N256 NO-GO remains. A credible
potential niche is cold large N and a
statistical observation with sparse queries
where receiver chooses not to ingest exact
data; it must include SOURCE GUARD BUILD,
maintenance under append/delete, key reuse,
source crash/reconnect and true false-pass
rate/proof. It is not validated for an exact
set reconstruction product.

NOT measured/accepted: real WAN, 1/10Mbps,
10/50ms app RTT, Q1/Q10/Q100 sweep,
common hot receiver vs restarted exactly
durable source, device-sector I/O and
fsync wall clock, CPU-core-normalized
decoder cost, worst-case mixed workload,
authenticated hostile source, concurrent
source updates or power cut. The fixed
gen2 receipt is safe only in D64/D65
bounded one-gen test scope; arbitrary
multi-key/multi-epoch protocol still open.

**RESEARCH ACCEPT, NO SYSTEM_PRODUCT_GO**.
Leave #109/#107/#105/#103/#101/#99/#97/#92
and security #86 / API #69 OPEN. Next
B1B1B1-B1B1-B1 must freeze Q1/Q10/Q100,
app rate/latency and identical exact-capability
guard RESOLVED including full fallback,
paired n20 fully durable fsync wall and
receiver hot maintenance vs cold restarts
before any product verdict.
