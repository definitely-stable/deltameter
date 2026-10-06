# Source snapshot — DeepSeek post-M3 strict-Parity report

Date received: 2026-10-06
Source form: user-provided text
Canonical status: evidence only; see ../../../STRICT-PARITY-POST-M3.md.

## Main claims

The report agrees with NO-GO for published strict Parity but recommends REPLACE with a hybrid of:

- Energy;
- IBLT quadratic estimator;
- Minisketch;
- Parity as experimental only.

It also claims:

- a cancellation counterexample proves non-monotonicity of W_i/full statistic;
- per-level S_j is stochastically monotone because its expectation increases;
- an IBLT quadratic estimator has exact mean/variance and chi-square confidence;
- adaptive L0 attacks are a strict-Parity stop condition.

## Accepted contributions

- published W_i remains unsuitable for current Proven coverage;
- richer state statistics deserve separate research;
- exact-small-d recovery remains an important possible product lane;
- adaptive robustness is a distinct threat model worth documenting.

## Rejected or unverified claims

### Generic cancellation does not settle the actual statistic

A parity process can be non-monotone, but a generic two-step cancellation example is not automatically a counterexample for the actual lazy geometric GF(2)-F-PCSA W_i or sum_i W_i distribution.

Canonical status of the full statistic remains OPEN.

### Mean monotonicity does not imply stochastic monotonicity

An increasing E[S_j] is insufficient to conclude stochastic ordering of S_j.

### IBLT finite-sample confidence is not established here

Exact mean/variance plus a chi-square limit would still be asymptotic for confidence coverage.

The cited 2026 IBLT quadratic-estimator source was not independently verified in this audit.

No Coverage::Proven backend is created from this claim.

### Energy memory comparison is inconsistent with the repository oracle

The report lists memory numbers that omit the current Energy table amplification.

Canonical example:

~~~text
epsilon=10%
delta=1e-6
B=2048
R=23
state=376832 bytes
~~~

### Adaptive attacks are out of the accepted v0 threat model

The project currently proves/claims only the oblivious-input model.

Adaptive-query attacks do not invalidate that model.

### REPLACE is premature

Neither IBLT nor Minisketch is adopted in this research slice.

Each needs an explicit primary-source theorem, failure semantics, state accounting, API contract and implementation-cost audit.

## Architectural effect

The report adds candidate questions but does not change the current backend architecture.
