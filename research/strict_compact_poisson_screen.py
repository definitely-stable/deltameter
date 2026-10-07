#!/usr/bin/env python3
"""Optimistic Poissonized information screen for STRICT-COMPACT-001.

This is NOT a fixed-d estimator and NOT a coverage proof.

Under D~Poisson(lambda), stored GF(2)-F-PCSA cells split independently. For level j
and m rows:

    p_j(lambda) = (1 - exp(-lambda / (m * 2^j))) / 2.

Thus S_j~Binomial(m,p_j), independently across stored levels in this Poissonized
model. We compute the Fisher information in lambda carried by the complete vector
of level counts and report the corresponding local Cramer-Rao relative standard
deviation lower bound.

The result is only an optimistic opportunity screen: if even this lower bound were
poor, deeper fixed-d inference would be unattractive. A good bound does not prove
that an estimator or a strict finite-sample interval achieves it.
"""

from __future__ import annotations

import json
import math


def cell_one_probability(lam: float, rows: int, level: int) -> float:
    if lam <= 0.0:
        raise ValueError("lambda must be positive")
    if rows <= 0:
        raise ValueError("rows must be positive")
    if level <= 0:
        raise ValueError("level must be positive")

    exponent = -lam / (rows * float(1 << level))
    return -0.5 * math.expm1(exponent)


def cell_one_derivative(lam: float, rows: int, level: int) -> float:
    exponent = -lam / (rows * float(1 << level))
    return math.exp(exponent) / (2.0 * rows * float(1 << level))


def fisher_information(lam: float, rows: int, stored_levels: int) -> float:
    total = 0.0
    for level in range(1, stored_levels + 1):
        p = cell_one_probability(lam, rows, level)
        dp = cell_one_derivative(lam, rows, level)
        if 0.0 < p < 1.0:
            total += rows * dp * dp / (p * (1.0 - p))
    return total


def relative_crlb(lam: float, rows: int, stored_levels: int) -> float:
    info = fisher_information(lam, rows, stored_levels)
    if info <= 0.0:
        return math.inf
    return 1.0 / (lam * math.sqrt(info))


def build_payload() -> dict:
    levels = 64
    profiles = [
        (256, 2048),
        (1024, 8192),
        (4096, 32768),
        (8192, 65536),
    ]
    ratios = [0.1, 0.3, 1.0, 3.0, 10.0, 30.0, 100.0, 1000.0]

    rows = []
    for m, state_bytes in profiles:
        for ratio in ratios:
            lam = m * ratio
            rse = relative_crlb(lam, m, levels)
            rows.append(
                {
                    "rows": m,
                    "state_bytes": state_bytes,
                    "lambda_over_m": ratio,
                    "relative_crlb": rse,
                    "sqrt_m_scaled_coefficient": rse * math.sqrt(m),
                }
            )

    central = [
        row["sqrt_m_scaled_coefficient"]
        for row in rows
        if row["lambda_over_m"] >= 10.0
    ]
    if not all(math.isfinite(value) and value > 0.0 for value in central):
        raise AssertionError("invalid information screen")

    return {
        "model": "strict-compact-poissonized-fisher-screen-v1",
        "scope": (
            "optimistic Poissonized local-information lower bound only; "
            "not fixed-d coverage and not an achievable estimator claim"
        ),
        "stored_levels": levels,
        "rows": rows,
        "central_scaled_coefficient_min": min(central),
        "central_scaled_coefficient_max": max(central),
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
