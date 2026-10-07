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
