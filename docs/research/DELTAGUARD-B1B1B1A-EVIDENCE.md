# DeltaGuard B1-B1B1-A — physical fault-induced FULL reset and ACK replay evidence

Parent [#101](https://github.com/definitely-stable/deltameter/issues/101),
research [PR #102](https://github.com/definitely-stable/deltameter/pull/102).
[Frozen preregistration](DELTAGUARD-B1B1B1A-PROTOCOL.md)
source commit `60f712de6f2e6a6ff125cec58c0f4e2d4e0bc9e7`
predates all B1-B1B1-A runs; D60 warm-N256 exact dominance
and near-T STOP are retained. B2A profile/cert unchanged.

## Actual hosted CI

**B1B1B1A_PHYSICAL_FULL_RESET_FAULTS_PASS_NO_PRODUCT_GO**.

First complete source-attested five-worker
[run 37918069691](https://github.com/definitely-stable/deltameter/actions/runs/37918069691)
at code SHA `d3b3fafdc7e49446acf9fa6936337999bb5be71e`:
certificate regenerated, 5 independently hosted GitHub workers,
fail-closed independent aggregator all **SUCCESS**. Exact **390/390**
unique probes: N{256,65536} × 3 fixtures × 13 fault kinds ×
5 workers. The certificate SHA256 remains
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.

For each probe:
1. Two independent child OS source processes physically send
   *two complete* canonical exact source inventories at generation1,
   after separately metered source hello/request/ACK.
   The receiver constructs two distinct exact vectors and
   verifies source oracle independently.
2. A new OS owner-1 sender physically transmits one faulting
   generation2 event. Failed corruption, truncated body/header,
   disconnection, oversized payload, wrong sender/epoch/
   generation/key, noncanonical token ordering and invalid
   exact membership do NOT change the receiver exact state.
   Oversized response is rejected before allocating its
   claimed body.
3. The real SIGKILL lane kills an actual source process
   *after sending header and 4 body bytes*, observes the
   incomplete frame and refuses to alter the receiver exact.
4. The lost-ACK lane physically receives and applies the
   correct delta ONCE, sends no ACK, observes the sender's
   real timeout and receives its identical retransmission,
   then replies with a physical ACK WITHOUT reapplying.
5. Because there is no durable incremental event log,
   every attempt is followed by **full physical re-sync
   at generation2 from two NEW independent owner processes**,
   with exact canonical source truth checked before replacing
   both receiver lists. Final post-transmission VmHWM and
   Linux user+system CPU ticks from healthy sender children
   are sent as 16B telemetry per source; the aggregator
   checks all source and byte identities.

Every worker passed **78** cases and 78 FULL resets,
6 ACK-loss idempotency probes and 6 real SIGKILL probes.
Across all five workers: **30 real process kills** and
**30 ACK-loss identical retransmissions**, 390 full two-owner
resets with exact truth. No missing, duplicated or
substituted record was accepted.

## Real physical byte accounting

| Source N per owner | Initial two-owner exact bootstrap | Full two-owner post-fault recovery |
|---|---:|---:|
| 256 (d48) | 4,338B | **4,354B** |
| 65,536 (d48) | 1,048,818B | **1,048,834B** |

These are application TCP bytes, including **each** source 1B hello,
24B request, complete 64B header+full canonical u64 data,
16B ACK and post-transfer 16B RSS+CPU telemetry.
The 16B increase at generation2 is the two real 8B shared-token
insertions; there is NO presumed invisible source projection.

The injected failed owner-1 transfer adds, per probe:
disconnect 1B, truncated header 17B, truncated body 68B,
checksum/identity/cursor/membership failure with NACK 90B,
noncanonical 99B, preallocation oversized 65B, lost ACK
and duplicate full event 163B, true SIGKILL mid-body 69B.
Across 195 independent source probes per N, those fault
transfers total **15,330B**, separately from two full transfers.

The B1B1B0 baseline retained delta hot query frame with
two source ACKs was 242B. Thus under this explicitly
conservative *no durable log* full-reset policy, even a
single broken incremental state requires thousands to
over one million physical bytes. Reliable regeneration
and recovery can dominate the sketch/query savings,
especially for large exact inventories.

## Scope and critical limitations

**NO crash-consistent retained-state protocol has been proven.**
The new healthy owners regenerate their authoritative sources
**deterministically from public fixtures in new processes**.
They do not prove on-disk fsync, journal provenance, generation
continuity after uncontrolled autonomous source writes, or
correctness of a dead source unable to regenerate its state.
The "recovery" here is the explicitly preregistered fallback
FULL reset with exact independent source oracle, not
incremental ACK journal resumption.

The B1-A integrity checksum uses a PUBLIC BLAKE3 fixture
and is NOT a malicious-peer authentication tag. The
ACK-loss replay demonstrates identical frame idempotency
in a running receiver, NOT a durable ACK after receiver
crash. Duplicate payload with *different* content
at the same generation, multi-peer Byzantine replay,
lost ACK across reboot and clock/epoch rollback after
crash remain unresolved. Strict source epoch/key binding
exists only in the transient lab. CPU user+system
**ticks** are real Linux process accounting counters,
but many tiny N256 transfers report 0 ticks at kernel
tick resolution; they are neither high-resolution CPU
time nor proof of lower CPU cost. Source VmHWM was
sampled **after** transfer as intended; sender-killed
child telemetry is unavailable and must not be inferred.

**No realtime WAN RTT, authenticated protocol, bandwidth/
latency product GO, or public API/Snapshot v1 change.**
Ideal nonadaptive one-sided B2A bound is independent
from all these link failure checks. D60's scoped warm
N256 exact win and near-T resolved STOP remain active.
The unresolved candidate is a cold, sparse, two-owner
large-N guard niche, which must now account for
full-recovery error frequency and actual source ownership.

## Next evidence gate

[Issue #101](https://github.com/definitely-stable/deltameter/issues/101)
remains open for a separately frozen B1-B1B1-B:
durable ACK/cursor records with forced process termination
and fsync/epoch/reconnect test, isolated source state
provenance independent of deterministic regeneration,
multi-session failure rates, actual CPU/RSS comparisons
and viable sparse cold N65536 vs warm exact and
direct-full with a bounded 1/10/100Mbps and 0/10/50ms
application-emulated matrix. Do not promise reliable
production replication based on this slice.
Security #86 and public lifecycle #69 remain blockers.
