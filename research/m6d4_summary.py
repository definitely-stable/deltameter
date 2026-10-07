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


EXPECTED_META = {
    "format": "deltameter.m6d4-trace-square.v1",
    "contract": "frozen_m6d2_protocol_root_trace_square_only",
    "source_keys": "8192",
    "samples": "4",
    "square_repeats": "128",
    "schedule": "1:2;2:3;4:5;8:9",
    "payloads": "17;25;41;73",
}
HEADER = "kind,scenario,d,degree,outcome,operations,final_k,attempts,rtts,payload_bytes,generic_ns,specialized_ns,locator_ns,false_success".split(",")
SCENARIOS = {"d0": 0, "d1-zero": 1, "d2": 2, "d3": 3, "d4": 4,
             "d5": 5, "d8": 8, "d9": 9, "d10": 10, "d16": 16}


def validate_run(metadata, rows):
    if metadata != EXPECTED_META:
        raise SystemExit("benchmark metadata differs from frozen contract")
    counts = defaultdict(int)
    for row in rows:
        if any(row[field] < 0 for field in INT_FIELDS):
            raise SystemExit("negative measurement or count")
        if row["generic_ns"] <= 0 or row["specialized_ns"] <= 0:
            raise SystemExit("nonpositive timing")
        if row["false_success"] != 0:
            raise SystemExit("false-success observed")
        kind = row["kind"]
        if kind == "square":
            degree = row["degree"]
            if degree not in {2, 4, 8} or row["scenario"] != f"degree{degree}":
                raise SystemExit("invalid square case")
            if row["outcome"] != "na" or row["operations"] != 128:
                raise SystemExit("invalid square outcome/operations")
            if any(row[field] != 0 for field in ["d", "final_k", "attempts", "rtts", "payload_bytes", "locator_ns"]):
                raise SystemExit("invalid square logical fields")
        elif kind in {"decode", "total"}:
            if row["scenario"] not in SCENARIOS:
                raise SystemExit("unknown decode scenario")
            d = SCENARIOS[row["scenario"]]
            k, attempts, size = expected_stage(d)
            expected = {"d": d, "degree": 0, "operations": 1, "final_k": k,
                        "attempts": attempts, "rtts": attempts, "payload_bytes": size,
                        "outcome": "exact" if d <= 8 else "rejected"}
            if any(row[field] != value for field, value in expected.items()):
                raise SystemExit("decode contract changed")
            if kind == "decode" and row["locator_ns"] <= 0:
                raise SystemExit("nonpositive locator timing")
            if kind == "total" and row["locator_ns"] != 0:
                raise SystemExit("total locator field must be zero")
        else:
            raise SystemExit("unknown record kind")
        counts[(kind, row["scenario"])] += 1
    expected_counts = {(kind, name): 4 for kind in ("decode", "total") for name in SCENARIOS}
    expected_counts.update({("square", f"degree{degree}"): 4 for degree in (2, 4, 8)})
    if dict(counts) != expected_counts:
        raise SystemExit("incomplete or duplicated per-run matrix")


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
                if header is not None or line.split(",")[1:] != HEADER:
                    raise SystemExit(f"{path}: duplicate or invalid header")
                header = HEADER
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
                if key in metadata or header is not None:
                    raise SystemExit(f"{path}: duplicate or misplaced metadata")
                metadata[key] = value
                continue
            raise SystemExit(f"{path}: unknown line")

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


def paired_reductions(samples):
    reductions = [(1 - row["specialized_ns"] / row["generic_ns"]) * 100 for row in samples]
    per_run = [median(reductions[start:start + 4]) for start in range(0, len(reductions), 4)]
    return [f"{value:.3f}" for value in (median(reductions), min(per_run), max(per_run))]


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d4_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if [os.path.basename(path) for path in paths] != ["result-1.txt", "result-2.txt", "result-3.txt"]:
        raise SystemExit("exactly three complete named runs required")

    parsed = [parse(path) for path in paths]
    base_meta = parsed[0][0]
    rows = []
    for metadata, run_rows in parsed:
        validate_run(metadata, run_rows)
        rows.extend(run_rows)

    if any(row["false_success"] != 0 for row in rows):
        raise SystemExit("false-success observed")

    decode = defaultdict(list)
    square = defaultdict(list)
    total = defaultdict(list)
    for row in rows:
        if row["kind"] == "decode":
            decode[row["scenario"]].append(row)
        elif row["kind"] == "total":
            total[row["scenario"]].append(row)
        elif row["kind"] == "square":
            square[row["degree"]].append(row)
        else:
            raise SystemExit(f"unknown row kind {row['kind']}")

    expected_scenarios = SCENARIOS

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
            "paired_reduction_percent",
            "run_median_reduction_min_percent",
            "run_median_reduction_max_percent",
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
                *paired_reductions(samples),
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
            "generic_phase_sum_ns",
            "specialized_phase_sum_ns",
            "phase_sum_speedup_percent",
            "samples",
            "paired_reduction_percent",
            "run_median_reduction_min_percent",
            "run_median_reduction_max_percent",
        ]
    )
    for scenario, d in sorted(expected_scenarios.items(), key=lambda item: item[1]):
        samples = decode[scenario]
        first = samples[0]
        locator = median(row["locator_ns"] for row in samples)
        generic = median(row["generic_ns"] for row in samples)
        specialized = median(row["specialized_ns"] for row in samples)
        generic_total = median(row["locator_ns"] + row["generic_ns"] for row in samples)
        specialized_total = median(row["locator_ns"] + row["specialized_ns"] for row in samples)
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
                f"{generic_total:.1f}",
                f"{specialized_total:.1f}",
                f"{total_speedup:.3f}",
                len(samples),
                *paired_reductions(samples),
            ]
        )

    writer.writerow(["total", "scenario", "generic_total_ns", "specialized_total_ns",
                     "paired_reduction_percent", "run_median_reduction_min_percent",
                     "run_median_reduction_max_percent", "samples"])
    for scenario in SCENARIOS:
        samples = total[scenario]
        writer.writerow(["total", scenario, f"{median(row['generic_ns'] for row in samples):.1f}",
                         f"{median(row['specialized_ns'] for row in samples):.1f}",
                         *paired_reductions(samples), len(samples)])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
