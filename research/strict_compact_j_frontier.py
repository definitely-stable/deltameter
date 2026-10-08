#!/usr/bin/env python3
"""Exact STRICT-COMPACT OPT-A state/range frontier over stored level count J.

This reuses the already independently certified Q32 table generated with
alpha=delta/64. For J<=64 the used familywise alpha mass only decreases.

For each J, construct the largest hole-free prefix [4096,d_max] for which the
existing exact-rational q95 U/d<=1.5 certificate closes using levels 1..=J.

Binary64 remains proposal-ranking only inside strict_compact_range_certify.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

from strict_compact_lookup_certify import ROWS
from strict_compact_range_certify import (
    DECLARED_D_MIN,
    DOMAIN,
    build_prefix_cover,
    load_table,
)

J_VALUES = (24, 32, 40, 48, 56, 64)
EXPECTED_TABLE_SHA256 = (
    "634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a"
)


def summarize(table: list[int], levels: int) -> dict:
    intervals, d_max, first_failure = build_prefix_cover(table, levels)
    exact = [row for row in intervals if row["kind"] == "exact_tail"]
    used = sorted({row["level"] for row in exact})

    state_bits = ROWS * levels
    if state_bits % 64 != 0:
        raise AssertionError("frozen J matrix must pack to whole u64 words")

    return {
        "stored_levels": levels,
        "state_bits": state_bits,
        "state_words": state_bits // 64,
        "state_bytes": state_bits // 8,
        "d_min": DECLARED_D_MIN,
        "d_max": d_max,
        "first_uncertified_d": first_failure,
        "covers_full_u64_domain": d_max == DOMAIN - 1,
        "interval_count": len(intervals),
        "exact_tail_interval_count": len(exact),
        "levels_used": used,
        "max_level_used": max(used) if used else None,
        "decision": (
            "FULL_RANGE"
            if d_max == DOMAIN - 1
            else "CERTIFIED_PREFIX"
            if d_max >= DECLARED_D_MIN
            else "NO_USEFUL_PREFIX"
        ),
    }


def build_payload(table_path: Path) -> dict:
    raw = table_path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != EXPECTED_TABLE_SHA256:
        raise AssertionError(
            f"unexpected Q32 table sha256 {digest}; expected {EXPECTED_TABLE_SHA256}"
        )

    table = load_table(table_path)
    workers = max(1, min(len(J_VALUES), os.cpu_count() or 1))
    with ProcessPoolExecutor(max_workers=workers) as executor:
        rows = list(executor.map(summarize, [table] * len(J_VALUES), J_VALUES))

    # Regression anchor. The independent legacy range workflow recomputes the full
    # J=64 certificate on every OPT-A head. Here we additionally require the
    # already accepted decomposition summary from STRICT-COMPACT-002.
    full = rows[-1]
    if full["stored_levels"] != 64 or not full["covers_full_u64_domain"]:
        raise AssertionError("J=64 no longer reproduces full-domain certificate")
    if full["interval_count"] != 876 or full["exact_tail_interval_count"] != 875:
        raise AssertionError("J=64 accepted interval decomposition changed")
    if full["levels_used"] != list(range(1, 53)):
        raise AssertionError("J=64 accepted certifying-level set changed")

    previous_d_max = -1
    previous_bytes = -1
    for row in rows:
        if row["state_bytes"] <= previous_bytes:
            raise AssertionError("state bytes must increase with J")
        if row["d_max"] < previous_d_max:
            raise AssertionError("certified prefix unexpectedly shrank as J increased")
        previous_bytes = row["state_bytes"]
        previous_d_max = row["d_max"]

    return {
        "format": "deltameter.strict-compact-opt-a-j-frontier.v1",
        "rows": ROWS,
        "delta": "1e-6",
        "per_level_alpha_source": "certified table at delta/64",
        "q95_width_ratio": "1.5",
        "declared_d_min": DECLARED_D_MIN,
        "domain_cardinality": DOMAIN,
        "table_sha256": digest,
        "profiles": rows,
        "decision": "J_FRONTIER_CERT_PASS",
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

    for row in payload["profiles"]:
        print(
            "STRICT_COMPACT_OPT_A_J "
            f"J={row['stored_levels']} "
            f"state_bytes={row['state_bytes']} "
            f"d_max={row['d_max']} "
            f"first_fail={row['first_uncertified_d']} "
            f"intervals={row['interval_count']} "
            f"max_level_used={row['max_level_used']}"
        )
    print("STRICT_COMPACT_OPT_A_J_FRONTIER_PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
