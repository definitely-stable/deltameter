#!/usr/bin/env python3
"""Monotone de-Poissonization checks for STRICT-COMPACT-001.

For one stored level let F_d(s)=P[S_j(d)<=s] in the exact fixed-d chain.
If N~Poisson(lambda), the Poissonized CDF is

    G_lambda(s)=E[F_N(s)].

Because F_n(s) is nonincreasing in n,

    G_lambda(s) >= P[N<=d] * F_d(s),

hence

    F_d(s) <= G_lambda(s) / P[N<=d].

Taking lambda=d and integer d, Choi's sharp Poisson-median bounds imply that d is
a median of Poisson(d), so P[N<=d]>=1/2. Therefore the simple universal bound is

    F_d(s) <= 2 * G_d(s).

Under Poissonization, S_j is Binomial(m,p_j(d)), where

    p_j(d)=(1-exp(-d/(m*2^j)))/2.

This script checks the inequality against the exact tiny Fraction oracle. The
mathematical statement does not depend on floating-point checks; they are regression
evidence only.
"""

from __future__ import annotations

import json
import math
from fractions import Fraction

from strict_compact_level_count import cdf, distribution
from strict_compact_poisson_width import binomial_cdf, level_probability


def poisson_cdf_at_integer_mean(d: int) -> float:
    if d < 0:
        raise ValueError("d must be non-negative")
    if d == 0:
        return 1.0

    term = math.exp(-float(d))
    total = term
    for k in range(1, d + 1):
        term *= d / k
        total += term
    return total


def checked_case(rows: int, level: int, d: int) -> dict:
    fixed = cdf(distribution(rows, level, d))
    p = level_probability(float(d), rows, level)
    poisson_denominator = poisson_cdf_at_integer_mean(d)

    max_ratio_to_poissonized = 0.0
    max_ratio_to_tight_bound = 0.0
    worst_s = 0

    if poisson_denominator < 0.5 - 1e-14:
        raise AssertionError("integer-mean Poisson median lower bound failed")

    for observed, exact_fixed in enumerate(fixed):
        poissonized = binomial_cdf(rows, p, observed)
        fixed_value = float(exact_fixed)

        if poissonized > 0.0:
            ratio = fixed_value / poissonized
            if ratio > max_ratio_to_poissonized:
                max_ratio_to_poissonized = ratio
                worst_s = observed

        tight_upper = min(1.0, poissonized / poisson_denominator)
        if fixed_value > tight_upper + 5e-12:
            raise AssertionError(
                f"de-Poissonization failed m={rows} j={level} d={d} "
                f"s={observed}: fixed={fixed_value} upper={tight_upper}"
            )

        simple_upper = min(1.0, 2.0 * poissonized)
        if fixed_value > simple_upper + 5e-12:
            raise AssertionError(
                f"factor-two bound failed m={rows} j={level} d={d} s={observed}"
            )

        if tight_upper > 0.0:
            max_ratio_to_tight_bound = max(
                max_ratio_to_tight_bound,
                fixed_value / tight_upper,
            )

    return {
        "rows": rows,
        "level": level,
        "d": d,
        "poisson_cdf_at_mean": poisson_denominator,
        "max_fixed_over_poissonized": max_ratio_to_poissonized,
        "max_fixed_over_tight_upper": max_ratio_to_tight_bound,
        "worst_s": worst_s,
    }


def build_payload() -> dict:
    cases = [
        checked_case(rows, level, d)
        for rows in (1, 2, 4, 8)
        for level in (1, 2, 3, 4)
        for d in range(1, 25)
    ]

    minimum_denominator = min(
        row["poisson_cdf_at_mean"] for row in cases
    )
    maximum_ratio = max(
        row["max_fixed_over_poissonized"] for row in cases
    )

    return {
        "model": "strict-compact-monotone-depoissonization-v1",
        "theorem_route": (
            "fixed-d stochastic monotonicity + Poisson mixture + "
            "integer-mean Poisson median"
        ),
        "universal_bound": "F_fixed_d(s) <= 2 * F_poissonized_lambda=d(s)",
        "poisson_median_reference": {
            "author": "K. P. Choi",
            "title": "On the Medians of Gamma Distributions and an Equation of Ramanujan",
            "venue": "Proceedings of the American Mathematical Society 121(1), 1994",
            "doi": "10.2307/2160389",
            "used_result": "mu-log(2) <= median(Poisson(mu)) < mu+1/3",
        },
        "scope": (
            "tiny numerical regression of a theorem-derived inequality; "
            "binary64 is not the proof boundary"
        ),
        "checked_cases": len(cases),
        "minimum_checked_poisson_cdf_at_integer_mean": minimum_denominator,
        "maximum_checked_fixed_over_poissonized": maximum_ratio,
        "cases": cases,
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
