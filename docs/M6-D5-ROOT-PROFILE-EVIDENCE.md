# M6-D5 — Root-factor degree and verification profile

Status: **PROFILE COMPLETE; deterministic quadratic specialization is GO-to-experiment.**  
Issue: #32. Parent: #19 / #14. PR: #33.  
Evidence date: 2026-10-07.

## Decision

D5 does not change the accepted D4 decoder. It instruments an isolated copy of the
same specialized root path to determine where the residual CPU goes.

The result justifies exactly one next optimization experiment:

**a deterministic safe-Rust degree-two root solver.**

This is not an ACCEPT of that solver in advance. D5 only establishes that degree-two
self work is large enough to deserve a separate causal experiment.

Verification is not the next target.

## Frozen contract

D2/D4 remain unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also unchanged:

- exact nonzero u64 -> GF(2^64) identity mapping;
- out-of-band zero bit;
- GF reduction polynomial 0x1B;
- one guard syndrome;
- fresh locator/BM;
- D4 characteristic-2 trace square;
- deterministic trace splitting;
- all-syndrome candidate guard;
- public API, snapshot-v1 and Coverage.

No root algorithm is changed in D5.

## Canonical provenance

Measured implementation head:

`0c86781079f6b88979b3e7ae0730cabe6db12cdd`

Base main:

`cd312660691ccfd542ffcb5d6d98684c1b686f9c`

Exact-head GitHub-hosted gates:

- M6-D5 root profile #8 / `37581499080` — SUCCESS
- Rust #226 / `37581499072` — SUCCESS
- Research #239 / `37581499073` — SUCCESS

Artifact:

`m6d5-root-profile-37581499080-1`

Artifact ID:

`11464807805`

Artifact SHA256:

`a379dd7067b39b1497831767f023773414360fa46bb878cc374b31d9df347b10`

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
5525087f5f5ec01d7ec1d4712890745757e2b48b5162f9c111002c5b85999c79  examples/m6d5_bench.rs
c6c27f1b0aa8c3f8f1b722551605870f8c296629e75d6689da7d9dcb5a8fc203  examples/support/m6d5_root_profile.rs
162a11e7feffb67797786c247e9a1b9a6e4108f5da3e5648ea34b70f94f98b9e  examples/support/m6d4_trace_square.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4  examples/support/m6d_pinsketch64.rs
25559f0dbe49c6d55251c686e17395e3506711ac5094c27533aa8a1fac0470bf  tests/m6d5_root_profile.rs
c0e1b8c9989af6cd866e552dd7b6153d217e9c80d009657445035e0bee9d3c8d  research/m6d5_summary.py
4ebb45c0cc00a265cd8e233b81cf098b8f52156afcd5a17afae8daa1019787e6  research/test_m6d5_summary.py
eaa443d828ef51723be4e298dd481ab3bae9af55f35f9bf2f899f201c6ac8db5  .github/workflows/m6d5.yml
~~~

Fail-closed summary:

~~~text
logical_gate=PASS
false_success_total=0
~~~

## Profiling method

Three fresh hosted processes, four samples per scenario, for 12 observations per
scenario.

The accepted D4 decoder is timed separately as an **uninstrumented control**.

The D5 path is an instrumented copy used only to decompose work. Its wall time is
not treated as the speed baseline.

For each recursive factor frame D5 records non-overlapping local work by current
polynomial degree:

- factor call count;
- trace split attempts;
- characteristic-2 square/mod calls;
- trace-loop self time;
- GCD self time;
- successful quotient/division self time.

Recursive child time is not charged to the parent degree bucket.

Candidate rebuild plus all-syndrome verification is measured separately.

Untimed controls compare:

- frozen D1 decoder;
- D4 generic decoder;
- accepted D4 specialized decoder;
- D5 profiled decoder.

The outputs/errors must agree on the frozen corpus before measurements are accepted.

## Main result

Median results:

| d | D4 control | factor wall | degree-2 self | degree-2 / control | verification / control |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 0.044 ms | 0.028 ms | 0.028 ms | **62.8%** | 1.20% |
| 3 | 0.172 ms | 0.138 ms | 0.089 ms | **51.7%** | 0.79% |
| 4 | 0.491 ms | 0.458 ms | 0.201 ms | **40.9%** | 0.34% |
| 5 | 5.233 ms | 5.146 ms | 0.188 ms | **3.6%** | 0.06% |
| 8 | 7.137 ms | 7.125 ms | 1.964 ms | **27.5%** | 0.06% |

For d=8:

~~~text
accepted D4 complete control     ~7.137 ms
profiled factor wall             ~7.125 ms
degree-two local self work       ~1.964 ms
degree >= 3 local self work      ~5.130 ms
candidate verification           ~0.0039 ms
~~~

Degree-two therefore represents approximately:

- **27.6% of factor wall time**;
- **27.5% of accepted D4 control latency**.

Higher-degree work is still larger, about 72% of factor wall time.

The theoretical maximum from deleting all measured degree-two local work is therefore
roughly one quarter of current d=8 latency, not another order-of-magnitude gain.

## Operation structure at d=8

Median deterministic operation counts over the complete retry schedule:

~~~text
degree 2:
  factor calls      3
  trace attempts   81
  square calls   5184

degree 3:
  factor calls      3
  trace attempts   77
  square calls   4928

degree 4:
  factor calls      2
  trace attempts   16
  square calls   1024

degree 5:
  factor calls      1
  trace attempts    6
  square calls    384

degree 8:
  factor calls      1
  trace attempts    1
  square calls     64
~~~

This shows that degree-two work is repeated frequently enough to be material even
though higher-degree work remains the majority.

## Verification result

Verification is decisively not the current bottleneck.

For d=8, candidate rebuild + all-syndrome verification is roughly:

`3.9 us`

or about:

`0.055%`

of accepted D4 control latency.

Therefore no verification optimization is justified before the quadratic experiment.

## Other workloads

Degree-two share of accepted D4 control:

~~~text
d=2   62.8%
d=3   51.7%
d=4   40.9%
d=5    3.6%
d=8   27.5%
~~~

The low d=5 share is real for this deterministic factor tree: degree-four work
dominates that workload.

Over-bound rejects also show nontrivial degree-two work, but production decisions
must not be based on reject-path speed alone.

d=0/1 are sub-microsecond noise-scale controls and do not exercise nontrivial root
splitting.

## Instrumentation sanity

For meaningful workloads the instrumented path remains close to the accepted D4
control. At d=8 the profile overhead is about 1.1%.

Every non-base trace frame satisfies:

~~~text
square_calls = 64 * trace_attempts
~~~

Operation counts are deterministic across the repeated runs.

The profile is therefore suitable for bottleneck selection, while the uninstrumented
D4 path remains the only performance baseline.

## D5 verdict

**GO-to-experiment for one deterministic quadratic specialization.**

Do not merge a quadratic solver into D5.

The next candidate must:

1. replace only degree-two factor resolution;
2. leave degree >= 3 trace splitting unchanged;
3. leave D4 characteristic-2 square/mod unchanged;
4. preserve exact roots, rejection/error order, guard verification and D2 protocol;
5. compare against the accepted D4 path with balanced paired hosted evidence;
6. report the realized end-to-end gain against the D5 upper-bound expectation.

A reasonable expectation is a bounded improvement, not a 7x result. On the current
d=8 corpus, the measured removable degree-two local work is about 27.5% of accepted
D4 latency.

If the solver does not produce a reproducible material end-to-end gain, reject it
and stop that candidate.

Unsafe/CLMUL/SIMD, field-representation changes and multiple root optimizations remain
out of scope.
