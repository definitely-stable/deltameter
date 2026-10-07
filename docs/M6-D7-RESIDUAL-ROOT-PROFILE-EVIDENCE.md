# M6-D7 — Post-quadratic residual root profile

Status: **PROFILE COMPLETE; NO-GO for an immediate degree-specific cubic/quartic solver.**  
Issue: #36. Parent: #19 / #14. PR: #37.  
Evidence date: 2026-10-07.

## Decision

D7 profiles the accepted D6 decoder after degree-two specialization without
changing any root algorithm.

The result rejects the next obvious degree-specific step.

For exact d=8 workloads, the dominant residual factor degree changes materially by
deterministic corpus:

~~~text
D4-derived corpus   degree 4 dominant
D5-derived corpus   degree 3 dominant
D6-derived corpus   degree 3/4 approximately split
D7a corpus          degree 4 dominant
D7b corpus          degree 3 dominant
~~~

Therefore an immediate cubic or quartic solver would optimize a factor-tree-specific
hot path rather than a stable general bottleneck.

The stable common bottleneck is the **trace path across degree >= 3**.

## Frozen contract

Unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also unchanged:

- accepted D6 deterministic degree-two solver;
- D4 characteristic-2 trace-square structure;
- degree >= 3 deterministic trace splitting;
- GF(2^64) polynomial basis and reduction 0x1B;
- exact nonzero u64 identity mapping;
- separate zero bit;
- one guard syndrome;
- all-syndrome verification;
- public API, snapshot-v1 and Coverage.

Production/public ExactSmallDelta remains NO-GO.

## Canonical provenance

Measured implementation head:

`4df88b3f2a2e713db2a5482adc8db0ff979219e8`

Base main:

`5796715eec578a49f67cec5d81ad9755ecd768d4`

Exact-head GitHub-hosted gates:

- M6-D7 residual profile #5 / `37588181951` — SUCCESS
- Rust #256 / `37588181958` — SUCCESS
- Research #269 / `37588181957` — SUCCESS

Artifact:

`m6d7-residual-profile-37588181951-1`

Artifact ID:

`11467960057`

Artifact SHA256:

`5b568d1de52177bf17689cbd7772438e404847f5a9d2b328d2bb85e11a6b44a8`

Environment:

~~~text
CPU  AMD EPYC 7763 64-Core Processor
VM   4 logical CPUs exposed
Rust 1.99.0
LLVM 23.1.1
host x86_64-unknown-linux-gnu
~~~

Source SHA256 for the original #5 artifact:

~~~text
57582884568c61c2ae2adb6bb5415a9ea60835ef4552f460d9af60a71981da12  examples/m6d7_bench.rs
2fc6650870da721d194886550c4766e8869541b1dff2255e88572f0faf2d778b  examples/support/m6d7_residual_profile.rs
38dc7cbca423e3670fc1ce0e2b66164f07d9722670074eb810fc4426db95676f  examples/support/m6d6_quadratic.rs
162a11e7feffb67797786c247e9a1b9a6e4108f5da3e5648ea34b70f94f98b9e  examples/support/m6d4_trace_square.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4  examples/support/m6d_pinsketch64.rs
7794613b4b9f8a4b592c595637ee73b6130cdb52f6d90c8dee793b104ab884e1  tests/m6d7_residual_profile.rs
fb33909946d539f4021f50525a51470db932b71cc9607d7cf3258385036e7335  research/m6d7_summary.py
07cc24d490eb40a4a5cffd8f0999a8f70752b5224ecfc5547646abe2f7303d54  research/test_m6d7_summary.py
3107c259834957dd742c0e981dd318dd562932d1be0d96f705d61587bda723b8  .github/workflows/m6d7.yml
~~~

Fail-closed summary:

~~~text
runs=3
corpora=5
logical_gate=PASS
false_success_total=0
~~~

### Automated residual-phase aggregation closure

The strongest D7 decision signal is now emitted directly by the fail-closed
summarizer rather than reconstructed manually from per-degree rows.

Reproducibility-closure head:

`291c8cf406b3b237bd0bedf95f8c734ff9cf53bc`

Exact-head GitHub-hosted gates:

- M6-D7 residual profile #11 / `37591616506` — SUCCESS
- Rust #262 / `37591616465` — SUCCESS
- Research #275 / `37591616440` — SUCCESS

Artifact:

`m6d7-residual-profile-37591616506-1`

Artifact ID:

`11468907221`

Artifact SHA256:

`78462d499c6f5a8ed0d48f93f39ccbd74ba4232e8382dc27d13d819d629efd4f`

For exact d=8 the automated aggregate rows report:

~~~text
corpus  trace/control  gcd/control  division/control
D4      67.122%        30.470%      0.479%
D5      63.386%        33.362%      0.695%
D6      82.555%         6.684%      2.287%
D7a     66.930%        30.694%      0.485%
D7b     65.328%        31.383%      0.718%

cross-corpus trace range:    63.386-82.555%
cross-corpus GCD range:       6.684-33.362%
cross-corpus division range:  0.479-2.287%
~~~

This closes the provenance gap between the raw per-degree timing rows and the
general trace-path decision used to authorize the next experiment.

## Profiling method

Five deterministic 8192-key corpora are used:

- D4-derived;
- D5-derived;
- D6-derived;
- new D7a;
- new D7b.

For every corpus:

~~~text
d = 0,1,2,3,4,5,8,9,10,16
~~~

Three fresh hosted processes and four balanced samples per scenario give twelve
observations per corpus.

The accepted D6 decoder is the uninstrumented performance control.

The D7 path is an instrumented copy used only to assign non-overlapping local work
to factor degrees and phases.

Degree two calls the accepted D6 quadratic solver. Degree >= 3 retains the accepted
D4 trace-square/GCD/division path.

Recursive child time is not charged to the parent degree.

## Correctness

The profiled path matches accepted D6 result/error semantics across all corpora and
stages.

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

Zero, high-bit, u64::MAX, invalid-limit ordering and guard behavior remain frozen.

Every degree >=3 frame satisfies:

~~~text
square_calls = 64 * trace_attempts
~~~

Degree two records quadratic calls only and no trace-square work.

## d=8 residual degree distribution

Accepted D6 control and dominant residual degree:

| Corpus | D6 control | Degree 3 | Degree 4 | Degree 5 | Degree 8 | Dominant |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| D4 | ~9.43 ms | ~6.0% | **~82.1%** | ~6.8% | ~3.1% | 4 |
| D5 | ~7.79 ms | **~67.5%** | ~18.0% | ~8.2% | ~3.8% | 3 |
| D6 | ~2.74 ms | ~28.5% | **~29.2%** | ~23.4% | ~10.7% | 4, narrowly |
| D7a | ~9.32 ms | ~5.7% | **~82.3%** | ~6.9% | ~3.2% | 4 |
| D7b | ~7.56 ms | **~71.5%** | ~13.6% | ~8.5% | ~3.9% | 3 |

Five-corpus aggregate d=8 shares:

~~~text
degree 2 median ~0.7%, range ~0.6-2.7%
degree 3 median ~28.5%, range ~5.7-71.5%
degree 4 median ~29.2%, range ~13.6-82.3%
degree 5 median  ~8.2%, range ~6.8-23.4%
degree 8 median  ~3.8%, range ~3.1-10.7%
~~~

No single degree >=3 is consistently dominant.

## d=5 confirms factor-tree dependence

Five-corpus shares:

~~~text
degree 3 median ~38.8%, range ~2.9-81.7%
degree 4 median ~28.9%, range ~13.7-93.8%
degree 5 median  ~2.2%, range ~1.7-13.8%
~~~

A cubic or quartic solver would therefore have highly corpus-dependent realized
benefit, repeating the lesson from D6.

## Smaller exact cases

d=3:

~~~text
degree 3 ~44.7-45.0% of control across all corpora
degree 2 ~22%
~~~

d=4:

~~~text
degree 3 ~45.3-45.7%
degree 4 ~32.7-33.7%
degree 2 ~7.4%
~~~

These are stable, but selecting the next global decoder direction solely from d=3/4
would not address the max audited d=8 path.

## Stable general phase: trace

Summing degree >=3 local trace/GCD/division work over the raw hosted records shows
a more stable cross-corpus picture.

For exact d=8:

~~~text
trace self / accepted D6 control:
  D4 corpus   ~67%
  D5 corpus   ~63%
  D6 corpus   ~83%
  D7a corpus  ~67%
  D7b corpus  ~65%

GCD self:
  roughly ~7-34%, corpus dependent

successful division self:
  below ~2.3%
~~~

Thus **trace is the stable common residual bottleneck** after D6.

For d=4, trace is also about 67-69% of control across all five corpora.

For d=5, trace is roughly 56-63% across the corpora even though the dominant
polynomial degree changes.

This is the strongest D7 decision signal.

## Quadratic and verification after D6

At d=8:

- accepted quadratic self work is only about 0.6-2.7% of control;
- candidate verification remains below roughly 0.3%, normally around 0.07-0.09%.

Neither is the next optimization target.

## Instrumentation overhead

On meaningful exact workloads the profiled wall time remains close to the accepted
D6 control, generally within well below 1%.

The control remains the only performance baseline; instrumented timing is used only
for decomposition.

## D7 verdict

**NO-GO for an immediate cubic or quartic solver.**

The residual degree distribution is too factor-tree dependent.

**GO-to-experiment for one general trace-path optimization only if it is
bit-equivalent, safe-Rust and isolated.**

The next candidate should target work shared by every degree >=3 rather than another
degree-specific solver.

The narrowest promising target is GF(2^64) scalar squaring inside the trace
square/mod step:

~~~text
gf64_square(x) = gf64_mul(x, x)
~~~

is currently implemented by the frozen scalar 64-iteration field multiply.

Field squaring is GF(2)-linear, so the follow-up can use a dedicated bit-equivalent
safe-Rust transform (for example polynomial bit spreading followed by reduction)
instead of routing through generic multiplication. The exact implementation is not
frozen by D7; only the field representation and result are frozen.

A follow-up candidate must change only scalar squaring first. It must not bundle:

- generic field multiplication changes;
- GCD/division rewrites;
- cubic/quartic solvers;
- SIMD/CLMUL/unsafe;
- field representation changes;
- protocol changes.

If specialized scalar squaring does not produce material end-to-end gain across the
same five corpora, reject it and reassess whether decoder micro-optimization should
stop in favor of maintained-state system-level comparison.
