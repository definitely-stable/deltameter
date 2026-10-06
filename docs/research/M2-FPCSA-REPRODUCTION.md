# M2 — Published GF(2)-F-PCSA reproduction

Status: complete; merged in PR #5  
Primary source: Dingyu Wang, Probabilistic Counting in Generalized Turnstile Models, arXiv:2310.14977v1.

## Objective

M2 reproduces the published finite-field PCSA construction for F = GF(2) before DeltaMeter experiments with any custom set-specialized variant.

This is intentionally not M3. It does not freeze the public ParityDeltaMeter API and does not add strict capacity coverage.

## Source construction

Definition 6 in the paper stores:

~~~text
FIELDMAP[i,j] in F

h : U -> [m] x N
P(h(v) = (i,j)) = (1/m) * 2^-j

g : U -> F
g(v) uniform over F

update(v,k):
    FIELDMAP[h(v)] += k * g(v)
~~~

For F = GF(2) and a boolean toggle update k = 1, the cell update is XOR with the sampled coefficient bit.

The paper's estimator uses the highest non-zero index W_i per row and an exponential/geometric-mean estimator. Its Table 2 reports, for GF(2), the rounded asymptotic normalization 1.079*m and relative standard error 1.638/sqrt(m).

## What is reproduced in Rust

PublishedFpcsaF2 implements:

- one packed bit per finite FIELDMAP[i,j] cell;
- rows m >= 1, matching the construction rather than imposing an estimator-quality threshold;
- one-based levels 1..=J;
- deterministic row/level/coefficient mapping from a key;
- exact GF(2) cancellation on repeated toggles;
- XOR sketch composition;
- highest non-zero level per row;
- a diagnostic-only Table 2 reference estimate.

For J = 64, the implementation covers the full geometric level range representable from one uniform u64 word. J is a stored-level boundary, not a synonym for universe cardinality. A sampled level above J is returned as an explicit Truncated update and is not silently clamped.

## Random-oracle boundary

The paper assumes ideal random oracles for h and g.

The M2 Rust reproduction instead uses a deterministic, domain-separated 64-bit mixing function with three explicit seeds:

~~~text
row seed
level seed
coefficient seed
~~~

Row selection uses rejection sampling so that, if the underlying words were independent uniform values, the row distribution would be exactly uniform for arbitrary m.

Level selection uses the count of trailing zero bits:

~~~text
level = 1 + trailing_zeros(word)
~~~

which has P(level = j) = 2^-j under a uniform word.

The coefficient uses one independent pseudo-oracle bit.

This practical deterministic mapping is for reproducibility. It is not claimed to instantiate the paper's ideal random oracle theorem.

## Finite-state boundary

The paper's main analysis makes three simplifications:

1. ideal random oracle;
2. poissonization / independent cells;
3. random offsets that smooth multiplicative periodic fluctuations.

It also explicitly assumes the cardinality is in the middle range:

~~~text
d >> 1
d << N
~~~

and says separate methods are needed for constant-size and near-universe cardinalities.

M2 does not hide those assumptions.

The Rust state is finite and offset-free. Therefore table2_reference_estimate() is a reproduction aid only:

~~~text
point ≈ 1.079 * m * 2^(mean highest level)
RSE   ≈ 1.638 / sqrt(m)
~~~

It is not Coverage::Proven, not a finite-sample interval, and not the final M3 estimator contract.

## Exact small-case moment regression

The existing one-level analytical oracle uses:

~~~text
A = 1 - r/m
B = 1 - 2r/m

E[S] =
    m/2 * (1 - A^d)

Var(S) =
    m/4 * [1 + (m-1)B^d - m A^(2d)]
~~~

where r is the total hash mass of the selected level across all rows.

research/fpcsa_exact_small.py independently computes the exact parity-mask distribution for small m,d by dynamic programming.

For one active key at that level:

~~~text
P(toggle row i) = r/(2m)
P(no parity contribution at that level) = 1 - r/2
~~~

The exact DP then checks that its mean and variance match the analytical formulas. This is a finite exact regression, not Monte Carlo.

## Deliberate exclusions

M2 does not implement:

- g(v)=1 / ParityPcsaSetV1;
- classical PCSA leftmost-zero state;
- strict finite-sample upper capacity;
- de-Poissonization;
- random-offset implementation;
- Minisketch/FFI;
- persisted wire format;
- crypto/key-management;
- SIMD/unsafe.

## M2 acceptance interpretation

M2 is complete when:

- packed-state algebra and cancellations are correct;
- source construction and finite truncation are explicit;
- exact small-case moments match the analytical oracle;
- Table 2 constants are labelled asymptotic/reference-only;
- CI is green.

Whether this reproduction becomes the public ParityDeltaMeter is an M3 decision.
