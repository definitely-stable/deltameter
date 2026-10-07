# M6-D4 — Characteristic-2 trace-square specialization

Status: implementation in progress. Issue: #30. Parent: #19 / #14.

## Goal

Reduce the dominant D2/D3 root-factorization CPU without changing the reconciliation protocol or factorization algorithm.

D3 established:

- incremental Berlekamp–Massey reuse is correct;
- BM append is negligible at large d;
- factorization + candidate verification dominates decoder CPU.

D4 Candidate A replaces only the Frobenius square/modulus operation used inside deterministic Berlekamp trace splitting.

## Frozen baseline

Keep unchanged:

~~~text
D2 stage k             1   2   4   8
stored syndromes       2   3   5   9
cumulative payload    17  25  41  73 bytes
attempts / RTTs        1   2   3   4
~~~

Also frozen:

- D1 exact-u64 field mapping;
- out-of-band zero bit;
- GF(2^64) reduction polynomial 0x1B;
- D3 locator polynomial;
- deterministic trace split coefficients 1<<bit;
- guard revalidation against all stored syndromes;
- public API/snapshot/Coverage boundaries.

## Algebraic candidate

For polynomial P(x) over GF(2^64), characteristic two gives:

~~~text
P(x)^2 = sum_i a_i^2 x^(2i)
~~~

because every cross term appears twice and cancels.

The frozen generic trace step computes:

~~~text
poly_mul_mod(P, P, M)
~~~

which:

- computes all i,j coefficient pairs, including cross terms that cancel;
- runs generic polynomial division;
- computes inverse(leading(M)) although trace factorization passes an already-monic M.

Candidate A implements:

~~~text
poly_square_mod_monic(P, M)
~~~

with:

1. coefficient squares only at even exponents;
2. exact reduction against a monic modulus;
3. no inverse(1);
4. no multiply by the monic leading coefficient;
5. no change to field multiplication itself.

## Correctness gates

Differential square/mod:

- modulus degrees 1..8;
- monic deterministic moduli;
- polynomial degrees below modulus degree;
- coefficient corpus: 0, 1, 2, high bit, u64::MAX, fixed full-width values;
- deterministic mixed/random-looking vectors;
- candidate == frozen generic poly_mul_mod(P,P,M);
- output canonical degree < modulus degree.

Root equivalence:

- specialized trace factor roots equal frozen D3 roots;
- locator inputs from d=0,1,2,3,4,5,8 and reject cases d=9,10,16;
- zero/high-bit/u64::MAX cases;
- candidate verification remains identical;
- false-success remains zero in the frozen matrix.

## Measurement

Hosted release evidence separates:

- generic square/mod micro-cost;
- specialized square/mod micro-cost;
- frozen D3 root+verify;
- D4 root+verify;
- frozen D2/D3 cumulative retry decode;
- D4 cumulative retry decode.

Measure on the same immutable 8192-key workload matrix.

## Acceptance

ACCEPT requires:

- all differential and exact-oracle gates pass;
- D2 bytes/RTTs unchanged;
- reproducible material improvement in root+verify and cumulative decoder CPU.

NO-GO is valid if compiler/runtime effects erase the algebraic saving.

## Exclusions

No unsafe, SIMD, CLMUL, randomized splitting, quadratic formulas, alternative field representation, public API, snapshot-v1, Coverage change, FFI or runtime dependency in this candidate.

## Review corrections and implementation gates

- Preserve D1/D2/D3 files byte-for-byte. Use the existing D1 decode seam as an
  independent untimed control on every benchmark scenario/stage, isolated in
  `validate_control`; no duplicated sketch representation or lint suppression.
- Four balanced AB/BA samples per process, three complete process runs; both
  decoder paths warmed before measurements. Fixed D4 corpus seeds are shared
  between arms, but differ from D2/D3 seeds: cross-slice timings are not paired.
- Validate exact metadata and complete per-run matrices before printing PASS.
  Reject invalid counts/timings and test deliberate corruptions in hosted CI.
- Report medians of paired phase sums, paired reductions and per-run median
  ranges. Separately time complete cumulative decodes including prefix/merge,
  fresh locator, root factorization, guard verification and retries; source
  construction and network transport are outside this maintained-state scope.
- Add direct edge-coefficient, canonical trailing-zero and guard-mutation tests.
- Require exact-head Rust (including release example build), Research and D4
  hosted runs before the verdict; record source hashes and docs-only provenance.
