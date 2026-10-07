#!/usr/bin/env python3
"""Summarize M6-D1 PinSketch64 lab benchmark evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


HEADER = (
    "metric,stored_capacity,max_elements,d,operations,sample,total_ns,"
    "sketch_bytes,direct_set_payload_bytes"
)


def parse(path: str):
    metadata = {}
    rows = []
    inventory = {}

    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()

    header_index = None
    for index, line in enumerate(lines):
        if line == HEADER:
            header_index = index
            break
        if "=" in line and not line.startswith("inventory,"):
            key, value = line.split("=", 1)
            metadata[key] = value

    if header_index is None:
        raise SystemExit(f"{path}: benchmark header missing")

    csv_lines = []
    for line in lines[header_index:]:
        if line.startswith("inventory,"):
            parts = line.split(",")[1:]
            values = {}
            for part in parts:
                key, value = part.split("=", 1)
                values[key] = value
            key = (
                values["mode"],
                int(values["stored_capacity"]),
                int(values["max_elements"]),
                int(values["d"]),
            )
            inventory[key] = (
                int(values["trials"]),
                int(values["rejected"]),
                int(values["false_success"]),
            )
        elif line:
            csv_lines.append(line)

    for row in csv.DictReader(csv_lines):
        operations = int(row["operations"])
        if operations <= 0:
            raise SystemExit(f"{path}: non-positive operations")
        rows.append(
            {
                **row,
                "stored_capacity": int(row["stored_capacity"]),
                "max_elements": int(row["max_elements"]),
                "d": int(row["d"]),
                "operations": operations,
                "sample": int(row["sample"]),
                "total_ns": int(row["total_ns"]),
                "sketch_bytes": int(row["sketch_bytes"]),
                "direct_set_payload_bytes": int(row["direct_set_payload_bytes"]),
            }
        )

    return metadata, rows, inventory


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d1_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if not paths:
        raise SystemExit("no M6-D1 result files found")

    parsed = [parse(path) for path in paths]
    base_meta, _, base_inventory = parsed[0]

    grouped = defaultdict(list)
    first_rows = {}
    for metadata, rows, inventory in parsed:
        if metadata != base_meta:
            raise SystemExit("benchmark metadata changed across runs")
        if inventory != base_inventory:
            raise SystemExit("over-capacity inventory changed across runs")

        for row in rows:
            key = (
                row["metric"],
                row["stored_capacity"],
                row["max_elements"],
                row["d"],
            )
            grouped[key].append(row["total_ns"] / row["operations"])
            first_rows.setdefault(key, row)

    print("format=deltameter.m6d1-summary.v2")
    print(f"runs={len(paths)}")
    for key in [
        "contract",
        "source_keys",
        "build_samples",
        "merge_repeats",
        "decode_samples",
        "over_capacity_trials",
        "guard_field_bits",
        "guard_extra_syndromes",
    ]:
        print(f"{key}={base_meta[key]}")

    mode_totals = defaultdict(lambda: [0, 0, 0])
    for (mode, _, _, _), (trials, rejected, false_success) in base_inventory.items():
        totals = mode_totals[mode]
        totals[0] += trials
        totals[1] += rejected
        totals[2] += false_success

    for mode in sorted(mode_totals):
        trials, rejected, false_success = mode_totals[mode]
        print(f"{mode}_over_capacity_total_trials={trials}")
        print(f"{mode}_over_capacity_rejected={rejected}")
        print(f"{mode}_over_capacity_false_success={false_success}")

    print("inventory,mode,stored_capacity,max_elements,d,trials,rejected,false_success")
    for (mode, stored_capacity, max_elements, d), (
        trials,
        rejected,
        false_success,
    ) in sorted(base_inventory.items()):
        print(
            f"inventory,{mode},{stored_capacity},{max_elements},{d},"
            f"{trials},{rejected},{false_success}"
        )

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "summary",
            "metric",
            "stored_capacity",
            "max_elements",
            "d",
            "median_ns_per_op",
            "min_ns_per_op",
            "max_ns_per_op",
            "samples",
            "sketch_bytes",
            "direct_set_payload_bytes",
            "payload_ratio",
        ]
    )

    for key in sorted(grouped):
        metric, stored_capacity, max_elements, d = key
        values = grouped[key]
        row = first_rows[key]
        payload_ratio = row["sketch_bytes"] / row["direct_set_payload_bytes"]
        writer.writerow(
            [
                "summary",
                metric,
                stored_capacity,
                max_elements,
                d,
                f"{statistics.median(values):.3f}",
                f"{min(values):.3f}",
                f"{max(values):.3f}",
                len(values),
                row["sketch_bytes"],
                row["direct_set_payload_bytes"],
                f"{payload_ratio:.9f}",
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
