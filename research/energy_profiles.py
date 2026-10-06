#!/usr/bin/env python3
"""Derive conservative EnergyDeltaMeter profiles.

Model:
    E[T]   = d
    Var(T) = 2 d (d - 1) / B

Chebyshev gives:
    P(|T-d| >= epsilon*d) <= 2 / (B*epsilon^2)

Independent tables are amplified by taking the median and evaluating the
exact binomial upper tail.
"""

from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass

EPSILONS = (0.05, 0.10, 0.20)
DELTAS = (1e-3, 1e-6, 1e-9)
TARGET_SINGLE_TABLE_FAILURE = 0.10


@dataclass(frozen=True)
class Profile:
    epsilon: float
    delta: float
    buckets: int
    tables: int
    single_table_failure_bound: float
    median_failure_bound: float
    counters: int
    state_bytes_i64: int


def binomial_upper_tail(n: int, p: float, threshold: int) -> float:
    return sum(
        math.comb(n, k) * (p**k) * ((1.0 - p) ** (n - k))
        for k in range(threshold, n + 1)
    )


def choose_buckets(epsilon: float) -> int:
    buckets = 1
    while 2.0 / (buckets * epsilon * epsilon) > TARGET_SINGLE_TABLE_FAILURE:
        buckets *= 2
    return buckets


def choose_tables(p: float, delta: float) -> tuple[int, float]:
    tables = 1
    while True:
        threshold = (tables + 1) // 2
        bound = binomial_upper_tail(tables, p, threshold)
        if bound <= delta:
            return tables, bound
        tables += 2


def derive_profiles() -> list[Profile]:
    profiles: list[Profile] = []
    for epsilon in EPSILONS:
        buckets = choose_buckets(epsilon)
        p = 2.0 / (buckets * epsilon * epsilon)
        for delta in DELTAS:
            tables, failure = choose_tables(p, delta)
            counters = buckets * tables
            profiles.append(
                Profile(
                    epsilon=epsilon,
                    delta=delta,
                    buckets=buckets,
                    tables=tables,
                    single_table_failure_bound=p,
                    median_failure_bound=failure,
                    counters=counters,
                    state_bytes_i64=counters * 8,
                )
            )
    return profiles


def build_payload() -> dict:
    return {
        "model": "energy-countsketch-chebyshev-median-v0",
        "claim": (
            "conservative finite-sample upper configuration; "
            "not an optimality claim"
        ),
        "formula": {
            "variance": "Var(T) = 2 d (d - 1) / B",
            "single_table_failure": "p <= 2 / (B epsilon^2)",
            "amplification": (
                "exact binomial upper tail for median of odd R independent tables"
            ),
        },
        "profiles": [asdict(profile) for profile in derive_profiles()],
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
