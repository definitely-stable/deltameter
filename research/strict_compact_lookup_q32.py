#!/usr/bin/env python3
"""Generate the STRICT-COMPACT m=4096/J=64/delta=1e-6 Q32 prototype table.

This is a high-precision research generator, not yet the final public proof
certificate. It deliberately rounds normalized d/2^j thresholds upward so the
integer lookup cannot become narrower than the computed crossing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from decimal import (
    Decimal,
    ROUND_CEILING,
    localcontext,
)
from pathlib import Path

ROWS = 4096
LEVELS = 64
DELTA = Decimal(1) / Decimal(1_000_000)
ALPHA = DELTA / LEVELS
HALF = Decimal(1) / 2
ONE = Decimal(1)
TWO = Decimal(2)
Q_BITS = 32
Q_SCALE = 1 << Q_BITS
U64_MAX = (1 << 64) - 1
SENTINEL = U64_MAX
PRECISION = 80
BISECTION_STEPS = 96


def bernoulli_kl(x: Decimal, p: Decimal) -> Decimal:
    if not (Decimal(0) <= x <= ONE):
        raise ValueError("x")
    if not (Decimal(0) < p < ONE):
        raise ValueError("p")
    if x == 0:
        return -(ONE - p).ln()
    if x == ONE:
        return -p.ln()
    return x * (x / p).ln() + (ONE - x) * (
        (ONE - x) / (ONE - p)
    ).ln()


def log_bound_for_q(observed: int, q: int) -> Decimal:
    x = Decimal(observed) / ROWS
    y = Decimal(q) / Q_SCALE
    p = (ONE - (-y / ROWS).exp()) / TWO
    if p <= x:
        return Decimal(0)  # log(1)
    return TWO.ln() - Decimal(ROWS) * bernoulli_kl(x, p)


def _float_kl(x: float, p: float) -> float:
    if x == 0.0:
        return -math.log1p(-p)
    return x * math.log(x / p) + (1.0 - x) * math.log(
        (1.0 - x) / (1.0 - p)
    )


def threshold_for_count(observed: int, target_kl: Decimal) -> int:
    if not (0 <= observed < ROWS // 2):
        raise ValueError("observed")

    x_decimal = Decimal(observed) / ROWS
    max_kl = bernoulli_kl(x_decimal, HALF)

    if max_kl <= target_kl:
        return SENTINEL

    # Binary64 is proposal generation only. The emitted integer is accepted
    # solely after the Decimal inequality below is checked.
    x = observed / ROWS
    target = float(target_kl)
    low = x
    high = math.nextafter(0.5, 0.0)

    for _ in range(FLOAT_BISECTION_STEPS):
        middle = (low + high) / 2.0
        if _float_kl(x, middle) >= target:
            high = middle
        else:
            low = middle

    normalized = -ROWS * math.log1p(-2.0 * high)
    q = max(1, math.ceil(normalized * Q_SCALE))
    if q >= U64_MAX:
        raise OverflowError("finite Q32 proposal does not fit u64")

    log_alpha = ALPHA.ln()

    # Repair upward until the high-precision evaluator proves this quantized
    # point is on the conservative side of the crossing.
    while log_bound_for_q(observed, q) > log_alpha:
        q += 1
        if q >= U64_MAX:
            raise OverflowError("Q32 repair overflow")

    # Tighten to the smallest Q32 value still certified conservative by the
    # same high-precision evaluator. This makes generation deterministic and
    # prevents proposal precision from affecting the committed table.
    while q > 1 and log_bound_for_q(observed, q - 1) <= log_alpha:
        q -= 1

    if log_bound_for_q(observed, q) > log_alpha:
        raise AssertionError("final Q32 threshold is not conservative")
    if q > 1 and log_bound_for_q(observed, q - 1) <= log_alpha:
        raise AssertionError("final Q32 threshold is not minimal")

    return q


def build_thresholds() -> list[int]:
    with localcontext() as context:
        context.prec = PRECISION
        target_kl = (TWO / ALPHA).ln() / ROWS
        table = [
            threshold_for_count(observed, target_kl)
            for observed in range(ROWS // 2)
        ]

    saw_sentinel = False
    previous = 0
    for index, value in enumerate(table):
        if value == SENTINEL:
            saw_sentinel = True
            continue
        if saw_sentinel:
            raise AssertionError(
                f"finite threshold after sentinel at count {index}"
            )
        if value < previous:
            raise AssertionError("threshold table must be monotone")
        previous = value

    return table


def table_text(table: list[int]) -> str:
    return "".join(f"{value}\n" for value in table)


def build_payload(table: list[int] | None = None) -> dict:
    if table is None:
        table = build_thresholds()
    encoded = table_text(table).encode("ascii")
    finite = [value for value in table if value != SENTINEL]
    first_sentinel = next(
        (index for index, value in enumerate(table) if value == SENTINEL),
        None,
    )

    if len(table) != ROWS // 2:
        raise AssertionError("table length")
    if len(finite) != 1853:
        raise AssertionError(
            f"expected 1853 finite thresholds, got {len(finite)}"
        )

    return {
        "format": "deltameter.strict-compact-q32.v1",
        "rows": ROWS,
        "stored_levels": LEVELS,
        "state_bytes": ROWS * LEVELS // 8,
        "delta": str(DELTA),
        "alpha_per_level": str(ALPHA),
        "q_bits": Q_BITS,
        "entries": len(table),
        "finite_entries": len(finite),
        "sentinel_entries": len(table) - len(finite),
        "first_sentinel_count": first_sentinel,
        "max_finite_q32": max(finite),
        "table_bytes": len(table) * 8,
        "table_sha256": hashlib.sha256(encoded).hexdigest(),
        "generator_precision_digits": PRECISION,
        "float_proposal_bisection_steps": FLOAT_BISECTION_STEPS,
        "proof_status": (
            "prototype conservative high-precision generator; "
            "independent interval/rational certification still required"
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--table-out", type=Path)
    parser.add_argument("--summary-out", type=Path)
    args = parser.parse_args()

    table = build_thresholds()
    summary = build_payload(table)

    if args.table_out is not None:
        args.table_out.parent.mkdir(parents=True, exist_ok=True)
        args.table_out.write_text(table_text(table), encoding="ascii")

    if args.summary_out is not None:
        args.summary_out.parent.mkdir(parents=True, exist_ok=True)
        args.summary_out.write_text(
            json.dumps(summary, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    if args.table_out is None and args.summary_out is None:
        print(json.dumps(summary, indent=2, sort_keys=True))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
