#!/usr/bin/env python3
"""Summarize M6-D7 post-quadratic residual root profiles."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

HEADER = [
    "kind",
    "corpus",
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
    "quadratic_calls",
    "self_ns",
    "quadratic_ns",
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
    "quadratic_calls",
    "self_ns",
    "quadratic_ns",
    "trace_ns",
    "gcd_ns",
    "division_ns",
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
        "format": "deltameter.m6d7-residual-profile.v1",
        "contract": "frozen_m6d6_profile_only",
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
        if row["d"] != SCENARIOS[row["scenario"]]:
            raise SystemExit(f"{row['corpus']}/{row['scenario']}: wrong d")
        if row["sample"] not in range(4):
            raise SystemExit("invalid sample")
        if row["false_success"] != 0:
            raise SystemExit("false-success observed")
        grouped[(row["corpus"], row["scenario"], row["sample"], row["kind"])].append(row)

    for corpus in CORPORA:
        for scenario, d in SCENARIOS.items():
            final_k, attempts, payload = expected_stage(d)
            outcome = "exact" if d <= 8 else "rejected"

            for sample in range(4):
                tops = grouped[(corpus, scenario, sample, "scenario")]
                degree_rows = grouped[(corpus, scenario, sample, "degree")]
                if len(tops) != 1 or len(degree_rows) != 8:
                    raise SystemExit(
                        f"{corpus}/{scenario} sample={sample}: incomplete row set"
                    )

                top = tops[0]
                if (
                    top["degree"] != 0
                    or top["outcome"] != outcome
                    or top["final_k"] != final_k
                    or top["attempts"] != attempts
                    or top["rtts"] != attempts
                    or top["payload_bytes"] != payload
                ):
                    raise SystemExit(
                        f"{corpus}/{scenario} sample={sample}: frozen protocol changed"
                    )
                if top["control_ns"] <= 0 or top["profile_wall_ns"] <= 0:
                    raise SystemExit(
                        f"{corpus}/{scenario} sample={sample}: non-positive timing"
                    )
                for field in [
                    "factor_calls",
                    "trace_attempts",
                    "square_calls",
                    "quadratic_calls",
                    "self_ns",
                    "quadratic_ns",
                    "trace_ns",
                    "gcd_ns",
                    "division_ns",
                ]:
                    if top[field] != 0:
                        raise SystemExit(
                            f"{corpus}/{scenario} sample={sample}: top {field} must be zero"
                        )

                seen = set()
                for row in degree_rows:
                    degree = row["degree"]
                    if degree not in range(1, 9) or degree in seen:
                        raise SystemExit(
                            f"{corpus}/{scenario} sample={sample}: invalid degree rows"
                        )
                    seen.add(degree)

                    for field in ["outcome", "final_k", "attempts", "rtts", "payload_bytes"]:
                        if row[field] != top[field]:
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample} degree={degree}: {field} mismatch"
                            )
                    for field in [
                        "control_ns",
                        "profile_wall_ns",
                        "factor_wall_ns",
                        "verification_ns",
                    ]:
                        if row[field] != 0:
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample} degree={degree}: top timing must be zero"
                            )

                    if degree == 2:
                        if (
                            row["trace_attempts"] != 0
                            or row["square_calls"] != 0
                            or row["quadratic_calls"] != row["factor_calls"]
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample}: invalid degree-two accounting"
                            )
                    elif degree >= 3:
                        if (
                            row["square_calls"] != row["trace_attempts"] * 64
                            or row["quadratic_calls"] != 0
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample} degree={degree}: invalid trace accounting"
                            )
                    else:
                        if (
                            row["trace_attempts"] != 0
                            or row["square_calls"] != 0
                            or row["quadratic_calls"] != 0
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample}: invalid degree-one accounting"
                            )

                    measured = (
                        row["quadratic_ns"]
                        + row["trace_ns"]
                        + row["gcd_ns"]
                        + row["division_ns"]
                    )
                    if row["self_ns"] < measured:
                        raise SystemExit(
                            f"{corpus}/{scenario} sample={sample} degree={degree}: self time undercounts"
                        )

    expected_rows = len(CORPORA) * len(SCENARIOS) * 4 * 9
    if len(rows) != expected_rows:
        raise SystemExit(f"expected {expected_rows} rows, got {len(rows)}")


def median(values):
    return statistics.median(values)


def percent(part, whole):
    return 0.0 if whole == 0 else part / whole * 100.0


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d7_summary.py ARTIFACT_DIR")

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
            top[(row["corpus"], row["scenario"])].append(row)
        elif row["kind"] == "degree":
            degrees[(row["corpus"], row["scenario"], row["degree"])].append(row)
        else:
            raise SystemExit(f"unexpected kind {row['kind']}")

    print("format=deltameter.m6d7-summary.v1")
    print(f"runs={len(paths)}")
    print(f"corpora={len(CORPORA)}")
    print("logical_gate=PASS")
    print("false_success_total=0")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "scenario",
            "corpus",
            "name",
            "d",
            "outcome",
            "control_ns",
            "profile_wall_ns",
            "factor_wall_ns",
            "verification_ns",
            "degree2_self_ns",
            "degree3_self_ns",
            "degree4_self_ns",
            "degree5_self_ns",
            "degree6_self_ns",
            "degree7_self_ns",
            "degree8_self_ns",
            "dominant_residual_degree",
            "dominant_residual_of_control_percent",
            "verification_of_control_percent",
            "profile_overhead_percent",
            "samples",
        ]
    )

    ordered_scenarios = sorted(SCENARIOS.items(), key=lambda item: item[1])
    for corpus in CORPORA:
        for scenario, d in ordered_scenarios:
            tops = top[(corpus, scenario)]
            control = median(row["control_ns"] for row in tops)
            profile_wall = median(row["profile_wall_ns"] for row in tops)
            factor_wall = median(row["factor_wall_ns"] for row in tops)
            verification = median(row["verification_ns"] for row in tops)
            degree_self = {
                degree: median(
                    row["self_ns"] for row in degrees[(corpus, scenario, degree)]
                )
                for degree in range(2, 9)
            }
            dominant = max(range(3, 9), key=lambda degree: degree_self[degree])
            dominant_ns = degree_self[dominant]

            writer.writerow(
                [
                    "scenario",
                    corpus,
                    scenario,
                    d,
                    tops[0]["outcome"],
                    f"{control:.0f}",
                    f"{profile_wall:.0f}",
                    f"{factor_wall:.0f}",
                    f"{verification:.0f}",
                    *[f"{degree_self[degree]:.0f}" for degree in range(2, 9)],
                    dominant,
                    f"{percent(dominant_ns, control):.3f}",
                    f"{percent(verification, control):.3f}",
                    f"{percent(profile_wall - control, control):.3f}",
                    len(tops),
                ]
            )

    writer.writerow(
        [
            "degree",
            "corpus",
            "scenario",
            "d",
            "degree",
            "factor_calls",
            "trace_attempts",
            "square_calls",
            "quadratic_calls",
            "self_ns",
            "quadratic_ns",
            "trace_ns",
            "gcd_ns",
            "division_ns",
            "self_of_control_percent",
            "self_of_factor_percent",
            "samples",
        ]
    )

    for corpus in CORPORA:
        for scenario, d in ordered_scenarios:
            control = median(row["control_ns"] for row in top[(corpus, scenario)])
            factor_wall = median(row["factor_wall_ns"] for row in top[(corpus, scenario)])
            for degree in range(1, 9):
                samples = degrees[(corpus, scenario, degree)]
                for field in [
                    "factor_calls",
                    "trace_attempts",
                    "square_calls",
                    "quadratic_calls",
                ]:
                    values = {row[field] for row in samples}
                    if len(values) != 1:
                        raise SystemExit(
                            f"{corpus}/{scenario} degree={degree}: nondeterministic {field}"
                        )
                self_ns = median(row["self_ns"] for row in samples)
                writer.writerow(
                    [
                        "degree",
                        corpus,
                        scenario,
                        d,
                        degree,
                        samples[0]["factor_calls"],
                        samples[0]["trace_attempts"],
                        samples[0]["square_calls"],
                        samples[0]["quadratic_calls"],
                        f"{self_ns:.0f}",
                        f"{median(row['quadratic_ns'] for row in samples):.0f}",
                        f"{median(row['trace_ns'] for row in samples):.0f}",
                        f"{median(row['gcd_ns'] for row in samples):.0f}",
                        f"{median(row['division_ns'] for row in samples):.0f}",
                        f"{percent(self_ns, control):.3f}",
                        f"{percent(self_ns, factor_wall):.3f}",
                        len(samples),
                    ]
                )

    writer.writerow(
        [
            "aggregate_degree",
            "scenario",
            "d",
            "degree",
            "median_control_share_percent",
            "corpus_min_percent",
            "corpus_max_percent",
            "corpora_nonzero",
        ]
    )

    for scenario, d in ordered_scenarios:
        for degree in range(2, 9):
            shares = []
            nonzero = 0
            for corpus in CORPORA:
                control = median(row["control_ns"] for row in top[(corpus, scenario)])
                self_ns = median(
                    row["self_ns"] for row in degrees[(corpus, scenario, degree)]
                )
                share = percent(self_ns, control)
                shares.append(share)
                if self_ns > 0:
                    nonzero += 1
            writer.writerow(
                [
                    "aggregate_degree",
                    scenario,
                    d,
                    degree,
                    f"{median(shares):.3f}",
                    f"{min(shares):.3f}",
                    f"{max(shares):.3f}",
                    nonzero,
                ]
            )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
