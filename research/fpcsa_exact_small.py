#!/usr/bin/env python3
"""Exact small-case regression for one GF(2)-F-PCSA level.

For a fixed level with total hash mass r across m rows, one active element:
- toggles row i with probability r/(2m), because g(v) is uniform in GF(2);
- makes no parity contribution at this level with probability 1-r/2.

The DP below computes the exact parity-occupancy distribution for small m,d and
checks the first two moments against research/parity_level_moments.py.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass

from parity_level_moments import level_moments


@dataclass(frozen=True)
class ExactCase:
    m: int
    d: int
    level_mass_r: float
    total_probability: float
    exact_mean: float
    formula_mean: float
    exact_variance: float
    formula_variance: float
    mean_abs_error: float
    variance_abs_error: float


def exact_distribution(m: int, d: int, r: float) -> list[float]:
    if not (1 <= m <= 12):
        raise ValueError("exact DP intentionally supports 1 <= m <= 12")
    if d < 0:
        raise ValueError("d must be non-negative")
    if not (0.0 <= r <= 1.0):
        raise ValueError("r must be in [0,1]")

    state = [0.0] * (1 << m)
    state[0] = 1.0

    toggle_probability = r / (2.0 * m)
    no_contribution_probability = 1.0 - r / 2.0

    for _ in range(d):
        nxt = [0.0] * len(state)
        for mask, probability in enumerate(state):
            if probability == 0.0:
                continue

            nxt[mask] += probability * no_contribution_probability
            for row in range(m):
                nxt[mask ^ (1 << row)] += probability * toggle_probability
        state = nxt

    return state


def exact_case(m: int, d: int, r: float) -> ExactCase:
    distribution = exact_distribution(m, d, r)
    total = sum(distribution)

    mean = sum(mask.bit_count() * probability for mask, probability in enumerate(distribution))
    second = sum(
        (mask.bit_count() ** 2) * probability
        for mask, probability in enumerate(distribution)
    )
    variance = second - mean * mean

    formula = level_moments(m, d, r)
    formula_mean = float(formula["mean_hamming_weight"])
    formula_variance = float(formula["variance_hamming_weight"])

    return ExactCase(
        m=m,
        d=d,
        level_mass_r=r,
        total_probability=total,
        exact_mean=mean,
        formula_mean=formula_mean,
        exact_variance=variance,
        formula_variance=formula_variance,
        mean_abs_error=abs(mean - formula_mean),
        variance_abs_error=abs(variance - formula_variance),
    )


def build_payload() -> dict:
    cases = [
        exact_case(m, d, r)
        for m in (4, 8)
        for r in (0.5, 0.25, 0.125)
        for d in (0, 1, 2, 3, 4, 6, 8)
    ]

    tolerance = 1e-12
    max_probability_error = max(abs(case.total_probability - 1.0) for case in cases)
    max_mean_error = max(case.mean_abs_error for case in cases)
    max_variance_error = max(case.variance_abs_error for case in cases)

    if max_probability_error > tolerance:
        raise AssertionError(f"probability mass error {max_probability_error}")
    if max_mean_error > tolerance:
        raise AssertionError(f"mean regression error {max_mean_error}")
    if max_variance_error > tolerance:
        raise AssertionError(f"variance regression error {max_variance_error}")

    return {
        "model": "published-fpcsa-f2-single-level-exact-small-v1",
        "claim": "exact finite small-case regression for the published random-coefficient level model",
        "tolerance": tolerance,
        "max_probability_error": max_probability_error,
        "max_mean_abs_error": max_mean_error,
        "max_variance_abs_error": max_variance_error,
        "cases": [asdict(case) for case in cases],
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
