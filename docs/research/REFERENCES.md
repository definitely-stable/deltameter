# Research references

This file maps the primary literature and upstream systems used by the DeltaMeter research. It is not intended to be a complete streaming-algorithms bibliography.

## Core estimation

- Noga Alon, Yossi Matias, Mario Szegedy — *The Space Complexity of Approximating the Frequency Moments*.
  - Role: AMS/frequency-moment foundation; fourth-moment style analysis for F2.

- Moses Charikar, Kevin Chen, Martin Farach-Colton — *Finding Frequent Items in Data Streams*.
  - Role: CountSketch family; hash buckets plus random signs.

- Dingyu Wang — *Probabilistic Counting in Generalized Turnstile Models*.
  - Role: finite-field counting / F-PCSA; key prior art for GF(2) ParityDeltaMeter.

- Kane, Nelson, Woodruff and related turnstile norm-estimation work.
  - Role: generic Fp/F2/L0 upper/lower-bound context.
  - Caution: generic bounds are not automatically tight for set-only GF(2) Hamming weight.

## High-confidence and lower bounds

- *High Probability Streaming Lower Bounds for F2 Estimation* (2026 preprint/work).
  - Role: modern high-probability F2 lower-bound context.
  - Caution: broader problem than DeltaMeter's GF(2) set-only promise.

- *Tight Bounds for Low-Error Frequency Moment Estimation* and related low-error work.
  - Role: dependence on (arepsilon,delta) in broader streaming models.

- distinct-elements lower-bound/upper-bound literature, including low-failure-probability results.
  - Role: comparison point showing how promise/model changes the memory frontier.

## Embeddings

- sparse Johnson-Lindenstrauss / sparse sign embedding literature.
  - Role: general norm-preservation baseline.
  - Caution: do not directly transfer subspace-embedding conclusions to specialized EnergyDeltaMeter.

## Set similarity/cardinality references

- Broder/min-wise hashing literature.
  - Role: explains MinHash/Jaccard conditioning.

- HyperLogLog literature.
  - Role: cardinality baseline, not subtractable symmetric-difference estimator.

## Reconciliation / small-d references

- Bitcoin Core minisketch.
  - Role: BCH-based set reconciliation and practical evidence that a difference-size oracle is useful.

- *Simple Set Sketching*.
  - Role: compact set-sketch/recovery reference for a possible future exact-small-d lane.

- IBLT and recent memory-improved IBLT literature.
  - Role: another possible future exact-small-d lane.

- rate-compatible / rateless set-reconciliation literature.
  - Role: broader reconciliation context, not v0 implementation scope.

## Adversarial streaming

- Hardt and Woodruff — adaptive attacks on linear sketches for norm estimation.
  - Role: why repeated adaptive observation is a different threat model.

- recent 2024–2026 work on adaptive attacks and robust turnstile streaming.
  - Role: confirms that robust streaming is not obtained merely by renaming a seed “secret”.

## Sequential inference

- anytime-valid inference / confidence-sequence literature.
  - Role: possible future progressive-confidence machinery.
  - Not a v0 dependency.

## Rust verification

- Verus.
- Alerus / probabilistic verification work.
- Creusot.

Role: optional future machine-checked proof tooling, not v0 merge-gate infrastructure.

## Rust / CI upstream documentation

- Rust standard-library and `std::arch` documentation.
- Rust portable-SIMD tracking/documentation.
- GitHub Actions official documentation for hosted runners and artifacts.

These should be checked at implementation time because toolchain/runner details can change.
