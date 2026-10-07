# M6-D10 — trace-internal replay measurement plan

Issue: #43. Parent: #19 / #14.  
Base: D9 merge `146fa66bc20c3421403864b7aa26379e3286c184`.  
Scope: private measurement only.

## Why replay instead of hot-path timers

D9 shows that trace remains 57.5-80.6% of accepted D8 d=8 latency. The remaining
primitive operations are small enough that placing `Instant` around every field
operation would materially perturb the code being measured.

D10 therefore collects exact trace operands first and performs offline batched
replays with one outer timer per component.

## Exact operand collection

The collector mirrors accepted D8 factorization and records, for every degree>=3
trace attempt:

- coefficient;
- monic modulus;
- all 64 term polynomials before square/mod;
- exact trace result.

Collector output must equal accepted D8 at every k stage and the final d=8 result
must equal the exact symmetric-difference oracle.

## Replay components

For every recorded term:

1. full accepted square/mod;
2. modulus clone/trim/monic preparation;
3. unreduced square-build with accepted D8 scalar square;
4. reduction of the prebuilt square using frozen generic GF(2^64) multiplication.

Reduction replay is destructive, so it includes a clone of the prebuilt remainder.
A separately measured clone-only batch is subtracted:

```text
net reduction = reduction_with_copy - clone_only
```

At trace-attempt level D10 also measures:

- full trace replay;
- XOR accumulation over the recorded terms.

All component batches use the same exact operands and repeat count.

## Frozen workload

~~~text
corpora = d4,d5,d6,d7a,d7b
scenario = d8
full attempt schedule = k 1,2,4,8
samples/process = 4
batch repeats = 8
hosted processes = 3
~~~

## Validity

For every corpus:

- replay inventories are deterministic;
- each trace attempt has exactly 64 terms;
- full replay reproduces the collected trace;
- split square-build + reduction equals full square/mod;
- reduction-with-copy exceeds clone-only;
- median additive closure
  `(prepare + square-build + net-reduction) / full-square-mod`
  is between 70% and 130%.

Outside that closure range the selector is inconclusive.

## Predeclared selector

Require full-square-mod/full-trace >=70% in every corpus before selecting a
square/mod subcomponent.

Then choose the first common/material component:

1. net reduction >=50% of full square/mod in every corpus;
2. else square-build >=25%;
3. else modulus preparation >=20%;
4. else trace accumulation >=20% of full trace;
5. else stop algebraic micro-optimization.

D10 does not implement the selected optimization.

Production/public ExactSmallDelta remains NO-GO.
