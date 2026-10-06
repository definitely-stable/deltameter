#!/usr/bin/env python3
"""Exact finite-J truncation budgets for published GF(2)-F-PCSA.

DeltaMeter uses one-based geometric levels:
    P(level = j) = 2^-j, j >= 1.

Two events are intentionally separated:

1. truncation signal:
   the sampled level lies above the stored boundary J.
   This matches the current implementation's Truncated outcome and does not
   depend on the GF(2) coefficient.

2. state-distorting truncation:
   level > J and g(v)=1, so an un-stored nonzero contribution would have
   changed the ideal infinite-level state.

For one active element:
    p_signal     = 2^-J
    p_distortion = 2^-(J+1)

The row choice does NOT add a 1/m factor after summing over all rows.
"""

from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass


@dataclass(frozen=True)
class Budget:
    d: int
    stored_levels: int
    single_signal_probability: float
    any_signal_probability: float
    single_state_distortion_probability: float
    any_state_distortion_probability: float


def any_event_probability(d: int, p_single: float) -> float:
    if d < 0:
        raise ValueError("d must be non-negative")
    if not (0.0 <= p_single <= 1.0):
        raise ValueError("p_single must be in [0,1]")
    if d == 0 or p_single == 0.0:
        return 0.0
    if p_single == 1.0:
        return 1.0
    return -math.expm1(d * math.log1p(-p_single))


def truncation_budget(d: int, stored_levels: int) -> Budget:
    if not (1 <= stored_levels <= 1024):
        raise ValueError("stored_levels must be positive")

    p_signal = math.ldexp(1.0, -stored_levels)
    p_distortion = math.ldexp(1.0, -(stored_levels + 1))
    if p_distortion * 2.0 != p_signal:
        raise AssertionError("state-distortion probability must be half the signal probability")

    return Budget(
        d=d,
        stored_levels=stored_levels,
        single_signal_probability=p_signal,
        any_signal_probability=any_event_probability(d, p_signal),
        single_state_distortion_probability=p_distortion,
        any_state_distortion_probability=any_event_probability(d, p_distortion),
    )


def max_d_for_budget(stored_levels: int, delta: float, *, distortion_only: bool) -> int:
    if not (0.0 < delta < 1.0):
        raise ValueError("delta must be in (0,1)")

    p_single = math.ldexp(
        1.0,
        -(stored_levels + 1 if distortion_only else stored_levels),
    )
    ratio = math.log1p(-delta) / math.log1p(-p_single)
    return max(0, math.floor(ratio))


def build_payload() -> dict:
    levels = 64
    ds = [1 << exponent for exponent in (20, 32, 48, 60, 64)]
    deltas = (1e-3, 1e-6, 1e-9)

    return {
        "model": "published-fpcsa-f2-finite-j-truncation-v1",
        "level_indexing": "one-based: P(level=j)=2^-j",
        "claims": {
            "signal": "P(level>J)=2^-J",
            "state_distortion": "P(level>J and g=1)=2^-(J+1)",
            "no_row_factor": "row probabilities sum to one across all m rows",
        },
        "j64_grid": [asdict(truncation_budget(d, levels)) for d in ds],
        "max_d_by_budget": {
            str(delta): {
                "signal": max_d_for_budget(levels, delta, distortion_only=False),
                "state_distortion": max_d_for_budget(
                    levels, delta, distortion_only=True
                ),
            }
            for delta in deltas
        },
    }


def main() -> None:
    print(json.dumps(build_payload(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
