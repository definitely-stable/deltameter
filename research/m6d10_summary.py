#!/usr/bin/env python3
"""Summarize M6-D10 trace-internal replay measurements."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

HEADER = [
    "corpus",
    "sample",
    "trace_attempts",
    "term_cases",
    "repeats",
    "full_trace_ns",
    "full_square_mod_ns",
    "prepare_ns",
    "square_build_ns",
    "reduction_with_copy_ns",
    "clone_ns",
    "trace_accumulate_ns",
]

INT_FIELDS = set(HEADER) - {"corpus"}
CORPORA = ["d4", "d5", "d6", "d7a", "d7b"]
SAMPLES = 4
REPEATS = 8

CLOSURE_MIN_PERCENT = 70.0
CLOSURE_MAX_PERCENT = 130.0
SQUARE_MOD_MIN_PERCENT = 70.0
REDUCTION_MIN_PERCENT = 50.0
SQUARE_BUILD_MIN_PERCENT = 25.0
PREPARE_MIN_PERCENT = 20.0
TRACE_ACCUMULATE_MIN_PERCENT = 20.0


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


def validate_run(metadata, rows):
    expected = {
        "format": "deltameter.m6d10-trace-internal.v1",
        "contract": "frozen_m6d8_offline_replay",
        "source_keys": "8192",
        "samples": "4",
        "repeats": "8",
        "corpora": "d4;d5;d6;d7a;d7b",
        "scenario": "d8",
        "schedule": "1:2;2:3;4:5;8:9",
        "payload_bytes": "73",
    }
    if metadata != expected:
        raise SystemExit(f"invalid metadata: {metadata}")

    grouped = defaultdict(list)
    for row in rows:
        if row["corpus"] not in CORPORA:
            raise SystemExit(f"unexpected corpus {row['corpus']}")
        if row["sample"] not in range(SAMPLES):
            raise SystemExit("invalid sample")
        if row["repeats"] != REPEATS:
            raise SystemExit("repeat count drift")
        if row["trace_attempts"] <= 0:
            raise SystemExit("non-positive trace attempts")
        if row["term_cases"] != row["trace_attempts"] * 64:
            raise SystemExit("term-case accounting drift")
        for field in [
            "full_trace_ns",
            "full_square_mod_ns",
            "prepare_ns",
            "square_build_ns",
            "reduction_with_copy_ns",
            "clone_ns",
            "trace_accumulate_ns",
        ]:
            if row[field] <= 0:
                raise SystemExit(f"non-positive {field}")
        if row["reduction_with_copy_ns"] <= row["clone_ns"]:
            raise SystemExit("reduction replay did not exceed clone baseline")
        grouped[(row["corpus"], row["sample"])].append(row)

    for corpus in CORPORA:
        attempts = set()
        terms = set()
        for sample in range(SAMPLES):
            records = grouped[(corpus, sample)]
            if len(records) != 1:
                raise SystemExit(f"{corpus} sample={sample}: incomplete row")
            attempts.add(records[0]["trace_attempts"])
            terms.add(records[0]["term_cases"])
        if len(attempts) != 1 or len(terms) != 1:
            raise SystemExit(f"{corpus}: nondeterministic replay inventory")

    expected_rows = len(CORPORA) * SAMPLES
    if len(rows) != expected_rows:
        raise SystemExit(f"expected {expected_rows} rows, got {len(rows)}")


def median(values):
    return statistics.median(values)


def percent(part, whole):
    return 0.0 if whole == 0 else part / whole * 100.0


def metrics(row):
    net_reduction = row["reduction_with_copy_ns"] - row["clone_ns"]
    closure = row["prepare_ns"] + row["square_build_ns"] + net_reduction
    return {
        "net_reduction_ns": net_reduction,
        "square_mod_of_trace_percent": percent(
            row["full_square_mod_ns"], row["full_trace_ns"]
        ),
        "prepare_of_square_percent": percent(
            row["prepare_ns"], row["full_square_mod_ns"]
        ),
        "square_build_of_square_percent": percent(
            row["square_build_ns"], row["full_square_mod_ns"]
        ),
        "net_reduction_of_square_percent": percent(
            net_reduction, row["full_square_mod_ns"]
        ),
        "trace_accumulate_of_trace_percent": percent(
            row["trace_accumulate_ns"], row["full_trace_ns"]
        ),
        "closure_percent": percent(closure, row["full_square_mod_ns"]),
    }


def evaluate(grouped):
    corpus_metrics = {}
    invalid = []

    for corpus in CORPORA:
        values = [metrics(row) for row in grouped[corpus]]
        aggregate = {
            key: median(value[key] for value in values)
            for key in values[0]
            if key != "net_reduction_ns"
        }
        aggregate["net_reduction_ns"] = median(
            value["net_reduction_ns"] for value in values
        )
        corpus_metrics[corpus] = aggregate

        closure = aggregate["closure_percent"]
        if not (CLOSURE_MIN_PERCENT <= closure <= CLOSURE_MAX_PERCENT):
            invalid.append(
                f"{corpus} closure {closure:.3f}% outside "
                f"[{CLOSURE_MIN_PERCENT:.0f},{CLOSURE_MAX_PERCENT:.0f}]%"
            )

    if invalid:
        return "INCONCLUSIVE_REPLAY_CLOSURE", corpus_metrics, invalid

    square_mod_common = min(
        corpus_metrics[corpus]["square_mod_of_trace_percent"] for corpus in CORPORA
    )
    if square_mod_common >= SQUARE_MOD_MIN_PERCENT:
        reduction_common = min(
            corpus_metrics[corpus]["net_reduction_of_square_percent"]
            for corpus in CORPORA
        )
        if reduction_common >= REDUCTION_MIN_PERCENT:
            return "GO_REDUCTION_EXPERIMENT", corpus_metrics, []

        square_build_common = min(
            corpus_metrics[corpus]["square_build_of_square_percent"]
            for corpus in CORPORA
        )
        if square_build_common >= SQUARE_BUILD_MIN_PERCENT:
            return "GO_SQUARE_BUILD_EXPERIMENT", corpus_metrics, []

        prepare_common = min(
            corpus_metrics[corpus]["prepare_of_square_percent"]
            for corpus in CORPORA
        )
        if prepare_common >= PREPARE_MIN_PERCENT:
            return "GO_MODULUS_PREP_EXPERIMENT", corpus_metrics, []

    accumulate_common = min(
        corpus_metrics[corpus]["trace_accumulate_of_trace_percent"]
        for corpus in CORPORA
    )
    if accumulate_common >= TRACE_ACCUMULATE_MIN_PERCENT:
        return "GO_TRACE_ACCUMULATION_EXPERIMENT", corpus_metrics, []

    return "STOP_ALGEBRAIC_MICRO_OPT", corpus_metrics, []


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d10_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.txt")))
    if [os.path.basename(path) for path in paths] != [
        "result-1.txt",
        "result-2.txt",
        "result-3.txt",
    ]:
        raise SystemExit("exactly three complete named runs required")

    grouped = defaultdict(list)
    for path in paths:
        metadata, rows = parse(path)
        validate_run(metadata, rows)
        for row in rows:
            grouped[row["corpus"]].append(row)

    decision, corpus_metrics, invalid = evaluate(grouped)

    print("format=deltameter.m6d10-summary.v1")
    print(f"runs={len(paths)}")
    print(f"corpora={len(CORPORA)}")
    print("logical_gate=PASS")
    print("validity_gate=" + ("PASS" if not invalid else "INCONCLUSIVE"))
    print("decision=" + decision)
    print("validity_reasons=" + ("none" if not invalid else " | ".join(invalid)))

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "corpus",
            "trace_attempts",
            "term_cases",
            "full_trace_ns",
            "full_square_mod_ns",
            "prepare_ns",
            "square_build_ns",
            "net_reduction_ns",
            "trace_accumulate_ns",
            "square_mod_of_trace_percent",
            "prepare_of_square_percent",
            "square_build_of_square_percent",
            "net_reduction_of_square_percent",
            "trace_accumulate_of_trace_percent",
            "closure_percent",
            "samples",
        ]
    )

    for corpus in CORPORA:
        rows = grouped[corpus]
        values = corpus_metrics[corpus]
        writer.writerow(
            [
                "corpus",
                corpus,
                rows[0]["trace_attempts"],
                rows[0]["term_cases"],
                f"{median(row['full_trace_ns'] for row in rows):.0f}",
                f"{median(row['full_square_mod_ns'] for row in rows):.0f}",
                f"{median(row['prepare_ns'] for row in rows):.0f}",
                f"{median(row['square_build_ns'] for row in rows):.0f}",
                f"{values['net_reduction_ns']:.0f}",
                f"{median(row['trace_accumulate_ns'] for row in rows):.0f}",
                f"{values['square_mod_of_trace_percent']:.3f}",
                f"{values['prepare_of_square_percent']:.3f}",
                f"{values['square_build_of_square_percent']:.3f}",
                f"{values['net_reduction_of_square_percent']:.3f}",
                f"{values['trace_accumulate_of_trace_percent']:.3f}",
                f"{values['closure_percent']:.3f}",
                len(rows),
            ]
        )

    writer.writerow(
        [
            "range",
            "metric",
            "median_percent",
            "corpus_min_percent",
            "corpus_max_percent",
        ]
    )
    for metric_name in [
        "square_mod_of_trace_percent",
        "prepare_of_square_percent",
        "square_build_of_square_percent",
        "net_reduction_of_square_percent",
        "trace_accumulate_of_trace_percent",
        "closure_percent",
    ]:
        shares = [corpus_metrics[corpus][metric_name] for corpus in CORPORA]
        writer.writerow(
            [
                "range",
                metric_name,
                f"{median(shares):.3f}",
                f"{min(shares):.3f}",
                f"{max(shares):.3f}",
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
