# Research questions and decision gates

The original five questions remain useful, but not all of them are implementation blockers anymore.

## Q1 — Finite-sample Parity capacity

### v0 gate

**Resolved for the published W_i estimator: NO-GO for strict Parity `Coverage::Proven`; the current strict-W_i track is stopped.**

The broader GF(2)-linear question remains open only through separate candidates such as `ParityLevelCounts`.

See [STRICT-PARITY-POST-M3.md](STRICT-PARITY-POST-M3.md) and [Q1-FINITE-SAMPLE-PARITY.md](Q1-FINITE-SAMPLE-PARITY.md).

### Why

The current material does not establish a finite-(m), finite-(V), all-(d) upper confidence bound for the concrete GF(2)-F-PCSA construction.

In particular, these are insufficient:

- asymptotic RSE inserted into Chebyshev/Cantelli;
- fitted empirical tails at (10^{-9});
- median amplification without a valid per-copy bound;
- unproved independence assumptions;
- saddlepoint approximations without rigorous remainder control.

### Post-M3 status

The optional research produced three useful exact results:

1. exact tiny full-state enumeration and a direct W_i non-sufficiency witness;
2. an exact finite-J truncation budget separating the implementation signal from state distortion;
3. an exact unconditional Poissonized cell/row law.

The remaining published-W_i gaps are not implementation blockers and are no longer an active proof program:

- practical fixed-d de-Poissonization/coefficient extraction;
- full-statistic stochastic monotonicity;
- certified 1e-6/1e-9 one-sided inversion;
- useful strict width/power.

### Narrow continuation

Only a separate `ParityLevelCounts` track is worth considering:

~~~text
S_j = popcount(FIELDMAP[:,j])
~~~

It must not become a public backend until a certified fixed-d tail and useful width exist.

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

Minisketch remains a strong future candidate if small-(d) exact recovery becomes a demonstrated product requirement. Simple Set Sketching and IBLT-derived candidates were also raised in the post-M3 reports, but none is adopted before a separate primary-source/failure-semantics audit.

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

## Current implementation blockers

None from strict-Parity theory.

M4 performance/API work may proceed independently of the optional ParityLevelCounts or exact-small-d research tracks.
