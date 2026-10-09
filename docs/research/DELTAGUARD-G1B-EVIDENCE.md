# DeltaGuard G1-B1 evidence — exact fixed-d parity and unit-coefficient specialization

Issue #89, parents #84/#85; threat model #86.
Frozen protocol: [DELTAGUARD-G1B-PROTOCOL.md](DELTAGUARD-G1B-PROTOCOL.md).
Research-only: **NOT a public finite-sample API/secret-PRF security theorem**.

## Outcome

**G1B_EXACT_CHAIN_CERT_PASS** (24 independent table profiles,
162 independent finite-bitmask oracle comparisons);
**G1B_B1_XOR_PASS_EMPIRICAL_UNIT_CANDIDATE**
(1,800 source-bound two-owner parity rows, five GitHub-hosted workers).

Evidence source commit: `b622d556efcee4cb7bd11d784f131ae3a4ff3918`.
Hosted [workflow 37881841587](https://github.com/definitely-stable/deltameter/actions/runs/37881841587)
completed exact certificate, worker 1..5 and fail-closed aggregator successfully.
Certified TSV SHA256:
`90c05149db6c81bbc615bc4dbaf23a1b7295c22c765d0125036b6c81ceeb4244`.

No new general sketch theorem is claimed. This is a classical lazy
Ehrenfest/hypercube occupancy model with an EXACT integer evaluation and
a scoped application engineering result.

## Mathematical objects and guarantee

A fixed, nonadaptive set symmetric difference has d distinct tokens,
one secret independent ideal oracle, m=4096 parity cells and a
predetermined one-level configuration (T,j,profile). Every token
toggles each cell of j with probability 1/D, where

- original profile: D=m*2^(j+1), from random coefficient bit g;
- unit profile: D=m*2^j, from g(v)=1, **domain-separated** keyed
  BLAKE3 oracle and incompatible with original.

The exact odd-cell-count law n_d[s]/D^d starts with n_0[0]=1 and

    n_(d+1)[s] = (D-m)n_d[s]
                + (m-s+1)n_d[s-1]
                + (s+1)n_d[s+1].

Every exact numerator sum equals D^d. The birth/death transition
kernel is stochastically monotone since (m+1)/D<=1
for all named profiles; S_d increases in first-order stochastic
order as d increases from its zero initial state. Therefore
`sup_{d>T} P_d[S<=c] = P_{T+1}[S<=c]`.
The maximal cutoff c (or -1/ALWAYS_UNKNOWN) is accepted only if

    1_000_000 * sum_{s=0..c} n_(T+1)[s] <= D^(T+1),

an **exact arbitrary-precision integer comparison** independent of
Poisson/Chernoff/logs/binary64 or rare-event sampling.

For each committed (T,j,profile), the ideal-oracle one-sided
false-safe probability is thus <=1e-6 for *every integer d>T*.
An unrestricted decoder choosing among several j/T after looking at
the bitmap does not inherit this one-profile bound; it would need
its own joint/cumulative alpha budget. The keyed BLAKE3 replacement
is only a computational assumption, with no numerical PRF bound proven.
Repeating queries on adaptively mutated token sets or introducing a
malicious sender is excluded. Snapshot/wire authentication not supplied.

The original profile has an exact **single-level T=32 STOP**:
for j=1, d=33 and no token updating that level,
P>= (3/4)^33>1e-6, which alone exceeds the budget.
For j>=2 the no-toggle event is still more likely.
Therefore the exact fixed-d single-level estimator cannot
report SAFE for T32 under the original g random oracle,
irrespective of its confidence inversion algorithm.
This is NOT a lower bound on joint-level estimators or new oracles.

## Exact cutoff table: j=1

| T | original cutoff S<=c | unit cutoff S<=c |
|---:|---:|---:|
| 32 | -1 (never SAFE) | 3 |
| 64 | 1 | 13 |
| 256 | 32 | 86 |

Other levels j2..4 have exact independent cutoff records in
`cutoffs.tsv` and complete `certificate.json` in workflow artifact;
no Q32 table or old snapshot was reused for the unit profile.

## Certified *ideal-oracle* usefulness, not Monte Carlo

Numbers below are exact-derived probability quantizations,
rounded **down** to parts per million by integer arithmetic.
They are NOT confidence intervals nor finite test frequencies.

| T | true d | original j1 Pr(SAFE) | unit j1 Pr(SAFE) |
|---:|---:|---:|---:|
| 32 | 8 | 0 | 0.364215 |
| 32 | 24 | 0 | 0.000145 |
| 64 | 16 | 0.063661 | 0.997956 |
| 64 | 48 | 0.000017 | 0.001189 |
| 256 | 64 | 0.999995 | 1.000000 (exact for d<=cutoff) |
| 256 | 192 | 0.004663 | 0.145960 |

The useful range depends strongly on slack (T-d).
**At T=64 and d=0.75T, even the unit oracle confirms SAFE
only about 0.12% of ideal random-oracle outcomes**; it is
unsuitable for positioning as a near-boundary high-power predicate.
At T=32,d=0.25T, unit profile is informative 36.4% of the time
but is nowhere near a universal >=95% usefulness promise.

## Two-owner Rust evidence

Both owners maintain actual 512B per-owner bitmap payload
(1,024B combined), not a 26KiB compressed facade.
Each independently hashes/toggles tokens then XOR-merges.
For the original oracle the selected level equals the projection
of the unchanged J52 reference **word for word**.
Unit uses distinct derive_key context; mismatched unit/original
or different stored j yields an error rather than XOR.
No candidate source state silently allocates full J52.
A 26KiB J52 sketch exists only in the *test oracle*.

Frozen raw comparison: 5 GitHub-hosted workers × 3 thresholds ×
5 ratios × 3 seeds × 4 levels × 2 profiles =
**1,800 complete scenario rows** (deterministic seed windows
with one PUBLIC fixture key, not 1,800 independent PRF draws).
Exact source and TSV SHA match required; duplicate, absent,
incorrect cutoff/row, conflicting profile rejected in aggregator.

Named T/d cells (j1, 15 source ranges per cell):

| T | d | original SAFE | unit SAFE |
|---:|---:|---:|---:|
| 32 | 8 | 0/15 | **7/15** |
| 32 | 24 | 0/15 | 0/15 |
| 64 | 16 | 0/15 | **15/15** |
| 64 | 48 | 0/15 | 0/15 |

The tiny independent oracle enumerates all 2^m masks for
m=2/3/5, j=1/2/3, original/unit, d=0..8
(3*3*2*9=**162** complete non-approximated comparisons).
The proof script checks state normalization at each time,
exact sampled stochastic-order CDF comparisons, invalid inputs,
and deliberately malformed probability inputs. Compilation/format/Clippy
are separate Rust CI gates.

## Artifact reproducibility and erratum

Actual named [hosted proof and integration run 37881841587](https://github.com/definitely-stable/deltameter/actions/runs/37881841587):
- Certificate artifact ID `11595330078` SHA256
  `6ab27d3f23e6030c80d04da0a42c915ed6c3917dd2cf5a6f669077b4e215fbef`;
- Full aggregate artifact ID `11594262784` SHA256
  `1707e55525b7f6b0527fdee57c30411a71ba3a8a43bd6a45c0838730b29b2b20`;
- Worker 1/2/3/4/5 artifact IDs:
  `11594852132`, `11594656417`, `11594292384`,
  `11594762285`, `11594802124`.

Protocol arithmetic erratum: an earlier statement called
5*3*5*3*4*2 “900”; its correct product is 1,800. All
factors and experiment cells were fixed *before observations*;
no experimental change or cherry-picked subset was made.
The initial matrix aggregator and success-marker
were updated to require all 1,800 records.

## Research-only decision and next gates

**ACCEPT G1-B1 mathematical/integration foundation**, because it
supplies a useful *exact* finite-sample single-level GF(2) predicate
and a fully separated unit-coefficient candidate while preserving
two-party XOR. This is not permission for public DeltaGuard.

- Do NOT claim universal near-T power, WAN latency savings, or a
  new statistical theorem. Compare trusted single-sender scalar,
  direct exact small sets and two-owner maintained state.
- Remaining #89 G1-B2: decide if joint-level observations are
  worth the complexity; any joint process must account for
  mutually exclusive level selection and full familywise alpha,
  not assume per-level independence.
- Add realistic key/session/profile binding, adversarial-input
  and repeated-query scope from #86 before shipping.
- Only benchmark real memory (key/Box/alloc), update/hash, XOR,
  and two-owner network semantics after an independent G1-B2
  protocol; a 512B bitmap alone is not product proof.
- If the use case requires high power at d≈T, consider an
  alternative specialized primitive or STOP_NEAR_T rather than
  optimizing a weak single-level test indefinitely.
