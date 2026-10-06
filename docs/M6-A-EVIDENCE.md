# M6-A — Estimator-assisted value evidence

Status: **NO-GO for a generic/public distributed workflow**.  
Issue: #16. Parent: #14. PR: #21.  
Evidence date: 2026-10-06.

## Scope

M6-A tests whether a DeltaMeter snapshot can justify skipping an expensive exact-set exchange.

It does not recover missing elements. A rejected admission is a distinct outcome and is never reported as successful reconciliation.

The three measured arms are:

1. direct exact sorted-set transfer;
2. snapshot plus the identical exact transfer as an overhead control;
3. snapshot admission, with exact transfer only after an admit decision.

CI uses deterministic custom Energy rows for reproducibility. These rows are not theorem draws, so the decision experiment is empirical and does not claim Coverage::Proven.

## Reproducible run

Final hosted run: **m6a-value #12 / 37514920381**.

~~~text
head  15d89fdaaf9c24900548e29e685d757f8cb76d50
CPU   AMD EPYC 7763 64-Core Processor
Rust  1.99.0
runs  5
peer set size 8192
admission threshold 256
~~~

Artifact: `m6a-value-37514920381-1`.

The workflow pins the pull-request head, hashes the experiment and summarizer sources, repeats the experiment five times, verifies that all logical outcomes/byte counts are identical, and fails closed if summarization fails.

## Correctness and protocol result

PASS:

- direct exact results equal the harness oracle;
- snapshot-control exact results equal the oracle;
- admitted snapshot paths finish through the same exact oracle;
- rejected paths never claim exact completion;
- wrong identity is rejected;
- replayed request IDs are rejected;
- stale source generations are rejected;
- bounded frames reject truncation/oversize cases;
- timeout handling kills and reaps a deliberately stalled child;
- exact-set input must be canonical strictly increasing u64 values.

The fixed workload matrix covers d = 0, 1, 2, 8, 32, 128, 512, 2048, disjoint sets, and source-size asymmetry.

The deterministic estimator decisions produced zero false-admit and zero false-reject outcomes for this matrix. This is a regression observation, not a statistical guarantee.

## Byte accounting

Across the 11-scenario matrix:

~~~text
direct exact       722,356 bytes
snapshot control 1,134,900 bytes
snapshot admission 937,896 bytes
~~~

Therefore:

- snapshot-control costs about **+57.111%** versus direct exact;
- snapshot-admission costs about **+29.838%** versus direct exact;
- only 3 of 11 scenarios (27.27%) reject the exact exchange.

For equal-size 8192-element sets:

~~~text
direct exact exchange     65,668 bytes
snapshot-only reject      37,504 bytes
snapshot + admitted exact 103,172 bytes
~~~

The byte-only break-even reject fraction is:

~~~text
37,504 / 65,668 = 0.571115
~~~

So more than **57.111%** of comparisons must reject the exact exchange before this snapshot-admission policy beats direct exact transfer on application bytes for this shape.

This is a workload condition, not evidence that a real product workload meets it.

## Cold versus maintained cost

Median diagnostic costs on the hosted runner are approximately:

~~~text
direct peer setup          0.10 ms
sketch peer setup         20.0  ms
source Energy build       19.9  ms
snapshot encode            0.19 ms
direct local exchange      0.16 ms
admission local exchange   0.39 ms
~~~

The exact values are diagnostic local-process timings, not WAN or SLA measurements.

The implication is robust: constructing the estimator on demand is far more expensive than this in-memory exact baseline. M6-A therefore finds no case for cold one-shot snapshot admission.

A maintained-sketch deployment has different economics because sketch construction is sunk/incremental. In that regime the byte break-even above is the relevant first gate, but M6-A has no product workload proving the required reject rate.

RTT is kept separate as round-trip count:

- direct exact: 1 request/response;
- rejected admission: 1 request/response;
- admitted snapshot path: 2 request/responses.

## Decision

**NO-GO** for adding a public distributed reconciliation/admission API from the current evidence.

Reasons:

1. Energy/Parity still do not recover elements.
2. Snapshot-control is pure overhead when the exact exchange still occurs.
3. The fixed workload matrix is +29.838% bytes for admission.
4. Cold construction cost is orders of magnitude above the local exact baseline.
5. No actual product workload has established a reject rate above the ~57.1% byte break-even.
6. Deterministic CI rows cannot supply a theorem-backed admission guarantee.

## What remains useful

M6-A does establish a precise conditional result:

A maintained-sketch, reject-heavy workload may benefit from estimator-assisted admission when avoided downstream work exceeds snapshot transfer and decision overhead.

That is a future workload-specific optimization, not a generic DeltaMeter networking feature.

## Handoff

Do not expand M6-A into a public network API.

Next priorities:

1. continue the read-only M6-D exact-lane source/failure-semantics audit;
2. evaluate M6-C Energy sign hashing because maintained-sketch update/build cost is now a measured bottleneck;
3. evaluate M6-B codec work only as a separate measured optimization; codec savings alone do not change the M6-A product verdict.

Production ExactSmallDelta/hybrid adoption remains gated by a separate GO decision.
