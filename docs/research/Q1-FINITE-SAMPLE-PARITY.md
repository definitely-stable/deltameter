# Q1 — Finite-sample Parity capacity

Status: **published W_i decision resolved: NO-GO for Coverage::Proven**  
Research status: **strict W_i track stopped; ParityLevelCounts remains optional research**  
Date: 2026-10-06

See [STRICT-PARITY-POST-M3.md](STRICT-PARITY-POST-M3.md) for the current canonical synthesis. This file preserves the earlier Q1 proof program and remains useful historical context.

## Question

For x in GF(2)^N with true Hamming weight

~~~text
d = ||x||_0
~~~

can a practical GF(2)-F-PCSA construction produce a computable upper bound

~~~text
U(S, m, N, delta)
~~~

such that

~~~text
Pr[d <= U] >= 1 - delta
~~~

for finite parameters, including d = 0 and small d, without unsupported asymptotic normality?

## v0 decision

**No for the current published W_i estimator. The repository now treats that strict path as stopped unless a new theorem changes the gate.**

The repository still has no theorem-backed finite-sample construction that justifies strict Parity capacity at:

~~~text
delta in {1e-3, 1e-6, 1e-9}
epsilon in {5%, 10%, 20%}
~~~

Therefore:

~~~text
ParityDeltaMeter
    point estimate: allowed
    asymptotic metadata: allowed if labelled
    empirical diagnostics: allowed if labelled
    Coverage::Proven: not allowed
    recommended_capacity(Proven): not allowed

EnergyDeltaMeter
    strict finite-sample capacity: allowed
~~~

This is a product decision, not an impossibility theorem.

## Canonical notation

The newest report overloaded several symbols. Q1 now fixes notation:

~~~text
N = universe cardinality
m = number of F-PCSA rows
d = true Hamming weight / symmetric-difference size
j = level index
J = maximum stored level / finite-level truncation boundary
F = field; for DeltaMeter Parity, F = GF(2)
~~~

Never reuse d for fringe width or truncation depth.

## Exact published construction first

The proof effort must start from the published F-PCSA definition, not from a generic classical-PCSA approximation.

For the published construction:

~~~text
state:
    FIELDMAP[i,j] in F

hash h:
    h : U -> [m] x N_levels
    P(h(v) = (i,j)) = (1/m) * 2^-j

hash/coefficient g:
    g : U -> F
    g(v) is uniform over F

update:
    add k * g(v) to FIELDMAP[h(v)] in the field

row statistic:
    W_i = highest level j whose FIELDMAP[i,j] is non-zero

estimator:
    normalized function of the W_i values
~~~

For F = GF(2), a field cell that “should contain a 1” in classical PCSA can cancel back to zero. That cancellation is part of the published model.

The published paper explicitly analyzes the middle-cardinality regime and does not claim that the same estimator handles the d = O(1) and d near N regimes without separate treatment.

## Published F-PCSA vs set-specialized parity

Two candidates remain separate.

### PublishedFpcsaF2

Uses the published random coefficient g(v) in GF(2).

Published asymptotic claims, including the relative-error constant near

~~~text
1.638 / sqrt(m)
~~~

belong only to this construction and its assumptions.

### ParityPcsaSetV1

Potential DeltaMeter specialization:

~~~text
g(v) = 1
cell ^= 1
~~~

This is statistically a different estimator.

It inherits no published asymptotic constant and no strict tail result automatically.

Repository rule:

> Reproduce PublishedFpcsaF2 before evaluating ParityPcsaSetV1.

## What is now established

The latest report strengthens the following exclusions.

1. Asymptotic RSE is not finite-sample high-confidence coverage.
2. Empirical tails are diagnostics, not proof of 1e-6 or 1e-9 coverage.
3. Median/group amplification only amplifies a valid per-copy failure bound.
4. Independence assumptions must be proved for the exact F-PCSA state/statistic.
5. Saddlepoint or other asymptotic approximations need explicit rigorous remainder control.
6. Finite levels/truncation, d = 0, small d, and practical randomness are part of the theorem.
7. A one-sided capacity bound does not require a full two-sided exact confidence interval.

## Critical corrections to the newest attached report

### Classical PCSA is not the exact proof object

The report repeatedly reasons about first-1 positions and classical PCSA bitmaps.

That is useful historical intuition, but Q1 concerns the published finite-field FIELDMAP construction and its rightmost-nonzero row statistic.

Exact finite analysis must use the actual state or a proved sufficient statistic.

### GF(2^V) is not the definition of GF(2)-F-PCSA

The report speculates that GF(2)-F-PCSA may mean hashing through an extension field GF(2^V).

That is rejected.

For DeltaMeter:

~~~text
field = GF(2)
universe size = N
~~~

These are separate concepts.

### Poissonization is a tool, not a free independence theorem

If fully independent hashing splits a Poisson number of active keys into disjoint row/level categories, the category counts may become independent by Poisson thinning.

That can simplify the exact field-cell law.

But the proof must derive this for:

- the exact h distribution;
- the g coefficient model;
- finite J;
- the chosen row statistic.

No blanket “the registers are independent” assumption is accepted.

### De-Poissonization is promising but conditional

The 2025 elementary de-Poissonization result is potentially useful because it gives explicit finite-order Poisson-Charlier remainder bounds in terms of higher forward differences.

For DeltaMeter it becomes applicable only after we define a concrete coefficient sequence, for example:

~~~text
a_n(s) = P_n[ T(S) <= s ]
~~~

or another acceptance/tail probability, and prove computable bounds on the forward differences required by the theorem.

Until then its status is:

~~~text
promising theorem candidate
not a finished F-PCSA tail bound
~~~

### Truncated multivariate-normal estimation is not the right truncation model

The cited high-dimensional truncated-sample paper studies parameter estimation from truncated multivariate Gaussian observations.

F-PCSA finite-level storage is discrete algorithmic censoring of sketch state.

That paper is not a direct justification for Q1 and is not part of the main proof path.

### One-sided inversion is the minimal statistical target

We need a valid level-delta test for large cardinalities, then invert it.

A conservative test with Type-I error <= delta is sufficient.

The test does not need to attain delta exactly.

Two-sided central/minlike/Blaker machinery is optional and currently unnecessary.

### Binary search requires monotonicity

A numeric binary search for U is only valid if the rejection/acceptance rule is monotone in d.

If stochastic ordering cannot be proved, test inversion must use explicit certified search over the candidate cardinalities or another method that handles non-monotone acceptance sets.

## Correct research path

### A. Freeze the exact published state machine

Write an implementation-independent mathematical specification for:

- h;
- g;
- field update;
- finite J;
- row statistic W_i;
- final estimator.

Do not introduce the g(v)=1 variant here.

### B. Exact small finite distribution

For small N, m, J, and d:

- enumerate or dynamic-program the exact FIELDMAP distribution;
- compare exact moments with the existing moment oracle where the models overlap;
- identify a smallest sufficient statistic, if one exists;
- test stochastic ordering in d;
- produce golden distributions/vectors.

Do not use a classical-PCSA first-1-position Markov chain unless equivalence to the actual FIELDMAP statistic is proved.

### C. Finite-level model

Treat levels above J as an explicit event.

Track a failure/error budget such as:

~~~text
delta_total =
    delta_model
  + delta_truncation
  + delta_depoissonization
  + delta_numerical
~~~

Only terms that are actually probabilistic failures belong in the budget; deterministic numerical interval enclosure can instead be handled by outward rounding/certified arithmetic.

### D. Poissonized exact law, if useful

Derive the exact cell/row law under a Poissonized active-key count.

Do not jump directly to CLT or asymptotic RSE.

### E. Rigorous return to fixed d

Preferred order:

1. direct fixed-d combinatorial/DP analysis;
2. exact conditioning/coefficient extraction;
3. a de-Poissonization theorem with verified hypotheses and explicit remainder;
4. only then looser concentration inequalities whose assumptions are verified.

### F. One-sided test inversion

For each candidate d0, construct a test of a one-sided hypothesis suitable for an upper confidence limit.

The exact direction depends on the monotone statistic eventually chosen.

Then define U from the non-rejected parameter set.

Do not assume that the confidence set is an interval until monotonicity is established.

### G. Width/power gate

Coverage alone is not enough.

For each requested epsilon/delta pair, measure or rigorously bound:

- state bytes;
- update cost;
- query cost;
- median/p95 U/d where meaningful;
- the smallest d where relative width is useful;
- behavior near d = 0 and d near N.

## GO / NO-GO / REPLACE rule

### GO

Only if:

- finite-sample coverage is theorem-backed for the concrete implementation;
- small-d and finite-J behavior are included;
- the Rust randomness contract matches the proof;
- width is useful for the requested profiles;
- memory/update cost is competitive enough to justify a second strict backend.

### NO-GO

If the theorem is unavailable or the resulting U is too wide/expensive.

Then:

~~~text
Parity = fast experimental/asymptotic estimate
Energy = strict capacity backend
~~~

### REPLACE

Only if another GF(2)-linear estimator directly demonstrates a better practical frontier under the same model.

## Minimal reproducible Q1 tooling

Do not build a research framework.

The next useful tools are:

~~~text
research/
    parity_exact_small.py
    parity_truncation_budget.py
~~~

A strict tail/profile generator is added only after a valid theorem exists.

Monte Carlo remains diagnostic.

## v0 consequence

This newest research **does not delay M1**.

Implementation order remains:

~~~text
EnergyDeltaMeter
-> faithful PublishedFpcsaF2 reproduction
-> experimental ParityDeltaMeter
-> performance comparison
-> optional continuation of strict-Parity theory
~~~
