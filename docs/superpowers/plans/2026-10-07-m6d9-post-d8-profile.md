# M6-D9 — post-D8 residual decoder profile plan

Issue: #41. Parent: #19 / #14.  
Base: D8 merge `6fd743204c42c58647302b3a3c15cbfeb6e7e78f`.  
Scope: private profiling only.

## Decision question

After accepting D8 scalar GF(2^64) squaring, does a common material algebraic
decoder bottleneck still remain across the five deterministic corpora?

D9 does not authorize or implement another optimization.

## Control and instrumentation

Control: accepted D8 decoder.

Profiled path: an instrumented copy of D8 factorization that:

- calls the accepted D6 quadratic solver for degree two;
- uses the exact accepted D8 `gf64_square_candidate` primitive;
- keeps generic multiplication, trace splitting, GCD/division and verification;
- records the same non-overlapping local phase timers as D7.

Trace-internal work gets counts only:

- coefficient-square operations;
- generic reduction-multiplication operations.

There are deliberately no per-coefficient timers because their observer cost would
be comparable to the primitive itself.

## Frozen matrix

~~~text
corpora = d4,d5,d6,d7a,d7b
d = 0,1,2,3,4,5,8,9,10,16
samples/process = 4 balanced control/profile order
hosted processes = 3
schedule = k 1 -> 2 -> 4 -> 8
payload = 17 -> 25 -> 41 -> 73 bytes
~~~

## Validity gate

- D9 result/error equals accepted D8 at every stage.
- Successful d<=8 recovery equals the exact oracle.
- d=9/10/16 remain reject.
- False-success total is zero in the deterministic matrix.
- Operation counts are deterministic per corpus/scenario/degree.
- d=8 median profile-wall overhead is <= +5% for every corpus.

If the overhead condition fails, the phase-selection result is INCONCLUSIVE.

## Frozen decision gate

A phase is common/material only when its d=8 share of accepted D8 latency is >=25%
in every one of the five corpora.

Measured phases:

- degree>=3 trace;
- degree>=3 GCD;
- successful division;
- degree-two quadratic;
- candidate verification.

Decision:

- common trace >=25% -> only a narrower trace-internal measurement slice is allowed;
- another common phase >=25% -> only a narrower evidence slice for that phase;
- no common phase >=25% -> stop algebraic micro-optimization and move to the
  maintained-state system comparison.

Counts alone never authorize an optimization.

Production/public ExactSmallDelta remains NO-GO.
