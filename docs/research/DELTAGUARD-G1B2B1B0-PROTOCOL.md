# DeltaGuard G1-B2-B1-B0 — TCP retained exact / fallback correctness protocol

Parent [#92](https://github.com/definitely-stable/deltameter/issues/92).
Previous merged B1-A PR #95 / D57.
This B1-B0 protocol is frozen **before new B1-B0 CI runs**.
It refines the earlier [B1 preregistration](DELTAGUARD-G1B2B1-PROTOCOL.md)
without replacing any prior B1-B final p95/bandwidth gate. B1-B0
is only the physical correctness and wire-accounting prerequisite to B1-B1.

## Explicitly independent owner and receiver states

Owners A/B each maintain distinct canonical sorted u64 vectors and keyed
B2A11 b=11,m=2047 bitmap. A separate receiver starts with neither list.
On session 1, the receiver obtains **two complete lists over TCP** and
retains them, then receives **one physically sent event frame per owner
per source generation**. Each complete bootstrap and every append event,
control header, exact response, fallback request and guard bitmap transfer
is charged. Receiving an event does not grant free access to the owners'
source lists. Source and receiver vectors are separately allocated.

All events carry sender, profile, configuration ID, source epoch,
monotone per-owner generation number, canonical token/value and
unkeyed fixture integrity checksum, checked before update. The checksum
does NOT authenticate malicious peers or prove PRF security. Replaying
a generation, dropping a generation, duplicate owner, mixed epoch,
wrong profile/key, non-canonical token order, delete of absent token,
double insert, short frame or bad checksum must fail closed.
No secret material on wire.

B1-B0 physical matrix: N={256,65536}, d={16,48,57,65},
three deterministic fixture windows per worker, session generation
checkpoints S={1,10,100}, five GitHub-hosted workers.
Check every actual generation in retained exact, not just milestones.
The source mutation lane is unchanged from B0: both owners append the
SAME previously unseen token; symmetric difference remains exactly d.
An independent unit correctness lane must also test one-sided
insert/delete and canceled overlap; do not claim performance
representativeness for a workload without churn.

At checkpoint S: physically send BOTH 256B guard states in 64B frames
(two distinct TCP socket senders), independently verify receiver XOR
against keyed direct-hash reference of true A triangle B, and check
SAFE iff observed odd rows <= exact cutoff 48. Direct FULL sends
both canonical exact lists via separate TCP senders, counted independently.
On UNKNOWN, measure a separate real two-owner **fallback request/response**
for the full canonical lists; count both physical requests (24B each)
rather than silently reusing B0's hypothetical single 24B request,
and verify receiver exact truth. Fallback is not a second SAFE test.

## Critical fairness: separate two *query cadences*

**Sparse checks**: guard sends once at S={1,10,100}; exact cache receives
EVERY intervening incremental event and computes exact truth at the
three checkpoints. Count total cumulative bytes up to each S.

**Dense checks**: one guard query for EVERY source generation 1..S,
while exact cache receives exactly the same per-generation event stream.
For identical source histories, guard cumulative cost is 640*S; exact
cache is the initial full transfer plus 146*(S-1) *provided* the 64+9
physical delta frame is accepted without ACK. B1-B1 must measure
real ACK/retry instead of treating those as zero. Report both regimes,
do not select the more favorable cadence after measurement.

B1-B0 must emit actual byte counters and measured construction/transfer
wall-clock values, but no p95 performance judgement may be made before
a separate B1-B1 frozen run with paired-order trials and RTT/pacing
{0,10,50}ms/{1,10,100}Mbps. B1-B0 raw 3-repeat timings are diagnostics
only. B2A false SAFE may occur with rare statistically permitted
probability when d>T: record empirical instances; do not treat every
such occurrence as software corruption. The exact ideal/nonadaptive
guarantee comes from pinned B2A certificate, not from these samples.

## B1-B0 acceptance

- all five workers, all 2*4*3*3 = 72 fixture milestones per worker
  and every intervening generation source/exact-receiver synchronization,
  no omitted/duplicate scenario;
- independent source exact truth, full-list equality to retained
  receiver, source-to-XOR and independent rehash correctness, actual
  bidirectional transport byte counts and fail-closed negative cases;
- mandatory explicit STOP near-T when real resolved fallback bytes
  exceed direct full; no overall product gate claim;
- source-attested complete workers and fail-closed aggregation;
- no public API, no Snapshot v1 change, no new runtime dependencies.

Next B1-B1: physical ACK/retry, process separation, paced/RTT transport,
five-worker paired p95/full RSS and independent authenticated lifecycle
model under #86. Public product boundary #69 remains blocked.
