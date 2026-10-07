# STRICT-COMPACT-001 — research and implementation plan

Issue: #59

Status: Phase A foundation. No public/API change and no Proven promotion.

## Product question

Can DeltaMeter replace the current strict-state Pareto point — EnergyDeltaMeter at
10% relative error, failure <=1e-6 and 376,832 bytes of primary state — with a
materially smaller GF(2)-linear sketch that still exposes a genuine fixed-d
finite-sample one-sided upper bound on d = |A △ B|?

The desired primitive is: small mergeable state, O(1)-style toggle, XOR merge,
certified U with P_d(U >= d) >= 1-delta, and useful U/d width.

Phase A targets state <=64 KiB, preferably <=32 KiB.

## Exact single-level reduction

For one stored F-PCSA level j and m rows, let S_j(d) be the number of rows whose
level-j GF(2) cell is one after exactly d active difference elements.

One element selects level j with probability 2^-j, has GF(2) coefficient one with
probability 1/2, and then chooses a row uniformly. Therefore one element toggles one
level-j row with probability r_j = 2^-(j+1).

S_j is the lazy Ehrenfest / odd-occupancy birth-death chain:

P(s -> s+1) = r_j (m-s)/m
P(s -> s-1) = r_j s/m
P(s -> s)   = 1-r_j

started at zero. This is an exact fixed-d law for the stored statistic.

### Finite-J correction

The old truncation budget is not a proof blocker for S_j. Events assigned to levels
above J simply belong to the idle probability for any fixed stored level j<=J.
They do not change that stored level. The useful d-range is instead limited by
informativeness/saturation when even high stored levels approach half occupancy.

## Monotonicity route

Write p_s = r_j(m-s)/m and q_s = r_j s/m. The standard adjacent-state sufficient
condition for a monotone discrete birth-death kernel is p_s + q_(s+1) <= 1.

Here p_s + q_(s+1) = r_j(m+1)/m. Since j>=1, r_j<=1/4; for m>=1 the expression is
at most 1/2. The chain therefore satisfies the classical monotone-kernel condition
with margin.

The point mass at zero is stochastically dominated by its one-step law, so repeated
application of a monotone kernel gives S_j(d) <=_st S_j(d+1).

This will be cited as an application of classical birth-death monotonicity, not
claimed as new mathematics.

## One-sided inversion

Let F_(j,d)(s) = P_d[S_j <= s]. Monotonicity makes this nonincreasing in d.

For a per-level error budget alpha_j, conceptually define the conservative discrete
upper inversion U_j(s) as the supremum of d for which F_(j,d)(s) > alpha_j.
Then P_d[U_j(S_j) < d] <= alpha_j.

The first multilevel construction deliberately avoids the fixed-d joint law.
Predeclare deterministic alpha_j with sum alpha_j <= delta; baseline is equal
Bonferroni alpha_j = delta/J. Then U = min_j U_j(S_j) has familywise one-sided
coverage at least 1-delta without assuming level independence.

## Phase sequence

### A0 — cheap optimistic screen

Use the already-derived Poissonized model only as a diagnostic opportunity screen.
Under D~Poisson(lambda), stored cells split independently and one level count is
binomial. Compute a cheap multilevel width frontier for the same state budgets.

This cannot prove fixed-d coverage and cannot yield GO_COMPACT_STRICT. It exists to
stop early if even a favorable tractable model is far outside product width targets.

### A1 — exact tiny fixed-d reference

Add a dependency-free Fraction oracle for small m,d:
- exact transitions and PMF/CDF;
- exact moment checks;
- stochastic-order regression;
- finite-grid confidence inversion coverage checks.

This is the reference implementation for accelerated/certified work.

### A2 — scalable fixed-d law

The chain has Krawtchouk eigenstructure with eigenvalues
lambda_k = 1 - 2 r_j k/m, k=0..m.

Choose a stable scalable evaluator and cross-check it against A1. Ordinary binary64
output may be used for diagnostic width exploration but is not a proof boundary.
A product GO requires auditable tail enclosures / certified numerical error.

Potential path:
1. exact tiny oracle;
2. stable spectral recurrence for production m;
3. directed-rounding/high-precision enclosure or another numerical certificate;
4. monotone inversion in d.

### A3 — frozen width frontier

For J=64:
- 2 KiB -> m=256
- 8 KiB -> m=1024
- 32 KiB -> m=4096
- 64 KiB -> m=8192

Primary delta=1e-6; also report 1e-3 and 1e-9 when feasible.

Use a log-spaced d grid plus refinement at level hand-offs and saturation. Report
coverage separately from width, including at least median/p95/p99 of U/max(d,1)
under the certified fixed-d law.

### A4 — deeper inference only if earned

If equal-Bonferroni is close but misses the product gate, quantify the exact gap
before deeper mathematics. Allowed follow-ups include deterministic nonuniform
alpha allocation, data-independent level selection, independent pilot rows,
exact joint inversion, or closed-testing/e-value/confidence-sequence constructions.

No data-dependent alpha tuning may be silently introduced.

## Decision gates

GO_COMPACT_STRICT requires at least one profile with:
- state <=64 KiB, preferably <=32 KiB;
- certified fixed-d P(U>=d) >= 1-1e-6 over a declared useful d-range;
- XOR merge and current O(1)-style update retained;
- U/d <=1.5 at the declared product width gate across that range;
- preferably <=1.25 through the central operating range.

GO_DEEPER_INFERENCE means the conservative construction misses narrowly and a
quantified plausible multilevel improvement would cross the gate.

STOP_COMPACT_STRICT if the favorable Poissonized screen is already far outside the
target, the fixed-d construction remains far outside it at 64 KiB, certification is
impractical, or the useful d-range is too narrow to justify a standalone crate.

## Public boundary

Phase A adds research artifacts only. Do not change ParityDeltaMeter::estimate,
Coverage::Asymptotic, snapshot v1, public profiles/defaults, or publication status.
A public strict compact backend needs a later issue after research GO.
