# Snapshot — “Доказательное DeltaMeter: Архитектурный план для v0…”

Date imported: 2026-10-06.

## Main thesis

The report argues that strict Parity capacity is still a finite-sample research problem and that using the published asymptotic GF(2)-F-PCSA error constant directly at very small (delta) is unsafe.

It prioritizes:

- exact or rigorous de-Poissonization;
- finite-(d) analysis;
- explicit proof-status metadata;
- a two-level CI model.

## Important new distinction

The report separates:

```text
published F-PCSA:
    random coefficient g(v) in GF(2)

proposed ParityPcsaSetV1:
    g(v) = 1
    cell ^= 1
```

and correctly notes that the published (1.638/sqrt m) asymptotic constant cannot automatically be reused for the set-specialized algorithm.

This distinction is now canonical.

## Exact-small-d recommendation in the report

The report recommends PinSketch/Minisketch as `ExactSmallDelta`, with very small theoretical sketch sizes for small capacities.

It also acknowledges the engineering cost:

- mature upstream is C/C++;
- FFI complicates the Rust crate;
- pure-Rust BCH decoding is non-trivial.

Canonical decision: preserve Minisketch as a future candidate, but do not add it to v0.

## CI recommendation

The report proposes:

- fast deterministic PR gate;
- heavier main/weekly research jobs;
- Monte Carlo only as diagnostic;
- committed generated theory/profile artifacts.

That structure is accepted in simplified form.

## Status

Historical research input with several accepted conclusions and one major non-adopted architecture choice: immediate ExactSmallDelta/FFI.
