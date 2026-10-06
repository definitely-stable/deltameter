#!/usr/bin/env python3
"""Derive and verify conservative EnergyDeltaMeter profiles.

Model:
    E[T]   = d
    Var(T) = 2 d (d - 1) / B

Chebyshev gives:
    P(|T-d| >= epsilon*d) <= 2 / (B*epsilon^2)

Independent tables are amplified by taking the median and evaluating the
exact binomial upper tail.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path

EPSILONS = (0.05, 0.10, 0.20)
DELTAS = (1e-3, 1e-6, 1e-9)
TARGET_SINGLE_TABLE_FAILURE = 0.10

CSV_HEADER = (
    "epsilon",
    "delta",
    "buckets",
    "tables",
    "single_table_failure_bound",
    "median_failure_bound",
    "counters",
    "state_bytes_i64",
)


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


def build_csv() -> str:
    output = io.StringIO()
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(CSV_HEADER)

    for profile in derive_profiles():
        writer.writerow(
            (
                format(profile.epsilon, ".17g"),
                format(profile.delta, ".17g"),
                profile.buckets,
                profile.tables,
                format(profile.single_table_failure_bound, ".17g"),
                format(profile.median_failure_bound, ".17g"),
                profile.counters,
                profile.state_bytes_i64,
            )
        )

    return output.getvalue()


def check_csv(path: Path) -> None:
    expected = build_csv()
    actual = path.read_text(encoding="utf-8")
    if actual != expected:
        raise SystemExit(
            f"{path} is stale; regenerate with "
            f"'python research/energy_profiles.py --write-csv {path}'"
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-csv", type=Path)
    parser.add_argument("--check-csv", type=Path)
    args = parser.parse_args()

    if args.write_csv is not None:
        args.write_csv.write_text(build_csv(), encoding="utf-8")

    if args.check_csv is not None:
        check_csv(args.check_csv)

    if args.write_csv is None and args.check_csv is None:
        print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
