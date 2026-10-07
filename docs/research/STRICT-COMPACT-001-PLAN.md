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

### A2 — scalable fixed-d tails by monotone de-Poissonization

A direct Krawtchouk evaluator is no longer the primary path.

Let F_d(s)=P[S_j(d)<=s] be the exact fixed-d CDF and let N~Poisson(lambda).
Poissonization gives

G_lambda(s)=E[F_N(s)],

and the stored level count is exactly Binomial(m,p_j(lambda)) with

p_j(lambda)=(1-exp(-lambda/(m*2^j)))/2.

Because F_n(s) is nonincreasing in n,

G_lambda(s) >= P[N<=d] * F_d(s),

hence

F_d(s) <= G_lambda(s) / P[N<=d].

Choose lambda=d. Choi's sharp Poisson-median bound
mu-log(2) <= median(Poisson(mu)) < mu+1/3 implies that for integer d, d itself is
a Poisson(d) median, so P[N<=d]>=1/2. Therefore

F_d(s) <= 2 * G_d(s).

This converts an easily evaluated Poissonized binomial lower tail into a rigorous
fixed-d upper bound. The initial strict inversion can therefore use a per-level
Poissonized threshold alpha_j/2 rather than evaluating the Krawtchouk expansion
directly.

Primary A2 path:
1. exact tiny fixed-d oracle from A1;
2. theorem-derived factor-two de-Poissonization checked against that oracle;
3. apply the standard binomial KL-Chernoff lower-tail bound
   `G_d(s) <= exp(-m D(s/m || p_j(d)))`;
4. obtain the closed theorem-derived fixed-d bound
   `F_d(s) <= min(1, 2 exp(-m D(s/m || p_j(d))))`;
5. invert this monotone bound in d and combine levels with the frozen Bonferroni
   budget.

This route is preferable because the coverage theorem no longer depends on a
production-size incomplete-beta/Krawtchouk numerical tail evaluator. Numerical
implementation still must round conservatively, but the probability bound itself
is analytic.

Exact binomial tails remain a possible width-tightening optimization after the
closed-form KL construction is evaluated. The Krawtchouk fixed-d law remains an
independent validator or further tightening route, not a prerequisite.

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

## De-Poissonization breakthrough

Phase A now has a substantially cheaper strict fixed-d route.

The combination of fixed-d stochastic monotonicity, Poisson splitting and the
integer-mean Poisson median gives the universal single-level bound

F_fixed_d(s) <= 2 * F_poissonized_lambda=d(s).

This means the current Poissonized binomial law is not merely an optimistic
diagnostic. With a factor-two tightening of the per-level tail budget, it can certify
fixed-d one-sided coverage once the binomial numerical tail itself is enclosed
rigorously.

A tiny exact Fraction oracle plus binary64 Poissonized regression is committed to
check this theorem-derived inequality on small grids. The proof claim does not rest
on the floating-point regression.

## Closed-form strict candidate

Combining monotone de-Poissonization with the standard binomial KL-Chernoff tail
gives a direct finite-sample fixed-d candidate:

`F_fixed_d(s) <= min(1, 2 exp(-m D(s/m || p_j(d))))`.

This is substantially stronger engineering-wise than the original A2 plan: coverage
does not require evaluating tiny exact binomial probabilities at production m.

A committed tiny-grid regression checks the resulting bound against the exact
Fraction fixed-d PMFs. A separate width screen evaluates the strict rule under the
Poissonized observation model only as a diagnostic.

Preliminary diagnostic result at delta=1e-6, J=64:
- 2 KiB: clearly too wide;
- 8 KiB: still outside the desired <=1.5x p95 region;
- 32 KiB: promising, worst tested p95 about 1.35x;
- 64 KiB: strong, worst tested p95 about 1.24x.

These width figures are not fixed-d guarantees. They are sufficient to justify
continuing the strict fixed-d construction and to focus product attention on the
32/64 KiB profiles rather than 2/8 KiB.

## Initial opportunity signal

A Poissonized local-information screen is now committed as a diagnostic only.

Using the full vector of independent level counts in the Poissonized model, the
local Cramer-Rao relative standard-deviation lower bound approaches a scaled
coefficient of about 1.30/sqrt(m) through the central lambda/m range.

This is more informative than the current published W_i asymptotic coefficient
1.638/sqrt(m), which is plausible because level counts retain information discarded
by the rightmost-nonzero statistic.

Interpretation:

- this is NOT an achievable estimator claim;
- this is NOT fixed-d;
- this is NOT finite-sample coverage;
- but it is strong enough that the <=32/64 KiB strict-state target is not obviously
  information-theoretically dead.

Therefore Phase A should continue to exact/certified fixed-d tails rather than stop
at A0.

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
