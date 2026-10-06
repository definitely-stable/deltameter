# Audit of 2026-10-06 research inputs

This document reconciles the latest Deep Research run and five newly attached reports with the canonical DeltaMeter design.

The source reports are preserved under `archive/2026-10-06/`. They are evidence, not normative specifications.

## Executive result

The new material strengthens the existing simple architecture rather than expanding it.

Accepted direction:

```text
EnergyDeltaMeter
    first implementation
    strict finite-sample backend

ParityDeltaMeter
    second implementation
    experimental/asymptotic until Q1 is solved

ExactSmallDelta
    deferred
    Minisketch/PinSketch recorded as a strong future candidate

one crate
public GitHub-hosted CI
no default cryptography
no FFI in v0
no frozen persisted wire format yet
```

## Conflict matrix

| Topic | New reports say | Canonical repository decision |
|---|---|---|
| Strict Parity from asymptotic RSE | Some reports derive strict-looking profiles by combining the asymptotic (1.638/sqrt m) constant with Chebyshev/median wrappers | **Rejected as proof.** Asymptotic variance cannot be silently promoted to finite-sample variance. |
| Empirical (delta=10^{-9}) | One report proposes fitted empirical CDFs as a path to strict profiles | **Calibration only.** Monte Carlo cannot certify such extreme failure probabilities. |
| De-Poissonization | One report correctly identifies it as a central finite-sample obstacle | **Accepted research direction.** Must carry an explicit error budget or exact conditioning. |
| `g(v)=1` set specialization | One report identifies it as more natural for set XOR | **Important new distinction.** It is a new estimator; published F-PCSA constants do not transfer automatically. |
| ExactSmallDelta | One report recommends immediate Minisketch; another recommends deferring | **Deferred from v0.** Keep Minisketch/PinSketch as a future candidate, not a current dependency. |
| Energy hash independence | Some material suggests 2-wise may be enough generally | **Energy contract remains bucket pairwise + sign 4-wise** for the stated variance proof. |
| UMASH/ChainHash as proof hash | Suggested as practical examples | **Not frozen.** Collision bounds/universal hashing are not automatically the exact k-wise-independence theorem needed by the estimator. |
| Secret seed | New material mostly agrees it is unnecessary in the oblivious model | **Accepted.** Public deterministic seed is allowed for v0. |
| Wire format now | Some reports want a complete persisted header immediately | **Deferred.** Freeze only in-memory config identity first; persisted format after state/API stabilization. |
| Heavy CI | Some material suggests very large Monte Carlo research jobs | **Diagnostic only.** Public runners can be used freely, but proof does not come from brute-force rare-event simulation. |
| Formal verification | Some earlier material proposed Verus/Alerus | **Post-v1 unless a concrete proof boundary appears.** |
| Classical PCSA vs published F-PCSA | The newest report frequently analyzes first-1 / bitmap PCSA mechanics | **Corrected.** Q1 must start from the published FIELDMAP + rightmost-nonzero F-PCSA construction. |
| De-Poissonization theorem | The newest report treats the 2025 Poisson-Charlier result as nearly plug-and-play | **Promising but conditional.** We still need a concrete coefficient/tail sequence and computable forward-difference bounds. |
| Truncated-sample theory | The newest report imports truncated multivariate-normal methodology | **Not on the main proof path.** Algorithmic finite-level censoring in F-PCSA is a different model. |
| Exact CI construction | The newest report emphasizes two-sided exact tests | **Narrowed to one-sided inversion.** DeltaMeter only needs Pr[d <= U] >= 1-delta; conservative level <= delta is sufficient. |

## Source-specific audit

### A. “Доказательное DeltaMeter: Архитектурный план для v0…”

Strong contributions:

- correctly refuses to turn the published asymptotic F-PCSA constant into a finite-sample (10^{-9}) guarantee;
- identifies de-Poissonization as a real technical obstacle;
- explicitly separates published random-coefficient F-PCSA from a proposed `g(v)=1` set specialization;
- keeps Parity `Experimental/Asymptotic`;
- correctly separates fast PR CI from heavier research workflows.

Not adopted:

- immediate Minisketch FFI;
- three-component v0 architecture;
- early wire-format freeze.

Reason: the project goal is a small estimator crate first. A decoder/FFI layer is additional product scope.

### B. “From Theory to Practice: Resolving Implementation Barriers…”

Strong contributions:

- supports Energy as the first/default strict backend;
- supports Parity as secondary until Q1 is resolved;
- recommends deferring exact small-d recovery;
- supports pairwise bucket + 4-wise sign randomness for Energy;
- supports deterministic PBT/reference checks in PR CI.

Rejected/narrowed:

- an empirical fitted CDF cannot justify `Coverage::Proven` at (delta=10^{-9});
- “billion-trial” CI/research jobs are not a sensible proof strategy;
- qualitative Pareto claims need actual measured profiles.

### C. “Минимальный контракт случайности v0…”

Strong contributions:

- no secret seed is needed for the chosen oblivious-input model;
- reproducible canonical encoding is important once persistence exists;
- language-default hashing must not become wire identity.

Corrections:

- for the current Energy variance proof, 4-wise sign independence is not treated as optional;
- 2-universal collision guarantees are not interchangeable with full 2-wise output independence without checking the theorem;
- UMASH/ChainHash are not approved merely because they are fast or have collision guarantees;
- v0 does not need a persisted `RAND-V0` container before the estimator state is frozen.

### D. “Строгие верхние границы для GF(2)-PCSA…”

Strong contributions:

- emphasizes that direct Chernoff/Bennett/Bernstein use is invalid without their assumptions;
- highlights dependence, small-cardinality bias and truncation;
- treats test inversion and stronger concentration machinery as research directions;
- stresses that strict bounds need more than first/second moments.

Not adopted:

- the final GO recommendation is not supported by a completed finite-sample proof in the report;
- median wrappers do not manufacture a theorem if the single-copy failure probability is only asymptotic/empirical;
- Fast-AGMS is not accepted as a replacement without a direct apples-to-apples GF(2) Hamming-weight evaluation.

### E. Latest Deep Research Q1 run

The run usefully explored Chebyshev/Cantelli and median amplification, but its claimed strict profiles depend on substituting the published asymptotic RSE into finite-sample concentration.

That step is not valid as a proof.

Canonical status:

```text
useful exploratory calculation
not a Proven profile generator
Q1 v0 verdict = NO-GO for strict Parity
```

### F. “От Эвристики к Доказательству…”

Strong contributions:

- correctly rejects asymptotic RSE as a finite-sample theorem;
- correctly rejects empirical 1e-9 tails as proof;
- correctly notes that median amplification needs a valid per-copy bound;
- usefully prioritizes exact finite distributions, truncation, de-Poissonization, test inversion, and width/power;
- identifies strict remainder control as the key requirement if asymptotic/Poissonized machinery is used.

Critical corrections:

- much of the derivation is for classical PCSA rather than the published finite-field F-PCSA sketch;
- the report overloads m and d, making several “fixed-parameter” passages ambiguous;
- the speculative GF(2^V) explanation is not the published definition of GF(2)-F-PCSA;
- “register independence after Poissonization” must be proved for the exact categorical decomposition and hash model;
- the cited truncated-Gaussian estimation paper is not directly applicable to discrete F-PCSA fringe/level censoring;
- a valid one-sided test need only have Type-I error at most delta; exact equality is unnecessary;
- binary-search inversion is only valid after monotonicity/stochastic ordering is proved.

Canonical effect: no architecture change; Q1 proof plan becomes more precise and more source-faithful.

## New architecture implication

The most important new architectural input is not another backend. It is a naming/proof boundary:

```text
PublishedFpcsaF2
    published algorithm
    reproduce first
    published asymptotic claims apply only here

ParityPcsaSetV1
    proposed g(v)=1 specialization
    new research algorithm
    no inherited 1.638/sqrt(m) claim
```

This prevents a future implementation from accidentally mixing two different estimators under one `ParityDeltaMeter` name.

## What remains intentionally simple

The new inputs do **not** justify:

- a multi-crate workspace;
- plugin systems;
- crypto/key-management infrastructure;
- Minisketch FFI in the core;
- a generalized wire protocol before state freeze;
- formal verification in CI;
- nightly SIMD;
- a large research orchestration layer.


## Post-M3 Gemini / Qwen / DeepSeek round

Three additional reports were received after M3. Their common high-level conclusion is NO-GO for making the current published W_i Parity estimator a strict backend. Their detailed mathematics and architecture recommendations conflict substantially.

The canonical reconciliation is now [STRICT-PARITY-POST-M3.md](STRICT-PARITY-POST-M3.md).

### New conflict matrix

| Topic | Reports | Canonical decision |
|---|---|---|
| Published W_i strict backend | All three reports say NO-GO | **Accepted as a stop gate.** Energy remains the only Proven backend. |
| Exact Poissonized law | Gemini/Qwen derive independent cells/rows | **Accepted.** Poisson splitting gives an exact unconditional Poissonized law. It is not a fixed-d theorem. |
| Full-state exact law | Qwen gives a Walsh/Fourier formula; Gemini gives row Walsh calculations | **Accepted in principle.** Exact but exponential at target m,J. |
| W_i sufficiency | Qwen gives a direct finite counterexample; other reports invoke richer PCSA estimators | **Accepted via the direct finite counterexample.** No transfer from classical-PCSA Fish-number work is required. |
| Individual W_i monotonicity | Gemini/Qwen argue yes | **Accepted for one row** via a positive Walsh-sum CDF formula. |
| sum_i W_i monotonicity | Gemini infers yes; DeepSeek says cancellation gives a counterexample; Qwen keeps it open | **OPEN.** Neither inference is accepted as a general theorem. |
| Per-level S_j monotonicity | DeepSeek infers stochastic order from increasing expectation | **Rejected.** Mean monotonicity is not stochastic monotonicity. |
| Truncation | Reports mix formulas with and without 1/m | **Corrected.** Whole-sketch signal: 2^-J per key. State distortion: 2^-(J+1). No 1/m after summing rows. |
| Small-d zero floor | Qwen gives 10/20/30 for target deltas | **Corrected.** Exact theorem-implied floor is max{d:2^-d>delta}: 9/19/29. |
| Simple Set Sketching | Gemini recommends immediate Coverage::Exact hybrid | **Candidate only.** High-probability exact recovery is not unconditional Coverage::Exact. |
| IBLT quadratic estimator | DeepSeek recommends a Proven hybrid from exact moments + chi-square | **Not accepted.** Chi-square limit is asymptotic unless a finite-sample tail theorem is supplied; cited source requires verification. |
| Minisketch | DeepSeek recommends immediate exact lane | **Still deferred.** Strong candidate, separate product/implementation scope. |
| Energy memory Pareto | DeepSeek lists much smaller strict Energy states | **Rejected.** Canonical sizes come from research/energy_profiles.py and include all R tables. |
| Adaptive attacks | DeepSeek treats them as a stop condition | **Context only.** Current theorem model is oblivious input. |
| Richer FIELDMAP statistics | Reports propose level counts / occupancy information | **Research candidate.** Raw FIELDMAP remains XOR-mergeable; richer query statistics do not destroy mergeability. |

### Source snapshots

- [archive/2026-10-06/gemini-post-m3-strict-parity.md](archive/2026-10-06/gemini-post-m3-strict-parity.md)
- [archive/2026-10-06/qwen-post-m3-strict-parity.md](archive/2026-10-06/qwen-post-m3-strict-parity.md)
- [archive/2026-10-06/deepseek-post-m3-strict-parity.md](archive/2026-10-06/deepseek-post-m3-strict-parity.md)

### Canonical effect

The new round changes one thing materially: the repository no longer treats strict published-W_i Parity as an open proof program.

Instead:

~~~text
published W_i strict track
    STOP / NO-GO

ParityLevelCounts
    optional research-only track

Energy
    only current Coverage::Proven backend

Parity
    Coverage::Asymptotic
~~~

No new production backend is introduced by this research import.
