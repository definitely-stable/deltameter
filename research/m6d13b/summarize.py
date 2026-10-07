#!/usr/bin/env python3
"""Aggregate five M6-D13-B hosted workers under the frozen decision gates."""
from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path

from model import CONTRACT, median, modeled_total_ns, require

ARMS = ("direct", "d11", "riblt_pull", "riblt_stream_lb")


def expected_rows_per_worker() -> int:
    core = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["core"]["n"])
        * len(CONTRACT["core"]["updates_per_session"])
        * len(ARMS)
        * CONTRACT["measured_sessions"]
    )
    boundary = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["boundary"]["d"])
        * len(ARMS)
    )
    scaling = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["scaling"]["d"])
        * len(ARMS)
    )
    return core + boundary + scaling


def validate_row(row: dict[str, object]) -> None:
    required = (
        "kind", "worker", "process", "arm", "n", "u_or_d", "session",
        "build_ns", "update_native_ns", "sync_native_ns", "session_native_ns",
        "application_bytes", "rounds", "fallback", "vmrss_bytes", "vmhwm_bytes",
        "candidate_capacity",
    )
    for key in required:
        require(key in row, f"missing raw field {key}")

    require(row["kind"] in ("core", "boundary", "scaling"), "bad raw kind")
    require(row["arm"] in ARMS, "bad arm")
    require(1 <= int(row["worker"]) <= CONTRACT["hosted_workers"], "bad worker")
    require(0 <= int(row["process"]) < CONTRACT["processes_per_worker"], "bad process")
    for key in (
        "n", "u_or_d", "session", "build_ns", "update_native_ns",
        "sync_native_ns", "session_native_ns", "application_bytes", "rounds",
        "fallback", "vmrss_bytes", "vmhwm_bytes", "candidate_capacity",
    ):
        require(int(row[key]) >= 0, f"negative {key}")

    require(
        int(row["session_native_ns"])
        == int(row["update_native_ns"]) + int(row["sync_native_ns"]),
        "raw native phase closure",
    )
    require(int(row["application_bytes"]) > 0, "zero application bytes")
    require(int(row["rounds"]) > 0, "zero rounds")
    require(int(row["vmhwm_bytes"]) >= int(row["vmrss_bytes"]) > 0, "memory accounting")

    arm = str(row["arm"])
    d = int(row["u_or_d"])
    fallback = int(row["fallback"])
    if arm == "direct":
        require(fallback == 0, "direct fallback")
    if arm == "d11":
        false_candidate = int(row.get("false_candidate", -1))
        require(false_candidate in (0, 1), "missing D11 false-candidate flag")
        require(false_candidate <= fallback, "D11 false candidate without fallback")
        if d <= 8 and fallback:
            require(false_candidate == 1, "unexplained d<=8 D11 fallback")
        if d > 8:
            require(fallback == 1, "over-capacity D11 did not fall back")


def load_raw(directory: Path) -> list[dict[str, object]]:
    files = sorted(directory.rglob("raw-worker-*.json"))
    require(len(files) == CONTRACT["hosted_workers"], "missing worker artifacts")

    all_rows: list[dict[str, object]] = []
    seen_workers: set[int] = set()
    expected = expected_rows_per_worker()
    for path in files:
        report = json.loads(path.read_text())
        require(report.get("format") == "deltameter.m6d13b.raw.v1", "raw format")
        require(report.get("performance_decision") == "RAW_ONLY", "premature raw decision")
        worker = int(report["worker"])
        require(worker not in seen_workers, "duplicate worker artifact")
        seen_workers.add(worker)
        rows = report["rows"]
        require(len(rows) == expected, f"worker {worker} row count")
        for row in rows:
            validate_row(row)
            require(int(row["worker"]) == worker, "worker row mismatch")
            all_rows.append(row)

    require(seen_workers == set(range(1, CONTRACT["hosted_workers"] + 1)), "worker set")
    unique: set[tuple[object, ...]] = set()
    for row in all_rows:
        key = (
            row["worker"], row["process"], row["kind"], row["n"], row["u_or_d"],
            row["arm"], row["session"],
        )
        require(key not in unique, f"duplicate raw row {key}")
        unique.add(key)
    return all_rows


def core_summary(rows: list[dict[str, object]]) -> tuple[list[dict[str, object]], list[dict[str, object]], list[dict[str, object]]]:
    core = [row for row in rows if row["kind"] == "core"]
    grouped: dict[tuple[int, int, int, str], list[dict[str, object]]] = defaultdict(list)
    for row in core:
        grouped[
            (int(row["worker"]), int(row["n"]), int(row["u_or_d"]), str(row["arm"]))
        ].append(row)

    expected_group = CONTRACT["processes_per_worker"] * CONTRACT["measured_sessions"]
    for key, group in grouped.items():
        require(len(group) == expected_group, f"incomplete core group {key}")

    modeled: dict[
        tuple[int, int, int, int, int, int, str], float
    ] = {}
    arm_guardrails: list[dict[str, object]] = []

    for (worker, n, updates, arm), group in sorted(grouped.items()):
        arm_guardrails.append(
            {
                "worker": worker,
                "n": n,
                "updates": updates,
                "arm": arm,
                "native_session_median_ns": median(
                    [int(row["session_native_ns"]) for row in group]
                ),
                "bytes_median": median([int(row["application_bytes"]) for row in group]),
                "rounds_median": median([int(row["rounds"]) for row in group]),
                "vmhwm_median_bytes": median([int(row["vmhwm_bytes"]) for row in group]),
                "vmhwm_max_bytes": max(int(row["vmhwm_bytes"]) for row in group),
                "fallbacks": sum(int(row["fallback"]) for row in group),
                "false_candidates": sum(
                    int(row.get("false_candidate", 0)) for row in group
                ),
            }
        )
        for sessions_per_build in CONTRACT["core"]["sessions_per_build_model"]:
            for rtt_ms in CONTRACT["rtt_ms"]:
                for bandwidth_mbps in CONTRACT["bandwidth_mbps"]:
                    values = [
                        modeled_total_ns(
                            int(row["session_native_ns"]),
                            int(row["build_ns"]),
                            sessions_per_build,
                            int(row["application_bytes"]),
                            int(row["rounds"]),
                            rtt_ms,
                            bandwidth_mbps,
                        )
                        for row in group
                    ]
                    modeled[
                        (
                            worker, n, updates, sessions_per_build,
                            rtt_ms, bandwidth_mbps, arm,
                        )
                    ] = median(values)

    cells: list[dict[str, object]] = []
    d11_qualified: list[dict[str, object]] = []
    stream_qualified: list[dict[str, object]] = []

    for n in CONTRACT["core"]["n"]:
        for updates in CONTRACT["core"]["updates_per_session"]:
            for sessions_per_build in CONTRACT["core"]["sessions_per_build_model"]:
                for rtt_ms in CONTRACT["rtt_ms"]:
                    for bandwidth_mbps in CONTRACT["bandwidth_mbps"]:
                        worker_rows = []
                        d11_improvements = []
                        stream_improvements = []
                        pull_improvements = []
                        for worker in range(1, CONTRACT["hosted_workers"] + 1):
                            prefix = (
                                worker, n, updates, sessions_per_build,
                                rtt_ms, bandwidth_mbps,
                            )
                            direct = modeled[prefix + ("direct",)]
                            d11 = modeled[prefix + ("d11",)]
                            pull = modeled[prefix + ("riblt_pull",)]
                            stream = modeled[prefix + ("riblt_stream_lb",)]
                            require(direct > 0, "non-positive direct modeled cost")
                            d11_imp = 1.0 - d11 / direct
                            pull_imp = 1.0 - pull / direct
                            stream_imp = 1.0 - stream / direct
                            d11_improvements.append(d11_imp)
                            pull_improvements.append(pull_imp)
                            stream_improvements.append(stream_imp)
                            worker_rows.append(
                                {
                                    "worker": worker,
                                    "direct_ns": direct,
                                    "d11_ns": d11,
                                    "riblt_pull_ns": pull,
                                    "riblt_stream_lb_ns": stream,
                                    "d11_improvement": d11_imp,
                                    "riblt_pull_improvement": pull_imp,
                                    "riblt_stream_lb_improvement": stream_imp,
                                }
                            )

                        cell = {
                            "n": n,
                            "updates": updates,
                            "sessions_per_build": sessions_per_build,
                            "rtt_ms": rtt_ms,
                            "bandwidth_mbps": bandwidth_mbps,
                            "min_d11_improvement": min(d11_improvements),
                            "median_d11_improvement": median(d11_improvements),
                            "min_riblt_pull_improvement": min(pull_improvements),
                            "median_riblt_pull_improvement": median(pull_improvements),
                            "min_riblt_stream_lb_improvement": min(stream_improvements),
                            "median_riblt_stream_lb_improvement": median(stream_improvements),
                            "workers": worker_rows,
                        }
                        cells.append(cell)
                        threshold = CONTRACT["candidate_threshold_fraction"]
                        if min(d11_improvements) >= threshold:
                            d11_qualified.append(cell)
                        if min(stream_improvements) >= threshold:
                            stream_qualified.append(cell)

    return cells, d11_qualified, stream_qualified, arm_guardrails


def guard_summary(rows: list[dict[str, object]], kind: str) -> list[dict[str, object]]:
    selected = [row for row in rows if row["kind"] == kind]
    grouped: dict[tuple[int, int, str], list[dict[str, object]]] = defaultdict(list)
    for row in selected:
        grouped[(int(row["n"]), int(row["u_or_d"]), str(row["arm"]))].append(row)

    expected_group = CONTRACT["hosted_workers"] * CONTRACT["processes_per_worker"]
    out = []
    for (n, d, arm), group in sorted(grouped.items()):
        require(len(group) == expected_group, f"incomplete {kind} group")
        out.append(
            {
                "kind": kind,
                "n": n,
                "d": d,
                "arm": arm,
                "native_median_ns": median(
                    [int(row["session_native_ns"]) for row in group]
                ),
                "bytes_median": median([int(row["application_bytes"]) for row in group]),
                "rounds_median": median([int(row["rounds"]) for row in group]),
                "fallbacks": sum(int(row["fallback"]) for row in group),
                "false_candidates": sum(
                    int(row.get("false_candidate", 0)) for row in group
                ),
                "vmhwm_median_bytes": median([int(row["vmhwm_bytes"]) for row in group]),
                "vmhwm_max_bytes": max(int(row["vmhwm_bytes"]) for row in group),
            }
        )
    return out


def best_cell(cells: list[dict[str, object]], key: str) -> dict[str, object] | None:
    if not cells:
        return None
    return max(cells, key=lambda cell: float(cell[key]))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    rows = load_raw(args.input)
    cells, d11_qualified, stream_qualified, guardrails = core_summary(rows)
    boundary = guard_summary(rows, "boundary")
    scaling = guard_summary(rows, "scaling")

    if d11_qualified and stream_qualified:
        verdict = "MIXED_CONDITIONAL_PRIVATE_ONLY"
    elif d11_qualified:
        verdict = "D11_CONDITIONAL_PRIVATE_ONLY"
    elif stream_qualified:
        verdict = "RATELESS_RESEARCH_ONLY"
    else:
        verdict = "STOP_SYSTEM_PRODUCT"

    report = {
        "format": "deltameter.m6d13b.summary.v1",
        "verdict": verdict,
        "production_exactsmall_delta": "NO_GO",
        "workers": CONTRACT["hosted_workers"],
        "processes_per_worker": CONTRACT["processes_per_worker"],
        "raw_rows": len(rows),
        "named_network_cells": len(cells),
        "d11_qualified_cells": len(d11_qualified),
        "stream_lb_qualified_cells": len(stream_qualified),
        "best_d11_cell": best_cell(cells, "min_d11_improvement"),
        "best_stream_lb_cell": best_cell(cells, "min_riblt_stream_lb_improvement"),
        "qualified_d11": d11_qualified,
        "qualified_stream_lb": stream_qualified,
        "core_guardrails": guardrails,
        "boundary_guardrails": boundary,
        "scaling_guardrails": scaling,
        "cells": cells,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n")

    best_d11 = report["best_d11_cell"]
    best_stream = report["best_stream_lb_cell"]
    print(
        f"D13B_VERDICT={verdict} raw_rows={len(rows)} cells={len(cells)} "
        f"d11_qualified={len(d11_qualified)} "
        f"stream_lb_qualified={len(stream_qualified)}"
    )
    if best_d11 is not None:
        print(
            "BEST_D11 "
            f"n={best_d11['n']} u={best_d11['updates']} "
            f"S={best_d11['sessions_per_build']} rtt={best_d11['rtt_ms']} "
            f"bw={best_d11['bandwidth_mbps']} "
            f"min_improvement={best_d11['min_d11_improvement']:.6f} "
            f"median_improvement={best_d11['median_d11_improvement']:.6f}"
        )
    if best_stream is not None:
        print(
            "BEST_STREAM_LB "
            f"n={best_stream['n']} u={best_stream['updates']} "
            f"S={best_stream['sessions_per_build']} rtt={best_stream['rtt_ms']} "
            f"bw={best_stream['bandwidth_mbps']} "
            f"min_improvement={best_stream['min_riblt_stream_lb_improvement']:.6f} "
            f"median_improvement={best_stream['median_riblt_stream_lb_improvement']:.6f}"
        )

    for item in boundary:
        if item["arm"] == "d11":
            print(
                f"BOUNDARY_D11 d={item['d']} fallbacks={item['fallbacks']} "
                f"false_candidates={item['false_candidates']} "
                f"native_median_ns={item['native_median_ns']:.0f}"
            )
    for item in scaling:
        if item["arm"] in ("direct", "d11", "riblt_stream_lb"):
            print(
                f"SCALING arm={item['arm']} d={item['d']} "
                f"fallbacks={item['fallbacks']} "
                f"native_median_ns={item['native_median_ns']:.0f} "
                f"vmhwm_median={item['vmhwm_median_bytes']:.0f}"
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
