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

## Individual runs

```bash
python research/energy_profiles.py
python research/parity_level_moments.py --m 256 --d 1000 --r 0.5
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
