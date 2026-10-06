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

- exact small-(m,d) distribution;
- finite-(V) truncation accounting;
- direct fixed-(d) analysis or rigorous de-Poissonization;
- valid test inversion;
- explicit width/power.

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

Freeze only what is needed to reproduce the published construction. Stronger tail-related assumptions wait for future strict-Parity research.

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
