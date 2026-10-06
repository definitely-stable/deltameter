#!/usr/bin/env python3
"""Summarize M6-D1 PinSketch64 lab benchmark evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


def parse(path: str):
    metadata = {}
    rows = []
    inventory = {}

    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()

    header_index = None
    for index, line in enumerate(lines):
        if line == "metric,capacity,d,operations,sample,total_ns,sketch_bytes,direct_set_payload_bytes":
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
                values[key] = int(value)
            key = (values["capacity"], values["d"])
            inventory[key] = (
                values["trials"],
                values["rejected"],
                values["false_success"],
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
                "capacity": int(row["capacity"]),
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
    for metadata, rows, inventory in parsed:
        if metadata != base_meta:
            raise SystemExit("benchmark metadata changed across runs")
        if inventory != base_inventory:
            raise SystemExit("over-capacity inventory changed across runs")

        for row in rows:
            key = (row["metric"], row["capacity"], row["d"])
            grouped[key].append(row["total_ns"] / row["operations"])

    print("format=deltameter.m6d1-summary.v1")
    print(f"runs={len(paths)}")
    for key in [
        "contract",
        "source_keys",
        "build_samples",
        "merge_repeats",
        "decode_samples",
        "over_capacity_trials",
    ]:
        print(f"{key}={base_meta[key]}")

    total_trials = sum(item[0] for item in base_inventory.values())
    total_rejected = sum(item[1] for item in base_inventory.values())
    total_false_success = sum(item[2] for item in base_inventory.values())
    print(f"over_capacity_total_trials={total_trials}")
    print(f"over_capacity_rejected={total_rejected}")
    print(f"over_capacity_false_success={total_false_success}")

    print("inventory,capacity,d,trials,rejected,false_success")
    for (capacity, d), (trials, rejected, false_success) in sorted(base_inventory.items()):
        print(f"inventory,{capacity},{d},{trials},{rejected},{false_success}")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "summary",
            "metric",
            "capacity",
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

    first_rows = {}
    for _, rows, _ in parsed:
        for row in rows:
            first_rows.setdefault((row["metric"], row["capacity"], row["d"]), row)

    for key in sorted(grouped):
        metric, capacity, d = key
        values = grouped[key]
        row = first_rows[key]
        payload_ratio = row["sketch_bytes"] / row["direct_set_payload_bytes"]
        writer.writerow(
            [
                "summary",
                metric,
                capacity,
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
