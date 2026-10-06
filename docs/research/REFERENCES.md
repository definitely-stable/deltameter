# Research references

This file maps the primary literature and upstream systems used by the DeltaMeter research. It is not intended to be a complete streaming-algorithms bibliography.

## Core estimation

- Noga Alon, Yossi Matias, Mario Szegedy — *The Space Complexity of Approximating the Frequency Moments*.
  - Role: AMS/frequency-moment foundation; fourth-moment style analysis for F2.

- Moses Charikar, Kevin Chen, Martin Farach-Colton — *Finding Frequent Items in Data Streams*.
  - Role: CountSketch family; hash buckets plus random signs.

- Dingyu Wang — *Probabilistic Counting in Generalized Turnstile Models*, arXiv:2310.14977.
  - Role: finite-field counting / F-PCSA; key prior art for GF(2) ParityDeltaMeter.
  - Canonical caution: published relative-error constants are asymptotic and do not by themselves give fixed-d high-confidence coverage.

- Kane, Nelson, Woodruff and related turnstile norm-estimation work.
  - Role: generic Fp/F2/L0 upper/lower-bound context.
  - Caution: generic bounds are not automatically tight for set-only GF(2) Hamming weight.

## High-confidence and lower bounds

- *High Probability Streaming Lower Bounds for F2 Estimation* (2026 preprint/work).
  - Role: modern high-probability F2 lower-bound context.
  - Caution: broader problem than DeltaMeter's GF(2) set-only promise.

- *Tight Bounds for Low-Error Frequency Moment Estimation* and related low-error work.
  - Role: dependence on `epsilon` and `delta` in broader streaming models.

- distinct-elements lower-bound/upper-bound literature, including low-failure-probability results.
  - Role: comparison point showing how promise/model changes the memory frontier.

## Embeddings

- sparse Johnson-Lindenstrauss / sparse sign embedding literature.
  - Role: general norm-preservation baseline.
  - Caution: do not directly transfer subspace-embedding conclusions to specialized EnergyDeltaMeter.

## Set similarity/cardinality references

- Seth Pettie, Dingyu Wang — *Information Theoretic Limits of Cardinality Estimation: Fisher Meets Shannon*, arXiv:2007.08051.
  - Role: Fish-number / information-efficiency context for classical cardinality sketches.
  - Caution: this does not automatically transfer a sufficiency or finite-sample theorem to GF(2)-F-PCSA.


- Broder/min-wise hashing literature.
  - Role: explains MinHash/Jaccard conditioning.

- HyperLogLog literature.
  - Role: cardinality baseline, not subtractable symmetric-difference estimator.

## Reconciliation / small-d references

- Bitcoin Core minisketch.
  - Role: BCH-based set reconciliation and practical evidence that a difference-size oracle is useful.

- Jakob Bæk Tejs Houen, Rasmus Pagh, Stefan Walzer — *Simple Set Sketching*, arXiv:2211.03683 / SOSA 2023.
  - Role: compact XOR-based set recovery with high-probability decoding below a random-hypergraph load threshold.
  - Caution: exact recovery on successful decoding is not automatically unconditional `Coverage::Exact`.

- IBLT and recent memory-improved IBLT literature.
  - Role: another possible future exact-small-d lane.

- rate-compatible / rateless set-reconciliation literature.
  - Role: broader reconciliation context, not v0 implementation scope.

## Adversarial streaming

- Hardt and Woodruff — adaptive attacks on linear sketches for norm estimation.
  - Role: why repeated adaptive observation is a different threat model.

- *A Strong Separation for Adversarially Robust l0 Estimation* and related 2024–2026 work on adaptive attacks and robust turnstile streaming.
  - Role: confirms that adaptive robustness is a separate threat model for linear sketches.
  - Caution: DeltaMeter's accepted v0 theorem model is oblivious input, so adaptive attacks are not a v0 correctness blocker.

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


## Post-M3 source-verification notes

The post-M3 reports introduced an IBLT quadratic/cardinality estimator claim tied to a 2026 identifier. That claim is not canonical until the primary source is independently verified and its confidence statement is checked for a genuinely finite-sample tail theorem rather than an asymptotic chi-square limit.

The canonical mathematical conclusions in STRICT-PARITY-POST-M3.md do not depend on that unverified source.
