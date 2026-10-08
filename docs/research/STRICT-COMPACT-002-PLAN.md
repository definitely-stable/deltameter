# STRICT-COMPACT-002 — declared-range and oracle-contract closure plan

Issue: #63. Parent: #59.

Status: research proof-closure slice. No public API or Coverage promotion.

## Objective

Finish the 32 KiB STRICT-COMPACT proof boundary after #62 establishes a practical
integer lookup and an independently certified Q32 threshold table.

Two gates are independent:

1. RANGE_CERT_PASS — q95 U/d <= 1.5 for every integer d in a declared useful range;
2. ORACLE_CONTRACT_PASS — implementation probability claim no stronger than the
   randomness assumption actually provided by the Rust oracle.

## Frozen 32 KiB profile

- m = 4096 rows
- J = 64 stored levels
- primary state = 32,768 B
- delta = 1e-6
- Q32 threshold table from #62
- table SHA256 must match the exact-head generated table
- declared width target: P_d(U/d > 1.5) <= 0.05

The first useful-range target is d in [4096, 2^64-1]. Coverage itself remains
one-sided for all d; the lower endpoint applies only to the 1.5x q95 width claim.

## Continuous interval proof

The range verifier consumes the actual runtime Q32 table.

For each integer interval [a,b]:

- K_a = floor(1.5a);
- derive exactly from the runtime table the minimum count t_j(K_a) for which a
  chosen level's integer upper bound exceeds K_a;
- use monotonicity in K and in fixed-d level occupancy to reduce every d in [a,b]
  to the single worst-case tail P_b[S_j >= t_j(K_a)];
- use factor-two de-Poissonization and KL-Chernoff for that upper tail;
- prove the <=0.05 inequality with exact rational/integer arithmetic.

Binary64 is allowed only to rank candidate levels. It cannot make an interval pass.

Intervals begin as octave-sized blocks and recursively split only when the
monotone endpoint reduction is too pessimistic. The final artifact must be a
hole-free exact integer cover.

Near the top of the u64 universe, if floor(1.5d) >= 2^64, the width property is
trivial because every valid upper bound is <= domain cardinality 2^64. Such
intervals are marked domain_ceiling rather than presented as statistical evidence.

## Exact arithmetic route

For z=d/(m*2^j), build exact Taylor lower/upper bounds on exp(z):

- finite Taylor sum is a lower bound;
- the omitted tail is upper-bounded by a geometric series once z/(n+2)<1.

This yields an exact rational upper bound on

p_j(d) = (1-exp(-z))/2.

Round p upward to Q64. For upper-tail x=t/m>p, D(x||p) decreases as p increases,
so it is sufficient to prove the KL inequality at that larger rational p.

Exponentiating mD removes logarithms. For q95 beta=0.05 the proof target is exactly

2/beta = 40.

The final check is therefore an integer likelihood-ratio comparison only.

## Oracle contract

The probability theorem assumes ideal independent random row/level/coefficient
sampling.

Current PublishedFpcsaOracle is a deterministic pseudo-random engineering oracle
based on fixed seeds and mix64. Its own docs already say it is not the ideal random
oracle.

Therefore #63 must not expose Coverage::Proven on this implementation.

Preferred outcome for this slice:

GO_RESEARCH_ONLY_ORACLE_GAP

if range certification passes but no rigorous concrete hash-family/random-seed
contract is yet implemented.

A later production issue may evaluate:
- fresh unpredictable per-meter keyed hashing with a computational PRF assumption;
- or a concrete sufficiently independent hash family with a proof matching the
  concentration argument.

Fixed public deterministic seeds are not enough for an adversarial-key guarantee.

## Gates

RANGE_CERT_PASS requires:
- generated table independently re-certified;
- every integer d in [4096, 2^64-1] covered without holes;
- every statistical interval accepted only by exact rational/integer proof;
- domain-ceiling intervals explicitly separated;
- artifact records interval witnesses and levels used.

ORACLE_CONTRACT_PASS requires a separately justified probability source. This slice
is expected to document the gap rather than silently close it.

## Expected decision

- if continuous range passes and oracle remains deterministic: GO_RESEARCH_ONLY_ORACLE_GAP;
- if range needs a narrower lower endpoint: SHRINK_DECLARED_RANGE with exact endpoint;
- if range has internal holes: do not productize and investigate before any API;
- only a future oracle/implementation slice can reach GO_COMPACT_STRICT_PRIVATE_API.

No snapshot-v1 changes, public profiles, estimator defaults, or reconciliation work.
