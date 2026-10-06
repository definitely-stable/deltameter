#!/usr/bin/env python3
"""Summarize paired M6-C Energy performance rounds."""

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
        header = lines.index("metric,profile,content,operations,sample,total_ns")
    except ValueError as error:
        raise SystemExit(f"{path}: benchmark CSV header missing") from error

    for row in csv.DictReader(lines[header:]):
        if row["metric"] not in {"construct", "update", "decode", "difference", "query"}:
            continue
        operations = int(row["operations"])
        if operations <= 0:
            raise SystemExit(f"{path}: operations must be positive")
        yield (
            row["metric"],
            row["profile"],
            row["content"],
            int(row["total_ns"]) / operations,
        )


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6c_perf_summary.py ARTIFACT_DIR")

    root = sys.argv[1]
    base_paths = sorted(glob.glob(os.path.join(root, "[0-9]*-base.txt")))
    if not base_paths:
        raise SystemExit("no paired M6-C rounds found")

    grouped = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))
    rounds = []

    for base_path in base_paths:
        round_name = os.path.basename(base_path).split("-", 1)[0]
        head_path = os.path.join(root, f"{round_name}-head.txt")
        if not os.path.exists(head_path):
            raise SystemExit(f"missing head result for round {round_name}")
        rounds.append(round_name)

        for variant, path in [("base", base_path), ("head", head_path)]:
            for metric, profile, content, ns_per_op in read_samples(path):
                grouped[(metric, profile, content)][round_name][variant].append(ns_per_op)

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "metric",
            "profile",
            "content",
            "base_median_ns_per_op",
            "head_median_ns_per_op",
            "paired_ratio_median",
            "delta_percent",
            "paired_ratio_min",
            "paired_ratio_max",
            "rounds",
        ]
    )

    for key in sorted(grouped):
        ratios = []
        base_values = []
        head_values = []

        for round_name in rounds:
            variants = grouped[key].get(round_name)
            if variants is None or not variants["base"] or not variants["head"]:
                raise SystemExit(f"{key}: incomplete round {round_name}")
            base = statistics.median(variants["base"])
            head = statistics.median(variants["head"])
            if base <= 0:
                raise SystemExit(f"{key}: non-positive base timing")
            base_values.append(base)
            head_values.append(head)
            ratios.append(head / base)

        ratio = statistics.median(ratios)
        writer.writerow(
            [
                *key,
                f"{statistics.median(base_values):.3f}",
                f"{statistics.median(head_values):.3f}",
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
