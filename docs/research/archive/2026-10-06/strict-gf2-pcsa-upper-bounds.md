# Snapshot — “Строгие верхние границы для GF(2)-PCSA…”

Date imported: 2026-10-06.

## Valuable research directions

The report correctly emphasizes that strict finite-sample Parity cannot be obtained by blindly applying standard Chernoff/Bennett/Bernstein inequalities because their independence/boundedness assumptions may fail.

It highlights:

- finite-(m) bias;
- dependence between registers/cells;
- truncation effects;
- test inversion;
- MGF/CGF;
- martingale-style concentration;
- median/quantile wrappers;
- modern PCSA/GRA estimator choices.

These are retained as research directions.

## Where the report overreaches

The report eventually recommends GO for strict Parity while no complete finite-sample coverage proof is actually supplied.

In particular:

- a median wrapper needs a valid per-copy failure bound;
- a saddlepoint approximation is not exact coverage without a rigorous remainder;
- ordinary PCSA/HLL literature cannot automatically be transferred to finite-field GF(2) ParityDeltaMeter;
- alternative “Fast-AGMS” claims need direct verification in the actual Hamming-weight/XOR model.

Canonical result: **NO-GO for strict Parity in v0, research remains open.**

## Pareto comparison

The report's broader lesson is retained:

A replacement is justified only by an apples-to-apples improvement for the actual model:

```text
GF(2)-linear/composable
set-only
sparse update
finite-sample high-confidence estimate
```

Generic cardinality or JL results are comparison points, not automatic replacements.
