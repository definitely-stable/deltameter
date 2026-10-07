#!/usr/bin/env python3
"""Summarize M6-D2 guarded incremental-prefix evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


LOGICAL_FIELDS = [
    "arm",
    "scenario",
    "d",
    "source_n",
    "outcome",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "false_success",
]


def parse(path: str):
    metadata = {}
    rows = []
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()

    header = None
    for line in lines:
        if not line:
            continue
        if line.startswith("result,arm,"):
            header = line.split(",")[1:]
            continue
        if line.startswith("result,"):
            if header is None:
                raise SystemExit(f"{path}: result header missing")
            values = line.split(",")[1:]
            if len(values) != len(header):
                raise SystemExit(f"{path}: malformed result row")
            row = dict(zip(header, values, strict=True))
            for field in [
                "d",
                "source_n",
                "final_k",
                "attempts",
                "rtts",
                "payload_bytes",
                "decode_ns",
                "extension_ns",
                "merge_ns",
                "cold_build_ns",
                "direct_diff_ns",
                "false_success",
            ]:
                row[field] = int(row[field])
            rows.append(row)
            continue
        if "=" in line:
            key, value = line.split("=", 1)
            metadata[key] = value

    if not rows:
        raise SystemExit(f"{path}: no result rows")
    return metadata, rows


def stage_for_d(d: int) -> int:
    if d <= 1:
        return 1
    if d <= 2:
        return 2
    if d <= 4:
        return 4
    return 8


def validate_logical(rows):
    by_key = defaultdict(list)
    for row in rows:
        by_key[(row["arm"], row["scenario"])].append(row)

    for (arm, scenario), samples in by_key.items():
        first = samples[0]
        for row in samples[1:]:
            for field in LOGICAL_FIELDS:
                if row[field] != first[field]:
                    raise SystemExit(
                        f"logical result changed for {(arm, scenario)} field {field}"
                    )

    scenarios = sorted({row["scenario"] for row in rows})
    for scenario in scenarios:
        sample = next(row for row in rows if row["scenario"] == scenario)
        d = sample["d"]
        fixed = by_key[("fixed-guarded", scenario)][0]
        incremental = by_key[("incremental-prefix", scenario)][0]
        naive = by_key[("naive-resend", scenario)][0]
        direct = by_key[("direct-exact", scenario)][0]

        if any(row["false_success"] != 0 for row in [fixed, incremental, naive, direct]):
            raise SystemExit(f"{scenario}: false-success observed")

        expected_outcome = "exact" if d <= 8 else "rejected"
        for row in [fixed, incremental, naive]:
            if row["outcome"] != expected_outcome:
                raise SystemExit(
                    f"{scenario}: {row['arm']} outcome {row['outcome']} != {expected_outcome}"
                )

        expected_k = stage_for_d(d)
        if fixed["final_k"] != expected_k or incremental["final_k"] != expected_k:
            raise SystemExit(f"{scenario}: unexpected final k")

        if incremental["payload_bytes"] != fixed["payload_bytes"]:
            raise SystemExit(
                f"{scenario}: incremental bytes must equal ideal fixed-known-k bytes"
            )

        if incremental["attempts"] > 1:
            if incremental["payload_bytes"] >= naive["payload_bytes"]:
                raise SystemExit(
                    f"{scenario}: nested prefix did not beat naive resend"
                )
        elif incremental["payload_bytes"] != naive["payload_bytes"]:
            raise SystemExit(
                f"{scenario}: one-attempt incremental and naive bytes differ"
            )

        if incremental["rtts"] != incremental["attempts"]:
            raise SystemExit(f"{scenario}: modeled RTT count must equal attempts")

        if direct["outcome"] != "exact":
            raise SystemExit(f"{scenario}: direct baseline must be exact")

    return by_key


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d2_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if not paths:
        raise SystemExit("no M6-D2 result files found")

    parsed = [parse(path) for path in paths]
    base_meta = parsed[0][0]
    all_rows = []
    for metadata, rows in parsed:
        if metadata != base_meta:
            raise SystemExit("benchmark metadata changed across runs")
        all_rows.extend(rows)

    by_key = validate_logical(all_rows)

    print("format=deltameter.m6d2-summary.v1")
    print(f"runs={len(paths)}")
    for key in [
        "contract",
        "source_keys",
        "samples",
        "direct_repeats",
        "schedule",
        "first_zero_metadata_bytes",
        "syndrome_word_bytes",
    ]:
        print(f"{key}={base_meta[key]}")
    print("logical_gate=PASS")
    print("false_success_total=0")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "summary",
            "arm",
            "scenario",
            "d",
            "outcome",
            "final_k",
            "attempts",
            "rtts",
            "payload_bytes",
            "median_decode_ns",
            "median_extension_ns",
            "median_merge_ns",
            "median_cold_build_ns",
            "median_direct_diff_ns",
        ]
    )

    for key in sorted(by_key):
        arm, scenario = key
        samples = by_key[key]
        first = samples[0]
        writer.writerow(
            [
                "summary",
                arm,
                scenario,
                first["d"],
                first["outcome"],
                first["final_k"],
                first["attempts"],
                first["rtts"],
                first["payload_bytes"],
                f"{statistics.median(row['decode_ns'] for row in samples):.0f}",
                f"{statistics.median(row['extension_ns'] for row in samples):.0f}",
                f"{statistics.median(row['merge_ns'] for row in samples):.0f}",
                f"{statistics.median(row['cold_build_ns'] for row in samples):.0f}",
                f"{statistics.median(row['direct_diff_ns'] for row in samples):.0f}",
            ]
        )

    print("bytes,scenario,d,direct,fixed,incremental,naive,retry_savings_percent")
    scenarios = sorted(
        {row["scenario"] for row in all_rows},
        key=lambda name: next(row["d"] for row in all_rows if row["scenario"] == name),
    )
    for scenario in scenarios:
        d = by_key[("direct-exact", scenario)][0]["d"]
        direct = by_key[("direct-exact", scenario)][0]["payload_bytes"]
        fixed = by_key[("fixed-guarded", scenario)][0]["payload_bytes"]
        incremental = by_key[("incremental-prefix", scenario)][0]["payload_bytes"]
        naive = by_key[("naive-resend", scenario)][0]["payload_bytes"]
        savings = 0.0 if naive == 0 else (1.0 - incremental / naive) * 100.0
        print(
            f"bytes,{scenario},{d},{direct},{fixed},{incremental},{naive},{savings:.3f}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
