# STRICT-COMPACT-001 — Phase A foundation evidence

Issue: #59. PR: #61.

Status: **FOUNDATION_GO — continue to conservative numerical implementation and
declared-range closure. No public/API promotion yet.**

This document records the first post-M6 product-revalidation result for DeltaMeter.

## Result in one sentence

The current GF(2)-F-PCSA/Parity state admits a new theorem-derived one-sided
fixed-d upper-bound construction based on per-level odd-row counts, and the
32 KiB / 64 KiB profiles already cross the frozen 1.5x width target on the
reported fixed-d certificate grid.

This does **not** yet promote ParityDeltaMeter to Coverage::Proven. The remaining
blocking work is conservative floating/numerical implementation plus closure of a
declared useful d-range.

## 1. Exact statistic

For stored level j and m rows define S_j(d) as the number of odd GF(2) cells after
exactly d active symmetric-difference elements.

One element toggles one uniformly selected level-j row with total probability

    r_j = 2^-(j+1).

Therefore S_j is exactly the lazy Ehrenfest birth-death chain

    P(s->s+1) = r_j (m-s)/m
    P(s->s-1) = r_j s/m
    P(s->s)   = 1-r_j.

The existing finite-J truncation problem for the published W_i statistic is not a
proof blocker for one stored S_j. Outcomes assigned to higher levels are simply
idle for level j.

The dependency-free Fraction oracle in
research/strict_compact_level_count.py checks exact mass, exact moments and
stochastic monotonicity on small grids.

## 2. Classical monotonicity, not claimed novelty

For the birth-death kernel,

    p_s + q_(s+1) = r_j (m+1)/m <= 1/2,

because j>=1 and r_j<=1/4.

This satisfies the standard adjacent-state monotone-kernel condition. Starting from
zero, the resulting laws are stochastically nondecreasing in d:

    S_j(d) <=_st S_j(d+1).

DeltaMeter treats this as an application of classical monotone birth-death theory.

## 3. Monotone de-Poissonization

Let

    F_d(s) = P[S_j(d) <= s].

For N~Poisson(lambda), define

    G_lambda(s) = E[F_N(s)].

Because F_n(s) is nonincreasing in n,

    G_lambda(s) >= P[N<=d] F_d(s).

Set lambda=d. K. P. Choi's sharp Poisson-median result gives

    mu-log(2) <= median(Poisson(mu)) < mu+1/3.

For integer d, d is therefore a median of Poisson(d), hence

    P[N<=d] >= 1/2

and

    F_d(s) <= 2 G_d(s).

Reference:
K. P. Choi, "On the Medians of Gamma Distributions and an Equation of
Ramanujan", Proceedings of the American Mathematical Society 121(1), 1994,
DOI 10.2307/2160389.

research/strict_compact_depoisson.py checks the theorem-derived inequality against
the exact small Fraction oracle. The floating check is regression evidence only;
the theorem does not depend on binary64.

## 4. Closed-form finite-sample lower-tail bound

Under Poissonization the stored level cells split independently:

    S_j ~ Binomial(m, p_j(d))

with

    p_j(d) = (1-exp(-d/(m*2^j)))/2.

For x=s/m < p, standard binomial KL-Chernoff gives

    G_d(s) <= exp(-m D(x || p_j(d))).

Combining with the factor-two de-Poissonization result:

    F_d(s)
      <= min(1, 2 exp(-m D(s/m || p_j(d)))).

This is the primary strict single-level candidate. It avoids requiring a
production-size Krawtchouk solver or tiny incomplete-beta probabilities in the
coverage theorem.

For deterministic per-level budgets alpha_j with sum alpha_j<=delta, invert the
bound monotonically to obtain U_j. Then

    U = min_j U_j

satisfies the mathematical familywise target

    P_d(U < d) <= delta,

subject to a future conservative numerical implementation of the analytic formulas.

The exact tiny full-state regression in
research/strict_compact_multilevel_exact.py checks the complete composition:
full GF(2) state law -> level counts -> per-level inversion -> Bonferroni min ->
exact failure probability.

## 5. Fixed-d width certificate

The same argument applies to fixed-d upper tails. Since upper-tail probability is
nondecreasing in d and integer d is a Poisson(d) median,

    P_fixed_d[S_j >= t]
      <= 2 P_poissonized_d[S_j >= t].

For t/m > p_j(d), KL-Chernoff yields

    P_fixed_d[S_j >= t]
      <= 2 exp(-m D(t/m || p_j(d))).

For a target K>=d, event U>K implies U_j>K for every j. Selecting the best single
level therefore gives a valid upper bound on P_d(U>K), without assuming independence
between levels.

research/strict_compact_width_certificate.py evaluates these theorem-derived
fixed-d width bounds.

Primary settings:

    J = 64
    coverage delta = 1e-6
    q95 width failure = 0.05
    q99 width failure = 0.01

Reported log2-phase grid uses 16 phases in one octave and octave shifts
0,1,2,4,8,16,32.

### Width results

| State | rows m | worst reported q95 U/d | worst reported q99 U/d | frozen 1.5x q95 gate |
|---|---:|---:|---:|---|
| 8 KiB | 1024 | ~2.056 | ~2.145 | MISS |
| 32 KiB | 4096 | ~1.429 | ~1.458 | **PASS** |
| 64 KiB | 8192 | ~1.285 | ~1.306 | **PASS** |

The 32 KiB profile also stays below 1.5x on the reported q99 grid.

Low-d anchor scan is intentionally reported separately because relative width is
poor when d is tiny:

- 32 KiB: first reported anchor crossing q95 <=1.5 is d=2560
  (U/d ~=1.476);
- 64 KiB: first reported anchor crossing q95 <=1.5 is d=2048
  (U/d ~=1.472).

These anchor values are not yet a proof that every d above the anchor satisfies the
same width gate. Declared-range closure is still required.

## 6. Product interpretation

The strongest current candidate is **32 KiB**, not the existing 2 KiB Standard
profile.

Compared with default strict Energy state:

    Energy 10%, delta<=1e-6: 376,832 B
    STRICT-COMPACT candidate: 32,768 B

That is about an 11.5x state reduction while retaining the same linear/XOR primary
Parity state representation.

The candidate does not change toggle or merge semantics; the new work is inference
over the existing packed state.

The 64 KiB profile has a larger safety/width margin but only about a 5.75x state
reduction versus Energy.

The 8 KiB profile is not currently product-competitive at the frozen 1.5x q95 gate.

## 7. What remains before GO_COMPACT_STRICT

Foundation evidence is strong enough to continue, but not enough for public Proven
coverage.

Required next work:

1. implement the KL/de-Poissonized inversion with explicitly conservative numerical
   rounding so floating evaluation cannot understate U;
2. close a declared fixed-d useful range, including log2 phase and upper-domain
   boundaries, rather than only sampled points;
3. verify extraction of S_j from the actual packed Parity state and measure estimate
   CPU/allocation cost;
4. test deterministic pseudo-oracle engineering assumptions separately from the
   ideal-oracle probability theorem;
5. freeze the exact product contract: returned integer domain, saturation /
   unavailable semantics, delta profiles and state profiles;
6. only then decide GO_COMPACT_STRICT vs GO_DEEPER_INFERENCE.

## 8. Public boundary

No production/public claim follows from this PR.

Keep unchanged:

- ParityDeltaMeter::estimate();
- Coverage::Asymptotic;
- snapshot v1;
- current public profiles/defaults;
- crate publish=false.

A Proven compact API requires a later implementation/evidence slice.
