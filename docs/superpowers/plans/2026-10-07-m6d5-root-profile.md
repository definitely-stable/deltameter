# M6-D5 — Root-factor degree and verification profiling gate

Status: profiling/evidence complete in PR #33; **quadratic specialization GO-to-experiment**. Issue: #32. Parent: #19 / #14.

## Goal

Determine whether a deterministic degree-two root solver is the next justified optimization after D4.

D4 is frozen and accepted. D5 changes no factorization algorithm and no reconciliation semantics.

## Frozen controls

Keep unchanged:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also frozen:

- exact nonzero u64 -> GF(2^64) identity mapping;
- out-of-band zero bit;
- GF reduction polynomial 0x1B;
- one guard syndrome;
- fresh locator/BM;
- D4 characteristic-2 trace square;
- deterministic trace coefficients;
- all-syndrome candidate guard;
- public API, snapshot-v1 and Coverage.

## Profiling model

Use a D5-only instrumented copy of the accepted D4 specialized factor path.

Do not modify D4 implementation files.

For each recursive factor frame record by current polynomial degree:

- factor call count;
- trace split attempt count;
- characteristic-2 square/mod invocation count;
- trace-loop elapsed self time;
- GCD elapsed self time;
- successful quotient/division elapsed self time;
- other local self time where needed.

Recursive child time is **not** charged to the parent degree bucket.

Also record:

- factorization wall-clock time;
- candidate sort/root validation;
- candidate rebuild + all-syndrome verification;
- complete D4 specialized decode wall time as an uninstrumented control.

The purpose is to estimate what a degree-two solver can remove without double-counting recursive work.

## Workload

Same maintained-state shape as D4:

~~~text
source size = 8192
d = 0,1,2,3,4,5,8,9,10,16
stage schedule = 1,2,4,8
~~~

Use deterministic full-width keys and zero case.

Three hosted processes, at least four samples per scenario.

## Correctness

For every stage:

- profiled candidate/outcome == accepted D4 specialized result;
- successful result == exact oracle;
- d=9/10/16 remains reject;
- zero/high-bit/u64::MAX semantics unchanged;
- bytes/RTTs unchanged;
- false-success remains zero in the frozen matrix.

Profiling counters must satisfy internal accounting:

- trace square count == 64 * trace attempts for non-base factor frames;
- all recorded degrees are <= 8;
- factor calls are positive for nontrivial successful/rejected root paths;
- verification is measured separately and never omitted from the complete decoder control.

## Decision gate

Compute, for meaningful retry cases:

1. degree-two self factor CPU / profiled factor wall time;
2. degree-two self factor CPU / accepted D4 complete decoder time;
3. candidate verification share;
4. higher-degree self factor share.

Quadratic specialization is **GO** only if degree-two self work is a material, reproducible fraction and its theoretical upper-bound removal would meaningfully improve end-to-end latency.

If not, record **NO-GO quadratic** and do not implement it.

If verification is the material residual, select verification as the next isolated candidate.

If neither is material enough, stop decoder micro-optimization and move to maintained-state system-level reconciliation comparison.

## Boundaries

Profile-only slice. No algorithmic optimization in D5. Safe stable Rust only, no unsafe, SIMD, CLMUL, FFI, new dependency, public API, snapshot or Coverage change. GitHub-hosted runners only.


## Completion record

Canonical measured head: `0c86781079f6b88979b3e7ae0730cabe6db12cdd`.

Hosted gates:

- M6-D5 #8 / `37581499080`;
- Rust #226 / `37581499072`;
- Research #239 / `37581499073`.

Canonical evidence: [M6-D5 root profile](../../M6-D5-ROOT-PROFILE-EVIDENCE.md).

Decision:

- degree-two self work is material for d=2/3/4/8;
- d=8 degree-two share is ~27.5% of accepted D4 control latency;
- verification is ~0.055% at d=8 and is not a justified optimization target;
- higher-degree work remains the majority at d=8;
- next slice: one isolated deterministic safe-Rust quadratic solver candidate;
- D5 itself contains no root algorithm change.
