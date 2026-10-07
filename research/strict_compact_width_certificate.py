#!/usr/bin/env python3
"""Theorem-derived fixed-d width certificates for STRICT-COMPACT-001.

Coverage uses the lower-tail construction from strict_compact_chernoff.py.

For width, fix an integer target K>=d. If U=min_j U_j exceeds K, then every
per-level bound exceeds K. For any chosen level j this implies S_j is above the
first count t_j(K) where the lower-tail inversion accepts a value >K.

The fixed-d upper tail H_d(t)=P[S_j(d)>=t] is nondecreasing in d. Poissonizing at
lambda=d and using that integer d is a median of Poisson(d) gives

    H_fixed_d(t) <= 2 * H_poissonized_d(t).

The Poissonized level count is Binomial(m,p_j(d)); for x=t/m > p_j(d), the standard
KL-Chernoff upper-tail bound gives

    H_fixed_d(t) <= 2 * exp(-m D(x || p_j(d))).

Taking the best level yields a valid upper bound on P[U>K]. This gives a
theorem-derived fixed-d width certificate without assuming independence between
levels.

Floating arithmetic here is research evidence. A production Proven API still needs
a conservative numerical-rounding implementation/audit.
"""

from __future__ import annotations

import json
import math

from strict_compact_chernoff import (
    bernoulli_kl,
    crossing_for_candidate,
)
from strict_compact_poisson_width import level_probability


def fixed_upper_tail_bound(
    rows: int,
    level: int,
    d: int,
    threshold: int,
) -> float:
    if d < 0:
        raise ValueError("negative d")
    if not (0 <= threshold <= rows):
        raise ValueError("threshold")
    if threshold <= 0:
        return 1.0
    if d == 0:
        return 0.0

    p = level_probability(float(d), rows, level)
    x = threshold / rows

    if x <= p:
        return 1.0

    return min(1.0, 2.0 * math.exp(-rows * bernoulli_kl(x, p)))


def width_failure_bound(
    rows: int,
    stored_levels: int,
    delta: float,
    d: int,
    target_k: int,
) -> tuple[float, int, int]:
    if target_k < d:
        raise ValueError("target must be >= true d")

    alpha = delta / stored_levels
    best = 1.0
    best_level = 1
    best_threshold = 0

    for level in range(1, stored_levels + 1):
        threshold = crossing_for_candidate(
            rows,
            level,
            float(target_k),
            alpha,
        )
        bound = fixed_upper_tail_bound(
            rows,
            level,
            d,
            threshold,
        )
        if bound < best:
            best = bound
            best_level = level
            best_threshold = threshold

    return best, best_level, best_threshold


def certified_ratio(
    rows: int,
    stored_levels: int,
    delta: float,
    d: int,
    width_failure: float,
) -> dict:
    if d <= 0:
        raise ValueError("d must be positive")
    if not (0.0 < width_failure < 1.0):
        raise ValueError("width failure")

    low = d
    high = max(d + 1, 2 * d)

    for _ in range(80):
        bound, _level, _threshold = width_failure_bound(
            rows, stored_levels, delta, d, high
        )
        if bound <= width_failure:
            break
        high *= 2
    else:
        raise ArithmeticError("failed to bracket width certificate")

    while low < high:
        middle = (low + high) // 2
        bound, _level, _threshold = width_failure_bound(
            rows, stored_levels, delta, d, middle
        )
        if bound <= width_failure:
            high = middle
        else:
            low = middle + 1

    bound, level, threshold = width_failure_bound(
        rows, stored_levels, delta, d, low
    )
    return {
        "target_k": low,
        "u_over_d": low / d,
        "failure_upper_bound": bound,
        "certifying_level": level,
        "count_threshold": threshold,
    }


def profile_grid(rows: int, state_bytes: int) -> dict:
    levels = 64
    delta = 1e-6

    # One octave samples the log2 phase; selected higher octaves check that the
    # finite J=64 boundary does not silently invalidate the phase picture.
    phase = [2.0 ** (index / 16.0) for index in range(16)]
    octave_shifts = [0, 1, 2, 4, 8, 16, 32]
    points = []

    for octave in octave_shifts:
        for multiplier in phase:
            d = max(1, round(rows * multiplier * (1 << octave)))
            if d >= 1 << 64:
                continue

            q95 = certified_ratio(
                rows, levels, delta, d, width_failure=0.05
            )
            q99 = certified_ratio(
                rows, levels, delta, d, width_failure=0.01
            )
            points.append(
                {
                    "d": d,
                    "d_over_rows": d / rows,
                    "octave": octave,
                    "phase_multiplier": multiplier,
                    "q95": q95,
                    "q99": q99,
                }
            )

    return {
        "rows": rows,
        "state_bytes": state_bytes,
        "points": points,
        "worst_q95_u_over_d": max(
            row["q95"]["u_over_d"] for row in points
        ),
        "worst_q99_u_over_d": max(
            row["q99"]["u_over_d"] for row in points
        ),
        "best_q95_u_over_d": min(
            row["q95"]["u_over_d"] for row in points
        ),
    }


def low_d_anchors(rows: int, state_bytes: int) -> dict:
    levels = 64
    delta = 1e-6
    ds = [
        1, 2, 4, 8, 16, 32, 64, 128, 256, 512,
        1024, 1536, 2048, 2560, 3072, 4096,
    ]
    return {
        "rows": rows,
        "state_bytes": state_bytes,
        "points": [
            {
                "d": d,
                "q95": certified_ratio(
                    rows, levels, delta, d, width_failure=0.05
                ),
                "q99": certified_ratio(
                    rows, levels, delta, d, width_failure=0.01
                ),
            }
            for d in ds
        ],
    }


def build_payload() -> dict:
    profiles = [
        profile_grid(1024, 8192),
        profile_grid(4096, 32768),
        profile_grid(8192, 65536),
    ]
    anchors = [
        low_d_anchors(4096, 32768),
        low_d_anchors(8192, 65536),
    ]

    by_state = {
        profile["state_bytes"]: profile
        for profile in profiles
    }
    profile_32k = by_state[32768]
    profile_64k = by_state[65536]
    profile_8k = by_state[8192]

    if profile_32k["worst_q95_u_over_d"] > 1.5:
        raise AssertionError("32 KiB profile missed frozen q95 width gate")
    if profile_32k["worst_q99_u_over_d"] > 1.5:
        raise AssertionError("32 KiB profile missed 1.5x q99 diagnostic")
    if profile_64k["worst_q95_u_over_d"] > 1.5:
        raise AssertionError("64 KiB profile missed frozen q95 width gate")
    if profile_8k["worst_q95_u_over_d"] <= 1.5:
        raise AssertionError("8 KiB profile unexpectedly crossed frozen q95 gate")

    anchor_summary = {}
    for profile in anchors:
        passing = [
            row["d"]
            for row in profile["points"]
            if row["q95"]["u_over_d"] <= 1.5
        ]
        anchor_summary[str(profile["state_bytes"])] = {
            "first_reported_d_with_q95_at_most_1_5": (
                min(passing) if passing else None
            )
        }

    return {
        "model": "strict-compact-fixed-d-width-certificate-v1",
        "coverage_delta": 1e-6,
        "width_claim": (
            "for each reported fixed d, q95/q99 entries upper-bound "
            "P[U > target_k] by 0.05/0.01 respectively"
        ),
        "numeric_scope": (
            "theorem-derived probability bounds evaluated in binary64; "
            "production Proven status requires conservative rounding audit"
        ),
        "profiles": profiles,
        "low_d_anchors": anchors,
        "anchor_summary": anchor_summary,
        "product_signal": {
            "8_kib_q95_gate": "MISS",
            "32_kib_q95_gate": "PASS",
            "32_kib_q99_at_1_5x": "PASS",
            "64_kib_q95_gate": "PASS",
            "phase_a_direction": "CONTINUE_NUMERICAL_ROUNDING_AND_RANGE_CLOSURE",
        },
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
