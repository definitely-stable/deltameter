#!/usr/bin/env python3
"""Cross-runner replication closure for M6-D12."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

import m6d12_summary as base

EXPECTED_WORKERS = 5
EXPECTED_RUNS_PER_WORKER = 3
SAMPLES_PER_RUN = 4
COMMON_MATERIAL_MIN_PERCENT = 25.0
MAX_PROFILE_OVERHEAD_PERCENT = 5.0
SELECTABLE_PHASES = base.SELECTABLE_PHASES


def median(values):
    return statistics.median(values)


def percent(part, whole):
    return 0.0 if whole == 0 else part / whole * 100.0


def sample_phase_times(top_row, degree_rows):
    phase_ns = {
        "prefix": top_row["prefix_ns"],
        "locator": top_row["locator_ns"],
        "validation": top_row["validation_ns"],
        "verification": top_row["verification_ns"],
        "quadratic": degree_rows[2]["quadratic_ns"],
    }
    for name, field in [
        ("plan_build", "plan_build_ns"),
        ("trace", "trace_ns"),
        ("gcd", "gcd_ns"),
        ("division", "division_ns"),
    ]:
        phase_ns[name] = sum(degree_rows[degree][field] for degree in range(3, 9))
    return phase_ns


def summarize_worker(paths):
    if len(paths) != EXPECTED_RUNS_PER_WORKER:
        raise SystemExit(
            f"worker requires {EXPECTED_RUNS_PER_WORKER} result files, got {len(paths)}"
        )

    shares = {
        corpus: {phase: [] for phase in SELECTABLE_PHASES}
        for corpus in base.CORPORA
    }
    overheads = {corpus: [] for corpus in base.CORPORA}

    for path in sorted(paths):
        metadata, rows = base.parse(path)
        base.validate_run(metadata, rows)

        top = {}
        degrees = defaultdict(dict)
        for row in rows:
            if row["scenario"] != "d8":
                continue
            key = (row["corpus"], row["sample"])
            if row["kind"] == "scenario":
                if key in top:
                    raise SystemExit(f"{path}: duplicate d8 scenario row {key}")
                top[key] = row
            else:
                degree = row["degree"]
                if degree in degrees[key]:
                    raise SystemExit(f"{path}: duplicate d8 degree row {key}/{degree}")
                degrees[key][degree] = row

        for corpus in base.CORPORA:
            for sample in range(SAMPLES_PER_RUN):
                key = (corpus, sample)
                top_row = top.get(key)
                degree_rows = degrees.get(key)
                if top_row is None or degree_rows is None or set(degree_rows) != set(range(1, 9)):
                    raise SystemExit(f"{path}: incomplete paired d8 rows for {key}")

                control = top_row["control_ns"]
                if control <= 0:
                    raise SystemExit(f"{path}: non-positive control for {key}")
                phases = sample_phase_times(top_row, degree_rows)
                for phase, value in phases.items():
                    shares[corpus][phase].append(percent(value, control))
                overheads[corpus].append(
                    percent(top_row["profile_wall_ns"] - control, control)
                )

    summary = {}
    for corpus in base.CORPORA:
        summary[corpus] = {
            "shares": {
                phase: median(shares[corpus][phase])
                for phase in SELECTABLE_PHASES
            },
            "overhead": median(overheads[corpus]),
            "samples": len(overheads[corpus]),
        }
        if summary[corpus]["samples"] != EXPECTED_RUNS_PER_WORKER * SAMPLES_PER_RUN:
            raise SystemExit(f"{corpus}: incomplete worker sample count")

    return summary


def evaluate_replication(workers):
    if len(workers) != EXPECTED_WORKERS:
        raise SystemExit(f"expected {EXPECTED_WORKERS} workers, got {len(workers)}")

    invalid = []
    for worker, corpora in workers.items():
        if set(corpora) != set(base.CORPORA):
            raise SystemExit(f"{worker}: incomplete corpus set")
        for corpus in base.CORPORA:
            overhead = corpora[corpus]["overhead"]
            if overhead > MAX_PROFILE_OVERHEAD_PERCENT:
                invalid.append(
                    f"{worker}/{corpus} overhead {overhead:.3f}% > "
                    f"{MAX_PROFILE_OVERHEAD_PERCENT:.1f}%"
                )

    stable_common = []
    phase_medians = {}
    for phase in SELECTABLE_PHASES:
        values = [
            workers[worker][corpus]["shares"][phase]
            for worker in sorted(workers)
            for corpus in base.CORPORA
        ]
        phase_medians[phase] = median(values)
        if min(values) >= COMMON_MATERIAL_MIN_PERCENT:
            stable_common.append(phase)

    selected = None
    if stable_common:
        selected = max(
            stable_common,
            key=lambda phase: (phase_medians[phase], phase),
        )

    if invalid:
        decision = "INCONCLUSIVE_REPLICATION"
    elif selected is None:
        decision = "STOP_ALGEBRAIC_MICRO_OPT"
    else:
        decision = "GO_NARROW_PHASE_PROFILE"

    return {
        "decision": decision,
        "selected": selected,
        "stable_common": stable_common,
        "phase_medians": phase_medians,
        "invalid": invalid,
    }


def discover_workers(root):
    paths = sorted(glob.glob(os.path.join(root, "**", "result-*.txt"), recursive=True))
    if not paths:
        raise SystemExit("no worker result files found")

    grouped = defaultdict(list)
    for path in paths:
        parent = os.path.basename(os.path.dirname(path))
        if not parent.startswith("m6d12-worker-"):
            raise SystemExit(f"unexpected worker directory {parent}")
        grouped[parent].append(path)

    if len(grouped) != EXPECTED_WORKERS:
        raise SystemExit(
            f"expected {EXPECTED_WORKERS} worker directories, got {len(grouped)}"
        )
    for worker, worker_paths in grouped.items():
        names = sorted(os.path.basename(path) for path in worker_paths)
        expected = [f"result-{index}.txt" for index in range(1, EXPECTED_RUNS_PER_WORKER + 1)]
        if names != expected:
            raise SystemExit(f"{worker}: expected {expected}, got {names}")

    return dict(grouped)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6d12_replication_summary.py DOWNLOADED_ARTIFACT_ROOT")

    worker_paths = discover_workers(sys.argv[1])
    workers = {
        worker: summarize_worker(paths)
        for worker, paths in sorted(worker_paths.items())
    }
    decision = evaluate_replication(workers)

    print("format=deltameter.m6d12-replication-summary.v1")
    print(f"workers={len(workers)}")
    print(f"runs_per_worker={EXPECTED_RUNS_PER_WORKER}")
    print(f"samples_per_worker={EXPECTED_RUNS_PER_WORKER * SAMPLES_PER_RUN}")
    print("logical_gate=PASS")
    print("validity_gate=" + ("PASS" if not decision["invalid"] else "INCONCLUSIVE"))
    print("decision=" + decision["decision"])
    print("selected_phase=" + (decision["selected"] or "none"))
    print(
        "stable_common_phases="
        + (
            ";".join(decision["stable_common"])
            if decision["stable_common"]
            else "none"
        )
    )
    print(
        "validity_reasons="
        + ("none" if not decision["invalid"] else " | ".join(decision["invalid"]))
    )

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "worker_phase",
            "worker",
            "corpus",
            "phase",
            "median_paired_share_percent",
            "median_paired_overhead_percent",
            "samples",
        ]
    )
    for worker in sorted(workers):
        for corpus in base.CORPORA:
            item = workers[worker][corpus]
            for phase in SELECTABLE_PHASES:
                writer.writerow(
                    [
                        "worker_phase",
                        worker,
                        corpus,
                        phase,
                        f"{item['shares'][phase]:.3f}",
                        f"{item['overhead']:.3f}",
                        item["samples"],
                    ]
                )

    writer.writerow(
        [
            "stability",
            "phase",
            "median_percent",
            "worker_corpus_min_percent",
            "worker_corpus_max_percent",
            "stable_common",
        ]
    )
    for phase in SELECTABLE_PHASES:
        values = [
            workers[worker][corpus]["shares"][phase]
            for worker in sorted(workers)
            for corpus in base.CORPORA
        ]
        writer.writerow(
            [
                "stability",
                phase,
                f"{median(values):.3f}",
                f"{min(values):.3f}",
                f"{max(values):.3f}",
                "yes" if phase in decision["stable_common"] else "no",
            ]
        )

    gcd_d6 = [
        workers[worker]["d6"]["shares"]["gcd"]
        for worker in sorted(workers)
    ]
    writer.writerow(
        [
            "boundary",
            "gcd_d6_worker_medians",
            ";".join(f"{value:.3f}" for value in gcd_d6),
            f"{min(gcd_d6):.3f}",
            f"{max(gcd_d6):.3f}",
        ]
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
