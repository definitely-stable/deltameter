# Research questions and decision gates

The original five questions remain useful, but not all of them are implementation blockers anymore.

## Q1 — Finite-sample Parity capacity

### v0 gate

**Resolved: NO-GO for strict Parity `Coverage::Proven`.**

The mathematical question remains open.

See [Q1-FINITE-SAMPLE-PARITY.md](Q1-FINITE-SAMPLE-PARITY.md).

### Why

The current material does not establish a finite-(m), finite-(V), all-(d) upper confidence bound for the concrete GF(2)-F-PCSA construction.

In particular, these are insufficient:

- asymptotic RSE inserted into Chebyshev/Cantelli;
- fitted empirical tails at (10^{-9});
- median amplification without a valid per-copy bound;
- unproved independence assumptions;
- saddlepoint approximations without rigorous remainder control.

### Research continuation

Focus only on:

1. faithful mathematical specification of the published FIELDMAP/rightmost-nonzero F-PCSA construction;
2. exact small finite distributions on the actual F-PCSA state or a proved sufficient statistic;
3. finite-level truncation/censoring accounting;
4. Poissonization only after the exact category decomposition is written down;
5. direct fixed-d analysis or de-Poissonization with verified theorem hypotheses and explicit remainder;
6. one-sided test inversion;
7. explicit width/power.

Canonical notation:

~~~text
N = universe cardinality
m = row count
d = true Hamming weight
j = level
J = finite level boundary
~~~

Do not reuse d as truncation width.

---

## Q2 — Pareto frontier and v1 backend choice

### Current decision

```text
strict/default candidate: Energy
fast/experimental candidate: Parity
oracle: Gaussian/chi-square
```

This remains subject to actual Rust measurements, not qualitative literature comparisons.

### Still needed

For (V=2^{32}) and (2^{64}):

- state bytes;
- cells touched/update;
- hash evaluations/update;
- merge/query cost;
- accuracy/coverage class;
- small-(d) behavior.

A replacement GF(2) estimator is considered only if it wins directly in the actual set/Hamming model.

---

## Q3 — Exact small-d lane

### Decision

**Deferred from v0.**

The latest inputs disagree: one strongly recommends Minisketch/PinSketch immediately, another recommends deferral.

The repository chooses deferral because DeltaMeter's first product is an estimator, and a decoder adds:

- a new algorithmic subsystem;
- either C/C++ FFI or substantial pure-Rust BCH work;
- additional API and CI surface.

Minisketch remains the strongest future candidate if small-(d) exact recovery becomes a demonstrated product requirement.

---

## Q4 — Randomness/hash/threat/wire contract

### Energy: enough to start implementation

Freeze for the first Rust slice:

- pairwise-uniform bucket collisions;
- 4-wise independent signs;
- independent bucket/sign families;
- deterministic public seed/config for reproducibility;
- oblivious-input theorem model.

Do not use secret-key infrastructure.

### Parity: still open

Freeze only what is needed to reproduce the published construction:

- h mapping keys to row/level with the published mass function;
- g mapping keys to uniform field coefficients;
- finite level policy;
- rightmost-nonzero row statistic.

Stronger tail-related assumptions wait for future strict-Parity research.

Do not replace the published model with classical PCSA first-1-position reasoning or the proposed g(v)=1 specialization during reproduction.

### Persistence

No wire format yet.

Only in-memory compatibility/config identity is required for M1/M2.

---

## Q5 — Verification and CI

### Decision

Resolved enough for implementation.

PR gate:

- format/lint/build/test;
- deterministic vectors;
- property/algebra tests;
- exact formula/profile tests.

Research jobs:

- exact finite grids;
- moderate Monte Carlo diagnostics;
- profile regeneration;
- benchmark artifacts.

No self-hosted runners. No absolute nanosecond merge gate. No Monte Carlo claim of (10^{-9}) proof.

## Remaining true blockers before M1

None at the research-architecture level.

M1 can start after M0/PR #1 is reviewed and merged.
