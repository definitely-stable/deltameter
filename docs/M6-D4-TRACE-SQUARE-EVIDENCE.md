# M6-D4 — Characteristic-2 trace-square evidence

Status: **ACCEPT D4 Candidate A for private research; production/public ExactSmallDelta remains NO-GO.**
Issue: #30. Parent: #19 / #14. PR: #31. Evidence date: 2026-10-07.

## Decision and scope

Accept the isolated safe-Rust characteristic-2 square plus monic reduction inside
Frobenius trace splitting. The final paired experiment shows material root/decoder
improvement after correcting the benchmark and strict-Clippy blocker.

The D2 protocol is frozen:

| decode limit k | 1 | 2 | 4 | 8 |
| --- | ---: | ---: | ---: | ---: |
| stored syndromes | 2 | 3 | 5 | 9 |
| cumulative payload bytes | 17 | 25 | 41 | 73 |
| attempts / RTTs | 1 | 2 | 3 | 4 |

Exact nonzero u64-to-field identity, the separate zero bit, GF polynomial 0x1B,
all-syndrome guard checks, public API, snapshot-v1 and Coverage are unchanged.
No unsafe, intrinsics, CLMUL/SIMD, randomized splitting, quadratic solver,
field-representation change, FFI or dependency was introduced.

## Canonical provenance

Measured implementation head: `cf6434d4007825b8fc05ebc9b064ce4860d4442b`.
Base main: `63736303eef75289fde60714010d2cf79a5ee4ab`.

All three GitHub-hosted workflows passed and checked out the exact implementation
head, rather than a synthetic PR merge ref:

- [D4 #6 / 37578505762](https://github.com/definitely-stable/deltameter/actions/runs/37578505762)
- [Rust #211 / 37578505725](https://github.com/definitely-stable/deltameter/actions/runs/37578505725)
- [Research #224 / 37578505740](https://github.com/definitely-stable/deltameter/actions/runs/37578505740)

CPU: AMD EPYC 9V45 96-Core Processor; hosted VM exposes 4 CPUs.
Rust: 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`), LLVM 23.1.1,
x86_64-unknown-linux-gnu; release build, ubuntu-latest. See the preserved
[cpu](research/evidence/m6d4-37578505762/cpu.txt) and
[compiler](research/evidence/m6d4-37578505762/rustc.txt) records.

Artifact: `m6d4-trace-square-37578505762-1`, ID `11463263845`.
ZIP SHA256: `bfd8f5d400979957e783078a2ea21ec1f8ad3af936b85893112dba5326a7fb9a`.
The downloaded ZIP digest, head and every source hash were verified; rerunning the
summarizer reproduced `summary.txt` byte-for-byte. Raw runs and provenance are
preserved in [the repository](research/evidence/m6d4-37578505762/) so artifact
expiry does not remove the evidence.

Documentation/evidence commits after this measured head change no implementation,
control, benchmark, test, summarizer or workflow source. Later docs-triggered runs
are integration checks; this first passing final-implementation run remains canonical.
Earlier runs #3 and #4 are superseded, not pooled into this result.

Source SHA256 inventory:

~~~text
9176c4d792df2ada20078c26b3335d4295da070faa32472fac82bd0c03196adb  examples/m6d4_bench.rs
162a11e7feffb67797786c247e9a1b9a6e4108f5da3e5648ea34b70f94f98b9e  examples/support/m6d4_trace_square.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4  examples/support/m6d_pinsketch64.rs
1f17cedc130c0863dfc2e4eed0134ac362e108299ecbe6d6097f71e405371972  examples/support/m6d3_incremental_bm.rs
994c2788b3859a431a6b79604def969504302dd12e8367314fabbb88340d3cc3  tests/m6d4_trace_square.rs
b60be5801680e7dfd7c599daa849aa4e127cbd40d8cd0a2de40ac235a866efd8  research/m6d4_summary.py
2a08ea2de8da33e13718c554d4cca0e2d34f1ae9040b5d174add63ac5ce2862f  research/test_m6d4_summary.py
82d699c18f40647dd4ee84fa1781ccc877932038568eb6254b62c53c04601c06  .github/workflows/m6d4.yml
~~~

Git-tree comparison against base main found no changes in D1/D2/D3 reference,
test, benchmark or evidence files, or in `src/` and Cargo files. D1 and D3 support
hashes above also match their historical canonical evidence.

## Correctness and causal review

In characteristic two, each cross term in P*P occurs twice and cancels exactly.
The candidate writes coefficient squares only at even exponents. At each monic
reduction step the leading term cancels by XOR with itself, without inverse(1),
and the trimmed remainder strictly drops in degree. Hence the result is canonical
and has degree less than the nonconstant monic modulus. Constant modulus 1 yields
the zero remainder; invalid nonmonic/zero moduli are rejected.

The copied generic arithmetic, fresh BM and deterministic factor/trace bodies were
compared to D1; they differ only in error wrapping/function names. Generic and
specialized recursive factor bodies are identical apart from arm names. The
candidate changes only the Frobenius square/mod operation. Both paths still sort,
check distinct/nonzero roots, enforce the decode budget, restore zero, rebuild the
candidate and compare **all** stored syndromes. An independent exact set oracle
checks successful candidates outside timed regions.

Coverage includes:

- 1,408 mixed full-width polynomial/modulus cases across degrees 1..8, plus
  200 direct edge-coefficient cases including zero, one, high bit and u64::MAX;
- canonical remainder/trailing-zero and invalid-modulus checks;
- D1/D3 locator/candidate equivalence, d=0/1/2/3/4/5/8, zero and full-width keys;
- d=9/10/16 stage-by-stage rejection and a mutated extra guard syndrome;
- frozen validation/error precedence before factorization, including invalid
  limit, zero budget and excessive locator degree.

The last regression first failed on hosted run 37578396298 with D4 DecodeFailure
versus frozen InvalidDecodeLimit, then passed after the shared pre-factor check.
Final D4 release tests: 14 passed, 0 failed (including six frozen D1 module tests).
Rust CI passed fmt, strict `clippy --all-targets -- -D warnings`, all-target tests,
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`, and `cargo build --release --examples`.

The Clippy fix uses the existing frozen D1 decoder as an independent **untimed**
control through `validate_control`, on every stage of every measured scenario.
This avoids duplicating sketch state or adding D4 APIs to frozen D1. No dead-code
allow/expect or lint weakening was added. The D4 generic adapter remains isolated
and agrees with frozen D1 Results on the benchmark corpus.

## Workload and measurement

Three fresh benchmark processes, four balanced AB/BA paired samples per process:
12 pairs per case, all included. Timings use single-threaded `Instant` elapsed
time, not OS process-CPU counters. Both complete decoder paths are warmed before
measurement; square microbenchmarks use 128 operations per sample. Source sketches
are constructed outside decoder timers. The left set has 8,192 keys; the right has
8,192 for even d and 8,193 for odd d (including the zero-only d=1 case). Both arms use
the same immutable D4 source sets. D4 seeds differ from D2/D3: cross-slice absolute
numbers are not paired comparisons.

Two measurements are deliberately separate:

1. Phase measurement: common fresh syndrome reconstruction/BM locator and each
   root+guard-verification path, accumulated over D2 retries. `decode` summary
   phase sums are medians of per-pair sums, not sums of independent medians.
2. Complete maintained-state decoder: independently timed prefix extraction/merge,
   fresh locator, factorization, candidate guard verification and retry loop.
   `total` summary records supply the decoder numbers below. They exclude source
   construction, oracle comparison, transport, RTT delay and production verification.

Strict summarization requires exactly three named runs, exact metadata, the complete
per-run matrix with four samples per case, positive timings and valid operation
counts, frozen outcomes/bytes/RTTs, and zero false-success before printing PASS.
Six regression tests include missing/duplicated rows, missing run, invalid metadata,
negative/zero timings, zero operation counts and non-additive median examples.

Percent reductions below are medians of **paired** ratios. The ranges show the
minimum/maximum of the three per-process paired-median reductions; they are not
confidence intervals and do not establish population-level performance promises.

## Square/mod cost

| modulus degree | generic ns/op | specialized ns/op | paired reduction | per-run median range |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 5677.410 | 198.930 | 96.491% | 96.480–96.534% |
| 4 | 6779.973 | 720.457 | 89.402% | 88.975–89.423% |
| 8 | 11078.742 | 2895.215 | 73.918% | 73.807–73.952% |

## Decoder phase costs

All values in milliseconds, including tiny d=0/1 values. Locator includes fresh
syndrome reconstruction and BM; root columns include candidate guard verification.

| d | result | locator/BM | generic root+verify | specialized root+verify |
| ---: | --- | ---: | ---: | ---: |
| 0 | exact | 0.000091 | 0.000050 | 0.000056 |
| 1 | exact | 0.000035 | 0.000050 | 0.000065 |
| 2 | exact | 0.016524 | 0.380588 | 0.029824 |
| 3 | exact | 0.033530 | 25.241926 | 1.807962 |
| 4 | exact | 0.038748 | 7.865712 | 0.465576 |
| 5 | exact | 0.069284 | 37.167656 | 4.062126 |
| 8 | exact | 0.086355 | 48.309288 | 6.445502 |
| 9 | rejected | 0.086240 | 100.033161 | 21.090316 |
| 10 | rejected | 0.087572 | 89.079145 | 10.162871 |
| 16 | rejected | 0.086365 | 97.729856 | 19.400639 |

## Complete cumulative decoder

| d | generic ms | specialized ms | paired reduction | per-run median range |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0.000210 | 0.000206 | 0.226% | -4.149–2.381% |
| 1 | 0.000181 | 0.000186 | -2.486% | -8.387–2.998% |
| 2 | 0.394088 | 0.046449 | 88.195% | 88.179–88.208% |
| 3 | 25.175563 | 1.827811 | 92.743% | 92.739–92.743% |
| 4 | 7.896767 | 0.506112 | 93.595% | 93.579–93.635% |
| 5 | 37.295800 | 4.145585 | 88.849% | 88.815–89.073% |
| 8 | 48.327663 | 6.574828 | 86.384% | 86.263–86.513% |
| 9 | 100.169541 | 21.139910 | 78.997% | 78.864–79.144% |
| 10 | 89.427720 | 10.255489 | 88.474% | 88.415–88.541% |
| 16 | 98.077887 | 19.520558 | 80.084% | 79.994–80.231% |

For d=8, complete decoder medians are **48.328 -> 6.575 ms**; paired reduction
**86.384%**, with per-run medians **86.263–86.513%**. The CPU model differs
from earlier runs, so cross-run absolute differences cannot be attributed to an
additional algorithmic gain. d=0 and zero-only d=1 do not exercise trace splitting;
sub-microsecond differences and percentages there are timer/noise-scale controls.

## False-success inventory

`logical_gate=PASS`, `false_success_total=0`.

Per timed mode (phase or complete), each arm has 120 final observations: 84 exact
and 36 rejected, across 360 attempted stages. Two arms therefore give 240 final
observations per mode. These are **repetitions of ten deterministic workloads**,
not 240 independent input trials. The three distinct final over-bound workloads
(d=9/10/16) each reject at all four stages; earlier stages of in-bound workloads
also reject until the frozen admitted limit. Untimed D1 control additionally checks
all four stages, even after a smaller stage succeeds.

The D1 0/896 guarded inventory remains historical evidence, not a new D4 sample
or a theorem. Reusing syndromes is not independent final verification. This result
does not establish a 2^-64 adversarial false-positive bound or a production contract.

## Residual bottleneck and next decision

At d=8, specialized root+verify is 6.446 ms versus 0.086 ms locator/BM: about 98.7%
of the phase sum. This is a phase decomposition, not a sampled CPU call-stack
profile; it does not identify the dominant internal factor degree. Verification
was retained, but is not separately timed in this slice. A D5 profiling gate should
separate it before asserting that a quadratic solver addresses most residual time.

Recommend exactly one next candidate after D4 merge: **safe-Rust deterministic
quadratic specialization**, with a profile-first go/no-go gate. Measure factor calls
and CPU by degree plus candidate verification on the frozen D4 corpus, then implement
only the degree-two solver if its measured share makes the end-to-end benefit material.
Keep higher-degree splitting, field multiplication and all protocol/guard semantics
fixed. If the share is negligible, stop that candidate and move to a maintained-state
system end-to-end comparison; do not automatically add another micro-optimization.

D3 BM reuse and even-syndrome caching remain unjustified. Field multiplication,
trace reuse and especially unsafe/CLMUL/SIMD are not bundled into the next candidate.
The maintained-state latency is greatly reduced, but D4 alone cannot establish
practical product crossover or production readiness.
