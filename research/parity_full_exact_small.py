#!/usr/bin/env python3
"""Exact tiny full-state GF(2)-F-PCSA checks.

The state walk is exact for fixed d and finite m,J:
- each active element flips one stored cell (row,level) with probability
      1 / (m * 2^(level+1))
  because h contributes 2^-level/m and g=1 with probability 1/2;
- all other outcomes are an idle step.

The script is intentionally tiny. It is evidence for small cases only and does
not claim that full-state enumeration scales to production profiles.
"""

from __future__ import annotations

import json
from fractions import Fraction


def transition_probabilities(rows: int, stored_levels: int) -> tuple[list[Fraction], Fraction]:
    flips = [
        Fraction(1, rows * (1 << (level + 1)))
        for _row in range(rows)
        for level in range(1, stored_levels + 1)
    ]
    idle = Fraction(1) - sum(flips, Fraction(0))
    return flips, idle


def distribution(rows: int, stored_levels: int, d: int) -> dict[int, Fraction]:
    flips, idle = transition_probabilities(rows, stored_levels)
    state = {0: Fraction(1)}

    for _ in range(d):
        nxt: dict[int, Fraction] = {}
        for mask, probability in state.items():
            nxt[mask] = nxt.get(mask, Fraction(0)) + probability * idle
            for bit, flip_probability in enumerate(flips):
                target = mask ^ (1 << bit)
                nxt[target] = (
                    nxt.get(target, Fraction(0))
                    + probability * flip_probability
                )
        state = nxt

    return state


def row_w(mask: int, row: int, stored_levels: int) -> int:
    highest = 0
    base = row * stored_levels
    for level in range(1, stored_levels + 1):
        if mask & (1 << (base + level - 1)):
            highest = level
    return highest


def sum_w(mask: int, rows: int, stored_levels: int) -> int:
    return sum(row_w(mask, row, stored_levels) for row in range(rows))


def cdf_for_stat(
    state: dict[int, Fraction],
    statistic,
) -> dict[int, Fraction]:
    pmf: dict[int, Fraction] = {}
    for mask, probability in state.items():
        value = statistic(mask)
        pmf[value] = pmf.get(value, Fraction(0)) + probability

    result: dict[int, Fraction] = {}
    cumulative = Fraction(0)
    for value in range(max(pmf) + 1):
        cumulative += pmf.get(value, Fraction(0))
        result[value] = cumulative
    return result


def cdf_is_stochastically_nondecreasing(
    previous: dict[int, Fraction],
    current: dict[int, Fraction],
) -> bool:
    max_value = max(max(previous), max(current))
    for value in range(max_value + 1):
        prev = previous.get(value, Fraction(1))
        cur = current.get(value, Fraction(1))
        if cur > prev:
            return False
    return True


def checked_monotonicity(rows: int, stored_levels: int, d_max: int) -> dict:
    previous_row = None
    previous_sum = None
    row_ok = True
    sum_ok = True

    for d in range(d_max + 1):
        state = distribution(rows, stored_levels, d)
        if sum(state.values(), Fraction(0)) != 1:
            raise AssertionError("probability mass must remain exactly one")

        row_cdf = cdf_for_stat(
            state,
            lambda mask: row_w(mask, 0, stored_levels),
        )
        sum_cdf = cdf_for_stat(
            state,
            lambda mask: sum_w(mask, rows, stored_levels),
        )

        if previous_row is not None:
            row_ok &= cdf_is_stochastically_nondecreasing(previous_row, row_cdf)
            sum_ok &= cdf_is_stochastically_nondecreasing(previous_sum, sum_cdf)

        previous_row = row_cdf
        previous_sum = sum_cdf

    return {
        "rows": rows,
        "stored_levels": stored_levels,
        "d_max": d_max,
        "row_w_monotone_on_checked_grid": row_ok,
        "sum_w_monotone_on_checked_grid": sum_ok,
        "scope": "exact finite checked grid only; no general theorem for sum_w",
    }


def non_sufficiency_example() -> dict:
    rows = 1
    stored_levels = 2
    state_level_2_only = 0b10
    state_both_levels = 0b11

    d1 = distribution(rows, stored_levels, 1)
    d2 = distribution(rows, stored_levels, 2)

    assert row_w(state_level_2_only, 0, stored_levels) == 2
    assert row_w(state_both_levels, 0, stored_levels) == 2
    assert d1.get(state_level_2_only, Fraction(0)) > 0
    assert d1.get(state_both_levels, Fraction(0)) == 0
    assert d2.get(state_both_levels, Fraction(0)) > 0

    return {
        "claim": "W is not sufficient for d",
        "same_W": 2,
        "state_level_2_only": {
            "p_d1": str(d1.get(state_level_2_only, Fraction(0))),
            "p_d2": str(d2.get(state_level_2_only, Fraction(0))),
        },
        "state_both_levels": {
            "p_d1": str(d1.get(state_both_levels, Fraction(0))),
            "p_d2": str(d2.get(state_both_levels, Fraction(0))),
        },
        "reason": "states with the same W have d-dependent conditional probabilities",
    }


def build_payload() -> dict:
    checks = [
        checked_monotonicity(1, 2, 8),
        checked_monotonicity(2, 2, 8),
        checked_monotonicity(2, 3, 6),
    ]

    return {
        "model": "published-fpcsa-f2-full-state-exact-small-v1",
        "checks": checks,
        "non_sufficiency": non_sufficiency_example(),
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
