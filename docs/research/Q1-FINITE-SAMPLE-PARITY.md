# Q1 — Finite-sample Parity capacity

Status: **v0 decision resolved: NO-GO for `Coverage::Proven`**  
Research status: **open**  
Date: 2026-10-06

## Question

For (xin GF(2)^V), (d=|x|_0), can a practical GF(2)-PCSA / finite-field sketch produce a computable

[
U(S,m,V,delta)
]

such that

[
Pr[dle U]ge 1-delta
]

for finite (m,V), all (d), including (d=0) and small (d), without relying on unsupported asymptotic normality?

## Decision for v0

**No. Not yet.**

The repository does not currently have a theorem-backed finite-sample construction that justifies strict Parity capacity at (delta=10^{-3},10^{-6},10^{-9}).

Therefore:

```text
ParityDeltaMeter
    point estimate: allowed
    asymptotic/calibrated metadata: allowed if labelled
    Coverage::Proven: not allowed
    recommended_capacity(Proven): not allowed

EnergyDeltaMeter
    strict finite-sample capacity: allowed
```

This is a product decision, not a theorem that strict Parity is impossible.

## Critical distinction: published F-PCSA vs set-specialized parity

The latest research surfaced an important distinction that must be frozen before implementation.

### Published F-PCSA over GF(2)

The published finite-field construction uses a random coefficient (g(v)in GF(2)). In the binary field that coefficient can be zero.

Claims such as the published asymptotic relative-error constant near

[
1.638/sqrt m
]

belong to that construction and its assumptions.

### Proposed set-specialized `g(v) = 1`

A tempting DeltaMeter specialization is:

```text
choose row/level
cell ^= 1
```

This removes coefficient randomness and is natural for set XOR semantics.

However, this is a **different estimator**. Published F-PCSA constants and tail claims must not be transferred to it automatically.

Repository rule:

> Until independently analyzed, `ParityPcsaSetV1` and published GF(2)-F-PCSA are separate research candidates.

The Rust implementation must reproduce the published construction first before any set-specialized variant is evaluated.

## What the latest research established

### Strong evidence

1. Published asymptotic RSE is not a finite-sample high-confidence theorem.
2. Very small (delta) cannot be certified by ordinary Monte Carlo.
3. Median/grouping amplification is valid only after a valid per-copy failure bound is established.
4. Poissonization can simplify occupancy/parity analysis, but returning to fixed (d) needs a justified de-Poissonization argument.
5. Naive de-Poissonization can introduce a large penalty and should not be hidden inside a `Proven` profile.
6. Small (d), level truncation, finite (V), and practical hash independence are part of the theorem, not implementation afterthoughts.

### Not established

The following are **not** accepted as proof-grade conclusions:

- plugging the asymptotic (1.638/sqrt m) variance into Chebyshev/Cantelli and calling the result finite-sample;
- fitting an empirical CDF and calling (delta=10^{-9}) coverage `Proven`;
- using a median wrapper without a theorem for the per-copy tail;
- assuming independent cells/registers in the fixed-(d) model;
- treating a saddlepoint approximation as an exact confidence guarantee;
- transferring published F-PCSA results to `g(v)=1`.

## Correct research path

The next proof attempt should be narrow.

### A. Reproduce the exact published construction

Freeze:

- row/register selection;
- level distribution;
- coefficient distribution;
- finite level range induced by (V);
- exact estimator statistic;
- exact independence model.

No implementation-specific simplification before reproduction.

### B. Exact finite cases

For small (m,d), compute the exact distribution by enumeration or dynamic programming.

Goals:

- validate first/second-moment formulas;
- detect non-monotonicity;
- test stochastic ordering assumptions needed by test inversion;
- produce golden vectors.

### C. Poissonized model

If Poissonization makes cells independent, use it only as an intermediate model.

Track explicitly:

```text
delta_total =
    delta_poisson_tail
  + delta_depoissonization
  + delta_truncation
  + delta_numerical
```

No hidden error budget.

### D. Fixed-d return

Prefer, in order:

1. exact conditioning / exact coefficient extraction;
2. rigorous finite-d dynamic program;
3. explicit de-Poissonization theorem with computable remainder;
4. conservative inequality whose assumptions are verified.

Saddlepoint/CGF approximations may guide search, but remain diagnostic until accompanied by a rigorous remainder bound.

### E. Test inversion

A strict upper bound should ideally come from a family of valid tests.

For observed statistic (S=s), define a valid rejection probability (p_d(s)). Then construct an upper confidence set by inversion.

A generic shape is:

```text
U(s) = max { d : p_d(s) > delta }
```

but only if the acceptance/rejection ordering is valid. Parity occupancy can saturate or become non-monotone, so stochastic monotonicity must be proved or the inversion must explicitly handle non-monotone acceptance regions.

## Engineering threshold for changing the decision

The decision may move from NO-GO to GO only if all of the following hold:

1. finite-sample coverage is theorem-backed for the concrete sketch;
2. (d=0) and small (d) are covered explicitly;
3. truncation and numerical error are inside the failure budget;
4. the randomness contract used in Rust matches the proof;
5. for (arepsilonin{0.05,0.10,0.20}) and (deltain{10^{-3},10^{-6},10^{-9}}), generated profiles have useful width;
6. memory/update cost is competitive enough to justify a second strict backend.

If coverage is valid but (U/d) is usually too large, the result remains NO-GO for strict Parity.

## Reproducible research runners to add later

Keep this small:

```text
research/
  parity_exact_small.py
  parity_truncation_budget.py
  parity_tail_profile.py
```

Their roles:

- `parity_exact_small.py`: exact enumeration/DP for small (m,d);
- `parity_truncation_budget.py`: finite-(V) level-mass accounting;
- `parity_tail_profile.py`: only after a valid tail theorem exists, generate strict profiles.

Monte Carlo remains diagnostic and belongs outside the proof path.

## v0 consequence

Q1 no longer blocks starting Rust implementation.

The implementation order is:

```text
EnergyDeltaMeter
-> published F-PCSA reproduction
-> ParityDeltaMeter experimental
-> performance comparison
-> optional continuation of strict-Parity theory
```

Strict Parity research continues, but it is not a release gate for the first useful crate.
