#!/usr/bin/env python3
"""Aggregate durable STRICT-COMPACT-003 worker timing records."""
from __future__ import annotations

import argparse
import json
import math
import re
import statistics
from pathlib import Path

PATTERN = re.compile(
    r"^STRICT_COMPACT_KEYED_RESULT "
    r"tokens=(?P<tokens>\d+) rounds=(?P<rounds>\d+) "
    r"parity_ns=(?P<parity>[0-9.]+) "
    r"keyed_ns=(?P<keyed>[0-9.]+) "
    r"energy_ns=(?P<energy>[0-9.]+) "
    r"keyed_over_parity=(?P<ratio>[0-9.]+) "
    r"energy_over_keyed=(?P<energy_ratio>[0-9.]+)$"
)

WORKERS = 5
ABSOLUTE_GATE_NS = 500.0
RELATIVE_GATE = 10.0


def parse(path: Path) -> dict:
    text = path.read_text(encoding="utf-8").strip()
    match = PATTERN.fullmatch(text)
    if match is None:
        raise ValueError(f"malformed result: {path}")

    row = {
        "file": path.name,
        "tokens": int(match["tokens"]),
        "rounds": int(match["rounds"]),
        "parity_ns": float(match["parity"]),
        "keyed_ns": float(match["keyed"]),
        "energy_ns": float(match["energy"]),
        "keyed_over_parity": float(match["ratio"]),
        "energy_over_keyed": float(match["energy_ratio"]),
    }
    for key in (
        "parity_ns",
        "keyed_ns",
        "energy_ns",
        "keyed_over_parity",
        "energy_over_keyed",
    ):
        if not math.isfinite(row[key]) or row[key] <= 0:
            raise ValueError(f"invalid {key}: {path}")
    if row["tokens"] != 250_000 or row["rounds"] != 3:
        raise ValueError(f"unexpected protocol row: {path}")
    if row["keyed_ns"] > ABSOLUTE_GATE_NS:
        raise ValueError(f"absolute gate failed: {path}")
    if row["keyed_over_parity"] > RELATIVE_GATE:
        raise ValueError(f"relative gate failed: {path}")
    return row


def stats(rows: list[dict], key: str) -> dict:
    values = [row[key] for row in rows]
    return {
        "min": min(values),
        "median": statistics.median(values),
        "max": max(values),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    paths = sorted(args.input.rglob("worker-*.txt"))
    if len(paths) != WORKERS:
        raise SystemExit(f"expected {WORKERS} worker results, got {len(paths)}")

    rows = [parse(path) for path in paths]
    payload = {
        "format": "deltameter.strict-compact-keyed-oracle.v1",
        "decision": "GO_COMPUTATIONAL_STRICT_PRIVATE_API",
        "workers": WORKERS,
        "absolute_gate_ns": ABSOLUTE_GATE_NS,
        "relative_gate": RELATIVE_GATE,
        "rows": rows,
        "summary": {
            key: stats(rows, key)
            for key in (
                "parity_ns",
                "keyed_ns",
                "energy_ns",
                "keyed_over_parity",
                "energy_over_keyed",
            )
        },
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    s = payload["summary"]
    print(
        "STRICT_COMPACT_KEYED_AGGREGATE_PASS "
        f"keyed_ns={s['keyed_ns']['min']:.3f}/"
        f"{s['keyed_ns']['median']:.3f}/"
        f"{s['keyed_ns']['max']:.3f} "
        f"ratio={s['keyed_over_parity']['min']:.3f}/"
        f"{s['keyed_over_parity']['median']:.3f}/"
        f"{s['keyed_over_parity']['max']:.3f} "
        f"energy_over_keyed_median={s['energy_over_keyed']['median']:.3f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
