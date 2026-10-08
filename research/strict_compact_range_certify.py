#!/usr/bin/env python3
"""Exact interval closure for STRICT-COMPACT-002.

Goal: prove, for every integer d in a declared range, the q95 width statement

    P_d[U/d > 1.5] <= 0.05

for the actual Q32 runtime lookup table.

The proof is interval-based, not a sampled scan.

For d in [a,b], let K_a=floor(3a/2). Since K_d>=K_a and the runtime per-level
upper bound is monotone in the observed count, the event U>K_d implies, for any
chosen level j,

    S_j(d) >= t_j(K_a),

where t_j(K) is the exact count threshold induced by the committed Q32 table.
Since S_j(d) is stochastically nondecreasing in d,

    P_d[S_j(d)>=t] <= P_b[S_j(b)>=t].

The fixed-d upper tail is bounded by monotone de-Poissonization plus KL-Chernoff:

    P_b[S_j>=t] <= 2 exp(-m D(t/m || p_j(b))).

This file certifies that final inequality without float/Decimal/exp/log. It uses
an exact rational upper bound on p_j(b) and an exact integer likelihood-ratio
comparison against 2/0.05 = 40.

Binary64 is used only to rank candidate levels. It never decides validity.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from fractions import Fraction
from pathlib import Path

from strict_compact_chernoff import bernoulli_kl
from strict_compact_lookup_certify import (
    P_Q,
    ROWS,
    SENTINEL,
    likelihood_crosses_target,
    p_upper_q64_from_z,
)
from strict_compact_poisson_width import level_probability

LEVELS = 64
Q32 = 1 << 32
DOMAIN = 1 << 64
TABLE_ENTRIES = ROWS // 2
WIDTH_NUMERATOR = 3
WIDTH_DENOMINATOR = 2
WIDTH_FAILURE_TARGET = 40  # 2 / 0.05
DECLARED_D_MIN = 4096


def load_table(path: Path) -> list[int]:
    values = [
        int(line)
        for line in path.read_text(encoding="ascii").splitlines()
        if line
    ]
    if len(values) != TABLE_ENTRIES:
        raise ValueError("table length")
    return values


def level_upper(q32: int, level: int) -> int:
    if q32 == SENTINEL:
        return DOMAIN
    numerator = q32 << level
    return min(DOMAIN, (numerator + Q32 - 1) // Q32)


def runtime_threshold(table: list[int], level: int, k: int) -> int:
    """Minimum count s with runtime U_j(s)>k."""
    if k >= DOMAIN:
        return ROWS + 1

    for observed, q32 in enumerate(table):
        if level_upper(q32, level) > k:
            return observed

    # Counts >= TABLE_ENTRIES are not looked up and therefore contribute DOMAIN.
    return TABLE_ENTRIES


def float_tail_score(level: int, d: int, threshold: int) -> float:
    """Proposal-only score. Exact rational arithmetic decides PASS."""
    if threshold <= 0:
        return 1.0
    if threshold > ROWS:
        return 0.0

    p = level_probability(float(d), ROWS, level)
    x = threshold / ROWS
    if x <= p:
        return 1.0
    return min(1.0, 2.0 * math.exp(-ROWS * bernoulli_kl(x, p)))


def exact_upper_tail_pass(level: int, d: int, threshold: int) -> bool:
    if threshold <= 0:
        return False
    if threshold > ROWS:
        return True

    z = Fraction(d, ROWS * (1 << level))

    # The exact Taylor enclosure helper intentionally refuses a too-small degree.
    # A useful certifying level is always in the nonsaturated scale regime.
    try:
        p_upper = p_upper_q64_from_z(z)
    except ValueError:
        return False

    # Need x=t/m strictly above the certified p upper bound.
    if p_upper * ROWS >= threshold * P_Q:
        return False

    # For x>p, D(x||p) decreases as p increases. Since p_true<=p_upper,
    # D(x||p_true)>=D(x||p_upper). Exact LR >=40 proves the 5% tail target.
    return likelihood_crosses_target(
        threshold, p_upper, WIDTH_FAILURE_TARGET
    )


def certify_interval(
    table: list[int], a: int, b: int
) -> dict | None:
    if not (DECLARED_D_MIN <= a <= b < DOMAIN):
        raise ValueError("interval")

    k_a = (WIDTH_NUMERATOR * a) // WIDTH_DENOMINATOR
    if k_a >= DOMAIN:
        return {
            "a": a,
            "b": b,
            "k_a": k_a,
            "kind": "domain_ceiling",
        }

    proposals = []
    for level in range(1, LEVELS + 1):
        threshold = runtime_threshold(table, level, k_a)
        score = float_tail_score(level, b, threshold)
        proposals.append((score, level, threshold))
    proposals.sort()

    for score, level, threshold in proposals:
        if exact_upper_tail_pass(level, b, threshold):
            return {
                "a": a,
                "b": b,
                "k_a": k_a,
                "kind": "exact_tail",
                "level": level,
                "count_threshold": threshold,
                "proposal_float_tail": score,
                "proof_target_2_over_beta": WIDTH_FAILURE_TARGET,
            }
    return None


def recursive_cover(
    table: list[int],
    a: int,
    b: int,
    out: list[dict],
) -> None:
    witness = certify_interval(table, a, b)
    if witness is not None:
        out.append(witness)
        return

    if a == b:
        raise AssertionError(f"uncovered declared d={a}")

    middle = (a + b) // 2
    recursive_cover(table, a, middle, out)
    recursive_cover(table, middle + 1, b, out)


def build_cover(table: list[int]) -> list[dict]:
    # Above this point U<=2^64 implies U/d<=1.5 deterministically.
    trivial_start = (2 * DOMAIN + 2) // 3
    proof_end = trivial_start - 1

    intervals: list[dict] = []
    start = DECLARED_D_MIN
    while start <= proof_end:
        # Start with an octave-sized interval and split only when exact
        # monotonicity closure is too pessimistic.
        end = min(proof_end, 2 * start - 1)
        recursive_cover(table, start, end, intervals)
        start = end + 1

    intervals.append(
        {
            "a": trivial_start,
            "b": DOMAIN - 1,
            "k_a": (WIDTH_NUMERATOR * trivial_start) // WIDTH_DENOMINATOR,
            "kind": "domain_ceiling",
        }
    )
    return intervals


def validate_cover(intervals: list[dict]) -> None:
    if not intervals:
        raise AssertionError("empty cover")
    expected = DECLARED_D_MIN
    for row in intervals:
        if row["a"] != expected:
            raise AssertionError(
                f"range hole/overlap: expected {expected}, got {row['a']}"
            )
        if row["b"] < row["a"]:
            raise AssertionError("reversed interval")
        expected = row["b"] + 1
    if expected != DOMAIN:
        raise AssertionError("cover does not reach u64 domain ceiling")


def build_payload(table_path: Path) -> dict:
    raw = table_path.read_bytes()
    table = load_table(table_path)
    intervals = build_cover(table)
    validate_cover(intervals)

    tail_intervals = [
        row for row in intervals if row["kind"] == "exact_tail"
    ]
    levels_used = sorted({row["level"] for row in tail_intervals})

    return {
        "format": "deltameter.strict-compact-range-certificate.v1",
        "profile": {
            "rows": ROWS,
            "stored_levels": LEVELS,
            "state_bytes": ROWS * LEVELS // 8,
            "delta": "1e-6",
            "width_quantile": "q95",
            "width_ratio": "1.5",
        },
        "declared_range": {
            "d_min": DECLARED_D_MIN,
            "d_max": DOMAIN - 1,
            "domain_cardinality": DOMAIN,
        },
        "proof": (
            "interval monotonicity + factor-two de-Poissonization + "
            "KL-Chernoff, with exact rational p upper bounds and integer "
            "likelihood-ratio comparisons"
        ),
        "proof_target_2_over_beta": WIDTH_FAILURE_TARGET,
        "table_sha256": hashlib.sha256(raw).hexdigest(),
        "interval_count": len(intervals),
        "exact_tail_interval_count": len(tail_intervals),
        "levels_used": levels_used,
        "intervals": intervals,
        "decision": "RANGE_CERT_PASS",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--table", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()

    payload = build_payload(args.table)
    encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"

    if args.out is None:
        print(encoded, end="")
    else:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(encoded, encoding="utf-8")
        print(
            "STRICT_COMPACT_RANGE_CERT_PASS "
            f"intervals={payload['interval_count']} "
            f"tail_intervals={payload['exact_tail_interval_count']} "
            f"d_min={payload['declared_range']['d_min']} "
            f"d_max={payload['declared_range']['d_max']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
