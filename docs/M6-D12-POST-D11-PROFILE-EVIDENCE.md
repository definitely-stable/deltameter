# M6-D12 — Post-D11 whole-decode residual profile

Status: **PROFILE COMPLETE; STOP algebraic decoder micro-optimization.**  
Issue: #48. Parent: #19 / #14. PR: #50.  
Evidence date: 2026-10-07.

## Decision

D12 re-profiles the complete accepted D11 staged decoder after the fixed-constant
trace-reduction optimization.

No accepted decoder algorithm is changed.

The predeclared selector required a phase to account for at least 25% of accepted
D11 d=8 control latency in **every one of the five frozen corpora** before another
narrow algebraic measurement slice could be authorized.

Measured verdict:

~~~text
validity_gate=PASS
decision=STOP_ALGEBRAIC_MICRO_OPT
selected_phase=none
common_material_phases=none
~~~

No selectable phase clears the frozen common-material threshold.

## Canonical measured provenance

Measured implementation head:

`d47b98ff3436219f031f4e6b4e13a921edcfe857`

Base main includes accepted D11 and M6-B1:

`703f8a39a7e4c7183197707fa5a1ed2d557bfe17`

Hosted measurement:

- M6-D12 #8 / `37611169160` — SUCCESS
- Research #332 / `37611169399` — SUCCESS

Artifact:

`m6d12-post-d11-profile-37611169160-1`

Artifact ID:

`11477283140`

Artifact SHA256:

`5a6bd123b32033d61de56d6c19b28a78eb65afc02b3256d712325a286c8ebbfd`

The artifact contains environment/source hashes, all three repeated raw result files
and the fail-closed summary.

This run is on the final measured source/evidence head before the final docs-only
closure. Exact-head integration gates for the final PR head are recorded in the PR
before merge.

## Frozen semantics

Unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also unchanged:

- D11 factor-frame-local fixed-constant reduction plan;
- D8 dedicated scalar GF(2^64) square;
- D6 deterministic degree-two solver;
- deterministic degree>=3 factor tree and trace sequence;
- generic GF(2^64) multiplication outside D11 trace reduction;
- GCD/division;
- frozen polynomial basis / reduction `0x1B`;
- exact nonzero-u64 identity mapping and separate zero bit;
- one guard syndrome;
- all-syndrome verification;
- public API, snapshot-v1 and Coverage.

Production/public ExactSmallDelta remains **NO-GO**.

## Method

D12 deliberately profiles the **whole staged decode**, not only factorization,
because D11 removed roughly half of the previous end-to-end d=8 cost.

The accepted D11 decoder remains the uninstrumented control.

The profiled copy measures cumulative non-overlapping phases:

- guarded-prefix materialization/merge;
- fresh locator/BM construction;
- decoder validation;
- D11 reduction-plan construction;
- degree>=3 trace;
- GCD;
- successful division;
- D6 quadratic solve;
- candidate verification.

Factor wall and profiled decoder wall are retained only as overlapping accounting
controls and are not selection candidates.

Five deterministic 8192-key corpora are reused:

- D4-derived;
- D5-derived;
- D6-derived;
- D7a;
- D7b.

For every corpus:

~~~text
d = 0,1,2,3,4,5,8,9,10,16
~~~

Three fresh hosted processes and four balanced control/profile samples per scenario
produce twelve d=8 observations per corpus.

## Correctness

The profiled path matches accepted D11 result/error semantics at every audited stage.

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

The deterministic profile inventory preserves factor/plan/trace call counts.

False-success total is zero in the repeated deterministic matrix.

This empirical zero count is not promoted to an over-capacity theorem.

## Instrumentation validity

d=8 median profile-wall overhead:

~~~text
D4    -0.440%
D5    -0.422%
D6    +0.717%
D7a   -0.501%
D7b   -0.478%
~~~

All five are below the predeclared +5% ceiling. The phase selector is therefore
valid.

## d=8 residual result

Share of accepted D11 end-to-end control latency:

| Phase | Median | Corpus min | Corpus max | Common >=25%? |
| --- | ---: | ---: | ---: | --- |
| prefix | 0.018% | 0.017% | 0.080% | no |
| locator | 3.775% | 3.435% | 17.372% | no |
| validation | 0.003% | 0.003% | 0.016% | no |
| plan build | 0.595% | 0.480% | 2.741% | no |
| trace | 20.027% | 18.860% | 36.503% | no |
| GCD | **72.325%** | **23.616%** | **72.762%** | **no** |
| division | 1.498% | 1.135% | 7.998% | no |
| quadratic | 1.517% | 1.392% | 9.275% | no |
| verification | 0.166% | 0.163% | 1.066% | no |

The factor-tree dependence is now decisive.

Four corpora are strongly GCD-heavy, while the D6-derived corpus distributes work
across trace (36.503%), GCD (23.616%), locator (17.372%), quadratic (9.275%) and
division (7.998%).

GCD misses the frozen 25% minimum-in-every-corpus gate by 1.384 percentage points in
the D6-derived corpus. That near miss is **not** grounds to move the threshold after
seeing the data. Doing so would turn the gate into post-hoc candidate selection.

Trace no longer clears the common threshold either.

## Interpretation

D4 through D11 successfully removed a sequence of stable or measured bottlenecks:

~~~text
generic trace square/mod
-> degree-two root work
-> scalar GF(2^64) square
-> polynomial reduction
~~~

After D11 the residual is no longer one stable algebraic primitive across factor-tree
shapes.

A GCD-only optimization could look compelling on four corpora and still target only
about one quarter of the D6-derived workload. A trace-only optimization has the
opposite problem. Selecting either now would resume factor-tree-specific
micro-optimization rather than improve a stable common primitive.

The appropriate next question is therefore no longer “which algebraic instruction
should be faster?” but “does maintaining and exchanging this exact sketch beat the
system alternatives under named workloads?”

## D12 verdict

**STOP algebraic decoder micro-optimization.**

Do not start, without new system-level evidence:

- GCD specialization;
- another trace specialization;
- cubic/quartic solver;
- generic GF(2^64) rewrite;
- D11 table-layout tuning;
- SIMD/CLMUL/unsafe;
- caches keyed by factor polynomial;
- protocol changes motivated only by decoder microbenchmarks.

Next M6-D action:

**maintained-state system comparison** of:

- accepted private D11 PinSketch lane;
- direct exact reconciliation;
- rateless-style reconciliation comparator.

That comparison must include maintained-state construction/update cost, memory,
application bytes, retries/RTTs, decoder CPU, verification and workload distribution.

Production/public ExactSmallDelta remains **NO-GO** until a separate product-level
GO decision is recorded.
