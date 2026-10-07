#!/usr/bin/env python3
"""Exact tiny multilevel regression for STRICT-COMPACT-001.

This script enumerates the full finite GF(2)-F-PCSA state for tiny m,J,d and applies
the proposed per-level KL/de-Poissonized upper bounds with a Bonferroni allocation.

The probability law is exact Fraction arithmetic. The per-level bound itself is
theorem-derived but evaluated in binary64 here, so this remains a regression oracle,
not the production numerical proof boundary.
"""

from __future__ import annotations

import json
from fractions import Fraction

from parity_full_exact_small import distribution
from strict_compact_chernoff import fixed_lower_tail_upper_bound


def level_count(mask: int, rows: int, stored_levels: int, level: int) -> int:
    if not (1 <= level <= stored_levels):
        raise ValueError("invalid level")
    count = 0
    for row in range(rows):
        bit = row * stored_levels + level - 1
        count += int(bool(mask & (1 << bit)))
    return count


def bounded_level_upper_table(
    rows: int,
    level: int,
    d_max: int,
    alpha: float,
) -> list[int]:
    table = []
    for observed in range(rows + 1):
        accepted = [
            d
            for d in range(d_max + 1)
            if fixed_lower_tail_upper_bound(
                rows, level, float(d), observed
            )
            > alpha
        ]
        table.append(max(accepted) if accepted else 0)
    return table


def exact_multilevel_case(
    rows: int,
    stored_levels: int,
    d_max: int,
    delta: Fraction,
) -> dict:
    alpha = float(delta / stored_levels)
    upper_tables = {
        level: bounded_level_upper_table(rows, level, d_max, alpha)
        for level in range(1, stored_levels + 1)
    }

    worst_failure = Fraction(0)
    worst_d = 0

    for true_d in range(d_max + 1):
        state = distribution(rows, stored_levels, true_d)
        failure = Fraction(0)

        for mask, probability in state.items():
            per_level = [
                upper_tables[level][
                    level_count(mask, rows, stored_levels, level)
                ]
                for level in range(1, stored_levels + 1)
            ]
            combined = min(per_level)
            if combined < true_d:
                failure += probability

        if failure > delta:
            raise AssertionError(
                f"multilevel coverage failed m={rows} J={stored_levels} "
                f"d={true_d}: failure={failure} delta={delta}"
            )

        if failure > worst_failure:
            worst_failure = failure
            worst_d = true_d

    return {
        "rows": rows,
        "stored_levels": stored_levels,
        "d_max": d_max,
        "delta": str(delta),
        "alpha_per_level": str(delta / stored_levels),
        "worst_failure": str(worst_failure),
        "worst_d": worst_d,
        "coverage_holds_on_exact_grid": True,
    }


def build_payload() -> dict:
    cases = [
        exact_multilevel_case(1, 2, 12, Fraction(1, 20)),
        exact_multilevel_case(2, 2, 10, Fraction(1, 20)),
        exact_multilevel_case(2, 3, 8, Fraction(1, 20)),
        exact_multilevel_case(3, 2, 8, Fraction(1, 20)),
    ]

    return {
        "model": "strict-compact-multilevel-exact-small-v1",
        "scope": (
            "exact tiny full-state coverage regression for the proposed "
            "Bonferroni + de-Poissonized KL construction"
        ),
        "cases": cases,
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
