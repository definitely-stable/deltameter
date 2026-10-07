#!/usr/bin/env python3
"""Summarize M6-D5 root-factor profiling evidence."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

HEADER = [
    "kind",
    "scenario",
    "d",
    "sample",
    "degree",
    "outcome",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "control_ns",
    "profile_wall_ns",
    "factor_wall_ns",
    "verification_ns",
    "factor_calls",
    "trace_attempts",
    "square_calls",
    "self_ns",
    "trace_ns",
    "gcd_ns",
    "division_ns",
    "false_success",
]

INT_FIELDS = {
    "d",
    "sample",
    "degree",
    "final_k",
    "attempts",
    "rtts",
    "payload_bytes",
    "control_ns",
    "profile_wall_ns",
    "factor_wall_ns",
    "verification_ns",
    "factor_calls",
    "trace_attempts",
    "square_calls",
    "self_ns",
    "trace_ns",
    "gcd_ns",
    "division_ns",
    "false_success",
}

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
        "format": "deltameter.m6d5-root-profile.v1",
        "contract": "frozen_m6d4_decoder_profile_only",
        "source_keys": "8192",
        "samples": "4",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }
    if metadata != expected_meta:
        raise SystemExit(f"invalid metadata: {metadata}")

    grouped = defaultdict(list)
    for row in rows:
        if row["scenario"] not in SCENARIOS:
            raise SystemExit(f"unexpected scenario {row['scenario']}")
        if row["d"] != SCENARIOS[row["scenario"]]:
            raise SystemExit(f"{row['scenario']}: wrong d")
        if row["sample"] not in range(4):
            raise SystemExit(f"{row['scenario']}: invalid sample")
        if row["false_success"] != 0:
            raise SystemExit("false-success observed")
        grouped[(row["scenario"], row["sample"], row["kind"])].append(row)

    for scenario, d in SCENARIOS.items():
        expected_outcome = "exact" if d <= 8 else "rejected"
        expected_k, expected_attempts, expected_bytes = expected_stage(d)
        for sample in range(4):
            scenario_rows = grouped[(scenario, sample, "scenario")]
            degree_rows = grouped[(scenario, sample, "degree")]
            if len(scenario_rows) != 1 or len(degree_rows) != 8:
                raise SystemExit(f"{scenario} sample={sample}: incomplete row set")

            top = scenario_rows[0]
            for field, expected in [
                ("degree", 0),
                ("final_k", expected_k),
                ("attempts", expected_attempts),
                ("rtts", expected_attempts),
                ("payload_bytes", expected_bytes),
            ]:
                if top[field] != expected:
                    raise SystemExit(
                        f"{scenario} sample={sample}: {field}={top[field]} != {expected}"
                    )
            if top["outcome"] != expected_outcome:
                raise SystemExit(f"{scenario} sample={sample}: unexpected outcome")
            if top["control_ns"] <= 0 or top["profile_wall_ns"] <= 0:
                raise SystemExit(f"{scenario} sample={sample}: non-positive top timing")
            if top["factor_wall_ns"] < 0 or top["verification_ns"] < 0:
                raise SystemExit(f"{scenario} sample={sample}: invalid phase timing")
            for field in [
                "factor_calls",
                "trace_attempts",
                "square_calls",
                "self_ns",
                "trace_ns",
                "gcd_ns",
                "division_ns",
            ]:
                if top[field] != 0:
                    raise SystemExit(f"{scenario} sample={sample}: top row {field} must be zero")

            seen = set()
            for row in degree_rows:
                degree = row["degree"]
                if degree not in range(1, 9) or degree in seen:
                    raise SystemExit(f"{scenario} sample={sample}: invalid degree rows")
                seen.add(degree)
                for field in ["outcome", "final_k", "attempts", "rtts", "payload_bytes"]:
                    if row[field] != top[field]:
                        raise SystemExit(
                            f"{scenario} sample={sample} degree={degree}: {field} mismatch"
                        )
                for field in [
                    "control_ns",
                    "profile_wall_ns",
                    "factor_wall_ns",
                    "verification_ns",
                ]:
                    if row[field] != 0:
                        raise SystemExit(
                            f"{scenario} sample={sample} degree={degree}: {field} must be zero"
                        )
                if row["square_calls"] != row["trace_attempts"] * 64:
                    raise SystemExit(
                        f"{scenario} sample={sample} degree={degree}: square accounting mismatch"
                    )
                measured = row["trace_ns"] + row["gcd_ns"] + row["division_ns"]
                if row["self_ns"] < measured:
                    raise SystemExit(
                        f"{scenario} sample={sample} degree={degree}: self time undercounts"
                    )

    expected_rows = len(SCENARIOS) * 4 * 9
    if len(rows) != expected_rows:
        raise SystemExit(f"expected {expected_rows} rows, got {len(rows)}")


def median(values):
    return statistics.median(values)


def percent(part, whole):
    return 0.0 if whole == 0 else part / whole * 100.0


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d5_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if [os.path.basename(path) for path in paths] != [
        "result-1.txt",
        "result-2.txt",
        "result-3.txt",
    ]:
        raise SystemExit("exactly three complete named runs required")

    parsed = [parse(path) for path in paths]
    rows = []
    for metadata, run_rows in parsed:
        validate_run(metadata, run_rows)
        rows.extend(run_rows)

    top = defaultdict(list)
    degrees = defaultdict(list)
    for row in rows:
        if row["kind"] == "scenario":
            top[row["scenario"]].append(row)
        elif row["kind"] == "degree":
            degrees[(row["scenario"], row["degree"])].append(row)
        else:
            raise SystemExit(f"unexpected kind {row['kind']}")

    print("format=deltameter.m6d5-summary.v1")
    print(f"runs={len(paths)}")
    print("logical_gate=PASS")
    print("false_success_total=0")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "scenario",
            "d",
            "outcome",
            "control_ns",
            "profile_wall_ns",
            "factor_wall_ns",
            "verification_ns",
            "degree2_self_ns",
            "degree3plus_self_ns",
            "degree2_of_factor_percent",
            "degree2_of_control_percent",
            "verification_of_control_percent",
            "higher_degree_of_factor_percent",
            "profile_overhead_percent",
            "samples",
        ]
    )

    ordered = sorted(SCENARIOS.items(), key=lambda item: item[1])
    for scenario, d in ordered:
        tops = top[scenario]
        control = median(row["control_ns"] for row in tops)
        profile_wall = median(row["profile_wall_ns"] for row in tops)
        factor_wall = median(row["factor_wall_ns"] for row in tops)
        verification = median(row["verification_ns"] for row in tops)
        degree2 = median(row["self_ns"] for row in degrees[(scenario, 2)])
        degree3plus = median(
            sum(
                next(
                    r["self_ns"]
                    for r in degrees[(scenario, degree)]
                    if r["sample"] == sample
                )
                for degree in range(3, 9)
            )
            for sample in range(4)
            for _run in [0]
        )
        # Recompute degree3+ across all 12 observations by aligned position.
        degree3plus_values = []
        for index in range(len(tops)):
            degree3plus_values.append(
                sum(degrees[(scenario, degree)][index]["self_ns"] for degree in range(3, 9))
            )
        degree3plus = median(degree3plus_values)

        writer.writerow(
            [
                scenario,
                d,
                tops[0]["outcome"],
                f"{control:.0f}",
                f"{profile_wall:.0f}",
                f"{factor_wall:.0f}",
                f"{verification:.0f}",
                f"{degree2:.0f}",
                f"{degree3plus:.0f}",
                f"{percent(degree2, factor_wall):.3f}",
                f"{percent(degree2, control):.3f}",
                f"{percent(verification, control):.3f}",
                f"{percent(degree3plus, factor_wall):.3f}",
                f"{percent(profile_wall - control, control):.3f}",
                len(tops),
            ]
        )

    writer.writerow(
        [
            "degree",
            "scenario",
            "d",
            "factor_calls",
            "trace_attempts",
            "square_calls",
            "self_ns",
            "trace_ns",
            "gcd_ns",
            "division_ns",
            "samples",
        ]
    )

    for scenario, d in ordered:
        for degree in range(1, 9):
            samples = degrees[(scenario, degree)]
            counts = {
                field: {row[field] for row in samples}
                for field in ["factor_calls", "trace_attempts", "square_calls"]
            }
            if any(len(values) != 1 for values in counts.values()):
                raise SystemExit(f"{scenario} degree={degree}: nondeterministic operation counts")
            writer.writerow(
                [
                    degree,
                    scenario,
                    d,
                    samples[0]["factor_calls"],
                    samples[0]["trace_attempts"],
                    samples[0]["square_calls"],
                    f"{median(row['self_ns'] for row in samples):.0f}",
                    f"{median(row['trace_ns'] for row in samples):.0f}",
                    f"{median(row['gcd_ns'] for row in samples):.0f}",
                    f"{median(row['division_ns'] for row in samples):.0f}",
                    len(samples),
                ]
            )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
