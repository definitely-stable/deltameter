# M6-D10 — Trace-internal replay measurement

Status: **MEASUREMENT COMPLETE; GO to one isolated polynomial-reduction experiment.**  
Issue: #43. Parent: #19 / #14. PR: #44.  
Evidence date: 2026-10-07.

## Decision

D10 decomposes the common post-D8 trace bottleneck selected by D9 without changing
the accepted decoder.

The predeclared selector returns:

~~~text
validity_gate=PASS
decision=GO_REDUCTION_EXPERIMENT
validity_reasons=none
~~~

No optimization is implemented in D10.

## Canonical provenance

Measured implementation head:

`c5a44cefcb5ddcb728db57460e9a4041b14b9d63`

Base main:

`146fa66bc20c3421403864b7aa26379e3286c184`

Hosted gates at the measured head:

- M6-D10 trace-internal #2 / `37597544507` — SUCCESS
- Research #302 / `37597544455` — SUCCESS
- Rust #289 / `37597544497` — integration gate

Artifact:

`m6d10-trace-internal-37597544507-1`

Artifact ID:

`11471266722`

Artifact SHA256:

`499fed280c34da304a2e6b5fcbf9852898c8dc9d5cb8271b9b3c47649329bc76`

## Method

D10 avoids per-field-operation timers in the decoder.

A collector copy of accepted D8 factorization records the exact degree>=3 trace
attempts and all 64 term states before square/mod. Collector results are
differential-checked against accepted D8 at every k stage.

After collection, the same operands are measured in batched offline replays:

- full trace;
- full accepted square/mod;
- modulus clone/trim/monic preparation;
- unreduced square-build using accepted D8 scalar squaring;
- destructive polynomial reduction using frozen generic GF(2^64) multiplication;
- clone-only baseline for the destructive replay;
- trace XOR accumulation.

Net reduction is:

~~~text
reduction_with_copy - clone_only
~~~

## Correctness

Across the five frozen d=8 corpora:

- collector result/error equals accepted D8 at every k stage;
- final candidate equals the exact symmetric-difference oracle;
- every collected trace has exactly 64 term states;
- full trace replay equals the collected trace;
- trace accumulation over the collected terms equals the collected trace;
- split square-build + reduction equals full square/mod;
- every recorded square/mod successor is reproduced;
- replay inventory is deterministic.

Accepted D8/public/snapshot/protocol code is unchanged.

## Validity: additive replay closure

The predeclared validity interval was 70-130%.

Observed closure:

~~~text
D4    99.055%
D5    97.829%
D6    99.003%
D7a   99.387%
D7b   97.910%
~~~

Range: **97.829-99.387%**.

The separately batched components therefore explain the accepted square/mod kernel
closely enough for selection.

## Trace structure

Full square/mod as a share of full trace replay:

~~~text
D4    99.868%
D5    99.821%
D6    99.753%
D7a   99.931%
D7b   99.889%
~~~

Range: **99.753-99.931%**.

Trace XOR accumulation is only **0.537-0.708%** of full trace.

Therefore the residual trace bottleneck is effectively the repeated polynomial
square/mod kernel, not trace accumulation.

## Square/mod decomposition

Share of full square/mod:

| Corpus | modulus prepare | square-build | net reduction | closure |
| --- | ---: | ---: | ---: | ---: |
| D4 | 1.949% | 5.061% | **92.080%** | 99.055% |
| D5 | 2.709% | 6.663% | **88.428%** | 97.829% |
| D6 | 2.061% | 5.112% | **91.804%** | 99.003% |
| D7a | 1.929% | 5.043% | **92.428%** | 99.387% |
| D7b | 2.842% | 6.719% | **88.299%** | 97.910% |

Cross-corpus ranges:

~~~text
modulus preparation   1.929-2.842%
square-build          5.043-6.719%
net reduction        88.299-92.428%
~~~

The predeclared reduction selector required >=50% in every corpus. The measured
minimum is 88.299%.

## Interpretation

D8 successfully removed scalar field squaring as a major cost, but the surrounding
polynomial reduction still invokes frozen generic bit-by-bit GF(2^64)
multiplication repeatedly.

D9's count-only evidence suggested this direction but did not authorize it. D10 now
provides timing evidence: polynomial reduction itself is the stable trace-internal
bottleneck across all five factor-tree shapes.

This does **not** authorize a general GF(2^64) rewrite, field-representation change,
SIMD/CLMUL, unsafe, or a protocol change.

## D10 verdict

**GO to one isolated polynomial-reduction experiment only.**

The follow-up must:

- preserve accepted D8 scalar squaring;
- preserve factor tree, GCD/division and D6 quadratic solver;
- change only the multiplication work used by polynomial reduction inside trace
  square/mod;
- remain safe Rust;
- be bit-equivalent to the frozen generic reduction;
- be evaluated end-to-end against accepted D8 on the same five corpora;
- have a predeclared performance gate before the first candidate run.

If the isolated reduction candidate does not produce consistent material
end-to-end gain, stop algebraic micro-optimization and move to maintained-state
system comparison.

Production/public ExactSmallDelta remains **NO-GO**.
