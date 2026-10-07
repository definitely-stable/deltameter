#!/usr/bin/env python3
"""Exact tiny fixed-d oracle for STRICT-COMPACT-001 ParityLevelCounts.

This is a Phase-A reference only. It proves/checks small instances exactly with
Fraction arithmetic and does not claim production-size numerical scalability.

For stored level j (one-based):
  P(level=j) = 2^-j
  P(GF(2) coefficient=1) = 1/2

Therefore one active difference element toggles one uniformly chosen row at this
level with probability r_j = 2^-(j+1). The number of odd rows S_j is a lazy
Ehrenfest birth-death chain.
"""

from __future__ import annotations

import json
from fractions import Fraction


def toggle_probability(level: int) -> Fraction:
    if level <= 0:
        raise ValueError("level must be one-based and positive")
    return Fraction(1, 1 << (level + 1))


def transition(rows: int, level: int, state: list[Fraction]) -> list[Fraction]:
    if rows <= 0:
        raise ValueError("rows must be positive")
    if len(state) != rows + 1:
        raise ValueError("state length")

    r = toggle_probability(level)
    out = [Fraction(0) for _ in range(rows + 1)]

    for odd, probability in enumerate(state):
        if probability == 0:
            continue

        up = r * Fraction(rows - odd, rows)
        down = r * Fraction(odd, rows)
        stay = Fraction(1) - r

        out[odd] += probability * stay
        if odd < rows:
            out[odd + 1] += probability * up
        if odd > 0:
            out[odd - 1] += probability * down

    if sum(out, Fraction(0)) != 1:
        raise AssertionError("probability mass")
    return out


def distribution(rows: int, level: int, d: int) -> list[Fraction]:
    if d < 0:
        raise ValueError("d must be non-negative")

    state = [Fraction(0) for _ in range(rows + 1)]
    state[0] = Fraction(1)

    for _ in range(d):
        state = transition(rows, level, state)
    return state


def moments_from_distribution(pmf: list[Fraction]) -> tuple[Fraction, Fraction]:
    mean = sum((Fraction(i) * p for i, p in enumerate(pmf)), Fraction(0))
    second = sum((Fraction(i * i) * p for i, p in enumerate(pmf)), Fraction(0))
    return mean, second - mean * mean


def closed_form_moments(rows: int, level: int, d: int) -> tuple[Fraction, Fraction]:
    r = toggle_probability(level)
    lambda1 = (Fraction(1) - Fraction(2) * r / rows) ** d
    lambda2 = (Fraction(1) - Fraction(4) * r / rows) ** d

    mean = Fraction(rows, 2) * (Fraction(1) - lambda1)
    variance = Fraction(rows, 4) * (
        Fraction(1)
        + Fraction(rows - 1) * lambda2
        - Fraction(rows) * lambda1 * lambda1
    )
    return mean, variance


def kernel_monotonicity_margin(rows: int, level: int) -> Fraction:
    if rows <= 0:
        raise ValueError("rows must be positive")
    r = toggle_probability(level)
    return Fraction(1) - r * Fraction(rows + 1, rows)


def stochastically_nondecreasing(
    previous: list[Fraction], current: list[Fraction]
) -> bool:
    if len(previous) != len(current):
        raise ValueError("distribution shape")

    prev_cdf = Fraction(0)
    cur_cdf = Fraction(0)

    for left, right in zip(previous, current):
        prev_cdf += left
        cur_cdf += right
        if cur_cdf > prev_cdf:
            return False
    return True


def checked_grid(rows: int, level: int, d_max: int) -> dict:
    previous = distribution(rows, level, 0)
    monotone = True
    moments_match = True

    for d in range(d_max + 1):
        current = distribution(rows, level, d)
        moments_match &= (
            moments_from_distribution(current)
            == closed_form_moments(rows, level, d)
        )

        if d:
            monotone &= stochastically_nondecreasing(previous, current)

        previous = current

    return {
        "rows": rows,
        "level": level,
        "d_max": d_max,
        "kernel_monotonicity_margin": str(
            kernel_monotonicity_margin(rows, level)
        ),
        "checked_stochastic_monotonicity": monotone,
        "exact_moments_match": moments_match,
    }


def build_payload() -> dict:
    cases = [
        checked_grid(rows, level, 24)
        for rows in (1, 2, 4, 8)
        for level in (1, 2, 3, 4)
    ]

    if not all(
        row["checked_stochastic_monotonicity"]
        and row["exact_moments_match"]
        and Fraction(row["kernel_monotonicity_margin"]) >= 0
        for row in cases
    ):
        raise AssertionError("STRICT-COMPACT exact reference failed")

    return {
        "model": "strict-compact-level-count-fixed-d-exact-v1",
        "scope": "tiny exact Fraction oracle; not production-size inference",
        "cases": cases,
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
