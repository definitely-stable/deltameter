# M6-C — Bit-equivalent Energy sign hashing evidence

Status: **ACCEPT**.  
Issue: #18. Parent: #14. PR: #22.  
Evidence date: 2026-10-06.

## Decision

Retain the private cached sign-mask implementation.

The candidate is algebraically equivalent to the existing cubic Energy sign hash, preserves snapshot-v1 bytes and public API, and reduces steady-state update cost by about 68–71% in paired hosted measurements.

The final implementation also replaces the initially expensive basis-by-basis mask builder with an exact multiply-by-x recurrence, reducing construction/decode overhead to a small absolute cost.

## Algebraic contract

For the existing polynomial basis let L(y) be bit zero. Multiplication by fixed c is GF(2)-linear.

Define:

~~~text
mask(c)[i] = L(c * x^i)
~~~

Then:

~~~text
L(c*y) = parity(mask(c) & y)
~~~

For the existing cubic sign polynomial:

~~~text
p(x) = c0 + c1*x + c2*x^2 + c3*x^3
~~~

the sign bit is computed as:

~~~text
L(c0)
xor parity(mask(c1) & x)
xor parity(mask(c2) & x^2)
xor parity(mask(c3) & x^3)
~~~

x^2 and x^3 are computed once per key, outside the row loop.

For B>1 the GF64 multiply count changes from:

~~~text
baseline  = 4R
candidate = R + 2
~~~

For default R=23: 92 -> 25 GF64 multiplications per update.

This operation count is explanatory; acceptance is based on measured behavior and differential correctness.

## Mask construction

The first candidate built each mask bit using a full gf64_mul(c, 1<<i), costing 192 full field multiplications per row.

The accepted implementation uses the identity:

~~~text
c*x^(i+1) = x * (c*x^i)
~~~

and advances one polynomial-basis product with a single shift plus the existing reduction constant 0x1B.

Each mask therefore costs 64 cheap multiply-by-x steps rather than 64 full field multiplications.

Tests compare the recurrence against full gf64_mul for every basis bit and multiple edge/full-width coefficients.

## Private state

EnergyDeltaMeter adds a private derived cache:

~~~text
3 u64 masks per row = 24R bytes
~~~

Representative raw cache sizes:

- default R=23: 552 bytes;
- small R=9: 216 bytes.

The cache:

- is not part of EnergyConfig or EnergyRowHash;
- is never serialized;
- is rebuilt on construction/decode;
- is cloned/reused by difference rather than recomputed.

## Correctness evidence

PASS:

- L(c*y) mask identity against direct field multiplication;
- all 64 polynomial basis vectors;
- zero/one/high-bit/all-ones/full-width coefficients;
- fast sign against the previous Horner sign oracle;
- every built-in Energy profile shape;
- signed-difference behavior;
- transactional overflow tests;
- post-decode continuation;
- existing unit/property tests;
- base/head snapshot-v1 compatibility: 13/13 tests pass on both revisions.

No change to:

- coefficient interpretation;
- GF polynomial/reduction;
- bucket hashing;
- profile dimensions;
- randomness/provenance contract;
- public API;
- snapshot-v1 bytes/semantics;
- Coverage::Proven.

## Paired performance evidence

Final M6-C hosted run: **m6c-performance #11 / 37517400587**.

~~~text
base  ce0179652cd596b8b597210577f16ee0433181ee
head  ee17c7e6d4620b223b4adfd3672a34926cdcdf2b
CPU   AMD EPYC 9V74 80-Core Processor
Rust  1.99.0 / LLVM 23.1.1
rounds 4, order AB / BA / AB / BA
samples per round 5
~~~

Artifact: `m6c-perf-37517400587-1`.

Source hashes are recorded in the artifact.

### Update

Paired median deltas:

| Profile | Content | Base ns/update | Head ns/update | Delta |
| --- | --- | ---: | ---: | ---: |
| default | full-width | 5431.635 | 1561.966 | -71.235% |
| default | sequential | 5460.135 | 1560.851 | -71.414% |
| small | full-width | 2106.601 | 673.222 | -68.083% |
| small | sequential | 2105.925 | 670.255 | -68.168% |

The gain is insensitive to sequential versus deterministic full-width keys in this harness.

### Construction and decode

| Metric | Profile | Base ns/op | Head ns/op | Delta |
| --- | --- | ---: | ---: | ---: |
| construct | default | 5,168 | 8,788 | +68.6% |
| construct | small | 501 | 2,424 | +384.3% |
| decode | default | 1,692,122 | 1,694,308 | +0.097% |
| decode | small | 167,125 | 169,091 | +1.227% |

Construction ratios look large because the baseline constructor is extremely small. Absolute added construction cost is about:

- default: 3.0 microseconds;
- small: 1.85 microseconds.

Using the full-width update savings, that overhead amortizes after roughly:

- default construction: 0.8 updates;
- small construction: 1.3 updates;
- default decode overhead: 1.4 continuation updates;
- small decode overhead: 1.3 continuation updates.

Therefore the accepted cache is effectively amortized immediately for ordinary non-empty sketches.

### Difference and query controls

Paired medians remain near baseline:

- default difference: -1.111%;
- small difference: +0.311%;
- default query: -1.349%;
- small query: -2.873%.

These controls are not performance claims; they show no material regression from the private cache.

## Memory trade-off

The derived cache adds 24R bytes:

- 552 bytes for default R=23 versus 376,832 primary counter bytes;
- 216 bytes for small R=9 versus 36,864 primary counter bytes.

This is private derived memory, not persisted state.

## Verdict

**ACCEPT M6-C.**

Reasons:

1. exact differential equivalence is tested against the Horner oracle;
2. snapshot-v1 compatibility is preserved;
3. update latency improves by roughly 3.1–3.5x;
4. optimized mask construction makes setup/decode overhead negligible after about 1–1.4 updates;
5. difference/query controls show no material regression;
6. memory overhead is small and derived only;
7. no unsafe, SIMD, nightly or dependency expansion is required.

The earlier 192-full-multiply-per-row builder is rejected and superseded by the multiply-by-x recurrence.

## Handoff

M6-C removes a major Energy update/build bottleneck observed during M6-A.

Next work should continue the independent M6-D exact-lane source/failure-semantics audit and evaluate M6-B codec/decoder improvements separately. Neither changes the M6-A generic-workflow NO-GO by itself.
