# Snapshot — “От Эвристики к Доказательству: Теоретическое Обоснование Доверительных Границ для GF(2)-F-PCSA при Конечных Параметрах”

Date imported: 2026-10-06.

## What the report gets right

The report strongly reinforces several already-canonical Q1 constraints:

- asymptotic RSE is not finite-sample coverage;
- empirical tails cannot prove very small failure probabilities;
- median amplification needs a valid per-copy bound first;
- unproved independence assumptions are not acceptable;
- saddlepoint approximations need explicit remainder control before they can support Coverage::Proven;
- exact small-parameter distributions, truncation accounting, fixed-cardinality analysis/de-Poissonization, test inversion, and interval width/power are the right research dimensions.

Those points are retained.

## Critical problems in the report

### 1. It analyzes classical PCSA more than the actual published F-PCSA construction

The report repeatedly describes:

- the position of the first 1 in a hash string;
- classical PCSA bitmaps;
- an estimator based on averaging those positions.

That is not the exact published finite-field construction we need to reproduce.

The published F-PCSA paper defines:

~~~text
FIELDMAP[i,j] in F
h : U -> [m] x N, with P(h(v)=(i,j)) = (1/m) * 2^-j
g : U -> F, uniform
update: add k * g(v) into FIELDMAP[h(v)]
statistic: per row, highest index of a non-zero field entry
~~~

Therefore the Q1 proof effort must start from the published FIELDMAP state and rightmost-nonzero statistic, not from a generic classical-PCSA first-1-position model.

### 2. The notation is internally inconsistent

The report uses m both as the number of rows/bitmaps and, in the de-Poissonization section, effectively as the number of inserted elements.

It also uses d as a fringe/truncation width even though DeltaMeter already uses d = ||x||_0 for the symmetric-difference cardinality.

Canonical notation is now:

~~~text
N = universe cardinality
m = F-PCSA row count
d = true Hamming weight / symmetric-difference size
j = level index
J = finite stored maximum level / truncation boundary
~~~

No proof document may reuse d for truncation width.

### 3. The GF(2^V) discussion is speculative and not the published model

The report says GF(2)-F-PCSA “probably” means hashing through an extension field GF(2^V).

That is not the definition in the published paper.

For DeltaMeter, F = GF(2). The universe has N keys. The field order and universe size are separate parameters.

This speculation is rejected.

### 4. Poissonization is promising, but independence must be derived for the exact state

Poisson splitting can make counts in disjoint categories independent under a fully independent assignment model.

However, “the registers become independent” is not accepted as a blanket fact for the concrete F-PCSA statistic until the exact category decomposition, coefficient randomness, finite levels, and hash assumptions are written down.

Poissonization remains a tool, not a proof shortcut.

### 5. The 2025 de-Poissonization paper is promising but not plug-and-play

The cited work on elementary de-Poissonization gives finite-order Poisson-Charlier remainder bounds controlled by higher forward differences of a coefficient sequence.

That is relevant.

But it does not automatically turn an asymptotic F-PCSA CDF into a finite-sample tail theorem.

Before using it, Q1 must define the target sequence a_n (for example a tail probability or test acceptance probability) and prove computable bounds on the required forward differences.

Canonical status:

~~~text
promising theorem candidate
not yet an applicable DeltaMeter theorem
~~~

### 6. The truncated-Gaussian statistics citation is not a direct model of F-PCSA truncation

The cited “Efficient Statistics, in High Dimensions, from Truncated Samples” work studies parameter estimation for a truncated multivariate normal distribution.

F-PCSA finite-level storage is algorithmic censoring/truncation of a discrete sketch state, not that statistical model.

The source may inspire terminology, but it is not a direct theoretical justification and is removed from the main proof path.

### 7. One-sided capacity does not require a two-sided exact test

DeltaMeter needs:

~~~text
Pr[d <= U(S)] >= 1 - delta
~~~

A level-delta one-sided test is enough.

The report spends substantial effort on two-sided central/minlike/Blaker intervals. Those may be useful later, but they are not the minimal Q1 objective.

Also, a valid test only needs Type-I error <= delta; it does not need to equal delta exactly. Conservative coverage is valid.

### 8. Binary search over cardinality is conditional on monotonicity

The report proposes numerical/binary search for confidence limits.

That is only safe if the acceptance region or p-value is monotone in the tested cardinality.

Parity/finite-field statistics can saturate or have non-trivial ordering.

Canonical rule:

> Prove stochastic/test monotonicity first; otherwise invert the test by explicit search over the finite parameter range or another certified method.

### 9. The proposed Markov state is not yet the right exact state

A toy exact dynamic program is a good idea.

But a state vector of first-one positions is classical-PCSA state, not the exact published F-PCSA state.

For Q1, exact enumeration/DP must operate on:

- the actual finite FIELDMAP state, or
- a compressed sufficient statistic whose sufficiency has been proved.

## Canonical effect

The report does not change the v0 architecture.

It sharpens the strict-Parity research path:

~~~text
1. faithful published F-PCSA model
2. exact small finite distribution
3. finite-level/truncation model
4. optional Poissonized representation
5. rigorous fixed-d return / de-Poissonization
6. one-sided test inversion
7. width/power and Pareto decision
~~~

Q1 remains:

~~~text
v0 implementation gate: NO-GO for Coverage::Proven
mathematical research: open
~~~
