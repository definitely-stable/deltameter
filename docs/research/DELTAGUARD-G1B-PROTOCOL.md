# DeltaGuard G1-B1 — preregistration for exact fixed-d odd-cell chain

Parent #89, related #85/#84/#86. GitHub-hosted CI only. Research only.
Frozen **before** any numerical experiment, utility measurement, or public code.

## Two **different** token oracle profiles (separate profile IDs)

Original GF(2) parity ("original"): same G0 keyed token mapping. Each distinct
nonadaptive token toggles cell i of stored level j with ideal probability
`1/(m*2^(j+1))`. Coefficient zero discards half of candidate updates.

Unit-coefficient set-specialized ("unit"): a NEW domain-separated keyed BLAKE3
context with coefficient g(v)=1, so the same probability is
`1/(m*2^j)`. Old and new sketches cannot XOR, share manifests, or reuse
confidence tables. Not F-PCSA general field coefficients; sets / XOR only.

We fix m=4096, j in {1,2,3,4}, T in {32,64,256}, δ=10^-6, one
precommitted (profile,T,j) per independent sketch session; **single-level
cutoffs** get the whole δ. Arbitrary search/select of j or T *after seeing
bitmap* does NOT inherit the single-profile theorem. Multiple simultaneous
profiles would need a separately allocated error budget. The test key is
public, not a cryptographic security credential.

## Exact integer proof and monotonicity

Let D=m*2^(j+1) for original and D=m*2^j for unit;
n_d[s] is the unnormalized exact probability numerator for s odd cells after
d independent distinct tokens, normalized by D^d.

`n_0[0]=1`; for d>=0,

`n_(d+1)[s] = (D-m)*n_d[s] + (m-(s-1))*n_d[s-1] + (s+1)*n_d[s+1]`,

interpreting outside-domain terms as 0.
Check `sum_s n_d[s] = D^d` at **every step**.

The associated birth/death/stay transition from state s is
`(m-s)/D`, `s/D`, `(D-m)/D`. Adjacent kernels are stochastically ordered
when `(m+1)/D <= 1`, exactly true here; starting at zero,
`S_d <=_st S_(d+1)` for all d. Thus for *any* d>T and any c,

`Pr_d[S <= c] <= Pr_(T+1)[S <= c]`.

The maximal safe integer cutoff c (may be -1, meaning ALWAYS UNCERTAIN)
is the largest c such that
`10^6 * sum_{s<=c} n_(T+1)[s] <= D^(T+1)`.
This is exact finite-sample, exact integer arithmetic; no Poissonization,
Chernoff, Decimal, float, asymptotic inference or 1e-6 Monte Carlo claim
is used for acceptance. The cutoff table is threshold- and profile-bound.

**Independent oracle** for tiny m<=5 and d<=8 enumerates the full
2^m bitmask Markov chain (not the lumped odd-count recurrence) and
aggregates it into odd counts. Compare the exact probability numerators;
reject corrupt probabilities and malformed parameters. Exact CDF
stochastic dominance at d and d+1 is checked for small and production
cases. Verify the no-update lower obstruction:
`10^6 * 3^33 > 4^33` for original T32,j1. Never oversell
this as a lower bound on all sketches.

## Frozen executable matrix AFTER independent proof passes

Five hosted workers × 3 T × 5 d/T values {0,.25,.75,1,1.125}
× 3 deterministic public-key token ranges × 4 levels × 2 profiles
= **1,800 physical two-owner observation rows**.
Each source independently creates and updates a real 512B bitmap, XOR
reconstructs, checks level count/cutoff, source/head binding and profile
rejection; original profile's selected 512B state must equal the original
G0 reference bit-for-bit. A unit profile is NOT compared bitwise to G0
because its mapping is domain separated and its coefficient differs.
A request for cross-profile XOR must fail closed. Changes to selected
level must also fail closed.

Decision is SAFE iff observed odd count<=certified cutoff.
Every row records (T,d,j,profile,cutoff,odd_count,SAFE).
Zero empirical false-safe rows is only a diagnostic, not the proof.
Also compute deterministic **exact ideal-oracle power** by running the
same recurrence at d=floor(T/4), floor(3T/4) and comparing CDF using
a rational numerator/denominator. Report transparent decimal only
as derived display, never as the proof gate.

## Gate

- EXACT_CHAIN_CERT_PASS: normalization, monotonically ordered kernel
  with independent finite-bitmask oracle, exact T+1 cutoffs and valid
  table-source digest. If no safe cutoff, explicitly report -1.
- G1B_B1_XOR_PASS: all 1,800 source-pinned rows and original projection,
  unit profile protection, no hidden full state in candidate;
  finished five-worker aggregate.
- Empirical utility candidate when *any predeclared unit-level j* at
  T=32,d=T/4 returns SAFE in >=5 of the 15 deterministic inputs,
  OR T=64,d=T/4 returns SAFE in >=12 of 15. This is not
  statistical precision and does not authorize release.
- If single-level insufficient, report G1B_B1_LOW_POWER; G1-B2
  may try explicit joint levels at separately certified combined δ,
  or STOP rather than importing false "independent levels" laws.
- A source-sampled Rust test does not establish the ideal-oracle
  probability or keyed PRF security; #86 remains unresolved.
- No new Rust crate API, public snapshot, or production defaults.

## Related literature / not novel

The finite-state count process is a *lazy Ehrenfest urn/hypercube
random walk*; odd-bit count is its standard lumped statistic.
Reference: Levin & Peres, *Markov Chains and Mixing Times*,
freely available at
https://pages.uoregon.edu/dlevin/MARKOV/mcmt2e.pdf
(chapter on Ehrenfest urn/hypercube), and
https://people.math.umass.edu/~lr7q/ps_files/teaching/math697/StochMC.pdf .
The monotone-kernel inequality and the exact DP are elementary known
Markov-chain techniques. G1-B's question is whether their strict
finite-sample **engineering profile** earns useful threshold power.

## Non-outcome protocol erratum

The preregistered Cartesian product was transcribed as 900 rows, but
`5 workers × 3 thresholds × 5 ratios × 3 seeds × 4 levels × 2 profiles
= 1,800`. **All factors were explicitly frozen before execution.**
No factors, endpoints, cutoffs, or acceptance thresholds changed;
only the arithmetic total and fail-closed aggregator expected row
count were corrected after the first successful five-worker execution.
This erratum is not a re-registration of a smaller/selected matrix.
