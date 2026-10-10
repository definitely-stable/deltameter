# DeltaGuard B1-B1B1-B1B1-B1A — preregistered exact-capability resolution audit

Parent #109, D66, merged PR #111. **Freeze before implementation-run / observations.**
Research only. Never modify public Snapshot v1/API, preexisting B2A certificates,
or claim a new mathematical lower bound.

## Falsifiable question and decisive distinction

User requires the **exact symmetric difference token list**, not just an
odd-parity/threshold observation. A B2A guard-only bitmap does not encode
the exact set difference; even `S<=48` is **not** a deterministic
certificate that `d<=48` and is not an exact result. With the
currently implemented protocols, a stateless guard must therefore
request an exact FULL inventory transfer on EVERY exact-result query,
regardless of `S`. The physical `guard→FULL` protocol transfers
**strictly 736 extra application TCP bytes per query** over direct FULL
(2 owners × (32B request+64B header+256B bitmap+16B ACK)).
This simple compositional accounting is algebra, **not a novel theorem**.
Both variants must use identical exact canonically framed FULL bytes
and the same long-lived durable independent source owner processes.
The order of two full-result candidate modes is rotated per query.
Do NOT cherry-pick `d=48` SAFE as a free exact result or d57 UNKNOWN
as a reason to treat guard-only bytes as resolved traffic.

A distinct third contract, `maintained_exact`, pays cold FULL + actual
two-owner WAL/commit fsync and then computes exact diff from hot receiver
vectors with no further source TCP on unchanged generation; its
initialization/RAM/disk must be accounted separately. Same source
private WAL/snapshot/commit/ACK chain is built before ALL query modes.

## Frozen physical matrix and feasibility

5 GitHub-hosted workers × 3 source fixture repeats × **six named lanes**:

| Lane | N | d | rate (Mbps) | app delay (ms) | paired exact-result query count |
|---|---:|---:|---:|---:|---:|
| A | 256 | 48 | 100 | 0 | 100 |
| B | 256 | 57 | 10 | 10 | 100 |
| C | 256 | 48 | 1 | 50 | 100 |
| D | 65,536 | 48 | 100 | 0 | 10 |
| E | 65,536 | 57 | 100 | 10 | 10 |
| F | 65,536 | 57 | 1 | 50 | 1 |

= **90 long-lived source sessions, 4,815 physically paired
fully exact-output rounds**, including 100-repetition Q prefixes
at Q1/10/100 where feasible. Each paired round does three REAL
two-source TCP interactions: one direct exact FULL query and,
in the other mode, a GUARD query followed by a second physical
exact FULL fallback, even when `S<=48`.
Every FULL frame is decoded and compared with each source's
disk-reopened exact inventory and receiver-maintained exact truth;
guard XOR is compared to independently hashed symmetric difference.
All source processes remain live throughout the queries in one session.

The first 20 paired repetitions/fixture report empirical nearest-rank
n20 p95 for lanes A/B/C (and any other lane with >=20 observations);
Q1/10/100 cumulative bytes and elapsed source-side physical mode
wall times are reported separately. For Q10 lanes report n10
nearest-rank p95 **labeled n10**, NOT misleading n20.
For lane F Q1 report a single raw latency ONLY.
No aggregate-window p95 that pretends Q1 is a 20-sample distribution.

One physically paced 1Mbps FULL N65536 transfer requires roughly
8.4s theoretical transmission time; physically running that at
Q100 across five workers and six lanes would be prohibitively
slow. N65536 1Mbps Q10/100 are **NOT RUN** and must never be
represented as measured; formulas for absent lanes, if shown,
must be labeled theoretical exact byte identities only.
1/10/100Mbps and 0/10/50ms are locally application-emulated;
parallel source sender pacing is NOT a shared WAN rate.

## Complete accounting / decision rules

Two source owners privately fsync their B1 gen1 canonical
`base.snap` and chained gen2 `chain.wal` with commit+ACK marker,
then a long-lived owner OS process reopens them from disk and
builds keyed near-full m2047 B2A bitmap once, charging wall build,
CPU ticks and owner VmHWM. A physically transferred gen1 pair FULL
is checkpointed at receiver, then a separate physical TCP gen2
event pair is durably committed via B1B0 256B atomic receiver WAL
and 48B committed watermark BEFORE any hot exact query.
Receiver checkpoint+WAL logical bytes and actual sync invocation
counts are recorded. Source seed/snapshot/WAL costs are shared
and must not disappear from total-cost interpretation. Physical
TCP request/response/ACK sizes include both owners. Signed/verified
by the fixed B1 fixture checksum, NOT sender authentication.

For `N256,d48`, each exact FULL Q body pair with 2 owners
has 240+16N = 4,336B; guard then FULL is 5,072B;
`N65536,d57` FULL pair has
240+16N+8 = 1,048,824B, resolved =1,049,560B.
Odd d57 source2 has N+1 sorted entries.
Cold maintained exact GEN1 physical pair includes two owner
1B hello and 224B control+headers+16N+8(d%2) bytes,
plus 356B physically committed gen2 source events.
The guard-only bitmap is NOT enough for precise set members
on any row. The original 24-profile B2A nonadaptive
fixed cutoff SHA256 remains
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.

No-go trigger, preregistered: if all confirmed
same-capability resolved queries pay guard+full and
exact-direct has fewer TCP bytes for every sample,
record **STOP_STATELESS_GUARD_AS_EXACT_RECONCILIATION_V1**.
Do not gate PASS on scheduler-sensitive p95 superiority:
report latency and per-worker n20/n10 data candidly.
The matched-output negative gate does NOT invalidate
distinct `S`-only cold screening applications.

Five independent GH-hosted workers, immutable source SHA,
exact source & receiver oracle, source subprocess identities,
real frame bytes, all rows and negative corruption vectors
validated by independent fail-closed aggregator before
research ACCEPT. Never call os SIGKILL power loss or
local pacing WAN. No source authentication, adversarial
adaptive query bound, mixed large batches, multiepoch,
fsync micro-latency/device writes, or remote loss claimed.

Only B1B1B1-B1B1-B1A bounded negative decision
`RESOLVED_EXACT_CAPABILITY_RESEARCH_ACCEPT_NO_PRODUCT_GO`.
#109 remains open for Q1/Q10/Q100 at additional
true-source-fault/cold/CPU/IO regimes if worthwhile;
#86 adaptive security and #69 public release still blocked.
