#!/usr/bin/env python3
"""Summarize M6-D4 characteristic-2 trace-square evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


INT_FIELDS = {
    "d",
    "degree",
    "operations",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "generic_ns",
    "specialized_ns",
    "locator_ns",
    "false_success",
}


def parse(path: str):
    metadata = {}
    rows = []
    header = None

    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            if not line:
                continue
            if line.startswith("record,kind,"):
                header = line.split(",")[1:]
                continue
            if line.startswith("record,"):
                if header is None:
                    raise SystemExit(f"{path}: result header missing")
                values = line.split(",")[1:]
                if len(values) != len(header):
                    raise SystemExit(f"{path}: malformed row")
                row = dict(zip(header, values, strict=True))
                for field in INT_FIELDS:
                    row[field] = int(row[field])
                rows.append(row)
                continue
            if "=" in line:
                key, value = line.split("=", 1)
                metadata[key] = value

    if not rows:
        raise SystemExit(f"{path}: no result rows")
    return metadata, rows


def expected_stage(d: int) -> tuple[int, int, int]:
    if d <= 1:
        return 1, 1, 17
    if d <= 2:
        return 2, 2, 25
    if d <= 4:
        return 4, 3, 41
    return 8, 4, 73


def median(values):
    return statistics.median(values)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d4_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if not paths:
        raise SystemExit("no M6-D4 result files found")

    parsed = [parse(path) for path in paths]
    base_meta = parsed[0][0]
    rows = []
    for metadata, run_rows in parsed:
        if metadata != base_meta:
            raise SystemExit("benchmark metadata changed across runs")
        rows.extend(run_rows)

    if any(row["false_success"] != 0 for row in rows):
        raise SystemExit("false-success observed")

    decode = defaultdict(list)
    square = defaultdict(list)
    for row in rows:
        if row["kind"] == "decode":
            decode[row["scenario"]].append(row)
        elif row["kind"] == "square":
            square[row["degree"]].append(row)
        else:
            raise SystemExit(f"unknown row kind {row['kind']}")

    expected_scenarios = {
        "d0": 0,
        "d1-zero": 1,
        "d2": 2,
        "d3": 3,
        "d4": 4,
        "d5": 5,
        "d8": 8,
        "d9": 9,
        "d10": 10,
        "d16": 16,
    }
    if set(decode) != set(expected_scenarios):
        raise SystemExit("decode scenario matrix changed")
    if set(square) != {2, 4, 8}:
        raise SystemExit("square degree matrix changed")

    for scenario, d in expected_scenarios.items():
        samples = decode[scenario]
        expected_k, expected_attempts, expected_bytes = expected_stage(d)
        expected_outcome = "exact" if d <= 8 else "rejected"
        for row in samples:
            if row["d"] != d:
                raise SystemExit(f"{scenario}: d changed")
            if row["outcome"] != expected_outcome:
                raise SystemExit(f"{scenario}: outcome {row['outcome']} != {expected_outcome}")
            if row["final_k"] != expected_k:
                raise SystemExit(f"{scenario}: final k changed")
            if row["attempts"] != expected_attempts or row["rtts"] != expected_attempts:
                raise SystemExit(f"{scenario}: attempts/RTTs changed")
            if row["payload_bytes"] != expected_bytes:
                raise SystemExit(f"{scenario}: D2 payload bytes changed")
            if row["operations"] != 1:
                raise SystemExit(f"{scenario}: decode operations must be one")

    print("format=deltameter.m6d4-summary.v1")
    print(f"runs={len(paths)}")
    for key in [
        "contract",
        "source_keys",
        "samples",
        "square_repeats",
        "schedule",
        "payloads",
    ]:
        print(f"{key}={base_meta[key]}")
    print("logical_gate=PASS")
    print("false_success_total=0")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "square",
            "degree",
            "generic_ns_per_op",
            "specialized_ns_per_op",
            "speedup_percent",
            "samples",
        ]
    )
    for degree in sorted(square):
        samples = square[degree]
        generic = median(row["generic_ns"] / row["operations"] for row in samples)
        specialized = median(
            row["specialized_ns"] / row["operations"] for row in samples
        )
        speedup = 0.0 if generic == 0 else (1.0 - specialized / generic) * 100.0
        writer.writerow(
            [
                "square",
                degree,
                f"{generic:.3f}",
                f"{specialized:.3f}",
                f"{speedup:.3f}",
                len(samples),
            ]
        )

    writer.writerow(
        [
            "decode",
            "scenario",
            "d",
            "outcome",
            "final_k",
            "attempts",
            "payload_bytes",
            "locator_ns",
            "generic_root_verify_ns",
            "specialized_root_verify_ns",
            "root_speedup_percent",
            "generic_total_ns",
            "specialized_total_ns",
            "total_speedup_percent",
            "samples",
        ]
    )
    for scenario, d in sorted(expected_scenarios.items(), key=lambda item: item[1]):
        samples = decode[scenario]
        first = samples[0]
        locator = median(row["locator_ns"] for row in samples)
        generic = median(row["generic_ns"] for row in samples)
        specialized = median(row["specialized_ns"] for row in samples)
        generic_total = locator + generic
        specialized_total = locator + specialized
        root_speedup = 0.0 if generic == 0 else (1.0 - specialized / generic) * 100.0
        total_speedup = (
            0.0
            if generic_total == 0
            else (1.0 - specialized_total / generic_total) * 100.0
        )
        writer.writerow(
            [
                "decode",
                scenario,
                d,
                first["outcome"],
                first["final_k"],
                first["attempts"],
                first["payload_bytes"],
                f"{locator:.0f}",
                f"{generic:.0f}",
                f"{specialized:.0f}",
                f"{root_speedup:.3f}",
                f"{generic_total:.0f}",
                f"{specialized_total:.0f}",
                f"{total_speedup:.3f}",
                len(samples),
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
