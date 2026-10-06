# Audit of 2026-10-06 research inputs

This document reconciles the latest Deep Research run and four newly attached reports with the canonical DeltaMeter design.

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
