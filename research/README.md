# Research harness

The research directory contains small, dependency-free scripts that keep the mathematical baseline executable.

The scripts are not production implementations.

## Run everything

```bash
python research/run_all.py --out research-out
```

Outputs:

- `energy-profiles.json` — conservative EnergyDeltaMeter `(epsilon, delta, B, R)` profiles.
- `parity-level-moments.json` — exact first two moments for selected finite-(d) F-PCSA single-level cases.
- `fpcsa-exact-small.json` — exact one-level parity-mask regression.
- `parity-full-exact-small.json` — tiny exact full-state checks, including the W non-sufficiency witness.
- `parity-poissonized.json` — theorem-derived Poissonized cell/row law diagnostics.
- `parity-truncation-budget.json` — exact finite-J signal and state-distortion budgets.

## Individual runs

```bash
python research/energy_profiles.py
python research/parity_level_moments.py --m 256 --d 1000 --r 0.5
python research/parity_full_exact_small.py
python research/parity_poissonized.py
python research/parity_truncation_budget.py
```

## Interpretation

`energy_profiles.py` is theorem-derived from:

```text
Var(T) = 2 d (d - 1) / B
P(|T-d| >= epsilon d) <= 2 / (B epsilon^2)
```

plus exact binomial-tail amplification for the median of independent tables.

`parity_level_moments.py` checks exact finite-(d) first and second moments for one level of the published GF(2) random-coefficient PCSA model. It does **not** provide a high-confidence tail bound.

The GitHub workflow intentionally does not run billion-trial Monte Carlo. Extreme confidence must come from a theorem and verified numerical implementation, not from an impractical test count.

## M2 F-PCSA exact-small regression

The script `fpcsa_exact_small.py` computes the exact finite parity-mask distribution for small row counts at one published GF(2)-F-PCSA level and verifies the analytical first/second-moment formulas without Monte Carlo.

```bash
python research/fpcsa_exact_small.py
```


## Post-M3 strict-Parity artifacts

The post-M3 research round added three narrow executable artifacts.

- parity_full_exact_small.py performs exact rational full-state enumeration only for tiny m,J. It is useful for counterexamples and regression, not production-size inference.
- parity_poissonized.py implements the exact unconditional Poissonized cell/row law. Its output is explicitly not a fixed-d theorem.
- parity_truncation_budget.py separates the implementation Truncated signal from actual ideal-state distortion. Whole-sketch probabilities have no 1/m factor after summing over rows.

These scripts support the canonical decision in docs/research/STRICT-PARITY-POST-M3.md. They do not create a strict Parity profile.
