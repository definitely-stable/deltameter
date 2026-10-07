#!/usr/bin/env python3
"""KL-Chernoff strict candidate for STRICT-COMPACT-001.

For stored level j with m rows and fixed true difference d:

1. monotone de-Poissonization gives
       F_fixed(d,s) <= 2 * G_d(s)
   where G_d is the Poissonized level-count CDF;

2. under Poissonization,
       S_j ~ Binomial(m, p_j(d)),
       p_j(d) = (1-exp(-d/(m*2^j)))/2;

3. the standard binomial KL-Chernoff lower-tail bound gives, for x=s/m < p,
       G_d(s) <= exp(-m * D(x || p)).

Therefore

    F_fixed(d,s) <= min(1, 2*exp(-m*D(s/m || p_j(d)))).

This is a theorem-derived finite-sample fixed-d bound. The width quantiles reported
below still use the Poissonized observation law only as a diagnostic distribution;
they are not a fixed-d width theorem.
"""

from __future__ import annotations

import json
import math

from strict_compact_level_count import cdf, distribution
from strict_compact_poisson_width import binomial_cdf, level_probability


def bernoulli_kl(x: float, p: float) -> float:
    if not (0.0 <= x <= 1.0 and 0.0 < p < 1.0):
        raise ValueError("invalid Bernoulli parameters")
    if x == 0.0:
        return -math.log1p(-p)
    if x == 1.0:
        return -math.log(p)
    return x * math.log(x / p) + (1.0 - x) * math.log(
        (1.0 - x) / (1.0 - p)
    )


def fixed_lower_tail_upper_bound(
    rows: int,
    level: int,
    d: float,
    observed: int,
) -> float:
    if rows <= 0 or level <= 0 or d < 0.0:
        raise ValueError("invalid shape")
    if not (0 <= observed <= rows):
        raise ValueError("observed out of range")
    if d == 0.0:
        return 1.0 if observed >= 0 else 0.0

    p = level_probability(d, rows, level)
    x = observed / rows

    if x >= p:
        return 1.0

    bound = 2.0 * math.exp(-rows * bernoulli_kl(x, p))
    return min(1.0, bound)


def crossing_for_candidate(
    rows: int,
    level: int,
    candidate_d: float,
    alpha: float,
) -> int:
    """Minimum s for which the strict fixed-d upper-tail bound exceeds alpha."""
    if not (0.0 < alpha < 1.0):
        raise ValueError("invalid alpha")

    low = 0
    high = rows
    while low < high:
        middle = (low + high) // 2
        if (
            fixed_lower_tail_upper_bound(
                rows, level, candidate_d, middle
            )
            > alpha
        ):
            high = middle
        else:
            low = middle + 1
    return low


def strict_bound_survival_under_poissonized_observations(
    candidate_u: float,
    true_lambda: float,
    rows: int,
    stored_levels: int,
    delta: float,
) -> float:
    alpha = delta / stored_levels
    log_probability = 0.0

    for level in range(1, stored_levels + 1):
        threshold = crossing_for_candidate(
            rows, level, candidate_u, alpha
        )
        p_true = level_probability(true_lambda, rows, level)
        survival = 1.0 - binomial_cdf(
            rows, p_true, threshold - 1
        )

        if survival <= 0.0:
            return 0.0
        log_probability += math.log(survival)

    return math.exp(log_probability)


def diagnostic_quantile_ratio(
    true_lambda: float,
    rows: int,
    stored_levels: int,
    delta: float,
    quantile: float,
) -> float:
    if true_lambda <= 0.0:
        raise ValueError("true lambda must be positive")

    target = 1.0 - quantile
    low = 0.0
    high = max(1.0, 2.0 * true_lambda)

    for _ in range(128):
        if (
            strict_bound_survival_under_poissonized_observations(
                high, true_lambda, rows, stored_levels, delta
            )
            <= target
        ):
            break
        high *= 2.0
    else:
        return math.inf

    for _ in range(52):
        middle = (low + high) / 2.0
        survival = strict_bound_survival_under_poissonized_observations(
            middle, true_lambda, rows, stored_levels, delta
        )
        if survival > target:
            low = middle
        else:
            high = middle

    return high / true_lambda


def tiny_exact_checks() -> dict:
    cases = []
    worst_ratio = 0.0

    for rows in (1, 2, 4, 8):
        for level in (1, 2, 3, 4):
            for d in range(1, 25):
                exact = cdf(distribution(rows, level, d))
                for observed, probability in enumerate(exact):
                    bound = fixed_lower_tail_upper_bound(
                        rows, level, float(d), observed
                    )
                    value = float(probability)
                    if value > bound + 5e-12:
                        raise AssertionError(
                            f"KL fixed-tail bound failed m={rows} j={level} "
                            f"d={d} s={observed}: exact={value} bound={bound}"
                        )
                    if bound > 0.0:
                        worst_ratio = max(worst_ratio, value / bound)
                cases.append((rows, level, d))

    return {
        "checked_cases": len(cases),
        "max_exact_over_bound": worst_ratio,
        "all_checked_fixed_d_cdfs_below_bound": True,
    }


def build_payload() -> dict:
    levels = 64
    delta = 1e-6
    profiles = [
        (256, 2048),
        (1024, 8192),
        (4096, 32768),
        (8192, 65536),
    ]
    lambda_over_m = [1.0, 10.0, 100.0, 1000.0]
    quantiles = [0.50, 0.95, 0.99]

    rows = []
    for m, state_bytes in profiles:
        for ratio in lambda_over_m:
            lam = m * ratio
            result = {
                "rows": m,
                "state_bytes": state_bytes,
                "lambda_over_m": ratio,
            }
            for quantile in quantiles:
                value = diagnostic_quantile_ratio(
                    lam, m, levels, delta, quantile
                )
                if not math.isfinite(value) or value < 1.0:
                    raise AssertionError("invalid diagnostic width")
                result[
                    f"q{int(quantile * 100):02d}_u_over_lambda"
                ] = value
            rows.append(result)

    by_state = {}
    for _m, state_bytes in profiles:
        selected = [
            row for row in rows if row["state_bytes"] == state_bytes
        ]
        by_state[str(state_bytes)] = {
            "max_q50": max(
                row["q50_u_over_lambda"] for row in selected
            ),
            "max_q95": max(
                row["q95_u_over_lambda"] for row in selected
            ),
            "max_q99": max(
                row["q99_u_over_lambda"] for row in selected
            ),
        }

    return {
        "model": "strict-compact-fixed-d-kl-chernoff-v1",
        "coverage_claim": (
            "theorem-derived single-level fixed-d lower-tail upper bound; "
            "Bonferroni over levels gives familywise one-sided coverage"
        ),
        "width_scope": (
            "diagnostic quantiles under Poissonized observations only; "
            "not a fixed-d width theorem"
        ),
        "stored_levels": levels,
        "delta": delta,
        "alpha_per_level": delta / levels,
        "tiny_exact_regression": tiny_exact_checks(),
        "rows": rows,
        "worst_reported_width_by_state": by_state,
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
