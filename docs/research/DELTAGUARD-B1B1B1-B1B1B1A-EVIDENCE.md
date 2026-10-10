# DeltaGuard B1B1B1-B1B1-B1A — exact-output guard→FULL physical system evidence

Parent [issue #109](https://github.com/definitely-stable/deltameter/issues/109),
[PR #112](https://github.com/definitely-stable/deltameter/pull/112).
Frozen [protocol](DELTAGUARD-B1B1B1-B1B1B1A-PROTOCOL.md)
before the first observations at commit
`d9414fc2e87201b2fad7604cab1bbdfd68aa304f`;
the unmeasured observation-total typo was corrected **before any
runs** at commit `46f7c28734da03deb1dd9ecc896b4f671b45d7a8`.
The bounded first-20 p95 window was clarified before any
successful code-bound measurement, not selected by timing outcome.

## Verified outcome

**B1B1B1_B1B1B1A_RESOLVED_EXACT_CAPABILITY_RESEARCH_ACCEPT
with STOP_STATELESS_GUARD_FOR_EXACT_V1, NO SYSTEM_PRODUCT_GO**.

First source-bound COMPLETE
[GitHub-hosted run #38028833535](https://github.com/definitely-stable/deltameter/actions/runs/38028833535),
code SHA `d0ba1d8d2fa345b0cb207be64241cdb7e3f5092a`:
all **5/5 independent hosted workers** and independently SHA-fixed
certificate+fail-closed aggregator **SUCCESS**. Exactly
**90/90** physical two-source process sessions,
**4,815/4,815** matched strict exact-output query pairs
and **210/210** query-prefix records across all
six preregistered workloads. Rust and Research workflows
at the SAME SHA completed SUCCESS. Original 24-profile B2A
cutoff unchanged, SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.

Two owner OS child processes reopen **different private**
durable source gen1 snapshots and chained gen2 WAL, real
File::sync_all and atomic synced COMMIT/ACK markers from
B1B1A. They build the public-fixture B2A keyed bitmap
ONCE and remain alive throughout the repeated requests.
Receiver exact inventory is genuinely physically
bootstrapped from both gen1 TCP FULL frames, persisted
through atomic two-owner checkpoint, then updated from
two actual source gen2 TCP events via the B1B0 256B
receiver WAL plus 48B commit watermark BEFORE the Q
hot questions. Both direct and resolved FULL frames
are independently decoded and matched to the two
disk-reopened exact source inventories and the
maintained exact receiver. GUARD XOR is checked
against an independently hashed true symmetric
difference. Every source response includes its
request, framed body and physical positive ACK in
actual measured app bytes. No free fixture reset
of owner process state on any query.

## Actual observed physical TCP query totals

Totals below are sums of **15 independent
sessions per lane** (5 hosted workers ×3 source
fixtures) and exclude separately and explicitly
reported exact cold/bootstrap and source gen2
update, which are common setup for that
specific maintained exact receiver contract.

| Lane, N,d | Pacing and app delay | Exact query Q/session | Direct FULL TCP, all 15 sessions | GUARD then FULL TCP, all 15 | GUARDED overhead |
|---|---|---:|---:|---:|---:|
| A, 256,48 | 100Mbps, 0ms | 100 | 6,504,000B | 7,608,000B | **+1,104,000B** |
| B, 256,57 | 10Mbps, 10ms | 100 | 6,516,000B | 7,620,000B | **+1,104,000B** |
| C, 256,48 | 1Mbps, 50ms | 100 | 6,504,000B | 7,608,000B | **+1,104,000B** |
| D, 65,536,48 | 100Mbps, 0ms | 10 | 157,322,400B | 157,432,800B | **+110,400B** |
| E, 65,536,57 | 100Mbps, 10ms | 10 | 157,323,600B | 157,434,000B | **+110,400B** |
| F, 65,536,57 | 1Mbps, 50ms | 1 | 15,732,360B | 15,743,400B | **+11,040B** |

**Every physical exact-result pair costs the same
additional 736B for guard→FULL**, independent of
threshold result, source size or host. All **4,815**
pairs collectively add **3,543,840B** of unnecessary
application traffic over direct full.

This confirms only a **compositional protocol identity**,
not a novel mathematical lower bound. A more capable
exact sketch, actual receiver-maintained exact or other
transport framing could change the premise.

## Query results, exact quality and receiver cost

- In the 3,000 d48 observations (lanes A,C:
  1,500 each) and 150 d48 observations in lane D,
  `S<=48` as expected from the pointwise S<=d
  inequality, but GUARD does NOT return exact differing
  token IDs nor a deterministic bound d<=48.
- For d57 in B/E/F, S>48 for every fixed fixture;
  every tested guard result UNKNOWN for the narrow
  cutoff, and physical FULL fallback was exercised.
- A distinct `maintained_exact` receiver pays
  cold gen1 FULL wire (N256 d48 4,322B;
  N256 d57 4,330B; N65536 d48 1,048,802B;
  N65536 d57 1,048,810B), plus actual gen2
  source event wire **356B**; initial receipt +
  atomic checkpoint/commit + gen2 two-owner
  receiver WAL serialized **9,448B/9,464B**
  N256 and **2,098,408B/2,098,424B**
  N65536, with 11 instrumented receiver sync
  invocations. It keeps exact inventory vectors
  in memory and computes exact symmetric difference
  without any per-query source TCP.
- Actual owner build wall time, source /proc CPU ticks,
  source/receiver VmHWM, per-query app RTT, cold
  exact bytes, WAL logical writes and sync count
  are included in each source-bound session artifact.
  **Physical device block writes and fsync elapsed
  latency were not measured**, so no complete
  all-in disk/CPU price inference.

## Latency: useful but not a universal ranking

The values are **ranges of 15 separate fixture/host
nearest-rank empirical p95s** from the *first
20 paired samples* for Q100 lanes A/B/C. For
Q10 lanes D/E, each fixture p95 is n10 (the
maximum observation), and Q1 F records one
elapsed time per fixture, **not n20 p95**.

| Lane | Direct FULL elapsed | Guard→FULL elapsed |
|---|---:|---:|
| A N256 100Mbps/0ms, n20 p95 | 0.298–0.421ms | 0.435–0.693ms |
| B N256 10Mbps/10ms, n20 p95 | 11.868–11.983ms | 22.329–22.567ms |
| C N256 1Mbps/50ms, n20 p95 | 67.205–67.406ms | 120.020–120.443ms |
| D N65536 100Mbps/0ms, n10 p95 | 42.875–263.012ms | 43.025–72.282ms |
| E N65536 100Mbps/10ms, n10 p95 | 52.947–54.229ms | 63.120–64.733ms |
| F N65536 1Mbps/50ms, **one sample** | 4245.975–4247.180ms | 4298.777–4300.347ms |

Lane D demonstrates runner/scheduler variation:
one direct-FULL n10 session was **263ms** even
though other sessions were around 43ms. Do NOT
conclude every worker's p95 guard→FULL is higher
or lower solely from these samples. The
preregistered decision is a strict positive
**actual TCP-byte** overhead under equivalent
exact-output capability; not an empirical
latency/p95 guarantee.

The application pacing is **per source socket**.
Both owner senders are live concurrently; a
1Mbps lane F transfers two ~0.5MiB owner frames
in parallel and completes around 4.25s, NOT
a single shared 1Mbps WAN link with a hypothetical
8.4s serialization time. Server-side 10/50ms
delay is injected once per response on the
individual source, not a measured WAN RTT or
packet loss.

## Critical verdict

**STOP_STATELESS_GUARD_FOR_EXACT_V1**: if a product
requires the same exact difference token list and
offers ONLY the currently implemented B2A guard
bitmap or FULL inventory fallback, sending GUARD
before mandatory FULL cannot save any bytes.
This remains true even if the B2A observation is
below the nearfull threshold. Implementing more
guard decoder micro-optimizations for *this exact*
fallback protocol should STOP.

Do **not** generalize STOP to cold S-only statistical
screening workflows: guard can be cheaper than exact
bootstrapping N65536 if the actual service truly
needs only a fallible S observation. This is a
different output contract, not a DeltaGuard
reconciliation guarantee. Its source key reuse,
false-pass/adaptive risk, source maintenance/
recovery cost, workload-specific decision value
and ownership/authentication remain unproven.
An independently capable exact reconciliation
sketch is likewise outside the premise.

Unmeasured: N65536 at 1Mbps Q10 or Q100,
WAN true shared bandwidth, drop/reorder,
multi-epoch/large batch/complex churn,
source restart during live query stream,
fsync wall latency/device-sector I/O,
cost of physical source snapshot/WAL sync
across all modes, hardware power cuts,
malicious-peer authenticated identity and
adaptive query threat. **NO SYSTEM_PRODUCT_GO**.
#109/#107/#105/#103/#101/#99/#97/#92
remain open, as do security #86 and public
API/release lifecycle #69.
