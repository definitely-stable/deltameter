# DeltaGuard B1-B1B0 — physically persistent sources and same-lifecycle hot/hot evidence

Parent [#99](https://github.com/definitely-stable/deltameter/issues/99),
[#97](https://github.com/definitely-stable/deltameter/issues/97)
and #92; PR [#100](https://github.com/definitely-stable/deltameter/pull/100).
[Frozen prior protocol](DELTAGUARD-B1B1B0-PROTOCOL.md) commit
`c2c0386f988ad37015a5a70392b88d984afa78c9`
predates the first B1-B1B0 run. B2A ideal T64/b11/c48 exact
certificate remains frozen, no Snapshot v1/public API edits.

## Status and provenance

**B1B1B0_PERSISTENT_HOT_HOT_CORRECTNESS_ACCEPT_NO_PRODUCT_GO**.

Source commit `1353dacac4fa65cd14225f6078bf4d0e4fb3efe0`:
[GitHub-hosted run 37913434241](https://github.com/definitely-stable/deltameter/actions/runs/37913434241).
All 5 independent hosted workers + B2A proof regeneration and
fail-closed aggregator **SUCCESS**; 1000/1000 unique source-bound
per-generation query observations (200 per worker), three lanes,
20 consecutive generations, physically distinct long-lived
sender OS processes. Five workers each completed 3
real socket corrupted-checksum → receiver NACK with no exact
state mutation → valid retry/ACK cases (15/15 recovery probes).

The source identity, full 24-profile cutoff certificate (SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`),
per-owner source truth and B2A XOR of *independently hashed*
true A triangle B, every retained exact receiver generation,
bidirectional bytes, final per-owner process VmHWM and strict
frame generation/owner/key/profile/epoch are checked. No source
worker process is restarted during its 20-generation session.
The physical sender source states stay live and incrementally
updated. Both protocols are measured with the SAME hot/live owner
processes and with the SAME per-query timer start/end, correcting
the invalid mixed cold/hot p95 comparison recorded in D59.

All reported times are real *localhost application control/pacing*
elapsed wall clock, NOT native process CPU time, shared physical
link bandwidth or WAN RTT. Delay injected per response via
server sleep. ACK confirms receipt for this run but is NOT crash-
durable or cryptographically authenticated. n20 nearest-rank
empirical p95 for each worker is the 19th ordered time, not a
distribution-free tail performance guarantee.

## Hot query physical byte totals across 20 generation queries

All bytes account BOTH actual sender 32B requests, 64B framed
payload, 16B physical ACK per query per owner. Sender initial
bootstrap and source ADVANCE are listed separately, not
silently omitted or charged 20 times.

| N, d, delay | Mode | 20-query application bytes |
|---|---|---:|
| 256, 48, 0ms | B2A guard-only | 14,720 |
| 256, 48, 0ms | receiver-retained exact delta | **4,840** |
| 256, 48, 0ms | direct exact full | 89,760 |
| 256, 57, 10ms | B2A guard-only | 14,720 |
| 256, 57, 10ms | receiver-retained exact delta | **4,840** |
| 256, 57, 10ms | direct full exact | 89,920 |
| 256, 57, 10ms | B2A resolved guard/fallback | **104,640** |
| 65536, 48, 10ms | B2A guard-only | **14,720** |
| 65536, 48, 10ms | receiver-retained exact delta | 4,840 warm, **1,048,802B cold bootstrap** |
| 65536, 48, 10ms | direct exact full | 20,979,360 |

Cold exact bootstrap physically transmitted to receiver at
generation1 (inclusive of both 1B sender hellos, requests, frames,
ACKs): N256,d48 **4,322B**, N256,d57 **4,330B**,
N65536,d48 **1,048,802B**. Therefore *cold + 20 warm query
bytes* for N256,d48 retained exact = **9,162B**, less than
B2A20 guard queries **14,720B**; for N65536,d48
retained exact = **1,053,642B**, much larger than B2A
14,720B. Source advancement commands 96B per generation
(20 benchmark generations + one negative test generation = 2,016B)
are physically sent and **shared/common** in this laboratory,
not included in the per-mode query totals. Graceful shutdown
+ child final VmHWM costs 112B per scenario. One checksum
NACK + exact retransmission probe costs 363B per lane.

At N256,d57 near T the 20 strict fixture queries
were UNKNOWN, so the resolved mode pays two 2-owner
source-transfer rounds rather than pretending UNKNOWN
means d>64. Its **104,640B** > direct full
**89,920B**: retain **STOP_RESOLVED_NEAR_T**.

## Empirical same-lifecycle hot/hot latency

All values are the range of the **five worker-specific
n=20 nearest-rank empirical p95**. No cross-worker pooling,
fit or tail probability theorem.

| Scenario | Guard p95 | Maintained exact p95 | Direct full p95 |
|---|---:|---:|---:|
| N256,d48, app delay 0ms | 0.42–0.51ms | **0.20–0.28ms** | 1.94–2.02ms |
| N256,d57, app delay 10ms | 10.54–10.59ms | **10.30–10.39ms** | 12.04–12.13ms |
| N65536,d48, app delay 10ms | 10.30–10.52ms | **10.28–10.45ms** | 53.51–54.66ms |

N256,d57 resolved guard p95 across workers was approximately
**22.60–22.71ms** (second response delay plus exact transfer),
worse than both the guard-only and direct-full lanes.
A few hundred microseconds in these synthetic conditions are
NOT a robust LAN/WAN p95 performance claim; performance
depends on schedule, host CPU, batching, cache state and
transport. Every mode starts after two fully ready same-process-
lifecycle owner children, and receiver exact bootstrap is paid
physically BEFORE the hot query sample stream.

## Decision and remaining gates

**NARROW SYSTEM NO-GO for B2A as a general replacement for
receiver-maintained exact on these warm N256/20-query/one-token-
per-source workloads.** Maintained exact sends fewer physical
bytes and has lower measured n20 p95 on every worker. This is
a *negative* but decision-grade scoped comparison, not grounds
for new algebraic microoptimizations. Against full direct
inventories, B2A guard is substantially cheaper, but direct
full is not the strongest legal comparator once the exact
receiver already has trusted/validated state.

**COLD large-N SPARSE GUARD niche remains plausible**, because
an exact receiver's 1MiB initial sync is not amortized across
20 queries; any future claim must explicitly include B2A
source construction/maintenance, retained warm-list capability,
query cadence and reliability overhead. Above/beside threshold
near-T resolved remains STOP.

This is still **NO SYSTEM_PRODUCT_GO**; neither adversarial
adaptive PRF guarantee nor malicious-peer authentication is
proven, and B1-B1B0 does not implement durable ACK log, process
crash/restart, dropped/duplicate/out-of-order physical socket
reconnection, replay-window/key rotation, or actual per-process
CPU clock. Next B1-B1B1 must stress those, quantify full
source+receiver RSS and CPU, and freeze broader relevant
query-rate/throughput/RTT application-emulation lane coverage
before a public implementation verdict. #99/#97/#92 remain
open alongside security #86 and public-product lifecycle #69.
