#!/usr/bin/env python3
"""Fail-closed summary for M6-D8 scalar-square evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

HEADER = [
    "corpus",
    "scenario",
    "d",
    "sample",
    "outcome",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "control_ns",
    "candidate_ns",
    "false_success",
]

INT_FIELDS = {
    "d",
    "sample",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "control_ns",
    "candidate_ns",
    "false_success",
}

CORPORA = ["d4", "d5", "d6", "d7a", "d7b"]
SCENARIOS = {
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
PRIMARY_MIN_PERCENT = 5.0
GUARDRAIL_MIN_PERCENT = -2.0


def parse(path: str):
    metadata = {}
    rows = []
    header = None

    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            if not line:
                continue
            if line.startswith("record,corpus,"):
                if header is not None or line.split(",")[1:] != HEADER:
                    raise SystemExit(f"{path}: duplicate or invalid header")
                header = HEADER
                continue
            if line.startswith("record,"):
                if header is None:
                    raise SystemExit(f"{path}: record before header")
                values = line.split(",")[1:]
                if len(values) != len(header):
                    raise SystemExit(f"{path}: malformed record")
                row = dict(zip(header, values, strict=True))
                for field in INT_FIELDS:
                    row[field] = int(row[field])
                    if row[field] < 0:
                        raise SystemExit(f"{path}: negative {field}")
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


def validate_run(metadata, rows):
    expected_meta = {
        "format": "deltameter.m6d8-square.v1",
        "contract": "frozen_m6d7_square_only",
        "source_keys": "8192",
        "samples": "4",
        "corpora": "d4;d5;d6;d7a;d7b",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }
    if metadata != expected_meta:
        raise SystemExit(f"invalid metadata: {metadata}")

    grouped = defaultdict(list)
    for row in rows:
        if row["corpus"] not in CORPORA:
            raise SystemExit(f"unexpected corpus {row['corpus']}")
        if row["scenario"] not in SCENARIOS:
            raise SystemExit(f"unexpected scenario {row['scenario']}")
        d = SCENARIOS[row["scenario"]]
        if row["d"] != d:
            raise SystemExit(f"{row['corpus']}/{row['scenario']}: wrong d")
        if row["sample"] not in range(4):
            raise SystemExit("invalid sample")
        if row["false_success"] != 0:
            raise SystemExit("false-success observed")

        final_k, attempts, payload = expected_stage(d)
        outcome = "exact" if d <= 8 else "rejected"
        if (
            row["outcome"] != outcome
            or row["final_k"] != final_k
            or row["attempts"] != attempts
            or row["rtts"] != attempts
            or row["payload_bytes"] != payload
        ):
            raise SystemExit(
                f"{row['corpus']}/{row['scenario']} sample={row['sample']}: frozen protocol changed"
            )
        if row["control_ns"] <= 0 or row["candidate_ns"] <= 0:
            raise SystemExit(
                f"{row['corpus']}/{row['scenario']} sample={row['sample']}: non-positive timing"
            )
        grouped[(row["corpus"], row["scenario"], row["sample"])].append(row)

    for corpus in CORPORA:
        for scenario in SCENARIOS:
            for sample in range(4):
                if len(grouped[(corpus, scenario, sample)]) != 1:
                    raise SystemExit(
                        f"{corpus}/{scenario} sample={sample}: incomplete row set"
                    )

    expected_rows = len(CORPORA) * len(SCENARIOS) * 4
    if len(rows) != expected_rows:
        raise SystemExit(f"expected {expected_rows} rows, got {len(rows)}")


def median(values):
    return statistics.median(values)


def reductions(rows):
    return [(1.0 - row["candidate_ns"] / row["control_ns"]) * 100.0 for row in rows]


def paired_summary(rows):
    values = reductions(rows)
    if len(values) % 4 != 0:
        raise SystemExit("paired summary requires complete four-sample runs")
    per_run = [median(values[start : start + 4]) for start in range(0, len(values), 4)]
    return median(values), min(per_run), max(per_run)


def evaluate_performance_gate(totals):
    reasons = []

    d8_all = []
    for corpus in CORPORA:
        rows = totals[(corpus, "d8")]
        reduction, run_min, _ = paired_summary(rows)
        d8_all.extend(rows)
        if reduction < PRIMARY_MIN_PERCENT:
            reasons.append(f"{corpus}/d8 median {reduction:.3f}% < {PRIMARY_MIN_PERCENT:.1f}%")
        if run_min <= 0.0:
            reasons.append(f"{corpus}/d8 process median not positive ({run_min:.3f}%)")

    aggregate = median(reductions(d8_all))
    if aggregate < PRIMARY_MIN_PERCENT:
        reasons.append(
            f"aggregate/d8 median {aggregate:.3f}% < {PRIMARY_MIN_PERCENT:.1f}%"
        )

    for scenario in ["d3", "d4", "d5"]:
        for corpus in CORPORA:
            reduction = median(reductions(totals[(corpus, scenario)]))
            if reduction < GUARDRAIL_MIN_PERCENT:
                reasons.append(
                    f"{corpus}/{scenario} median {reduction:.3f}% < {GUARDRAIL_MIN_PERCENT:.1f}%"
                )

    return not reasons, reasons


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d8_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if [os.path.basename(path) for path in paths] != [
        "result-1.txt",
        "result-2.txt",
        "result-3.txt",
    ]:
        raise SystemExit("exactly three complete named runs required")

    rows = []
    for path in paths:
        metadata, run_rows = parse(path)
        validate_run(metadata, run_rows)
        rows.extend(run_rows)

    totals = defaultdict(list)
    for row in rows:
        totals[(row["corpus"], row["scenario"])].append(row)

    gate_pass, reasons = evaluate_performance_gate(totals)

    print("format=deltameter.m6d8-summary.v1")
    print(f"runs={len(paths)}")
    print(f"corpora={len(CORPORA)}")
    print("logical_gate=PASS")
    print("false_success_total=0")
    print(f"performance_gate={'PASS' if gate_pass else 'FAIL'}")
    print("performance_gate_reasons=" + ("none" if gate_pass else " | ".join(reasons)))

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "decode",
            "corpus",
            "scenario",
            "d",
            "outcome",
            "final_k",
            "attempts",
            "payload_bytes",
            "control_ns",
            "candidate_ns",
            "median_reduction_percent",
            "run_median_min_percent",
            "run_median_max_percent",
            "samples",
        ]
    )

    ordered_scenarios = sorted(SCENARIOS.items(), key=lambda item: item[1])
    for corpus in CORPORA:
        for scenario, d in ordered_scenarios:
            samples = totals[(corpus, scenario)]
            reduction, run_min, run_max = paired_summary(samples)
            first = samples[0]
            writer.writerow(
                [
                    "decode",
                    corpus,
                    scenario,
                    d,
                    first["outcome"],
                    first["final_k"],
                    first["attempts"],
                    first["payload_bytes"],
                    f"{median(row['control_ns'] for row in samples):.0f}",
                    f"{median(row['candidate_ns'] for row in samples):.0f}",
                    f"{reduction:.3f}",
                    f"{run_min:.3f}",
                    f"{run_max:.3f}",
                    len(samples),
                ]
            )

    writer.writerow(
        [
            "aggregate",
            "scenario",
            "d",
            "control_ns",
            "candidate_ns",
            "median_reduction_percent",
            "corpus_median_min_percent",
            "corpus_median_max_percent",
            "samples",
        ]
    )

    for scenario, d in ordered_scenarios:
        samples = []
        corpus_medians = []
        for corpus in CORPORA:
            corpus_rows = totals[(corpus, scenario)]
            samples.extend(corpus_rows)
            corpus_medians.append(median(reductions(corpus_rows)))
        writer.writerow(
            [
                "aggregate",
                scenario,
                d,
                f"{median(row['control_ns'] for row in samples):.0f}",
                f"{median(row['candidate_ns'] for row in samples):.0f}",
                f"{median(reductions(samples)):.3f}",
                f"{min(corpus_medians):.3f}",
                f"{max(corpus_medians):.3f}",
                len(samples),
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
