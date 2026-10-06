# Research index

This directory is the research record for DeltaMeter.

Research is kept in layers so historical reports remain available without letting older or contradictory conclusions silently become current design requirements.

## Authority order

When documents disagree, use this order:

1. [DECISIONS.md](DECISIONS.md) — current implementation decisions.
2. [Q1-FINITE-SAMPLE-PARITY.md](Q1-FINITE-SAMPLE-PARITY.md) — current Q1 gate and proof boundary.
3. [FOUNDATION.md](FOUNDATION.md) — canonical technical synthesis.
4. [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) — remaining research and resolved gates.
5. [NEW-INPUTS-AUDIT.md](NEW-INPUTS-AUDIT.md) — reconciliation of the latest reports.
6. [SOURCE-AUDIT.md](SOURCE-AUDIT.md) — audit of the original F2-centric report.
7. [archive/](archive/) — historical research snapshots; useful evidence, not normative.
8. [REFERENCES.md](REFERENCES.md) — source map.

## Current research position

For sets (A,Bsubseteq[V]):

```text
x = 1_A - 1_B           over the integers
z = 1_A + 1_B mod 2     over GF(2)

d = |A △ B| = F2(x) = ||z||_0
```

The set-only GF(2) formulation is the primary model.

Current implementation direction:

- **EnergyDeltaMeter** — first implementation and strict finite-sample backend.
- **ParityDeltaMeter** — second implementation and experimental/asymptotic fast path.
- **Gaussian/chi-square oracle** — research/test oracle only.
- **ExactSmallDelta** — deferred; Minisketch/PinSketch remains a future candidate.

## Q1 status

The v0 implementation gate is resolved:

> **NO-GO for strict Parity `Coverage::Proven` in v0.**

This does not mean strict Parity is impossible. It means the current research does not yet provide a valid finite-(m), finite-(V), all-(d) theorem with practical width.

The latest reports additionally exposed a critical distinction:

```text
published GF(2)-F-PCSA
    random finite-field coefficient
    published asymptotic theory belongs here

proposed ParityPcsaSetV1
    g(v) = 1
    natural XOR-only set specialization
    statistically a new estimator
```

Published constants must not be transferred between those constructions without proof.

## What is proven vs open

### Proven/derivable in the current model

- (d=F_2(x)=|z|_0) for true set differences.
- For the Energy table under the documented randomness assumptions:
  `E[T]=d` and `Var(T)=2d(d-1)/B`.
- Median amplification gives an explicit finite-sample failure bound once the single-table bound is valid.
- A valid two-sided relative event can be converted into a one-sided capacity upper bound.

### Published but not enough for a strict API

- finite-field F-PCSA is linear and compact;
- the published GF(2) asymptotic RSE is useful engineering guidance;
- asymptotic RSE is not a finite-sample (10^{-6}) or (10^{-9}) theorem.

### Still open

- a useful strict finite-sample tail for the concrete published Parity construction;
- a full analysis of the proposed `g(v)=1` set specialization;
- tight lower bounds for the narrow GF(2)-linear, set-only, high-confidence model;
- final Parity randomness requirements if strict tails are pursued;
- whether ExactSmallDelta is worth adding after real measurements.

## Latest imported research

See [archive/2026-10-06/README.md](archive/2026-10-06/README.md) for the latest Deep Research run and four attached reports.

Their contradictory claims are reconciled in [NEW-INPUTS-AUDIT.md](NEW-INPUTS-AUDIT.md).

## Repository policy

Research scripts remain reproducible and small. New infrastructure is added only when a concrete mathematical or product requirement justifies it.
