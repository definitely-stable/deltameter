#!/usr/bin/env python3
"""Summarize deterministic M6-A two-process experiment artifacts."""

from __future__ import annotations

import csv
import glob
import os
import statistics
import sys


LOGICAL_FIELDS = [
    "arm",
    "scenario",
    "source_n",
    "peer_n",
    "exact_d",
    "threshold",
    "point",
    "decision",
    "outcome",
    "tx_bytes",
    "rx_bytes",
    "total_bytes",
    "round_trips",
    "exact_bytes_avoided",
    "false_admit",
    "false_reject",
]


def parse(path: str):
    metadata = {}
    setup = {}
    header = None
    results = {}

    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.strip()
            if not line:
                continue
            if "=" in line and not line.startswith(("setup,", "result,")):
                key, value = line.split("=", 1)
                metadata[key] = value
                continue
            if line == "setup,arm,peer_setup_ns,ready_rx_bytes":
                continue
            if line.startswith("setup,"):
                _, arm, setup_ns, ready_bytes = line.split(",")
                setup[arm] = (int(setup_ns), int(ready_bytes))
                continue
            if line.startswith("result,arm,"):
                header = line.split(",")[1:]
                continue
            if line.startswith("result,"):
                if header is None:
                    raise SystemExit(f"{path}: result header missing")
                values = line.split(",")[1:]
                row = dict(zip(header, values, strict=True))
                results[(row["arm"], row["scenario"])] = row

    if not results:
        raise SystemExit(f"{path}: no experiment results")
    return metadata, setup, results


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: m6a_value_summary.py ARTIFACT_DIR")

    paths = sorted(glob.glob(os.path.join(sys.argv[1], "result-*.csv")))
    if not paths:
        raise SystemExit("no M6-A result files found")

    parsed = [parse(path) for path in paths]
    base_meta, _, base_results = parsed[0]

    timing = {}
    setup_timing = {}
    for _, setup, results in parsed:
        if set(results) != set(base_results):
            raise SystemExit("result key set differs across runs")
        for key, base in base_results.items():
            row = results[key]
            for field in LOGICAL_FIELDS:
                if row[field] != base[field]:
                    raise SystemExit(
                        f"logical result changed for {key} field {field}: "
                        f"{base[field]} != {row[field]}"
                    )
            timing.setdefault(key, [[], []])
            timing[key][0].append(int(row["elapsed_ns"]))
            timing[key][1].append(int(row["source_sketch_build_ns"]))
        for arm, (setup_ns, ready_bytes) in setup.items():
            entry = setup_timing.setdefault(arm, [[], ready_bytes])
            if entry[1] != ready_bytes:
                raise SystemExit(f"ready bytes changed for {arm}")
            entry[0].append(setup_ns)

    arm_bytes = {}
    rejected = 0
    false_decisions = 0
    for row in base_results.values():
        arm_bytes[row["arm"]] = arm_bytes.get(row["arm"], 0) + int(row["total_bytes"])
        if row["arm"] == "snapshot-admission" and row["outcome"] == "rejected":
            rejected += 1
        false_decisions += int(row["false_admit"] == "true")
        false_decisions += int(row["false_reject"] == "true")

    direct_total = arm_bytes["direct-exact"]
    admission_total = arm_bytes["snapshot-admission"]
    control_total = arm_bytes["snapshot-control"]
    matrix_delta = (admission_total / direct_total - 1.0) * 100.0

    direct_equal = int(base_results[("direct-exact", "d512")]["total_bytes"])
    rejected_equal = int(base_results[("snapshot-admission", "d512")]["total_bytes"])
    break_even_reject = rejected_equal / direct_equal

    print("format=deltameter.m6a-summary.v1")
    print(f"runs={len(paths)}")
    print(f"peer_size={base_meta['peer_size']}")
    print(f"admission_threshold={base_meta['admission_threshold']}")
    print(f"matrix_scenarios={len(base_results) // 3}")
    print(f"matrix_rejected={rejected}")
    print(f"false_decisions={false_decisions}")
    print(f"direct_total_bytes={direct_total}")
    print(f"control_total_bytes={control_total}")
    print(f"admission_total_bytes={admission_total}")
    print(f"admission_vs_direct_percent={matrix_delta:.3f}")
    print(f"equal_size_break_even_reject_fraction={break_even_reject:.6f}")
    print(
        "matrix_byte_verdict="
        + ("NO_GO" if admission_total >= direct_total else "GO")
    )
    print("timing_contract=diagnostic_only_local_subprocess")
    print("setup,arm,median_peer_setup_ns,ready_rx_bytes")
    for arm in sorted(setup_timing):
        samples, ready_bytes = setup_timing[arm]
        print(
            f"setup,{arm},{statistics.median(samples):.0f},{ready_bytes}"
        )

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(
        [
            "summary",
            "arm",
            "scenario",
            "exact_d",
            "point",
            "decision",
            "outcome",
            "total_bytes",
            "round_trips",
            "median_elapsed_ns",
            "median_source_sketch_build_ns",
            "exact_bytes_avoided",
        ]
    )
    for key in sorted(base_results):
        row = base_results[key]
        elapsed, build = timing[key]
        writer.writerow(
            [
                "summary",
                row["arm"],
                row["scenario"],
                row["exact_d"],
                row["point"],
                row["decision"],
                row["outcome"],
                row["total_bytes"],
                row["round_trips"],
                f"{statistics.median(elapsed):.0f}",
                f"{statistics.median(build):.0f}",
                row["exact_bytes_avoided"],
            ]
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
