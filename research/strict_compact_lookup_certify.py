#!/usr/bin/env python3
"""Independent exact-rational certificate for the STRICT-COMPACT Q32 table.

The generator uses Decimal exp/log/KL. This verifier deliberately does not.

For observed s and normalized threshold y=q/2^32, let

    z = y/m
    p = (1-exp(-z))/2.

The KL crossing condition

    2 exp(-m D(s/m || p)) <= alpha

is equivalent to the exact likelihood-ratio inequality

    exp(m D) >= 2/alpha.

For x=s/m and rational p=P/Q,

    exp(mD) =
      [s/(m p)]^s * [(m-s)/(m(1-p))]^(m-s),

so the comparison can be performed with integers only.

To obtain a certified lower bound on the transcendental p(q), use

    e^z >= sum_(k=0)^n z^k/k!

which implies

    e^-z <= 1 / S_n(z)
    p(q) >= (1 - 1/S_n(z))/2.

That lower bound is rounded DOWN to Q64. Because D(x||p) is increasing in p for
p>x, proving the KL crossing at this smaller rational p proves that the committed
Q32 threshold is conservative.

Infinity sentinels are certified independently at the limiting p=1/2.

This verifier therefore has no exp/log/Decimal/binary64 dependency in its proof
path. Fraction arithmetic is exact Python integer arithmetic.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path

ROWS = 4096
LEVELS = 64
Q32 = 1 << 32
P_Q_BITS = 64
P_Q = 1 << P_Q_BITS
SENTINEL = (1 << 64) - 1
TAYLOR_DEGREE = 64
DELTA_DENOMINATOR = 1_000_000
# alpha = 1/(DELTA_DENOMINATOR * LEVELS)
# 2/alpha = 2 * DELTA_DENOMINATOR * LEVELS
LIKELIHOOD_TARGET = 2 * DELTA_DENOMINATOR * LEVELS


def load_table(path: Path) -> list[int]:
    values = [
        int(line)
        for line in path.read_text(encoding="ascii").splitlines()
        if line
    ]
    if len(values) != ROWS // 2:
        raise ValueError(f"expected {ROWS // 2} entries, got {len(values)}")
    return values


def exp_positive_bounds(
    z: Fraction, degree: int = TAYLOR_DEGREE
) -> tuple[Fraction, Fraction]:
    """Exact lower/upper bounds for exp(z), z>=0.

    Lower: finite Taylor sum S_n(z).

    Upper: after the first omitted term, every later ratio is at most
    z/(n+2). When this is <1, the omitted tail is bounded by a geometric
    series.
    """
    if z < 0:
        raise ValueError("z")
    if degree < 0:
        raise ValueError("degree")

    total = Fraction(1)
    term = Fraction(1)
    for k in range(1, degree + 1):
        term *= z
        term /= k
        total += term

    next_term = term * z / (degree + 1)
    ratio = z / (degree + 2)
    if ratio >= 1:
        raise ValueError("Taylor degree too small for exact upper bound")

    upper = total + next_term / (1 - ratio)
    return total, upper


def exp_positive_lower(z: Fraction, degree: int = TAYLOR_DEGREE) -> Fraction:
    return exp_positive_bounds(z, degree)[0]


def p_lower_q64_from_z(z: Fraction) -> int:
    """Certified Q64 lower bound on p=(1-exp(-z))/2."""
    exp_lower, _exp_upper = exp_positive_bounds(z)
    p_lower = (Fraction(1) - Fraction(1, 1) / exp_lower) / 2
    if p_lower <= 0:
        raise AssertionError("non-positive p lower bound")

    quantized = (p_lower.numerator * P_Q) // p_lower.denominator
    if not (0 < quantized < P_Q // 2):
        raise AssertionError("invalid Q64 p lower bound")
    return quantized


def p_upper_q64_from_z(z: Fraction) -> int:
    """Certified Q64 upper bound on p=(1-exp(-z))/2."""
    _exp_lower, exp_upper = exp_positive_bounds(z)
    p_upper = (Fraction(1) - Fraction(1, 1) / exp_upper) / 2
    if not (0 < p_upper < Fraction(1, 2)):
        raise AssertionError("invalid p upper bound")

    numerator = p_upper.numerator * P_Q
    quantized = (numerator + p_upper.denominator - 1) // p_upper.denominator
    if not (0 < quantized <= P_Q // 2):
        raise AssertionError("invalid Q64 p upper bound")
    return quantized


def p_lower_q64(q32: int) -> int:
    """Certified Q64 lower bound on p=(1-exp(-q32/(2^32*m)))/2."""
    if not (0 < q32 < SENTINEL):
        raise ValueError("finite positive q32 required")
    return p_lower_q64_from_z(Fraction(q32, Q32 * ROWS))


def likelihood_crosses_target(
    observed: int, p_num: int, target: int
) -> bool:
    """Exact test exp(m D(s/m || p_num/2^64)) >= target."""
    if not (0 <= observed < ROWS // 2):
        raise ValueError("observed")
    if not (0 < p_num < P_Q):
        raise ValueError("p")

    # The monotone argument requires p>x.
    if p_num * ROWS <= observed * P_Q:
        return False

    # exp(mD) =
    # s^s (m-s)^(m-s) Q^m / [m^m P^s (Q-P)^(m-s)].
    #
    # Here Q=2^64 and m=4096=2^12, so Q^m/m^m = 2^(52m).
    left_core = pow(observed, observed) * pow(
        ROWS - observed, ROWS - observed
    )
    left = left_core << ((P_Q_BITS - 12) * ROWS)

    if target <= 0:
        raise ValueError("target")
    right = (
        target
        * pow(p_num, observed)
        * pow(P_Q - p_num, ROWS - observed)
    )
    return left >= right


def likelihood_crosses(observed: int, p_num: int) -> bool:
    """Exact test exp(m D(s/m || p_num/2^64)) >= 2/alpha."""
    return likelihood_crosses_target(observed, p_num, LIKELIHOOD_TARGET)


def sentinel_is_valid(observed: int) -> bool:
    """At p->1/2 the KL divergence is maximal for fixed s<m/2.

    If even p=1/2 does not cross the target, no finite d can cross it.
    """
    p_half = P_Q // 2
    # Sentinel is valid when exp(mD(x||1/2)) <= 2/alpha.
    # likelihood_crosses uses >=, so compare directly and allow exact equality:
    left_core = pow(observed, observed) * pow(
        ROWS - observed, ROWS - observed
    )
    left = left_core << ((P_Q_BITS - 12) * ROWS)
    right = (
        LIKELIHOOD_TARGET
        * pow(p_half, observed)
        * pow(P_Q - p_half, ROWS - observed)
    )
    return left <= right


def certify(table: list[int]) -> dict:
    finite = 0
    sentinels = 0
    minimum_p_margin_numerator: int | None = None

    saw_sentinel = False
    previous = 0
    for observed, q32 in enumerate(table):
        if q32 == SENTINEL:
            saw_sentinel = True
            sentinels += 1
            if not sentinel_is_valid(observed):
                raise AssertionError(
                    f"invalid infinity sentinel at observed={observed}"
                )
            continue

        if saw_sentinel:
            raise AssertionError("finite threshold after sentinel")
        if q32 < previous:
            raise AssertionError("non-monotone Q32 table")
        previous = q32

        p_num = p_lower_q64(q32)
        margin = p_num * ROWS - observed * P_Q
        if margin <= 0:
            raise AssertionError(
                f"certified p lower bound did not exceed x at observed={observed}"
            )
        if minimum_p_margin_numerator is None or margin < minimum_p_margin_numerator:
            minimum_p_margin_numerator = margin

        if not likelihood_crosses(observed, p_num):
            raise AssertionError(
                f"Q32 threshold not independently certified at observed={observed}"
            )
        finite += 1

    if finite != 1853 or sentinels != 195:
        raise AssertionError(
            f"unexpected table split finite={finite} sentinel={sentinels}"
        )

    return {
        "format": "deltameter.strict-compact-q32-certificate.v1",
        "rows": ROWS,
        "stored_levels": LEVELS,
        "delta": "0.000001",
        "alpha_per_level": "0.000000015625",
        "q32_entries": len(table),
        "finite_entries": finite,
        "sentinel_entries": sentinels,
        "p_interval_q_bits": P_Q_BITS,
        "exp_lower_taylor_degree": TAYLOR_DEGREE,
        "likelihood_target_2_over_alpha": LIKELIHOOD_TARGET,
        "minimum_certified_p_minus_x_q64_numerator": minimum_p_margin_numerator,
        "certificate": (
            "exact rational/integer proof that every finite Q32 threshold is "
            "conservative and every infinity sentinel has no finite crossing"
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--table", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()

    raw = args.table.read_bytes()
    table = load_table(args.table)
    payload = certify(table)
    payload["table_sha256"] = hashlib.sha256(raw).hexdigest()

    encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if args.out is None:
        print(encoded, end="")
    else:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(encoded, encoding="utf-8")
        print(
            "STRICT_COMPACT_Q32_CERT_PASS "
            f"finite={payload['finite_entries']} "
            f"sentinels={payload['sentinel_entries']} "
            f"sha256={payload['table_sha256']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
