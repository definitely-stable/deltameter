# M6-D6 — Deterministic quadratic root solver evidence

Status: **ACCEPT for private research; performance benefit is factor-tree dependent.**  
Issue: #34. Parent: #19 / #14. PR: #35.  
Evidence date: 2026-10-07.

## Decision

D6 replaces only degree-two factorization in the accepted D4 decoder.

For a monic split quadratic over GF(2^64):

~~~text
x^2 + a*x + b = 0
~~~

with a != 0, D6 substitutes x=a*y and solves:

~~~text
y^2 + y = b / a^2
~~~

using deterministic safe-Rust GF(2) Gaussian elimination in the existing polynomial basis.

Degree >= 3 factorization remains the accepted D4 deterministic trace path.

The result is **ACCEPT for the private research decoder**, with an important limit:

**the realized end-to-end benefit depends materially on the deterministic factor tree.**

Do not summarize D6 as a universal 27% speedup.

## Frozen semantics

Unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also unchanged:

- exact nonzero u64 -> GF(2^64) identity mapping;
- out-of-band zero bit;
- field reduction polynomial 0x1B;
- one guard syndrome;
- D4 characteristic-2 trace square;
- degree >= 3 deterministic trace splitting;
- all-syndrome candidate verification;
- public API, snapshot-v1 and Coverage boundaries.

Production/public ExactSmallDelta remains NO-GO.

## Canonical provenance

Measured implementation head:

`1b982b9288f36a8a2e858ba9da846989a552360d`

Base main:

`8d10d6a38cf72c0c53bd2fa025d66fb8e801330f`

Exact-head GitHub-hosted gates:

- M6-D6 quadratic #6 / `37586264293` — SUCCESS
- Rust #245 / `37586264300` — SUCCESS
- Research #258 / `37586264319` — SUCCESS

Artifact:

`m6d6-quadratic-37586264293-1`

Artifact ID:

`11466691536`

Artifact SHA256:

`6e5024ee8cfb274ea9c202a22522f15ab183ac1d9a34c6545cd0ba548d75a7ee`

Environment:

~~~text
CPU  AMD EPYC 9V45 96-Core Processor
VM   4 logical CPUs exposed
Rust 1.99.0
LLVM 23.1.1
host x86_64-unknown-linux-gnu
~~~

Source SHA256:

~~~text
428e9d687d9c12dad8b622768f94eb35cdf9060a10fa0265c89bf11a96194611  examples/m6d6_bench.rs
38dc7cbca423e3670fc1ce0e2b66164f07d9722670074eb810fc4426db95676f  examples/support/m6d6_quadratic.rs
162a11e7feffb67797786c247e9a1b9a6e4108f5da3e5648ea34b70f94f98b9e  examples/support/m6d4_trace_square.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4  examples/support/m6d_pinsketch64.rs
88b4d0136d151e0c9d8f344a4275e283cdeb423d251d88886fce869a77320365  tests/m6d6_quadratic.rs
ba2f80c7b3a2708b822ffb1911632d1268b079d9b99a7b8b8603c283c7eb6efd  research/m6d6_summary.py
2965d614d7a6287f370d755ad00e0a21b95f8f3e0bcb44722360c9089cdc4182  research/test_m6d6_summary.py
531aba605ed47a15d58d147651039283df5679a8a14514601568058fc598d334  .github/workflows/m6d6.yml
~~~

Fail-closed summary:

~~~text
runs=3
corpora=3
logical_gate=PASS
false_success_total=0
~~~

## Correctness result

The candidate passes:

- deterministic Artin-Schreier image cases;
- inconsistent trace-one RHS rejection;
- deterministic distinct full-width root-pair reconstruction;
- direct polynomial evaluation of both returned roots;
- repeated-root quadratic rejection;
- D1 / D4 generic / D4 specialized / D6 differential outcome checks;
- exact-oracle completion for d=0/1/2/3/4/5/8;
- rejection for d=9/10/16;
- zero, high-bit and u64::MAX coverage;
- guard mutation rejection;
- invalid-limit/error-precedence compatibility;
- frozen D4 square-control equivalence.

No false-success occurs in the frozen repeated deterministic matrix.

That remains empirical guard evidence, not an independent adversarial theorem.

## Measurement method

Three deterministic corpora are used:

- D4-derived corpus;
- D5-derived corpus;
- new D6 corpus.

Each corpus uses the same 8192-key maintained-state workload shape.

For each scenario:

- four balanced AB/BA samples per hosted process;
- three fresh hosted processes;
- 12 observations per corpus;
- 36 observations in the aggregate.

Both arms include:

- prefix materialization;
- fresh locator construction;
- root factorization;
- candidate verification.

Only degree-two factorization differs.

## Exact workloads

### d=2

Across all three corpora the result is very stable:

~~~text
D4 control median   ~43.85 us
D6 median           ~26.48 us
aggregate reduction ~39.6%

corpus medians:
39.5%
39.6%
39.7%
~~~

Degree-two specialization is clearly material.

### d=3

Aggregate:

~~~text
~1.713 ms -> ~0.099 ms
median reduction ~94.0%
~~~

But corpus behavior is bimodal:

~~~text
D4 corpus ~94.3%
D5 corpus ~39.2%
D6 corpus ~94.2%
~~~

The variation comes from the deterministic factor tree, not from protocol changes.

### d=4

~~~text
aggregate reduction ~37.5%
corpus range ~36.5% to ~41.9%
~~~

This is stable and material.

### d=5

~~~text
aggregate reduction ~4.3%
corpus range ~2.7% to ~85.3%
~~~

This workload demonstrates why a single corpus is insufficient.

The D6 corpus happens to create a factor tree with dominant degree-two work; the
other two do not.

### d=8

~~~text
aggregate:
D4 ~6.247 ms
D6 ~4.894 ms
median reduction ~15.3%
~~~

Corpus medians:

~~~text
D4 corpus   ~3.7%
D5 corpus  ~27.1%
D6 corpus  ~15.3%
~~~

Every corpus remains positive, but the realized gain is strongly factor-tree dependent.

The D5 single-corpus profile predicted about 27.5% removable degree-two self work.
On the matching D5-style corpus D6 realizes about 27.1%, validating the profiling
gate. The lower gains on the other corpora show that this opportunity is not uniform.

## d=0/1 controls

d=0 and zero-only d=1 remain sub-microsecond noise-scale cases.

Relative regressions there are not interpreted as product regressions:

~~~text
d=0 aggregate delta roughly -10% relative, tens of nanoseconds absolute
d=1 aggregate roughly neutral/noise-scale
~~~

No protocol or correctness behavior changes.

## Reject paths

d=9/10/16 remain rejected.

Reject-path speed changes are positive overall but are not used as the primary
ACCEPT reason.

## Quadratic micro-cost

The per-solve deterministic Gaussian-elimination implementation is approximately:

~~~text
q0 ~10.28 us
q1 ~10.26 us
q2 ~10.22 us
q3 ~10.22 us
~~~

This is already small relative to the remaining multi-millisecond higher-degree
root work for the meaningful larger-d cases.

Therefore D6 does **not** justify bundling matrix precomputation/caching into the same
slice.

## Interpretation

D5 correctly identified degree-two work as a real opportunity, but its 27.5% d=8
number was a corpus-specific upper-bound observation rather than a universal property.

D6 establishes:

1. the quadratic solver is correct;
2. the solver converts degree-two trace work into a much cheaper deterministic solve;
3. d=2 and d=4 improve stably across all corpora;
4. d=8 improves on all corpora, with a broad 3.7–27.1% range;
5. the remaining latency is dominated by higher-degree factorization, not candidate
   verification or Artin-Schreier setup.

## D6 verdict

**ACCEPT as a private research decoder optimization.**

Do not claim:

- universal 15.3% or 27% latency reduction;
- production ExactSmallDelta readiness;
- a public API or wire-format change;
- a guard theorem from the deterministic corpus.

Keep:

- D6 degree-two solver;
- D4 path as independent control/reference;
- multi-corpus evidence for subsequent root work.

Do not immediately optimize the GF(2) Gaussian solver. Its ~10 us standalone cost is
too small to be the next bottleneck for d=5/8.

## Next step

Before another root algorithm is implemented, profile the **post-D6 residual factor
cost across multiple corpora**.

The next profile must answer:

- how much residual self time is degree 3, 4, 5, ... after degree-two replacement;
- whether one low-degree specialization remains consistently material across corpora;
- whether high-degree trace/GCD work is now too workload-dependent for another narrow
  specialization;
- whether it is time to stop decoder micro-optimization and move to maintained-state
  end-to-end system comparison.

No CLMUL/SIMD/unsafe, matrix caching, alternative field representation or bundled
root specializations should be started before that residual profile.
