# M6-D12 — post-D11 whole-decode residual profile

Issue: #48. Parent: #19 / #14.  
Base: D11 merge `035b3ff9414153df4fe7526f31c43a7a6d8f4748`.  
Scope: private profiling only.

## Decision question

D11 removed roughly half of d=8 end-to-end decode time. Does any one residual
phase remain a stable common material bottleneck across all five frozen corpora?

D12 does not implement another optimization.

## Why the profile starts outside factorization

After D11, work that used to be negligible may become material. The control is the
complete accepted D11 staged decode, including prefix reconstruction and locator
construction.

D12 measures cumulative non-overlapping phases:

1. guarded-prefix materialization and merge;
2. fresh locator/BM construction;
3. decoder validation;
4. D11 reduction-plan construction;
5. degree>=3 trace;
6. GCD;
7. successful division;
8. degree-two quadratic solve;
9. candidate verification.

Factor wall and decoder wall are recorded only as overlapping accounting controls.

## Frozen matrix

~~~text
corpora = d4,d5,d6,d7a,d7b
d = 0,1,2,3,4,5,8,9,10,16
samples/process = 4 balanced control/profile order
hosted processes = 3
schedule = k 1 -> 2 -> 4 -> 8
payload = 17 -> 25 -> 41 -> 73 bytes
~~~

## Validity

- profiled result/error equals accepted D11 at every stage;
- successful d<=8 recovery equals the exact oracle;
- d=9/10/16 remain reject;
- false-success total is zero in the deterministic matrix;
- factor/plan/trace call counts are deterministic;
- d=8 median profile-wall overhead is <= +5% in every corpus.

If the overhead condition fails, the selector is inconclusive.

## Predeclared selector

For d=8, a selectable phase is common/material only if it accounts for >=25% of
accepted D11 control latency in every corpus.

Selectable phases:

~~~text
prefix
locator
validation
plan_build
trace
gcd
division
quadratic
verification
~~~

If more than one phase clears the threshold, select only the one with the largest
cross-corpus median share.

- selected phase -> authorize only a narrower measurement slice for that phase;
- no selected phase -> stop algebraic decoder micro-optimization and move to the
  maintained-state system comparison;
- invalid profiler -> reduce instrumentation and rerun.

Production/public ExactSmallDelta remains NO-GO.


## Replication closure for the 25% boundary

Valid exact-head reruns disagreed on the D6-derived GCD share around the frozen
25% threshold: observed worker-level summaries included 23.616%, 24.721% and
26.359%. Correctness and instrumentation-overhead gates passed in all cases.

This is treated as a reproducibility problem, not as permission to choose the
favorable run.

Before collecting the canonical closure data, freeze this additional requirement:

~~~text
independent GitHub-hosted workers = 5
profiler processes / worker       = 3
balanced samples / process        = 4
paired samples / worker / corpus  = 12
~~~

For each raw sample, each selectable phase is divided by the accepted-D11 control
time from that same sample. Each worker/corpus/phase is summarized by the median of
its 12 paired shares.

Every worker must independently satisfy the original logical/matrix/false-success
checks and the <=+5% d=8 median paired profile-overhead ceiling.

A phase is replication-stable common/material only when its worker-level median is
>=25% for every corpus on every worker.

- stable phase(s) -> select only the phase with the largest median across all
  worker/corpus medians and authorize a narrower **measurement** slice;
- no stable phase -> STOP_ALGEBRAIC_MICRO_OPT and move to maintained-state system
  comparison;
- any invalid worker -> INCONCLUSIVE_REPLICATION.

The 25% threshold itself is unchanged. This closure only makes its reproducibility
requirement explicit after independent exact-head reruns demonstrated boundary
sensitivity.
