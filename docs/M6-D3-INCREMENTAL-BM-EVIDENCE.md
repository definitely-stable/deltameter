# M6-D3 — Incremental Berlekamp–Massey evidence

Status: **NO-GO as a performance optimization; retain only as correctness/diagnostic evidence.**  
Issue: #28. Parent: #19 / #14. PR: #29.  
Evidence date: 2026-10-07.

## Decision

Incremental Berlekamp–Massey state reuse is mathematically correct but does not materially reduce D2 retry CPU.

The frozen D2 protocol remains unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

D3 retains C/B/L/m/b plus the processed syndrome prefix and processes only newly available syndrome terms. Locator polynomials are byte-for-byte identical to fresh BM at every audited stage.

However, BM is not the dominant cost.

## Measured revision

Canonical hosted run:

~~~text
m6d3-incremental-bm #5 / 37567202464
head  872c4ee818777fcc26d722e68de74a2aed2750c6
CPU   AMD EPYC 7763 64-Core Processor
Rust  1.99.0 / LLVM 23.1.1
runs  3
source set size 8192
~~~

Artifact: `m6d3-incremental-bm-37567202464-1`.

Artifact digest:

~~~text
sha256:43d447d484ff36631fee573fe1b9c5a0051754758ea07a44171e9a0c860953fd
~~~

Source SHA256 values:

~~~text
m6d3_bench.rs
58ad0f1f17604e800b495fdd97fba02404b532837e54680b48c1426815576fb2

m6d3_incremental_bm.rs
1f17cedc130c0863dfc2e4eed0134ac362e108299ecbe6d6097f71e405371972

m6d_pinsketch64.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4

m6d3_incremental_bm test
22da26b6291ffd1903478c52c16262234c6fc64d4f5fe2aa79e5804dd10af8b7

m6d3_summary.py
840843439caa9e3a87855ee69e758387e755546cf2c79259f62ebba56275d9c5
~~~

Fail-closed summary:

~~~text
logical_gate=PASS
false_success_total=0
~~~

The ordinary Rust and Research workflows also pass on the exact measured head.

## Correctness result

D3 proves for every audited stage:

- reconstructed incremental syndrome sequence equals fresh reconstruction;
- incremental BM locator equals fresh BM locator;
- D3 candidate equals the frozen D1 decoder candidate;
- every d<=8 successful candidate equals the exact symmetric-difference oracle;
- d=9/10/16 remains reject in the frozen matrix;
- historical syndrome mutation fails closed;
- zero metadata mutation fails closed;
- full-width u64, high-bit values, zero and u64::MAX remain valid.

No D1/D2 production or evidence source file is modified.

## CPU result

Median hosted decoder results:

| d | frozen incremental D2 | D3 incremental BM | change | BM share of D3 |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 0.624 ms | 0.623 ms | -0.279% | 2.760% |
| 3 | 3.039 ms | 3.042 ms | +0.113% | 0.871% |
| 4 | 11.374 ms | 11.433 ms | +0.519% | 0.307% |
| 5 | 94.354 ms | 94.830 ms | +0.505% | 0.050% |
| 8 | 106.314 ms | 106.916 ms | +0.566% | 0.069% |

d=0/1 are sub-microsecond cases where relative percentages are noise-sized in absolute terms.

For d=8:

~~~text
incremental BM append         ~74 us
factor + candidate verification ~106.840 ms
total candidate                ~106.916 ms
~~~

So the retained BM work is less than one tenth of one percent of total decoder CPU.

## Why Candidate A is NO-GO

The D2 retry tax looked like repeated decoding, but phase decomposition shows that retry BM work is not the bottleneck.

State reuse removes only a tiny amount of computation, while each attempt still repeats root factorization and syndrome-based candidate verification.

Observed end-to-end changes are within roughly ±0.6%, with several retry cases slightly slower.

Therefore D3 does not justify additional complexity in the retained reference path.

The incremental BM implementation remains useful as a correctness oracle and for any future decoder architecture that can also reuse root-finding state.

## Candidate B — cached even syndromes

Do **not** implement the previously proposed even-syndrome cache as a separate optimization slice.

Measured fresh syndrome reconstruction at the largest audited stages is only about 1.3–1.4 us cumulative, versus roughly 100 ms root-factor/verification cost.

The maximum possible gain is therefore immaterial before measurement noise and implementation overhead.

This is an evidence-based skip, not an untested assumption.

## Dominant bottleneck

The next optimization target is root finding.

The D1/D3 reference uses deterministic absolute-trace splitting with:

- 64 Frobenius iterations per trace;
- generic polynomial multiplication for squaring;
- generic polynomial division/modulus;
- scalar bit-by-bit GF(2^64) multiplication.

The official Minisketch implementation likewise identifies root finding as the expensive decoder stage and uses Berlekamp trace root finding, explicit quadratic handling, fast squaring and field-specific optimizations.

For DeltaMeter's safe-Rust research boundary, the first isolated candidate should preserve the same deterministic factorization semantics and optimize only trace squaring/modulus.

## Next candidate

Start M6-D4 with a bit-equivalent polynomial-squaring specialization:

~~~text
(a0 + a1*x + ... + an*x^n)^2
=
a0^2 + a1^2*x^2 + ... + an^2*x^(2n)
~~~

in characteristic two.

Candidate:

1. compute only squared coefficients; cross terms vanish exactly;
2. reduce by the already-monic modulus without computing an inverse of 1;
3. use it only inside trace/Frobenius steps first;
4. differential-test against the frozen generic multiply+divide implementation for all audited degrees and deterministic full-width coefficient grids;
5. measure root-factor and end-to-end D2 retry CPU separately.

Do not combine this with CLMUL, unsafe SIMD, randomized splitting or low-degree closed forms in the same experiment.

## D3 verdict

**NO-GO Candidate A for performance.**

Keep D2 protocol and D1 correctness reference.

Skip cached-even-syndrome Candidate B.

Proceed to an isolated safe-Rust root-factorization optimization candidate.
