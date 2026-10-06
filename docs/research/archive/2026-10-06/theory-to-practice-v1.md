# Snapshot — “From Theory to Practice: Resolving Implementation Barriers for the DeltaMeter v1 Backend”

Date imported: 2026-10-06.

## Main recommendations in the report

1. Energy/CountSketch-like estimator as the first/default strict backend.
2. GF(2)-PCSA as a secondary backend until finite-sample tails are resolved.
3. Defer exact-small-d recovery.
4. Use pairwise bucket hashing and 4-wise sign randomness for Energy.
5. Use property-based/deterministic testing in fast CI and heavier diagnostics separately.

These points are broadly aligned with the current repository direction.

## Important limitation

For Q1 the report proposes an empirical CDF / fitted upper-bound model and suggests that, if stable, it could justify strict Parity profiles even at extreme (delta).

Canonical correction:

> empirical calibration may support `Coverage::Empirical`; it does not by itself justify `Coverage::Proven`, especially at (10^{-9}).

## Small-d

The report argues that exact recovery is orthogonal to DeltaMeter's core estimator mission and that the implementation cost of BCH/IBLT-style decoding is not justified for v1.

This matches the current v0 decision to defer `ExactSmallDelta`.

## CI

Accepted:

- fixed-seed property tests;
- exact numerical checks;
- moderate/heavy diagnostics outside the PR gate;
- no absolute hosted-runner timing gate.

Narrowed:

- billion-trial simulations are not a useful proof strategy even when public CI minutes are available.
