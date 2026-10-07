# M6-D12 — Post-D11 whole-decode residual profile

Status: **REPLICATION-CLOSED; STOP algebraic decoder micro-optimization.**  
Issue: #48. Parent: #19 / #14. PR: #50.  
Evidence date: 2026-10-07.

## Decision

D12 re-profiles the complete accepted D11 staged decoder after the fixed-constant
trace-reduction optimization.

The original predeclared selector required a phase to account for at least 25% of
accepted D11 d=8 control latency in **every one of the five frozen corpora** before
another narrow algebraic measurement slice could be authorized.

Independent exact-head reruns exposed a boundary-sensitive D6-derived GCD share:
valid single-run summaries included 23.616%, 24.721% and 26.359%. Correctness and
the <=+5% profiler-overhead gate passed in each case.

No favorable or unfavorable run was selected post hoc. Before collecting more data,
D12 froze an independent-runner replication closure that kept the original 25%
threshold unchanged.

Canonical replication verdict:

~~~text
validity_gate=PASS
decision=STOP_ALGEBRAIC_MICRO_OPT
selected_phase=none
stable_common_phases=none
~~~

No selectable phase reproduces the original >=25% common-material condition across
all five corpora on all five independent workers.

## Canonical replication provenance

Replication protocol/source head:

`f9ca7b42e6becc4e3feb47cb9f33bfa6a963e09f`

Base main:

`099cceb43e2d76a4b2b0890ecefc155965511cee`

The base contains accepted D11 plus the independently merged M6-B1/B2 work. M6-B
does not alter the private D11 decoder lane.

Hosted gates at the replication head:

- M6-D12 replication #12 / `37612343061` — SUCCESS
- Research #344 / `37612342976` — SUCCESS
- five independent `ubuntu-latest` workers — SUCCESS
- replication aggregation job — SUCCESS

Combined canonical artifact:

`m6d12-replication-37612343061-1`

Artifact ID:

`11477764214`

Artifact SHA256:

`de9425ac40184204297b21b76bb2f99c896c1ceb0f0444878e980bbb13f8f2ae`

Worker artifacts are retained separately as well:

~~~text
worker 1  artifact 11477949118
worker 2  artifact 11477974000
worker 3  artifact 11477694240
worker 4  artifact 11478019142
worker 5  artifact 11477429573
~~~

The combined artifact contains all worker environments, source hashes, raw result
files, local summaries and the cross-runner replication summary.

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

## Whole-decode method

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

## Replication closure

The canonical closure uses:

~~~text
independent GitHub-hosted workers = 5
profiler processes / worker       = 3
balanced samples / process        = 4
paired samples / worker / corpus  = 12
~~~

For each raw sample, each selectable phase is divided by the accepted-D11 control
time from that same sample. Each worker/corpus/phase is then summarized by the
median of its 12 paired shares.

A phase is replication-stable common/material only if its worker-level median is
>=25% for **every corpus on every worker**.

This does not move the original 25% threshold. It requires that threshold to be
reproducible after exact-head reruns demonstrated boundary sensitivity.

## Correctness

Every worker independently passes the existing deterministic logical matrix.

The profiled path matches accepted D11 result/error semantics at every audited stage.

Successful d<=8 workloads equal the exact symmetric-difference oracle.

d=9/10/16 remain reject.

The deterministic profile inventory preserves factor/plan/trace call counts.

False-success total is zero in the repeated deterministic matrix.

This empirical zero count is not promoted to an over-capacity theorem.

## Instrumentation validity

All 25 worker/corpus median paired overhead values pass the frozen +5% ceiling.

Observed worker/corpus overhead range:

~~~text
-1.343% .. +1.208%
~~~

Negative values are ordinary paired timing noise and are not interpreted as a
profiler speedup.

The replication selector is therefore valid.

## Replication-stable d=8 residual result

Across the 25 worker/corpus medians:

| Phase | Median | Worker/corpus min | Worker/corpus max | Stable common >=25%? |
| --- | ---: | ---: | ---: | --- |
| prefix | 0.020% | 0.017% | 0.092% | no |
| locator | 3.763% | 3.422% | 17.951% | no |
| validation | 0.003% | 0.003% | 0.017% | no |
| plan build | 0.496% | 0.363% | 2.716% | no |
| trace | 19.730% | 18.034% | 36.786% | no |
| GCD | **72.347%** | **23.574%** | **74.713%** | **no** |
| division | 1.493% | 1.120% | 8.434% | no |
| quadratic | 1.529% | 1.367% | 9.453% | no |
| verification | 0.169% | 0.161% | 1.113% | no |

The boundary corpus is D6-derived. Its GCD worker-level medians are:

~~~text
worker 1  23.657%
worker 2  24.750%
worker 3  24.575%
worker 4  23.574%
worker 5  23.583%
~~~

All five are below the original 25% threshold.

The earlier 26.359% exact-head observation is therefore not reproducible as a stable
common-material result.

## Interpretation

D4 through D11 successfully removed a sequence of measured bottlenecks:

~~~text
generic trace square/mod
-> degree-two root work
-> scalar GF(2^64) square
-> polynomial reduction
~~~

After D11 the residual is factor-tree dependent.

GCD dominates four corpus families, but in the D6-derived family the work is spread
across trace, GCD, locator, quadratic and division. The five-runner replication
demonstrates that GCD does not satisfy the original common-material criterion there.

A GCD-only optimization could therefore produce another large win for selected
factor trees while ceasing to be a stable project-wide primitive improvement. That
is the exact point at which the M6-D research protocol says to stop local algebraic
specialization and return to the product/system question.

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
application bytes, retries/RTTs, decoder CPU, verification and workload
distribution.

Production/public ExactSmallDelta remains **NO-GO** until a separate product-level
GO decision is recorded.
