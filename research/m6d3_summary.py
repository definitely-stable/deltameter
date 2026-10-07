#!/usr/bin/env python3
"""Summarize M6-D3 incremental Berlekamp-Massey evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict


INT_FIELDS = {
    "d",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "merge_ns",
    "reference_decode_ns",
    "incremental_bm_ns",
    "factor_verify_ns",
    "fresh_sequence_ns",
    "fresh_bm_ns",
    "candidate_decode_ns",
    "cold_build_ns",
    "false_success",
}

LOGICAL_FIELDS = [
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
    header = None

    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
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


def expected_stage(d: int) -> tuple[int, int]:
    if d <= 1:
        return 1, 17
    if d <= 2:
        return 2, 25
    if d <= 4:
        return 4, 41
    return 8, 73


def validate(rows):
    grouped = defaultdict(list)
    for row in rows:
        grouped[(row["arm"], row["scenario"])].append(row)

    scenarios = sorted({row["scenario"] for row in rows})
    arms = {"fixed-reference", "incremental-reference", "incremental-bm"}
    for scenario in scenarios:
        present = {arm for arm, name in grouped if name == scenario}
        if present != arms:
            raise SystemExit(f"{scenario}: missing arms {arms - present}")

    for key, samples in grouped.items():
        first = samples[0]
        for row in samples[1:]:
            for field in LOGICAL_FIELDS:
                if row[field] != first[field]:
                    raise SystemExit(f"{key}: logical field {field} changed across runs")

    for scenario in scenarios:
        ref = grouped[("incremental-reference", scenario)][0]
        candidate = grouped[("incremental-bm", scenario)][0]
        fixed = grouped[("fixed-reference", scenario)][0]

        if any(
            row["false_success"] != 0
            for row in (ref, candidate, fixed)
        ):
            raise SystemExit(f"{scenario}: false-success observed")

        for field in ["outcome", "final_k", "attempts", "rtts", "payload_bytes"]:
            if candidate[field] != ref[field]:
                raise SystemExit(
                    f"{scenario}: candidate changed frozen D2 field {field}"
                )

        expected_k, expected_bytes = expected_stage(ref["d"])
        expected_outcome = "exact" if ref["d"] <= 8 else "rejected"
        if ref["final_k"] != expected_k or ref["payload_bytes"] != expected_bytes:
            raise SystemExit(f"{scenario}: frozen D2 stage/bytes changed")
        if ref["outcome"] != expected_outcome or candidate["outcome"] != expected_outcome:
            raise SystemExit(f"{scenario}: unexpected final outcome")

        expected_attempts = {1: 1, 2: 2, 4: 3, 8: 4}[expected_k]
        if ref["attempts"] != expected_attempts or ref["rtts"] != expected_attempts:
            raise SystemExit(f"{scenario}: frozen D2 attempts/RTTs changed")

    return grouped


def median(samples, field):
    return statistics.median(row[field] for row in samples)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d3_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if not paths:
        raise SystemExit("no M6-D3 result files found")

    parsed = [parse(path) for path in paths]
    base_meta = parsed[0][0]
    rows = []
    for metadata, run_rows in parsed:
        if metadata != base_meta:
            raise SystemExit("benchmark metadata changed across runs")
        rows.extend(run_rows)

    grouped = validate(rows)

    print("format=deltameter.m6d3-summary.v1")
    print(f"runs={len(paths)}")
    for key in ["contract", "source_keys", "samples", "schedule", "payloads"]:
        print(f"{key}={base_meta[key]}")
    print("logical_gate=PASS")
    print("false_success_total=0")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "summary",
            "scenario",
            "d",
            "outcome",
            "final_k",
            "attempts",
            "payload_bytes",
            "fixed_reference_decode_ns",
            "incremental_reference_decode_ns",
            "candidate_decode_ns",
            "candidate_incremental_bm_ns",
            "candidate_factor_verify_ns",
            "reference_fresh_sequence_ns",
            "reference_fresh_bm_ns",
            "candidate_vs_reference_percent",
            "bm_fraction_of_candidate_percent",
        ]
    )

    scenario_names = sorted(
        {row["scenario"] for row in rows},
        key=lambda name: grouped[("incremental-reference", name)][0]["d"],
    )

    for scenario in scenario_names:
        fixed_samples = grouped[("fixed-reference", scenario)]
        ref_samples = grouped[("incremental-reference", scenario)]
        candidate_samples = grouped[("incremental-bm", scenario)]
        first = ref_samples[0]

        fixed_ns = median(fixed_samples, "reference_decode_ns")
        ref_ns = median(ref_samples, "reference_decode_ns")
        candidate_ns = median(candidate_samples, "candidate_decode_ns")
        bm_ns = median(candidate_samples, "incremental_bm_ns")
        factor_ns = median(candidate_samples, "factor_verify_ns")
        fresh_sequence_ns = median(ref_samples, "fresh_sequence_ns")
        fresh_bm_ns = median(ref_samples, "fresh_bm_ns")

        change = 0.0 if ref_ns == 0 else (candidate_ns / ref_ns - 1.0) * 100.0
        bm_fraction = 0.0 if candidate_ns == 0 else bm_ns / candidate_ns * 100.0

        writer.writerow(
            [
                "summary",
                scenario,
                first["d"],
                first["outcome"],
                first["final_k"],
                first["attempts"],
                first["payload_bytes"],
                f"{fixed_ns:.0f}",
                f"{ref_ns:.0f}",
                f"{candidate_ns:.0f}",
                f"{bm_ns:.0f}",
                f"{factor_ns:.0f}",
                f"{fresh_sequence_ns:.0f}",
                f"{fresh_bm_ns:.0f}",
                f"{change:.3f}",
                f"{bm_fraction:.3f}",
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
