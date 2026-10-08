#!/usr/bin/env python3
"""Independent OPT-A J=51/52 exact-rational refinement; existing Q32 table unchanged."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from strict_compact_j_frontier import EXPECTED_TABLE_SHA256
from strict_compact_range_certify import (
    DECLARED_D_MIN, DOMAIN, build_prefix_cover, load_table,
)

def certify(table: list[int], j: int) -> dict:
    intervals, d_max, first_fail = build_prefix_cover(table, j)
    used = sorted({int(x["level"]) for x in intervals if x["kind"] == "exact_tail"})
    assert all(1 <= level <= j for level in used)
    assert d_max >= DECLARED_D_MIN
    assert first_fail is None or first_fail == d_max + 1
    # Strings are REQUIRED for all d-domain values (safe across JSON/JS).
    record = {
        "J": j,
        "state_bytes": 4096 * j // 8,
        "d_max": str(d_max),
        "first_uncertified_d": None if first_fail is None else str(first_fail),
        "intervals": len(intervals),
        "max_witness_level": max(used) if used else None,
        "covers_full_domain": d_max == DOMAIN - 1,
    }
    assert int(record["d_max"]) == d_max
    if first_fail is not None:
        assert int(record["first_uncertified_d"]) == first_fail
    return record

def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--table", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args()
    data = args.table.read_bytes()
    assert hashlib.sha256(data).hexdigest() == EXPECTED_TABLE_SHA256
    assert str(DOMAIN) == "18446744073709551616"
    assert str(DOMAIN - 1) == "18446744073709551615"
    table = load_table(args.table)
    rows = [certify(table, j) for j in (51, 52)]
    assert rows[1]["covers_full_domain"], "J52 accepted full-range witness did not reproduce"
    assert rows[1]["state_bytes"] == 26624
    payload = {
        "format": "deltameter.strict-compact-opt-a-refinement-certificate.v1",
        "table_sha256": EXPECTED_TABLE_SHA256,
        "declared_d_min": str(DECLARED_D_MIN),
        "domain_cardinality": str(DOMAIN),
        "profiles": rows,
        "decision": "J52_EXACT_CERT_PASS",
        "note": "A failed J51 search is not a proof of minimality or impossibility.",
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("STRICT_COMPACT_OPT_A_REFINE_CERT_PASS")
    for row in rows:
        print("REFINE_CANDIDATE J={J} state_bytes={state_bytes} d_max={d_max} full={covers_full_domain}".format(**row))

if __name__ == "__main__":
    main()
