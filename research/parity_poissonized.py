#!/usr/bin/env python3
"""Exact Poissonized law for the published GF(2)-F-PCSA row statistic.

This script is theorem-derived for the Poisson model only.

If D ~ Poisson(lambda), Poisson splitting makes the stored cells independent.
For one-based level j:
    q_j = P(h(v)=(row,j)) = 2^-j / m
    rate(g=1 contribution) = lambda * q_j / 2
    P(cell=1) = (1 - exp(-lambda*q_j)) / 2

Rows are independent in the unconditional Poissonized model. Conditioning on
D=d returns to the dependent fixed-d model and is NOT performed here.
"""

from __future__ import annotations

import json
import math


def cell_one_probability(lam: float, rows: int, level: int) -> float:
    if lam < 0.0:
        raise ValueError("lambda must be non-negative")
    if rows <= 0:
        raise ValueError("rows must be positive")
    if level <= 0:
        raise ValueError("level must be one-based and positive")

    q = math.ldexp(1.0 / rows, -level)
    return -0.5 * math.expm1(-lam * q)


def row_pmf(lam: float, rows: int, stored_levels: int) -> list[float]:
    if stored_levels <= 0:
        raise ValueError("stored_levels must be positive")

    p_one = [
        0.0,
        *[
            cell_one_probability(lam, rows, level)
            for level in range(1, stored_levels + 1)
        ],
    ]

    suffix_zero_after = [1.0] * (stored_levels + 1)
    suffix = 1.0
    for level in range(stored_levels, 0, -1):
        suffix_zero_after[level] = suffix
        suffix *= 1.0 - p_one[level]

    pmf = [0.0] * (stored_levels + 1)
    pmf[0] = suffix
    for level in range(1, stored_levels + 1):
        pmf[level] = p_one[level] * suffix_zero_after[level]

    return pmf


def convolve(left: list[float], right: list[float]) -> list[float]:
    result = [0.0] * (len(left) + len(right) - 1)
    for i, a in enumerate(left):
        if a == 0.0:
            continue
        for j, b in enumerate(right):
            if b != 0.0:
                result[i + j] += a * b
    return result


def sum_w_pmf(lam: float, rows: int, stored_levels: int) -> list[float]:
    one_row = row_pmf(lam, rows, stored_levels)
    total = [1.0]
    for _ in range(rows):
        total = convolve(total, one_row)
    return total


def build_payload() -> dict:
    lam = 100.0
    rows = 8
    stored_levels = 8

    row = row_pmf(lam, rows, stored_levels)
    total = sum_w_pmf(lam, rows, stored_levels)

    return {
        "model": "published-fpcsa-f2-poissonized-v1",
        "coverage_scope": "Poissonized only; not a fixed-d theorem",
        "lambda": lam,
        "rows": rows,
        "stored_levels": stored_levels,
        "row_pmf": row,
        "row_probability_mass": sum(row),
        "sum_w_probability_mass": sum(total),
        "sum_w_support": len(total),
        "cell_one_level_1": cell_one_probability(lam, rows, 1),
        "cell_one_level_8": cell_one_probability(lam, rows, 8),
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
