#!/usr/bin/env python3
"""Summarize M6-D12 post-D11 whole-decode residual profiles."""

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
    "prefix_ns",
    "locator_ns",
    "decode_wall_ns",
    "validation_ns",
    "factor_wall_ns",
    "verification_ns",
    "factor_calls",
    "plan_build_calls",
    "trace_attempts",
    "quadratic_calls",
    "self_ns",
    "plan_build_ns",
    "quadratic_ns",
    "trace_ns",
    "gcd_ns",
    "division_ns",
    "false_success",
]

INT_FIELDS = set(HEADER) - {"kind", "corpus", "scenario", "outcome"}

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

COMMON_MATERIAL_MIN_PERCENT = 25.0
MAX_PROFILE_OVERHEAD_PERCENT = 5.0
SELECTABLE_PHASES = (
    "prefix",
    "locator",
    "validation",
    "plan_build",
    "trace",
    "gcd",
    "division",
    "quadratic",
    "verification",
)


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
                    raise SystemExit(
                        f"{path}: malformed record expected={len(header)} got={len(values)}"
                    )
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
        "format": "deltameter.m6d12-post-d11-profile.v1",
        "contract": "frozen_m6d11_whole_decode_profile",
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
        if row["kind"] not in {"scenario", "degree"}:
            raise SystemExit(f"unexpected kind {row['kind']}")
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
                for field in [
                    "control_ns",
                    "profile_wall_ns",
                    "prefix_ns",
                    "locator_ns",
                    "decode_wall_ns",
                ]:
                    if top[field] <= 0:
                        raise SystemExit(
                            f"{corpus}/{scenario} sample={sample}: non-positive {field}"
                        )
                if top["profile_wall_ns"] < top["decode_wall_ns"]:
                    raise SystemExit(
                        f"{corpus}/{scenario} sample={sample}: decoder exceeds profile wall"
                    )

                for field in [
                    "factor_calls",
                    "plan_build_calls",
                    "trace_attempts",
                    "quadratic_calls",
                    "self_ns",
                    "plan_build_ns",
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
                        "prefix_ns",
                        "locator_ns",
                        "decode_wall_ns",
                        "validation_ns",
                        "factor_wall_ns",
                        "verification_ns",
                    ]:
                        if row[field] != 0:
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample} degree={degree}: top timing must be zero"
                            )

                    if degree == 2:
                        if (
                            row["plan_build_calls"] != 0
                            or row["trace_attempts"] != 0
                            or row["quadratic_calls"] != row["factor_calls"]
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample}: invalid degree-two accounting"
                            )
                    elif degree >= 3:
                        if (
                            row["plan_build_calls"] != row["factor_calls"]
                            or row["quadratic_calls"] != 0
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample} degree={degree}: invalid D11 accounting"
                            )
                    else:
                        if (
                            row["plan_build_calls"] != 0
                            or row["trace_attempts"] != 0
                            or row["quadratic_calls"] != 0
                        ):
                            raise SystemExit(
                                f"{corpus}/{scenario} sample={sample}: invalid degree-one accounting"
                            )

                    measured = (
                        row["plan_build_ns"]
                        + row["quadratic_ns"]
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


def build_phase_summary(top, degrees, corpus):
    tops = top[(corpus, "d8")]
    control = median(row["control_ns"] for row in tops)
    if control <= 0:
        raise SystemExit(f"{corpus}: non-positive d8 control")

    phase_ns = {
        "prefix": median(row["prefix_ns"] for row in tops),
        "locator": median(row["locator_ns"] for row in tops),
        "validation": median(row["validation_ns"] for row in tops),
        "verification": median(row["verification_ns"] for row in tops),
        "quadratic": median(
            row["quadratic_ns"] for row in degrees[(corpus, "d8", 2)]
        ),
    }
    for name, field in [
        ("plan_build", "plan_build_ns"),
        ("trace", "trace_ns"),
        ("gcd", "gcd_ns"),
        ("division", "division_ns"),
    ]:
        phase_ns[name] = sum(
            median(row[field] for row in degrees[(corpus, "d8", degree)])
            for degree in range(3, 9)
        )

    shares = {name: percent(value, control) for name, value in phase_ns.items()}
    overhead = percent(
        median(row["profile_wall_ns"] for row in tops) - control,
        control,
    )

    return {
        "control_ns": control,
        "profile_wall_ns": median(row["profile_wall_ns"] for row in tops),
        "decode_wall_ns": median(row["decode_wall_ns"] for row in tops),
        "factor_wall_ns": median(row["factor_wall_ns"] for row in tops),
        "phase_ns": phase_ns,
        "shares": shares,
        "overhead": overhead,
    }


def evaluate_decision(top, degrees):
    summaries = {
        corpus: build_phase_summary(top, degrees, corpus)
        for corpus in CORPORA
    }
    invalid = [
        f"{corpus}/d8 profiler overhead {summary['overhead']:.3f}% > "
        f"{MAX_PROFILE_OVERHEAD_PERCENT:.1f}%"
        for corpus, summary in summaries.items()
        if summary["overhead"] > MAX_PROFILE_OVERHEAD_PERCENT
    ]

    common = []
    medians = {}
    for phase in SELECTABLE_PHASES:
        shares = [summaries[corpus]["shares"][phase] for corpus in CORPORA]
        medians[phase] = median(shares)
        if min(shares) >= COMMON_MATERIAL_MIN_PERCENT:
            common.append(phase)

    selected = None
    if common:
        selected = max(common, key=lambda phase: (medians[phase], phase))

    if invalid:
        decision = "INCONCLUSIVE_REDUCE_INSTRUMENTATION"
    elif selected is None:
        decision = "STOP_ALGEBRAIC_MICRO_OPT"
    else:
        decision = "GO_NARROW_PHASE_PROFILE"

    return {
        "decision": decision,
        "selected": selected,
        "common": common,
        "medians": medians,
        "summaries": summaries,
        "invalid": invalid,
    }


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d12_summary.py ARTIFACT_DIR")

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

    top = defaultdict(list)
    degrees = defaultdict(list)
    for row in rows:
        if row["kind"] == "scenario":
            top[(row["corpus"], row["scenario"])].append(row)
        else:
            degrees[(row["corpus"], row["scenario"], row["degree"])].append(row)

    # Counts must be deterministic across all three processes and four samples.
    for key, samples in degrees.items():
        for field in [
            "factor_calls",
            "plan_build_calls",
            "trace_attempts",
            "quadratic_calls",
        ]:
            if len({row[field] for row in samples}) != 1:
                raise SystemExit(f"{key}: nondeterministic {field}")

    decision = evaluate_decision(top, degrees)

    print("format=deltameter.m6d12-summary.v1")
    print(f"runs={len(paths)}")
    print(f"corpora={len(CORPORA)}")
    print("logical_gate=PASS")
    print("false_success_total=0")
    print("validity_gate=" + ("PASS" if not decision["invalid"] else "INCONCLUSIVE"))
    print("decision=" + decision["decision"])
    print("selected_phase=" + (decision["selected"] or "none"))
    print(
        "common_material_phases="
        + (";".join(decision["common"]) if decision["common"] else "none")
    )
    print(
        "validity_reasons="
        + ("none" if not decision["invalid"] else " | ".join(decision["invalid"]))
    )

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "phase",
            "corpus",
            "d",
            "control_ns",
            "phase_ns",
            "control_share_percent",
            "profile_overhead_percent",
            "samples",
        ]
    )
    for corpus in CORPORA:
        summary = decision["summaries"][corpus]
        for phase in SELECTABLE_PHASES:
            writer.writerow(
                [
                    "phase",
                    corpus,
                    8,
                    f"{summary['control_ns']:.0f}",
                    f"{summary['phase_ns'][phase]:.0f}",
                    f"{summary['shares'][phase]:.3f}",
                    f"{summary['overhead']:.3f}",
                    len(top[(corpus, "d8")]),
                ]
            )

    writer.writerow(
        [
            "phase_range",
            "phase",
            "median_control_share_percent",
            "corpus_min_percent",
            "corpus_max_percent",
            "common_material",
        ]
    )
    for phase in SELECTABLE_PHASES:
        shares = [
            decision["summaries"][corpus]["shares"][phase]
            for corpus in CORPORA
        ]
        writer.writerow(
            [
                "phase_range",
                phase,
                f"{median(shares):.3f}",
                f"{min(shares):.3f}",
                f"{max(shares):.3f}",
                "yes" if phase in decision["common"] else "no",
            ]
        )

    writer.writerow(
        [
            "wall",
            "corpus",
            "control_ns",
            "profile_wall_ns",
            "decode_wall_ns",
            "factor_wall_ns",
            "overhead_percent",
        ]
    )
    for corpus in CORPORA:
        summary = decision["summaries"][corpus]
        writer.writerow(
            [
                "wall",
                corpus,
                f"{summary['control_ns']:.0f}",
                f"{summary['profile_wall_ns']:.0f}",
                f"{summary['decode_wall_ns']:.0f}",
                f"{summary['factor_wall_ns']:.0f}",
                f"{summary['overhead']:.3f}",
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
