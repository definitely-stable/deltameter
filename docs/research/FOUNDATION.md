# DeltaMeter research foundation

This is the canonical synthesis of the DeltaMeter research performed before implementation.

It reconciles the attached F2-centric report with the later GF(2)/L0 analysis and turns the result into an implementation-facing technical record.

## 1. Problem statement

Given two sets (A,B\subseteq[V]), DeltaMeter estimates

[
d = |A\triangle B|.
]

Two equivalent representations are useful.

### Integer signed difference

[
x_i = \mathbf 1_A(i)-\mathbf 1_B(i) \in \{-1,0,+1\}.
]

Therefore,

[
d = \sum_i x_i^2 = F_2(x).
]

This makes AMS, CountSketch-like energy estimators and JL-style norm sketches valid mathematical candidates.

### GF(2) parity difference

For true set semantics,

[
z = \mathbf 1_A + \mathbf 1_B \pmod 2 = A\oplus B.
]

Then

[
d = \|z\|_0.
]

This is a stronger specialization than the generic F2 view. It makes the set-only DeltaMeter a Hamming-weight / finite-field L0 estimation problem.

That distinction drives the backend choice.

## 2. Semantic split

The API must not silently mix these two models.

### Set/toggle semantics

```text
membership is Boolean
toggle(x) is linear over GF(2)
merge difference is XOR
```

This is the natural domain for ParityDeltaMeter.

### General signed/multiset semantics

```text
updates may be +1/-1 or larger signed frequencies
state is over integers/reals
difference is subtraction
```

This is the natural domain for EnergyDeltaMeter or other F2/L0 turnstile estimators.

In GF(2), two identical updates cancel. Therefore repeated `insert(x)` cannot be given ordinary set-insert semantics unless uniqueness is guaranteed externally. A `toggle`-style API is clearer.

## 3. Candidate constructions

### 3.1 Published GF(2)-F-PCSA / finite-field counting

Finite-field PCSA is the strongest published fast-path candidate currently tracked for the narrow set-only direction.

A critical proof boundary now applies:

- the **published F-PCSA** construction includes its published finite-field coefficient/randomness model;
- a proposed DeltaMeter-specific **`g(v)=1` / `cell ^= 1` specialization** is a different estimator.

The published asymptotic constants below belong only to the published construction. They must not be transferred automatically to the set-specialized variant.

Published construction, in implementation-facing terms:

~~~text
FIELDMAP[i,j] in F
h(v) -> (i,j), with probability mass (1/m) * 2^-j
g(v) -> uniform element of F
update: FIELDMAP[h(v)] += k * g(v)
row statistic W_i: highest j with FIELDMAP[i,j] != 0
~~~

For DeltaMeter Parity, F = GF(2). Universe size and field size are separate parameters.

Relevant properties:

- linear over GF(2);
- one small state location changed per update in the published construction;
- packed bit state;
- XOR merge;
- published asymptotic relative-error constant around
  [
  1.638/\sqrt m
  ]
  for the GF(2) case;
- memory proportional to
  [
  m\log_2 V
  ]
  bits for GF(2).

For `N = 2^32`, this is about `4m` bytes:

| m | state | asymptotic RSE |
|---:|---:|---:|
| 128 | 512 B | 14.5% |
| 256 | 1 KiB | 10.2% |
| 512 | 2 KiB | 7.24% |
| 1024 | 4 KiB | 5.12% |
| 4096 | 16 KiB | 2.56% |

These RSE values are asymptotic point-estimation guidance, not strict tail guarantees.

The published analysis explicitly uses a middle-range assumption: very small cardinalities and cardinalities near the universe size need separate treatment. This is one reason Q1 cannot simply promote the asymptotic constant to a full-domain strict profile.

### 3.2 EnergyDeltaMeter

For one table with (B) buckets:

[
c_b = \sum_{i:h(i)=b} x_i\sigma_i,
]

and

[
T = \sum_{b=1}^{B} c_b^2.
]

Under pairwise-uniform bucket collisions and sufficient fourth-order sign independence:

[
\mathbb E[T]=d
]

and, specialized to set difference,

[
\operatorname{Var}(T)=\frac{2d(d-1)}B.
]

Hence

[
\Pr(|T-d|\ge\varepsilon d)
\le
\frac{2}{B\varepsilon^2}
]

by Chebyshev.

With (R) independent tables and median aggregation, the failure bound is the exact upper tail of a Binomial((R,p)), where

[
p\le\frac{2}{B\varepsilon^2}.
]

This is conservative but fully finite-sample.

The repository research harness currently chooses convenient power-of-two (B) values that make (p\approx0.09765625):

| epsilon | B |
|---:|---:|
| 0.05 | 8192 |
| 0.10 | 2048 |
| 0.20 | 512 |

and then the minimum odd `R` for selected `delta`:

| delta | R | bound |
|---:|---:|---:|
| 1e-3 | 9 | 7.98e-4 |
| 1e-6 | 23 | 3.61e-7 |
| 1e-9 | 35 | 5.71e-10 |

These are sufficient profiles, not claims of optimality.

### 3.3 Incremental energy

Energy does not require scanning all buckets on every query.

If a bucket changes from (c) to (c+\Delta),

[
(c+\Delta)^2-c^2 = 2c\Delta+\Delta^2.
]

For unit signed updates, (\Delta^2=1).

Therefore each table can maintain (T) incrementally. The point estimate then costs (O(R)), not (O(BR)).

### 3.4 Dense Gaussian / chi-square oracle

For research only, let (g_j\sim N(0,I)) and

[
Z_j = \langle g_j,x\rangle.
]

Because (\|x\|_2^2=d),

[
Z_j\sim N(0,d).
]

For

[
Q=\sum_{j=1}^{k} Z_j^2,
]

we have the exact pivot

[
Q/d \sim \chi_k^2.
]

Thus an exact one-sided upper confidence bound is

[
U_\delta = \frac{Q}{q_{\chi_k^2}(\delta)},
]

where (q_{\chi_k^2}(\delta)) is the lower (delta)-quantile.

This backend is useful as a statistical oracle because its finite-sample distribution is exact. It is not a production update path because every key touches many measurements.

### 3.5 Sparse JL / sparse sign embeddings

Sparse JL and sparse sign embeddings remain useful comparison points for general norm preservation.

They should not be used to conclude automatically that a one-bucket CountSketch-energy specialization is inferior. Results for general oblivious embeddings solve a broader problem and have different distortion objectives.

For DeltaMeter they are baselines, not the current default.

### 3.6 MinHash and HLL

These are not primary DeltaMeter backends.

MinHash estimates Jaccard similarity. If

[
s=|A|+|B|,
\qquad
J=\frac{|A\cap B|}{|A\cup B|},
]

then

[
d=s\frac{1-J}{1+J}.
]

As (J\to1), small error in (J) becomes large relative error in (d).

HLL estimates cardinalities and unions. Computing a small difference by subtracting several large noisy cardinalities is poorly conditioned when (A\approx B).

They are valuable comparison references, not subtractable symmetric-difference estimators.

## 4. Capacity guarantee

Suppose an estimator satisfies

[
\Pr[(1-\varepsilon)d\le\widehat d\le(1+\varepsilon)d]\ge1-\delta.
]

Then

[
C =
\left\lceil
\frac{\widehat d}{1-\varepsilon}
\right\rceil
]

satisfies

[
\Pr[C\ge d]\ge1-\delta.
]

On the same good event,

[
C
\le
\left\lceil
\frac{1+\varepsilon}{1-\varepsilon}d
\right\rceil.
]

For `epsilon = 0.1`, the multiplicative factor is about `1.2222`.

This distinction matters:

- a one-sided upper bound gives safety;
- a two-sided relative guarantee additionally controls overprovision.

The public API must not merge those two claims.

## 5. Finite-sample Parity problem

The v0 implementation gate is now resolved: **NO-GO for strict Parity `Coverage::Proven`**.

The post-M3 research round closes the current strict published-W_i proof track. The broader GF(2)-linear question remains open only through separate candidate statistics such as per-level parity counts.

The published asymptotic RSE curve does not itself justify

```text
recommended_capacity(delta = 1e-6)
```

or smaller failure probabilities.

For one published GF(2) level with (m) rows, level mass (r), and true difference (d), the executable research oracle tracks exact first two moments using

```text
A = 1 - r/m
B = 1 - 2r/m

E[S] =
    m/2 * (1 - A^d)

Var(S) =
    m/4 * [1 + (m-1)B^d - m A^(2d)]
```

for that model.

The published-W_i strict path is stopped because the exact full-state law is exponential at target dimensions, W_i is not sufficient, the Poissonized exact law lacks a practical certified fixed-d bridge, and full-statistic monotonicity/inversion remain unresolved.

Parity therefore remains a fast point-estimate backend but not a proof-grade capacity backend.

The latest research adds three explicit exclusions:

- asymptotic (1.638/\sqrt m) variance cannot be inserted into Chebyshev/Cantelli and relabelled finite-sample;
- empirical CDF fitting cannot by itself justify (10^{-6}) or (10^{-9}) `Proven` coverage;
- median/group amplification only amplifies a failure bound that is already valid for each independent copy.

Exact small-state enumeration, finite-J truncation budgeting and the unconditional Poissonized law are now executable research artifacts. Fixed-d de-Poissonization/inversion is no longer an active blocker for v0/M4. A narrow ParityLevelCounts track may continue separately.

The newest attached report sharpened this further, but also required corrections:

- Q1 must analyze the exact published FIELDMAP/rightmost-nonzero construction, not a classical-PCSA first-1-position surrogate;
- use N for universe cardinality, m for rows, d for true Hamming weight, j for level, and J for finite stored level bound;
- do not interpret GF(2)-F-PCSA as GF(2^V) hashing;
- Poissonized independence has now been derived for the exact category model; it applies only to the unconditional Poissonized law;
- the 2025 Poisson-Charlier de-Poissonization result is promising only after the target coefficient/tail sequence and its forward-difference bounds are established;
- truncated multivariate-normal estimation is not a direct model for discrete F-PCSA level censoring;
- the minimal capacity target is a one-sided level-delta test inversion, not a two-sided exact interval;
- binary-search inversion requires proved monotonicity.

## 6. Small-d regime

Relative error is awkward at

[
d=0,1,2,\ldots
]

and reconciliation often operates exactly in this regime.

A future hybrid could be

```text
ExactSmallDelta + ParityDeltaMeter
```

where the exact lane handles (d\le T) and falls back above (T).

Candidate families include:

- capped IBLT;
- Simple Set Sketching-style constructions;
- BCH/PinSketch/minisketch-like syndromes;
- power sums;
- XOR buckets plus fingerprints.

This is intentionally not in v0. It becomes worth implementing only if measurements show that the approximate backends leave a real product gap.

## 7. Lower-bound discipline

The following problems are related but not interchangeable:

- insertion-only distinct counting (F_0);
- integer turnstile (L_0);
- general (F_2);
- finite-field (L_0);
- GF(2)-linear Hamming-weight estimation;
- oblivious JL/subspace embedding.

Generic F2 lower bounds do not automatically prove optimality for the narrow GF(2) set-difference model.

Safe public language:

- Energy matches known generic-style dependence in its own conservative construction;
- Parity is based on a published finite-field L0 estimator;
- exact optimality for the narrow DeltaMeter model is not claimed.

## 8. Adversary model

### Oblivious input

The dataset/update sequence is fixed independently of the sketch randomness.

This is the v0 contract.

A secret seed is not required merely to state the classical oblivious-model probability theorem.

### Chosen-input after seed reveal

If an attacker can choose keys after seeing the mapping, ordinary oblivious proofs no longer apply.

A secret or post-commitment/session seed can be a practical mitigation, but that is a different computational/adversarial contract.

### Adaptive queries

If an attacker repeatedly observes sketch outputs and adaptively chooses future inputs, ordinary fixed linear-sketch guarantees should not be claimed.

This is out of scope for v0.

## 9. Randomness requirements

Energy currently needs a concrete family that realizes:

- uniform/pairwise bucket collision probabilities;
- sufficient fourth-order sign independence;
- independence between the bucket and sign families.

The implementation should use the smallest explicit construction that matches the proof. It does not need cryptographic hashing by default.

Parity randomness must be frozen only after the finite-sample proof path is chosen, because the exact independence assumptions may matter to the tail.

## 10. Rust implementation direction

Start with one crate and concrete types.

```rust
pub struct EnergyDeltaMeter { /* ... */ }
pub struct ParityDeltaMeter { /* ... */ }

pub enum Coverage {
    Proven { failure_probability: f64 },
    Asymptotic,
    Empirical,
}
```

Important semantic rules:

- Parity uses explicit set/toggle semantics.
- Strict capacity is exposed only from a backend/configuration with a theorem-backed finite-sample guarantee.
- Merge rejects incompatible configuration.
- Serialization is postponed until state layout is earned by working implementations.
- Stable Rust first.
- Scalar first.
- SIMD only after profiling.

## 11. CI and statistical verification

For extreme (delta), PR CI must not pretend to validate a (10^{-6}) or (10^{-9}) guarantee by naive Monte Carlo.

PR CI should check:

- algebraic identities;
- deterministic reference vectors;
- property tests;
- theorem-derived parameter calculations;
- exact finite formulas where available;
- moderate statistical smoke tests.

Heavier/manual research jobs can:

- regenerate profile tables;
- run broader moment grids;
- run moderate Monte Carlo as diagnostics;
- publish performance artifacts.

Absolute nanoseconds from hosted runners are not merge gates.

## 12. Current backend decision

After the latest Deep Research and attached-report audit, the current v0 direction is:

```text
EnergyDeltaMeter
    = first Rust implementation
    = proof-grade conservative baseline

PublishedFpcsaF2
    = second Rust implementation/reproduction
    = published asymptotic evidence applies only here

ParityDeltaMeter
    = experimental public fast path
    = no strict capacity in v0

ParityPcsaSetV1 (g(v)=1)
    = later research candidate
    = no inherited published error constant

Gaussian / chi-square
    = research oracle only
```

This is intentionally simpler than a three- or four-backend framework.


## 13. Latest-source reconciliation

Five new attached reports and the latest Q1 Deep Research run are preserved under `archive/2026-10-06/`.

Their conflicting recommendations are reconciled in [NEW-INPUTS-AUDIT.md](NEW-INPUTS-AUDIT.md).

The canonical simplifications are:

- no Minisketch/FFI in v0;
- no early persisted wire protocol;
- no default cryptography;
- Energy keeps pairwise bucket + 4-wise sign proof assumptions;
- public CI can be used freely, but rare-event Monte Carlo remains diagnostic rather than proof;
- formal verification remains post-v1 unless a narrow proof boundary justifies it.
