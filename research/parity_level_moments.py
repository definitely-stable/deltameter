#!/usr/bin/env python3
"""Exact finite-d first two moments for one published F-PCSA level over GF(2).

This is a research oracle for the published random-coefficient construction.
It is not a high-confidence tail theorem and not the set-specialized g(v)=1
variant.
"""

from __future__ import annotations

import argparse
import json
import math


def level_moments(m: int, d: int, r: float) -> dict[str, float | int]:
    if m <= 0:
        raise ValueError("m must be positive")
    if d < 0:
        raise ValueError("d must be non-negative")
    if not (0.0 <= r <= 1.0):
        raise ValueError("r must be in [0, 1]")

    a = 1.0 - r / m
    b = 1.0 - 2.0 * r / m

    mean = 0.5 * m * (1.0 - a**d)
    variance = 0.25 * m * (
        1.0 + (m - 1) * (b**d) - m * (a ** (2 * d))
    )

    # Protect JSON output from tiny negative roundoff near zero.
    variance = max(variance, 0.0)

    return {
        "m": m,
        "d": d,
        "level_mass_r": r,
        "mean_hamming_weight": mean,
        "variance_hamming_weight": variance,
        "sd_hamming_weight": math.sqrt(variance),
    }


def build_grid() -> dict:
    cases = []
    for m in (64, 256, 1024):
        for r in (1.0, 0.5, 0.25):
            for d in (0, 1, 2, 4, 16, 64, 256, 1024, 4096):
                cases.append(level_moments(m, d, r))
    return {
        "model": "published-fpcsa-f2-single-level-moments-v0",
        "coverage": "none; exact first two moments only",
        "cases": cases,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--m", type=int, default=256)
    parser.add_argument("--d", type=int, default=1000)
    parser.add_argument("--r", type=float, default=0.5)
    parser.add_argument("--grid", action="store_true")
    args = parser.parse_args()

    payload = (
        build_grid()
        if args.grid
        else {
            "model": "published-fpcsa-f2-single-level-moments-v0",
            "coverage": "none; exact first two moments only",
            "result": level_moments(args.m, args.d, args.r),
        }
    )
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
