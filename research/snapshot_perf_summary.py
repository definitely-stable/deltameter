#!/usr/bin/env python3
"""Summarize alternating base/head snapshot benchmark rounds.

The benchmark is diagnostic. This script intentionally reports raw paired
timing ratios without turning them into a pass/fail performance gate.
"""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


def read_samples(path: str):
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()

    try:
        header = lines.index("metric,case,content,bytes,sample,repeats,total_ns")
    except ValueError as error:
        raise SystemExit(f"{path}: benchmark CSV header is missing") from error

    for row in csv.DictReader(lines[header:]):
        repeats = int(row["repeats"])
        if repeats <= 0:
            raise SystemExit(f"{path}: repeats must be positive")
        yield (
            row["metric"],
            row["case"],
            row["content"],
            int(row["bytes"]),
            int(row["total_ns"]) / repeats,
        )


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: snapshot_perf_summary.py ARTIFACT_DIR")

    root = sys.argv[1]
    grouped = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))

    paths = sorted(glob.glob(os.path.join(root, "[0-9]*-base.txt")))
    if not paths:
        raise SystemExit("no paired benchmark rounds found")

    rounds = []
    for base_path in paths:
        name = os.path.basename(base_path)
        round_name = name.split("-", 1)[0]
        head_path = os.path.join(root, f"{round_name}-head.txt")
        if not os.path.exists(head_path):
            raise SystemExit(f"missing head result for round {round_name}")
        rounds.append(round_name)

        for variant, path in [("base", base_path), ("head", head_path)]:
            for metric, case, content, byte_count, ns_per_op in read_samples(path):
                key = (metric, case, content, byte_count)
                grouped[key][round_name][variant].append(ns_per_op)

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "metric",
            "case",
            "content",
            "bytes",
            "base_round_median_ns",
            "head_round_median_ns",
            "paired_ratio_median",
            "delta_percent",
            "paired_ratio_min",
            "paired_ratio_max",
            "rounds",
        ]
    )

    for key in sorted(grouped):
        round_map = grouped[key]
        ratios = []
        base_medians = []
        head_medians = []

        for round_name in rounds:
            variants = round_map.get(round_name)
            if variants is None or not variants["base"] or not variants["head"]:
                raise SystemExit(f"{key}: incomplete round {round_name}")
            base = statistics.median(variants["base"])
            head = statistics.median(variants["head"])
            if base <= 0:
                raise SystemExit(f"{key}: non-positive base timing")
            base_medians.append(base)
            head_medians.append(head)
            ratios.append(head / base)

        ratio = statistics.median(ratios)
        writer.writerow(
            [
                *key,
                f"{statistics.median(base_medians):.3f}",
                f"{statistics.median(head_medians):.3f}",
                f"{ratio:.6f}",
                f"{(ratio - 1.0) * 100.0:.3f}",
                f"{min(ratios):.6f}",
                f"{max(ratios):.6f}",
                len(ratios),
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
