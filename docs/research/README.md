# Research index

This directory is the research record for DeltaMeter.

The repository keeps research in layers so that historical reports are preserved without letting old conclusions silently become current design requirements.

## Authority order

When two documents disagree, use this order:

1. [DECISIONS.md](DECISIONS.md) — current implementation decisions.
2. [FOUNDATION.md](FOUNDATION.md) — canonical technical synthesis.
3. [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) — unresolved implementation-blocking research.
4. [SOURCE-AUDIT.md](SOURCE-AUDIT.md) — corrections to earlier reports.
5. [archive/](archive/) — historical research snapshots; useful evidence, not normative.
6. [REFERENCES.md](REFERENCES.md) — source map.

This order is deliberate. The attached F2-centric report is preserved because it contains useful derivations and references, but several conclusions were later narrowed or corrected.

## Current research position

For sets (A,B\subseteq[V]):

```text
x = 1_A - 1_B           over the integers
z = 1_A + 1_B mod 2     over GF(2)

d = |A △ B| = F2(x) = ||z||_0
```

The set-only formulation over GF(2) is the primary model.

Current candidate backends:

- **ParityDeltaMeter** — GF(2)/finite-field counting path; compact and fast candidate.
- **EnergyDeltaMeter** — CountSketch-like energy estimator; conservative proof-grade reference/strict backend.
- **Gaussian/chi-square oracle** — research/test oracle only.

## What is proven vs open

### Proven/derivable in the current model

- (d=F_2(x)=\|z\|_0) for true set differences.
- For the Energy table under the documented hash/sign assumptions:
  `E[T]=d` and `Var(T)=2d(d-1)/B`.
- Median amplification gives an explicit finite-sample failure bound from the single-table Chebyshev bound.
- If a two-sided ((1\pm\varepsilon)) estimate holds with failure probability (delta), then
  `ceil(d_hat/(1-epsilon))` is a one-sided capacity upper bound on the same good event.

### Published but not enough for a strict API

- GF(2)-PCSA/F-PCSA has strong asymptotic relative-error behavior and very small packed state.
- Published asymptotic RSE constants are not automatically finite-sample (10^{-6}) or (10^{-9}) coverage guarantees.

### Still open for DeltaMeter

- useful finite-sample one-sided tails for the exact practical Parity construction;
- tight lower bounds for the narrow GF(2)-linear, set-only, high-confidence model;
- whether an exact small-(d) lane is worth its complexity after the first two Rust backends exist;
- final randomness family for Parity once its finite-sample proof path is chosen.

## Repository policy

Research scripts must remain reproducible and small. New infrastructure is added only when a concrete research or implementation requirement justifies it.
