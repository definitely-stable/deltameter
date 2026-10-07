# M6-D7 — Post-quadratic residual root profile

Status: implementation in progress. Issue: #36. Parent: #19 / #14.

## Goal

Re-profile the accepted D6 decoder after degree-two specialization before selecting any further root optimization.

D6 is frozen and accepted. D7 is profiling-only.

## Frozen protocol and semantics

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
~~~

Also frozen:

- D6 deterministic degree-two solver;
- D4 characteristic-2 trace square;
- degree >=3 deterministic trace splitting;
- GF(2^64) polynomial basis and reduction 0x1B;
- exact nonzero u64 identity mapping;
- out-of-band zero bit;
- one guard syndrome;
- all-syndrome candidate verification;
- public API, snapshot-v1 and Coverage.

## Profiling model

Use a D7-only instrumented copy of the accepted D6 factor path.

For each recursive factor frame record non-overlapping local work:

- factor call count by degree;
- trace split attempts by degree;
- square/mod calls by degree;
- trace-loop self time;
- GCD self time;
- quotient/division self time;
- degree-two solver self time;
- local normalization/dispatch overhead;
- factorization wall time;
- candidate verification separately.

Recursive child time is not charged to the parent degree bucket.

The accepted D6 complete decoder is the uninstrumented performance control.

## Corpora

Use five deterministic 8192-key corpora:

- d4-derived salts;
- d5-derived salts;
- d6-derived salts;
- new d7a salts;
- new d7b salts.

Frozen scenario matrix:

~~~text
d = 0,1,2,3,4,5,8,9,10,16
~~~

Use three hosted processes and four balanced samples per scenario.

## Correctness gate

For every corpus and stage:

1. D7 profiled result/error equals accepted D6;
2. accepted D6 equals frozen D4/D1 controls where those controls are evaluated;
3. successful d<=8 candidate equals the exact oracle;
4. d=9/10/16 remains reject;
5. zero/high-bit/u64::MAX semantics are unchanged;
6. guard mutation and invalid-limit precedence remain unchanged;
7. D2 bytes/RTTs remain unchanged.

## Accounting gate

For degree >=3:

~~~text
square_calls = 64 * trace_attempts
~~~

Degree two must record quadratic solver calls/time and no trace-square work.

No recorded factor degree may exceed 8.

Instrumentation overhead is reported but is not used as the performance baseline.

## Decision gate

No algorithmic optimization is permitted in D7.

After evidence:

- authorize exactly one degree-specific candidate only if one degree >=3 is consistently material across corpora;
- if dominance changes substantially by factor tree, prefer a general trace/GCD candidate or stop;
- if residual decode CPU is already small relative to maintained-state network/RTT regimes, stop decoder micro-optimization and move to system-level comparison.

No Artin-Schreier cache, cubic/quartic solver, CLMUL, SIMD, unsafe, alternate field representation or bundled optimization in this slice.
