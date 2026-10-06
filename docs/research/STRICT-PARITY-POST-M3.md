# Post-M3 strict-Parity synthesis

Status: **canonical research decision**
Date: 2026-10-06
Tracks: published GF(2)-F-PCSA, ParityDeltaMeter, optional ParityLevelCounts

## Executive verdict

The three new research reports agree on the most important product boundary:

~~~text
EnergyDeltaMeter
    Coverage::Proven
    strict finite-sample backend

ParityDeltaMeter
    Coverage::Asymptotic
    experimental published GF(2)-F-PCSA wrapper
~~~

The repository therefore closes the current **published W_i strict-Parity track** as NO-GO.

This is not an impossibility theorem for every GF(2)-linear cardinality sketch. It is a stop decision for trying to turn the current published-W_i backend into a practical fixed-d high-confidence capacity estimator without a new theorem.

A narrow research candidate remains:

~~~text
ParityLevelCounts
    statistic S_j = popcount(FIELDMAP[:,j])
    same mergeable FIELDMAP state
    exact one-level finite laws available
    no Coverage::Proven yet
~~~

No hybrid/REPLACE architecture is accepted from the reports. Simple Set Sketching, Minisketch, IBLT-derived estimators and similar candidates require separate primary-source and contract audits.

## Evidence labels

Every statement below is classified as one of:

- **THEOREM** — exact mathematical statement under explicit assumptions;
- **DERIVED RESULT** — direct consequence of accepted exact results;
- **EXACT NUMERICAL RESULT** — finite computation without statistical approximation;
- **ASYMPTOTIC RESULT** — limiting result only;
- **HEURISTIC** — engineering/statistical approximation;
- **EMPIRICAL RESULT** — simulation/measurement;
- **OPEN** — not established.

## Canonical model

For the published GF(2)-F-PCSA reproduction:

~~~text
d = |A △ B|

rows:
    i in {0,...,m-1}

levels:
    j in {1,2,...}
    P(level=j) = 2^-j

row:
    uniform over m rows

coefficient:
    g(v) uniform in GF(2)
    P(g=1) = 1/2

stored state:
    FIELDMAP[i,j] for 1 <= j <= J
~~~

For a stored cell c=(i,j), define the h-mass

~~~text
q_c = 2^-j / m.
~~~

One active key flips that cell with probability q_c/2.

The implementation uses one-based levels. Research formulas must not silently switch to zero-based indexing.

## Accepted finite-state results

### 1. Full-state Fourier law

**THEOREM**

Let K=mJ and let S be the packed stored state in GF(2)^K after exactly d active elements.

For a in GF(2)^K define

~~~text
Q(a) = sum_{c : a_c=1} q_c.
~~~

Then

~~~text
P_d[S=s]
=
2^-K
sum_a
(-1)^(a dot s)
(1 - Q(a))^d.
~~~

This is exact fixed-d mathematics.

It does not make the full likelihood practical: direct evaluation still has 2^(mJ) Fourier terms.

### 2. One-level law

For one level whose total h-mass across all rows is r, let S_j be the number of non-zero parity cells at that level.

**THEOREM**

~~~text
G_{r,d}(u)
=
E[u^S_j]
=
2^-m
sum_{k=0}^m
C(m,k)
(1+u)^(m-k)
(1-u)^k
(1-r k/m)^d.
~~~

Therefore:

~~~text
E[S_j]
=
m/2 * (1 - (1-r/m)^d)

Var(S_j)
=
m/4 *
[
    1
    + (m-1)(1-2r/m)^d
    - m(1-r/m)^(2d)
].
~~~

The existing one-level moment oracle and exact-small DP are consistent with this model.

### 3. W_i is not sufficient

**THEOREM / finite counterexample**

For m=1 and J=2, the stored states

~~~text
level-2-only
both-levels-set
~~~

have the same highest level W=2.

With d=1, a single active key can set the level-2-only state but cannot set both levels simultaneously. With d=2, the both-levels state has positive probability.

Thus states with the same W have d-dependent conditional probabilities. W is not a sufficient statistic for the fixed-d full-state family.

This conclusion does not require importing Fisher-information claims from classical PCSA.

## Poissonized law

Let D ~ Poisson(lambda).

### Cell independence

**THEOREM**

Poisson splitting gives independent contribution counts for every stored cell.

For c=(i,j):

~~~text
rate(g=1 contribution to c)
=
lambda * q_c / 2.
~~~

The parity bit is one with probability

~~~text
pi_c(lambda)
=
(1 - exp(-lambda q_c)) / 2.
~~~

Therefore stored cells are independent in the unconditional Poissonized model.

Rows are also independent because each row depends on a disjoint group of independent cells.

### Row statistic

For one row:

~~~text
P_lambda(W=0)
=
product_{j=1}^J (1-pi_j)

P_lambda(W=j)
=
pi_j
product_{l=j+1}^J (1-pi_l).
~~~

Hence the distribution of

~~~text
Y = sum_i W_i
~~~

is a convolution of m iid row distributions in the Poissonized model.

This is exact for Poissonized cardinality. It is not a fixed-d confidence theorem.

## Fixed-d bridge

The exact relation is

~~~text
A_lambda(t)
=
sum_{d>=0}
P_d[T <= t]
exp(-lambda) lambda^d / d!.
~~~

Equivalently, fixed-d probabilities are coefficients of the exponential generating function.

This establishes an exact bridge in principle.

The remaining problem is computational certification:

- full-state coefficient extraction is exponential;
- generic analytic de-Poissonization does not automatically provide a useful explicit remainder for the concrete statistic;
- no repository artifact currently certifies a fixed-d 1e-6 or 1e-9 tail from the Poissonized law.

Canonical status:

~~~text
Poissonized law: THEOREM
fixed-d strict bridge: OPEN
~~~

## Monotonicity: reconciled result

The reports conflict strongly here.

### Individual row W_i

**THEOREM**

For a fixed row and threshold w, the event W_i <= w means all stored cells above w are zero.

Its exact Walsh expression is a positive average of terms

~~~text
(1 - Q(a))^d
~~~

with bases in [0,1].

Therefore

~~~text
P_d(W_i <= w)
~~~

is non-increasing in d, so one row W_i is stochastically non-decreasing.

### Full published statistic

**OPEN**

The reports make two incompatible claims:

- one report infers monotonicity of sum_i W_i from the row marginals;
- another asserts a cancellation counterexample.

Neither is accepted as a general theorem.

Marginal stochastic order does not by itself imply stochastic order of a sum when the joint dependence changes with d.

The repository therefore keeps:

~~~text
sum_i W_i monotonicity: OPEN
published point-estimator monotonicity: OPEN
~~~

The tiny exact-state script searches finite grids, but a passed finite grid is not a proof.

### Per-level S_j

The expectation E[S_j] is monotone in d.

That does **not** prove stochastic monotonicity of S_j.

Canonical status:

~~~text
mean monotonicity: THEOREM
distributional monotonicity: OPEN
~~~

This distinction is required before any binary-search inversion is accepted.

## Finite-J truncation: corrected contract

The new reports mix three different probabilities.

For one-based levels:

~~~text
P(level > J) = 2^-J.
~~~

### Implementation truncation signal

The current implementation surfaces Truncated when the sampled level is outside storage, before using the coefficient.

Therefore:

~~~text
p_signal = 2^-J
P(any signal among d)
=
1 - (1-2^-J)^d.
~~~

### State-distorting truncation

The ideal infinite state differs only if the out-of-range coefficient is nonzero:

~~~text
p_distortion
=
P(level>J and g=1)
=
2^-(J+1)

delta_distortion(d,J)
=
1 - (1-2^-(J+1))^d.
~~~

There is no 1/m factor after summing over all possible rows.

A report formula with 1/m is a per-row quantity and must not be used as the whole-sketch truncation probability.

Truncation is analytically manageable. It is not the main strict-Parity blocker.

## Small-d structural floor

**DERIVED RESULT**

For d active elements, the event that every field coefficient is zero has probability

~~~text
2^-d.
~~~

That event leaves the entire sketch zero.

Therefore, for a valid upper bound U at the zero observation:

~~~text
if 2^-d > delta,
then U(0) >= d.
~~~

Equivalently the forced floor is

~~~text
max { d in N : 2^-d > delta }.
~~~

For the current target deltas this lower floor is:

~~~text
delta=1e-3 -> 9
delta=1e-6 -> 19
delta=1e-9 -> 29
~~~

The 10/20/30 values appearing in one report are conservative round-ups, not the exact theorem-implied floor.

The true zero-state probability can be larger because cancellation creates additional zero states.

## One-sided inversion

The abstract construction remains valid.

For a candidate d0 and lower-tail statistic T, define

~~~text
p_d0(t)
=
sup_{d >= d0} P_d[T <= t].
~~~

Reject H0: d>=d0 only when p_d0(T_obs) <= delta.

This yields a valid upper confidence set if the supremum is correctly bounded.

If stochastic monotonicity is proved, the supremum may reduce to the boundary d0 and binary search may become valid.

Without that theorem, binary search over d is not accepted.

## ParityLevelCounts: narrow continuation track

The most promising non-production research direction from the new reports is:

~~~text
S_j = popcount(FIELDMAP[:,j]).
~~~

Why it is interesting:

- it uses the already mergeable FIELDMAP state;
- it retains information discarded by W_i;
- every one-level marginal has an exact finite-d law;
- a family of valid per-level upper bounds could be combined conservatively.

If valid per-level bounds U_j exist with failure budgets delta_j, then

~~~text
U = min_j U_j
~~~

has failure probability at most sum_j delta_j by the union bound.

What is still missing:

- a proved or certified sup_{d>=d0} tail for each level;
- high-precision/outward-rounded numerical evaluation at delta=1e-9;
- width/power showing the bound is useful;
- small-d handling;
- complete state/update/query cost accounting if independent copies are needed.

Therefore:

~~~text
ParityLevelCounts
    research candidate
    no public backend
    no Coverage::Proven
~~~

## Claims not accepted from the reports

### Full-statistic monotonicity from row marginals

Rejected.

Row-wise stochastic monotonicity is insufficient to order a sum under changing dependence.

### Cancellation counterexample as a proof against the actual published scalar

Not accepted without an explicit exact counterexample for the actual lazy geometric GF(2)-F-PCSA statistic.

Generic parity walks can be non-monotone, but that does not automatically settle this construction.

### S_j stochastic monotonicity from E[S_j]

Rejected.

Monotone expectation is weaker than stochastic order.

### 1/m truncation budget for the whole sketch

Rejected.

The row probabilities sum to one. Whole-sketch state-distorting probability per key is 2^-(J+1).

### IBLT quadratic estimator = finite-sample Proven confidence

Not accepted.

Exact mean/variance plus a chi-square limit would still leave the confidence interval asymptotic unless a separate finite-sample tail theorem is supplied.

The cited 2026 identifier was not source-verified in this audit, so it cannot change architecture.

### Simple Set Sketching = unconditional Coverage::Exact

Not accepted as stated.

Simple Set Sketching is an exact-recovery data structure on successful decoding, with high-probability recovery in its random-hypergraph regime. That is not the same contract as unconditional deterministic Coverage::Exact.

It remains a candidate for a future exact-small-d lane after a separate failure/verification/API audit.

### Full parity profile breaks mergeability

Rejected.

The raw FIELDMAP remains XOR-mergeable. A richer query statistic computed from the merged raw state does not destroy sketch mergeability.

A derived histogram by itself may not be linearly mergeable, but that is an API/storage choice, not a property of the underlying state.

### Energy memory numbers in the alternative report

Rejected.

Canonical Energy profile sizes are generated by research/energy_profiles.py.

For epsilon=10%, delta=1e-6:

~~~text
B=2048
R=23
47104 i64 counters
376832 bytes
~~~

Numbers such as 47 KB or 72 KB for that strict profile omit the table amplification or use a different accounting model.

### Adaptive attacks as a v0 stop condition

Rejected.

They are relevant to an adaptive/chosen-query threat model.

DeltaMeter's accepted theorem model is oblivious input, so adaptive-attack results are documented context, not a reason to invalidate the current backend.

## Alternative backends: current status

### Simple Set Sketching

Primary-source abstract supports linear-time exact recovery with high probability below a random-hypergraph load threshold.

Status:

~~~text
future exact-small-d candidate
not adopted
not Coverage::Exact by default
~~~

### Minisketch / PinSketch

Coding-theoretic exact recovery up to a configured capacity remains a strong small-d candidate.

Status:

~~~text
future candidate
still deferred
~~~

### IBLT-derived quadratic/cardinality estimator

The new report claims an exact-moment estimator and finite-sample confidence.

The confidence claim is not accepted without the primary theorem and a finite-sample tail audit.

Status:

~~~text
source verification required
no architecture change
~~~

### Classical PCSA Fish-number / richer estimators

The Pettie/Wang information-theoretic work confirms that richer PCSA state can contain more asymptotic information than a simple max statistic in classical cardinality sketches.

It is useful motivation, but it does not automatically transfer a theorem to the finite-field GF(2)-F-PCSA model.

The direct finite counterexample above is sufficient for the repository's W_i non-sufficiency statement.

## Stop gate

The current published-W_i strict track is stopped because all of the following are simultaneously true:

1. the exact full-state law is exponential at target m,J;
2. W_i discards finite-d information;
3. the exact Poissonized law does not yet have a certified practical fixed-d bridge;
4. full-statistic stochastic monotonicity is unresolved;
5. no practical 1e-6/1e-9 one-sided fixed-d inversion exists;
6. the strict Energy backend already exists.

This stop gate can be reopened only by a concrete new result that supplies at least one of:

- a practical fixed-d tail theorem for the published statistic;
- a proved monotone sufficient/reduced statistic with certified inversion;
- a certified ParityLevelCounts construction with useful width;
- a different GF(2)-linear backend whose complete strict Pareto frontier beats Energy.

## Repository consequences

Current architecture remains:

~~~text
EnergyDeltaMeter
    strict/default
    Coverage::Proven

ParityDeltaMeter
    experimental
    Coverage::Asymptotic

PublishedFpcsaF2
    research reproduction

ParityLevelCounts
    research-only candidate
~~~

M4 performance/API work remains the next implementation milestone.

Strict-Parity research is no longer an M4 blocker.
