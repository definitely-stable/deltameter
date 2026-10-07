# M6-D9 — Post-D8 residual decoder profile

Status: **PROFILE COMPLETE; GO only to narrower trace-internal measurement.**  
Issue: #41. Parent: #19 / #14. PR: #42.  
Evidence date: 2026-10-07.

## Decision

D9 profiles the accepted D8 decoder after scalar GF(2^64) square specialization.

No accepted decoder algorithm is changed.

The predeclared decision gate was:

- a phase is common/material only when its d=8 share of accepted D8 latency is
  at least 25% in every one of the five frozen corpora;
- d=8 median profiler wall overhead must remain <= +5% in every corpus.

Result:

~~~text
validity_gate=PASS
decision=GO_TRACE_INTERNAL_PROFILE
common_material_phases=trace
validity_reasons=none
~~~

Only trace passes the common-material gate.

## Canonical provenance

Measured implementation head:

`d7e47a885d197b4368833b4c5852c671d8ebf966`

Base main:

`6fd743204c42c58647302b3a3c15cbfeb6e7e78f`

Hosted gates at the measured head:

- M6-D9 post-D8 profile #1 / `37596060787` — SUCCESS
- Research #296 / `37596060776` — SUCCESS
- Rust #283 / `37596060890` — exact-head integration gate

Artifact:

`m6d9-post-d8-profile-37596060787-1`

Artifact ID:

`11470213546`

Artifact SHA256:

`2cd8786bd8d28adb5bd26b0d63a4307af9b4482ad643483dd5988611aaafa176`

The artifact contains runner environment, source hashes, all three raw result files
and the fail-closed summary.

## Frozen semantics

Unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also unchanged:

- D6 deterministic degree-two solver;
- D8 dedicated scalar GF(2^64) square;
- degree>=3 deterministic trace splitting;
- generic multiplication in normalization/reduction/GCD/division;
- GF(2^64) polynomial basis / reduction 0x1B;
- exact nonzero-u64 identity mapping;
- separate zero bit;
- one guard syndrome;
- all-syndrome verification;
- public API, snapshot-v1 and Coverage.

Production/public ExactSmallDelta remains NO-GO.

## Profiling method

Five deterministic 8192-key corpora:

- D4-derived;
- D5-derived;
- D6-derived;
- D7a;
- D7b.

For every corpus:

~~~text
d = 0,1,2,3,4,5,8,9,10,16
~~~

Three fresh hosted processes and four balanced control/profile samples per scenario
produce twelve observations per corpus.

The accepted D8 decoder is the uninstrumented performance control.

The D9 path is an instrumented copy used only for phase selection. It calls the
exact accepted D8 scalar-square primitive.

Non-overlapping local timings remain:

- quadratic;
- trace;
- GCD;
- successful division;
- factor wall;
- verification.

Inside trace square/mod, D9 records counts only:

- coefficient-square operations;
- generic reduction multiplications.

There are no per-coefficient timers.

## Correctness

The profiled path matches accepted D8 result/error semantics across all five corpora
and all stages.

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

Zero, high-bit, u64::MAX, invalid-limit ordering and guard behavior remain frozen.

False-success total is zero in the repeated deterministic matrix.

## Instrumentation validity

d=8 median profile-wall overhead:

~~~text
D4    +0.513%
D5    +0.678%
D6    +1.013%
D7a   +0.784%
D7b   +0.642%
~~~

All five are well below the predeclared +5% ceiling.

The phase-selection profile is therefore valid.

## d=8 residual phase result

Share of accepted D8 end-to-end control latency:

| Phase | Median | Corpus min | Corpus max | Common material? |
| --- | ---: | ---: | ---: | --- |
| trace | **62.208%** | **57.545%** | **80.590%** | **YES** |
| GCD | 35.486% | 8.088% | 39.307% | no |
| division | 0.820% | 0.554% | 2.773% | no |
| quadratic | 0.772% | 0.632% | 3.003% | no |
| verification | 0.088% | 0.079% | 0.358% | no |

Trace remains the only phase whose minimum across all five corpora exceeds 25%.

D8 reduced trace cost materially, but did not eliminate it as the shared bottleneck.

## Trace operation-count diagnostic

For d=8, deterministic degree>=3 totals are:

| Corpus | square/mod calls | coefficient-square ops | reduction-multiply ops | reduction/square-op ratio |
| --- | ---: | ---: | ---: | ---: |
| D4 | 6016 | 23283 | 71130 | 3.06x |
| D5 | 6400 | 20815 | 51938 | 2.50x |
| D6 | 2240 | 8285 | 25518 | 3.08x |
| D7a | 5888 | 22845 | 70004 | 3.06x |
| D7b | 6464 | 20879 | 51576 | 2.47x |

These are operation counts, not timing attribution.

They show that after D8 there are roughly 2.5-3.1 generic reduction multiplications
per coefficient-square operation in the observed trace paths. That is sufficient to
justify **measurement of trace internals**, but not sufficient to claim generic
multiplication or reduction is the next optimization target.

## D9 verdict

**GO only to a narrower trace-internal measurement slice.**

Do not start:

- cubic/quartic specialization;
- generic GF(2^64) multiplication optimization;
- GCD rewrite;
- division rewrite;
- SIMD/CLMUL/unsafe;
- tables/caches;
- protocol changes.

The next slice must measure trace internals with low observer effect and distinguish
at minimum:

- square/mod chain cost;
- polynomial reduction cost;
- trace accumulation/other overhead.

Only that evidence may select another algebraic candidate.

If no trace-internal component proves independently material, stop decoder
micro-optimization and move to maintained-state system comparison.

Production/public ExactSmallDelta remains **NO-GO**.
