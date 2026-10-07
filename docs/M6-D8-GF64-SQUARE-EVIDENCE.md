# M6-D8 — GF(2^64) scalar square specialization evidence

Status: **ACCEPT for private research.**  
Issue: #38. Parent: #19 / #14. PR: #39.  
Evidence date: 2026-10-07.

## Decision

D8 replaces only coefficient squaring inside the degree>=3 trace square/mod path.

Control:

~~~text
gf64_square(x) = gf64_mul(x, x)
~~~

Candidate:

~~~text
carryless bit-spread square
+ reduction in the frozen GF(2^64) polynomial basis
+ reduction constant 0x1B
~~~

The accepted D6 degree-two solver is called directly. Generic multiplication used
by normalization, inversion, polynomial reduction, GCD and division is unchanged.
The accepted D6 support source remains unchanged.

The candidate passes the performance threshold frozen before the first D8
performance run.

## Frozen performance gate

Primary d=8 gate:

- every corpus-level paired median reduction >= 5.0%;
- every per-process d=8 paired median > 0%;
- aggregate d=8 paired median reduction >= 5.0%.

Guardrail:

- for d=3/4/5, no corpus-level paired median may regress below -2.0%;
- d=0/1/2 are noise/control cases and do not determine acceptance.

## Canonical provenance

Measured implementation head:

`c2683dd7ec0f0a091d7d2aeff127db72e1ce5349`

Base main:

`fda9f3fbe7abe32a0c5961571211cf3ab72eb74d`

Exact-head GitHub-hosted gates:

- M6-D8 GF64 square #7 / `37593440166` — SUCCESS
- Rust #276 / `37593440167` — SUCCESS
- Research #289 / `37593440254` — SUCCESS

Artifact:

`m6d8-gf64-square-37593440166-1`

Artifact ID:

`11469378820`

Artifact SHA256:

`ab80093cd5acadb0f62d571c67483d0b04ee3ed04854f7056c3e53923cd353e9`

The artifact retains the exact runner environment, source hashes and all three raw
result files.

Fail-closed summary:

~~~text
runs=3
corpora=5
logical_gate=PASS
false_success_total=0
performance_gate=PASS
performance_gate_reasons=none
~~~

## Correctness

The candidate square matches the D8-local byte-equivalent frozen scalar reference
on all 64 polynomial basis vectors.

Additional deterministic checks cover:

- zero;
- one and small values;
- high bit;
- u64::MAX;
- 4096 deterministic full-width values.

The complete D8 decoder matches the accepted D6 decoder result/error at every
k stage across all five deterministic corpora.

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

Zero, high-bit, u64::MAX, invalid-limit ordering, one-guard semantics and
all-syndrome verification remain frozen.

## Candidate construction

Carryless squaring in characteristic two places input coefficient bit i at
polynomial degree 2i. D8 performs this by spreading the two 32-bit input halves
into even bit positions of a 128-bit value.

For the frozen field representation:

~~~text
x^64 = x^4 + x^3 + x + 1
~~~

The high 64 bits are folded with shifts 0/1/3/4. The first fold can leave only a
small bounded carry polynomial; one second fold removes it without another
overflow.

The implementation is safe Rust using integer shifts, masks and XOR only. It adds
no table, dependency, unsafe, SIMD or CLMUL path.

## d=8 primary result

Paired end-to-end median reductions versus accepted D6:

| Corpus | Control median | Candidate median | Paired reduction | Per-process median range |
| --- | ---: | ---: | ---: | ---: |
| D4 | ~9.464 ms | ~8.229 ms | **13.043%** | 12.854-13.226% |
| D5 | ~7.786 ms | ~6.674 ms | **14.213%** | 14.160-14.409% |
| D6 | ~2.747 ms | ~2.284 ms | **16.984%** | 16.797-17.547% |
| D7a | ~9.358 ms | ~8.125 ms | **13.144%** | 12.906-13.226% |
| D7b | ~7.551 ms | ~6.424 ms | **14.940%** | 14.891-15.014% |

Aggregate d=8 paired median:

~~~text
14.263%
~~~

Cross-corpus range:

~~~text
13.043-16.984%
~~~

Every corpus clears the predeclared 5% threshold and every per-process median is
positive.

## Guardrail result

Corpus-level paired-median ranges:

~~~text
d=3: ~5.732-6.034%
d=4: ~17.365-18.869%
d=5: ~12.002-14.108%
~~~

No d=3/4/5 corpus approaches the -2% regression guardrail.

d=0/1 are sub-microsecond noise controls and show expected timing noise. d=2 is
approximately neutral because the accepted D6 quadratic path is intentionally
unchanged.

## Interpretation

D7 selected scalar squaring because trace work was the stable common residual
bottleneck even though polynomial-degree dominance changed by corpus.

D8 validates that selection: unlike D6 quadratic specialization, the realized d=8
benefit is consistent across all five factor-tree shapes.

The result is empirical hosted performance evidence, not an SLA and not a theorem
about arbitrary workloads.

## D8 verdict

**ACCEPT the dedicated scalar GF(2^64) square for the private research decoder.**

Do not promote ExactSmallDelta to production/public status.

Do not immediately start another algebraic optimization. The next slice must
re-profile residual CPU after D8 and decide whether any remaining common decoder
cost is large enough to justify further work. If it is not, stop algebraic
micro-optimization and move to maintained-state system comparison:

- maintained PinSketch state;
- direct exact exchange;
- rateless-style reconciliation comparison.

Production/public ExactSmallDelta remains **NO-GO**.
