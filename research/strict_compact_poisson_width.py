#!/usr/bin/env python3
"""Deterministic Poissonized multilevel-width screen for STRICT-COMPACT-001.

This is NOT a fixed-d theorem and NOT production coverage evidence.

Under D~Poisson(lambda), stored level cells are independent. For level j and m rows:

    p_j(lambda) = (1 - exp(-lambda/(m*2^j))) / 2
    S_j ~ Binomial(m, p_j)

For alpha_j = delta/J, define the same one-sided per-level inversion planned for
fixed-d work. Because levels are independent in the Poissonized model, the
distribution of U=min_j U_j(S_j) can be evaluated deterministically without Monte
Carlo.

Binomial CDFs use a binary64 regularized incomplete-beta implementation. This is
diagnostic numerical evidence only; it is not an auditable proof boundary.
"""

from __future__ import annotations

import json
import math
from statistics import NormalDist


def _betacf(a: float, b: float, x: float) -> float:
    eps = 3e-14
    tiny = 1e-300
    qab = a + b
    qap = a + 1.0
    qam = a - 1.0
    c = 1.0
    d = 1.0 - qab * x / qap
    if abs(d) < tiny:
        d = tiny
    d = 1.0 / d
    h = d

    for iteration in range(1, 201):
        doubled = 2 * iteration
        aa = iteration * (b - iteration) * x / (
            (qam + doubled) * (a + doubled)
        )
        d = 1.0 + aa * d
        if abs(d) < tiny:
            d = tiny
        c = 1.0 + aa / c
        if abs(c) < tiny:
            c = tiny
        d = 1.0 / d
        h *= d * c

        aa = -(a + iteration) * (qab + iteration) * x / (
            (a + doubled) * (qap + doubled)
        )
        d = 1.0 + aa * d
        if abs(d) < tiny:
            d = tiny
        c = 1.0 + aa / c
        if abs(c) < tiny:
            c = tiny
        d = 1.0 / d
        delta = d * c
        h *= delta

        if abs(delta - 1.0) < eps:
            return h

    raise ArithmeticError("incomplete-beta continued fraction did not converge")


def regularized_beta(x: float, a: float, b: float) -> float:
    if x <= 0.0:
        return 0.0
    if x >= 1.0:
        return 1.0

    log_bt = (
        math.lgamma(a + b)
        - math.lgamma(a)
        - math.lgamma(b)
        + a * math.log(x)
        + b * math.log1p(-x)
    )
    bt = math.exp(log_bt)

    if x < (a + 1.0) / (a + b + 2.0):
        value = bt * _betacf(a, b, x) / a
    else:
        value = 1.0 - bt * _betacf(b, a, 1.0 - x) / b

    return min(1.0, max(0.0, value))


def binomial_cdf(n: int, p: float, k: int) -> float:
    if n < 0:
        raise ValueError("negative n")
    if not (0.0 <= p <= 1.0):
        raise ValueError("invalid p")
    if k < 0:
        return 0.0
    if k >= n:
        return 1.0
    if p == 0.0:
        return 1.0
    if p == 1.0:
        return 0.0

    # P[X<=k] = I_(1-p)(n-k,k+1)
    return regularized_beta(1.0 - p, n - k, k + 1)


def level_probability(lam: float, rows: int, level: int) -> float:
    if lam < 0.0:
        raise ValueError("lambda must be non-negative")
    if rows <= 0 or level <= 0:
        raise ValueError("invalid shape")
    return -0.5 * math.expm1(-lam / (rows * float(1 << level)))


def lower_alpha_crossing(rows: int, p: float, alpha: float) -> int:
    """Minimum s with BinomialCDF(s) > alpha."""
    if not (0.0 < alpha < 1.0):
        raise ValueError("invalid alpha")
    if p <= 0.0:
        return 0
    if p >= 1.0:
        return rows

    mean = rows * p
    variance = rows * p * (1.0 - p)
    if variance > 0.0:
        guess = int(
            math.floor(
                mean + NormalDist().inv_cdf(alpha) * math.sqrt(variance)
            )
        )
    else:
        guess = int(mean)

    guess = min(rows, max(0, guess))
    current = binomial_cdf(rows, p, guess)

    if current > alpha:
        while guess > 0 and binomial_cdf(rows, p, guess - 1) > alpha:
            guess -= 1
    else:
        while guess < rows and current <= alpha:
            guess += 1
            current = binomial_cdf(rows, p, guess)

    if binomial_cdf(rows, p, guess) <= alpha:
        raise AssertionError("alpha crossing not found")
    if guess > 0 and binomial_cdf(rows, p, guess - 1) > alpha:
        raise AssertionError("alpha crossing is not minimal")
    return guess


def upper_bound_survival(
    candidate_u: float,
    true_lambda: float,
    rows: int,
    stored_levels: int,
    delta: float,
    cdf_multiplier: float,
) -> float:
    """Poissonized P[U > candidate_u] for U=min_j U_j(S_j).

    cdf_multiplier=1 is the pure Poissonized inversion.
    cdf_multiplier=2 models the strict fixed-d de-Poissonized rule
    F_fixed <= 2 * F_poissonized.
    """
    if cdf_multiplier < 1.0:
        raise ValueError("cdf_multiplier must be >= 1")
    alpha = delta / (stored_levels * cdf_multiplier)
    log_probability = 0.0

    for level in range(1, stored_levels + 1):
        p_candidate = level_probability(candidate_u, rows, level)
        threshold = lower_alpha_crossing(rows, p_candidate, alpha)

        p_true = level_probability(true_lambda, rows, level)
        survival = 1.0 - binomial_cdf(rows, p_true, threshold - 1)

        if survival <= 0.0:
            return 0.0
        log_probability += math.log(survival)

    return math.exp(log_probability)


def upper_bound_quantile_ratio(
    true_lambda: float,
    rows: int,
    stored_levels: int,
    delta: float,
    quantile: float,
    cdf_multiplier: float,
) -> float:
    if true_lambda <= 0.0:
        raise ValueError("true lambda must be positive")
    if not (0.0 < quantile < 1.0):
        raise ValueError("invalid quantile")

    target_survival = 1.0 - quantile
    low = 0.0
    high = max(1.0, 2.0 * true_lambda)

    for _ in range(128):
        if (
            upper_bound_survival(
                high, true_lambda, rows, stored_levels, delta, cdf_multiplier
            )
            <= target_survival
        ):
            break
        high *= 2.0
    else:
        return math.inf

    for _ in range(52):
        middle = (low + high) / 2.0
        survival = upper_bound_survival(
            middle, true_lambda, rows, stored_levels, delta, cdf_multiplier
        )
        if survival > target_survival:
            low = middle
        else:
            high = middle

    return high / true_lambda


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
    rules = [
        ("poissonized", 1.0),
        ("fixed_d_depoissonized", 2.0),
    ]

    for m, state_bytes in profiles:
        for ratio in lambda_over_m:
            lam = m * ratio
            result = {
                "rows": m,
                "state_bytes": state_bytes,
                "lambda_over_m": ratio,
            }
            for rule_name, multiplier in rules:
                for quantile in quantiles:
                    value = upper_bound_quantile_ratio(
                        lam, m, levels, delta, quantile, multiplier
                    )
                    if not math.isfinite(value) or value < 1.0:
                        raise AssertionError("invalid width result")
                    result[
                        f"{rule_name}_q{int(quantile * 100):02d}_u_over_lambda"
                    ] = value
            rows.append(result)

    by_state = {}
    for _m, state_bytes in profiles:
        selected = [row for row in rows if row["state_bytes"] == state_bytes]
        by_state[str(state_bytes)] = {
            "poissonized": {
                "max_q50": max(
                    row["poissonized_q50_u_over_lambda"] for row in selected
                ),
                "max_q95": max(
                    row["poissonized_q95_u_over_lambda"] for row in selected
                ),
                "max_q99": max(
                    row["poissonized_q99_u_over_lambda"] for row in selected
                ),
            },
            "fixed_d_depoissonized_rule_under_poissonized_observations": {
                "max_q50": max(
                    row["fixed_d_depoissonized_q50_u_over_lambda"]
                    for row in selected
                ),
                "max_q95": max(
                    row["fixed_d_depoissonized_q95_u_over_lambda"]
                    for row in selected
                ),
                "max_q99": max(
                    row["fixed_d_depoissonized_q99_u_over_lambda"]
                    for row in selected
                ),
            },
        }

    return {
        "model": "strict-compact-poissonized-bonferroni-width-v2",
        "scope": (
            "diagnostic Poissonized observation model with binary64 binomial tails; "
            "includes the factor-two tail-budget penalty used by the strict fixed-d "
            "de-Poissonized rule, but width quantiles themselves are not fixed-d proof"
        ),
        "stored_levels": levels,
        "delta": delta,
        "poissonized_alpha_per_level": delta / levels,
        "fixed_d_depoissonized_alpha_per_level": delta / (2 * levels),
        "rows": rows,
        "worst_reported_width_by_state": by_state,
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
