# STRICT-COMPACT-002 — declared-range and oracle-contract evidence

Issue: #63. Parent: #59.

Status: **GO_RESEARCH_ONLY_ORACLE_GAP**.

The 32 KiB STRICT-COMPACT candidate now has:

- independently certified runtime Q32 thresholds;
- an actual integer-only packed-state inference path;
- a continuous exact-rational q95 width certificate over the declared useful range;
- but no public strict coverage claim for the current deterministic pseudo-oracle.

## Candidate profile

~~~text
rows                    4096
stored levels           64
primary state           32,768 bytes
static lookup table     16,384 bytes
coverage delta          1e-6
width contract          q95 U/d <= 1.5
declared useful d       4096 .. 2^64-1
~~~

Canonical runtime-table SHA-256:

`634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`.

## Continuous range proof

The range is not inferred from sampled anchor points.

For every certificate interval `[a,b]`:

1. set `K_a=floor(1.5a)`;
2. use the actual runtime Q32 table to derive the exact observed-count threshold
   `t_j(K_a)` for a certifying stored level;
3. monotonicity of the integer lookup in K and stochastic monotonicity of
   `S_j(d)` reduce every `d in [a,b]` to
   `P_b[S_j >= t_j(K_a)]`;
4. factor-two monotone de-Poissonization reduces the fixed-d upper tail to a
   Poissonized binomial upper tail;
5. KL-Chernoff gives the analytic tail bound;
6. the implementation certifies that inequality with exact rational/integer
   arithmetic.

The exact proof boundary does not use binary64, Decimal, exp, log or the generator's
KL evaluator. Binary64 only ranks candidate levels.

For q95 beta=0.05, the exact likelihood-ratio proof target is

`2/beta = 40`.

## Hosted result

Range workflow run `37720859516`:

~~~text
STRICT_COMPACT_Q32_CERT_PASS
STRICT_COMPACT_RANGE_CERT_PASS
  intervals=876
  exact_tail_intervals=875
  d_min=4096
  d_max=18446744073709551615
  levels_used=1..52
~~~

Artifact:

~~~text
id      11526035870
sha256  a0e0f282987367c556bb99b30a1ff9b0e2388a0c7ffe136f98a4bebe369eb8e2
~~~

The 875 statistical intervals form a hole-free integer cover from 4096 up to the
point where the width statement becomes deterministic from the domain ceiling.

The final interval is marked `domain_ceiling`: once `1.5d >= 2^64`, every valid
upper bound is already <= the u64 token-domain cardinality `2^64`. That region is
not presented as additional estimator information.

## What is now mathematically closed

Under the ideal independent-sampling model used by the theorem:

- each committed finite Q32 threshold is conservative;
- each infinity sentinel is valid;
- Bonferroni over 64 levels gives one-sided failure <=1e-6;
- for every integer d in [4096,2^64-1],
  `P_d(U/d > 1.5) <= 0.05`;
- primary state is 32 KiB, approximately 11.5x smaller than the default strict
  Energy state of 376,832 bytes.

This is no longer a sampled-width hypothesis.

## Oracle gap

The current Rust implementation does **not** yet realize the theorem's probability
space as a public guarantee.

`PublishedFpcsaOracle` uses deterministic `mix64` with fixed/configured u64 seeds.
It is explicitly an engineering pseudo-oracle, not:

- an ideal random oracle;
- a cryptographic keyed PRF with a stated computational assumption;
- or a proven finite-independence hash family sufficient for the tail theorem.

For fixed seeds and fixed keys, there is no sampling randomness left. An adversarial
key set is therefore outside the theorem's implementation guarantee.

Canonical analysis:
`docs/research/STRICT-COMPACT-002-ORACLE-CONTRACT.md`.

## Decision

**GO_RESEARCH_ONLY_ORACLE_GAP**.

Do not change:

- `ParityDeltaMeter::estimate()`;
- `Coverage::Asymptotic`;
- snapshot v1;
- public profiles/defaults;
- crate publication state.

The estimator/state/inference research has crossed its compactness, coverage and
width gates in the ideal model. The next product decision is now narrowly about the
oracle:

1. implement a fresh keyed cryptographic PRF/hash and expose a clearly
   computational coverage contract; or
2. prove a concrete finite-independence family sufficient for the theorem; or
3. retain STRICT-COMPACT as research-only.

No additional multilevel inference mathematics is currently justified.
