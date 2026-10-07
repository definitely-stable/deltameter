# M6-D11 — Fixed-constant polynomial reduction evidence

Status: **ACCEPT for private research.**  
Issue: #45. Parent: #19 / #14. PR: #46.  
Evidence date: 2026-10-07.

## Decision

D11 changes only multiplication inside degree>=3 trace polynomial reduction.

Accepted D8 remains the control.

For each monic factor frame, D11 builds an ephemeral fixed-constant
`ReductionPlan` for the modulus coefficients. Variable reduction factors are
multiplied by those fixed coefficients with positional nibble tables rather than the
frozen 64-step generic multiplier.

The predeclared end-to-end performance gate passes by a wide margin.

Production/public ExactSmallDelta remains **NO-GO**.

## Canonical provenance

Measured implementation head:

`fad05e10a070d790259cd39a2b23c2ecfc7e0091`

Base main:

`a0c860bd8071927e0f586cb00371ebb25af9ed0f`

Exact-head GitHub-hosted gates:

- M6-D11 fixed reduction #4 / `37598918009` — SUCCESS
- Rust #298 / `37598918119` — SUCCESS
- Research #311 / `37598918189` — SUCCESS

Artifact:

`m6d11-fixed-reduction-37598918009-1`

Artifact ID:

`11471822408`

Artifact SHA256:

`739df65de3ecaf6aef77cfbf6e64ca5b298b7ead474b0349c9d621ed97c5db6c`

Fail-closed summary:

~~~text
runs=3
corpora=5
logical_gate=PASS
false_success_total=0
performance_gate=PASS
performance_gate_reasons=none
~~~

## Frozen semantics

Unchanged:

- accepted D8 scalar GF(2^64) square;
- D6 deterministic degree-two solver;
- degree>=3 factor tree and trace coefficient sequence;
- polynomial reduction shape;
- generic GF(2^64) multiplication outside trace reduction;
- GCD and division;
- field representation and reduction constant `0x1B`;
- locator construction;
- k=1/2/4/8 attempt schedule;
- cumulative payload 17/25/41/73 bytes;
- one guard syndrome;
- exact nonzero-u64 + separate-zero mapping;
- all-syndrome verification;
- public API, snapshot-v1 and Coverage.

No unsafe, SIMD, CLMUL, FFI, nightly or new dependency is introduced.

## Candidate construction

Multiplication by a fixed field element is GF(2)-linear.

For each fixed modulus coefficient D11 constructs 64 basis images. Basis image zero
is the coefficient itself and each next image is obtained by one multiply-by-x
step in the frozen polynomial basis:

~~~text
carry = value >> 63
value <<= 1
if carry != 0:
    value ^= 0x1B
~~~

The 64 basis images are folded into:

~~~text
16 nibble positions
x 16 nibble values
x 8 bytes
= 2048 bytes per fixed coefficient
~~~

A variable multiplier is then evaluated with exactly 16 indexed table values and
XORs.

At maximum audited degree 8, one complete factor-frame plan therefore contains at
most about 16 KiB of table payload plus small Vec/coefficient overhead. The parent
plan is explicitly dropped before recursive child factorization so table memory does
not accumulate with factor-tree depth.

This memory trade-off is accepted only for the private research decoder. It is not
a production API/layout decision.

## Correctness

The fixed multiplier matches the frozen generic GF(2^64) multiplication for:

- all 64 variable basis vectors across edge/deterministic constants;
- 4096 deterministic full-width variable/constant pairs.

The complete D11 decoder matches accepted D8 result/error semantics at every k
stage across all five corpora and:

~~~text
d = 0,1,2,3,4,5,8,9,10,16
~~~

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

Zero, high-bit, u64::MAX and invalid-limit/guard behavior remain frozen.

False-success total is zero in the repeated deterministic matrix.

## Frozen performance gate

Recorded before the first candidate run:

- every d=8 corpus paired median >= 10.0%;
- aggregate d=8 paired median >= 15.0%;
- every d=8 per-process paired median > 0%;
- no d=3/4/5 corpus paired median < -2.0%.

d=0/1/2 are control/noise lanes.

## d=8 result

Paired end-to-end reductions versus accepted D8:

| Corpus | D8 control | D11 candidate | Paired reduction | Per-process median range |
| --- | ---: | ---: | ---: | ---: |
| D4 | ~8.208 ms | ~3.972 ms | **51.614%** | 51.596-51.680% |
| D5 | ~6.660 ms | ~3.606 ms | **45.916%** | 45.814-46.126% |
| D6 | ~2.286 ms | ~0.777 ms | **66.087%** | 65.927-66.187% |
| D7a | ~8.118 ms | ~3.959 ms | **51.228%** | 50.530-51.401% |
| D7b | ~6.403 ms | ~3.372 ms | **47.342%** | 47.308-47.500% |

Aggregate d=8 paired median:

~~~text
51.166%
~~~

Cross-corpus range:

~~~text
45.916-66.087%
~~~

Every corpus exceeds the frozen 10% threshold and every per-process median is
strongly positive.

## Guardrails

d=3 corpus-level paired medians:

~~~text
12.688-12.835%
~~~

d=4:

~~~text
48.757-50.292%
~~~

d=5:

~~~text
37.730-46.211%
~~~

No d=3/4/5 workload approaches the -2% regression guardrail.

## Interpretation

D10 measured net polynomial reduction at 88.299-92.428% of accepted D8 square/mod.
D11 directly attacks only that measured work and realizes a large, consistent
end-to-end effect across all five factor-tree shapes.

Unlike a general field-arithmetic rewrite, the candidate exploits a property local
to trace polynomial reduction: the modulus coefficients remain fixed while the
leading reduction factor changes.

The result is hosted empirical performance evidence, not an SLA and not evidence
that this table representation should become public or production state.

## D11 verdict

**ACCEPT the fixed-constant polynomial reduction plan for the private research
decoder.**

Do not immediately select another algebraic optimization.

The next M6-D slice must re-profile the accepted D11 decoder across the same five
corpora. That profile must include reduction-plan construction separately from
residual trace/GCD/division/verification work.

If no common material residual phase remains, stop algebraic decoder
micro-optimization and move to maintained-state system comparison.

Production/public ExactSmallDelta remains **NO-GO**.
