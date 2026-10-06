# Source snapshot — Gemini post-M3 strict-Parity report

Date received: 2026-10-06
Source form: attached PDF
Canonical status: evidence only; see ../../../STRICT-PARITY-POST-M3.md.

## Main claims

The report concludes NO-GO STRICT PARITY and proposes a hybrid fallback:

- EnergyDeltaMeter as Proven backend;
- ParityDeltaMeter as Asymptotic;
- Simple Set Sketching as an exact-small-d replacement candidate.

It also develops:

- exact finite-state/Walsh reasoning for row events;
- exact Poissonized cell and row independence;
- analytic de-Poissonization as the fixed-d obstacle;
- a claimed stochastic-monotonicity proof for W_i and then sum_i W_i;
- finite-J truncation calculations;
- a Simple Set Sketching architecture proposal.

## Accepted contributions

- NO-GO for strict published-W_i Parity is consistent with the repository gate.
- Poisson splitting gives exact independent cells/rows in the unconditional Poissonized model.
- Fixed-d conditioning restores dependence.
- Finite-J truncation can be budgeted analytically.
- Exact full-state methods explode exponentially.
- Simple Set Sketching is worth retaining as a future small-d candidate.

## Corrections

### Whole-sketch truncation has no 1/m factor

The report alternates between a per-row expression containing 1/m and a conservative whole-sketch expression without it.

Canonical whole-sketch probabilities are:

~~~text
P(level>J) = 2^-J
P(level>J and g=1) = 2^-(J+1)
~~~

### Sum monotonicity is not established

The report argues monotonicity for one row W_i, then concludes that sum_i W_i is monotone.

That implication is not valid without a joint stochastic-order/coupling argument because the row dependence changes with d in the fixed-d model.

Canonical status of the full statistic is OPEN.

### Simple Set Sketching is not unconditional Coverage::Exact

Exact recovery on successful decoding is different from deterministic guaranteed recovery.

The primary Simple Set Sketching result is high-probability recovery below a load threshold.

The backend remains a candidate, not an accepted replacement.

### "De-Poissonization impossible" is too strong

The report provides good reasons why a practical certified remainder is difficult, but not an impossibility theorem.

Canonical wording is "no practical certified fixed-d bridge is currently available."

## Architectural effect

No hybrid backend is added.

The report strengthens the stop gate for published-W_i strict Parity and motivates a separate exact-small-d source audit.
