# Implementation-blocking research questions

These are the five deep-research questions that must be resolved only as far as needed to make implementation decisions. They are not permission to build a large research framework.

## Q1 — Finite-sample Parity capacity

### Question

For (x\in GF(2)^V), (d=\|x\|_0), can the practical GF(2)-PCSA/finite-field construction produce a useful computable

[
U(S,m,V,\delta)
]

such that

[
\Pr[d\le U]\ge1-\delta
]

for finite (m,V), all (d), including (d=0) and small (d), without relying on unsupported asymptotic normality?

### Required investigation

- exact statistic and finite-(m) bias/variance;
- level truncation;
- dependence structure;
- MGF/CGF or other concentration tools;
- Chernoff/Bennett/Bernstein/test inversion where applicable;
- independent grouping/median/quantile wrappers;
- (delta=10^{-3},10^{-6},10^{-9});
- (arepsilon=5\%,10\%,20\%);
- memory and overprovision cost.

### Implementation decision

- **GO:** strict Parity profiles are practical.
- **NO-GO:** Parity remains fast point estimate, Energy remains strict backend.
- **REPLACE:** only if another GF(2) estimator demonstrably improves the Pareto frontier.

### Current status

Open. The repository has only first/second-moment executable checks.

---

## Q2 — Exact Pareto frontier and v1 backend choice

### Question

Within the actual DeltaMeter model

```text
set-only
GF(2)-linear/composable
oblivious input
sparse update
high-confidence relative estimation
```

what is the best practical frontier among:

- GF(2)-PCSA;
- Energy/CountSketch-like estimator;
- dense AMS/Gaussian oracle;
- sparse JL/SSE baseline?

### Required separation

Do not conflate:

- generic F2;
- insertion-only F0;
- integer turnstile L0;
- finite-field L0;
- GF(2) Hamming weight;
- JL/subspace embedding.

### Output required

For (V=2^{32}) and (2^{64}):

- state bytes;
- random cells touched/update;
- hash evaluations/update;
- merge cost;
- query cost;
- guarantee type;
- small-(d) behavior.

### Current provisional decision

```text
Energy first
Parity second
final v1 default after measurements and Q1
```

---

## Q3 — Exact small-d lane

### Question

Does v1 benefit enough from an exact/capped small-d lane to justify another algorithm and decoder?

Candidate shape:

```text
ExactSmallDelta + ParityDeltaMeter
```

### Candidates

- capped IBLT;
- Simple Set Sketching;
- BCH/PinSketch/minisketch-like syndrome;
- power sums;
- XOR + fingerprint buckets.

### Required comparison

For (T=8,16,32,64,128):

- state bytes;
- update/hash cost;
- decode cost;
- failure probability;
- XOR compatibility;
- dependency/licensing impact;
- implementation complexity.

### Current decision

Deferred from v0.

Do not add Minisketch FFI or a decoder before Energy and Parity measurements show a concrete gap.

---

## Q4 — Randomness/hash/threat/wire contract

### Question

What is the smallest randomness contract that exactly matches the proofs without importing unnecessary security infrastructure?

### Energy needs

- pairwise-uniform bucket collisions;
- fourth-order sign independence;
- separation between bucket/sign randomness.

### Parity needs

To be frozen after Q1, because tail proofs may require stronger structure than first/second moments.

### Threat modes

```text
v0 supported:
    oblivious input

not promised:
    chosen-input after seeing seed
    adaptive queries
```

A secret seed is not a v0 requirement.

### Wire/config direction

Do not freeze persistence yet.

When persistence becomes necessary, compatibility should at least include:

- algorithm/version;
- key/universe domain;
- estimator dimensions;
- randomness/hash suite identifier;
- seed/config identity;
- canonical endian-independent encoding.

Rust `Hash` is not a persisted wire identity.

---

## Q5 — Verification and CI for one developer

### Question

How do we maintain mathematical confidence without creating a research-infrastructure project?

### PR gate

Keep cheap and deterministic:

- compile/lint/test;
- algebraic identities;
- reference vectors;
- property tests;
- theorem-derived parameter tests;
- exact numerical formula checks.

### Research workflow

Allowed on GitHub-hosted public runners:

- regenerated JSON tables;
- wider finite-d grids;
- moderate Monte Carlo diagnostics;
- benchmarks as artifacts.

### Do not do

- billion-trial Monte Carlo for (10^{-9});
- absolute hosted-VM nanoseconds as merge gates;
- self-hosted runners;
- large orchestration frameworks.

### Formal verification

Verus/Alerus/Creusot are post-v1 unless a concrete proof/code boundary needs them.

### Current status

The first dependency-free research workflow is already implemented.
