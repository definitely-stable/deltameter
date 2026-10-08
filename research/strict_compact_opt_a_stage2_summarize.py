#!/usr/bin/env python3
"""Fail-closed five-worker aggregation for STRICT-COMPACT OPT-A Stage 2."""

from __future__ import annotations

import argparse
import json
import math
import statistics
from pathlib import Path

WORKERS = (1, 2, 3, 4, 5)
J_VALUES = (24, 40, 56, 64)
ROUNDS = tuple(range(8))
CONTROL_J = 64
FULL_DOMAIN = (1 << 64) - 1
MAX_UPDATE_REGRESSION = 0.15
MAX_ESTIMATE_REGRESSION = 0.25
MAX_XOR_REGRESSION = 0.25


def parse_sample(line: str) -> dict | None:
    prefix = "STRICT_COMPACT_OPT_A_STAGE2_SAMPLE "
    if not line.startswith(prefix):
        return None

    fields: dict[str, str] = {}
    for part in line[len(prefix) :].split():
        key, value = part.split("=", 1)
        fields[key] = value

    required = {
        "worker",
        "round",
        "J",
        "layout",
        "state_bytes",
        "update_ns_per_token",
        "estimate_ns",
        "xor_ns_per_kib",
    }
    if set(fields) != required:
        raise ValueError(f"unexpected sample fields {sorted(fields)}")

    row = {
        "worker": int(fields["worker"]),
        "round": int(fields["round"]),
        "stored_levels": int(fields["J"]),
        "layout": fields["layout"],
        "state_bytes": int(fields["state_bytes"]),
        "update_ns_per_token": float(fields["update_ns_per_token"]),
        "estimate_ns": float(fields["estimate_ns"]),
        "xor_ns_per_kib": float(fields["xor_ns_per_kib"]),
    }
    if row["worker"] not in WORKERS:
        raise ValueError("worker")
    if row["round"] not in ROUNDS:
        raise ValueError("round")
    if row["stored_levels"] not in J_VALUES:
        raise ValueError("J")
    if row["layout"] != "level":
        raise ValueError("Stage 2 must use level-major")
    expected_bytes = 4096 * row["stored_levels"] // 8
    if row["state_bytes"] != expected_bytes:
        raise ValueError("state bytes")
    for metric in ("update_ns_per_token", "estimate_ns", "xor_ns_per_kib"):
        if not math.isfinite(row[metric]) or row[metric] <= 0:
            raise ValueError(metric)
    return row


def regression(candidate: float, control: float) -> float:
    return candidate / control - 1.0


def build_payload(stage1_path: Path, logs_dir: Path, expected_head: str) -> dict:
    import re
    if re.fullmatch(r"[0-9a-f]{40}", expected_head) is None:
        raise AssertionError("invalid expected source SHA")
    stage1 = json.loads(stage1_path.read_text(encoding="utf-8"))
    if stage1["source_head"] != "0ab6824b6e5cd7e7967eb4bb66d2a09495b6121f":
        raise AssertionError("unexpected frozen Stage-1 source head")
    if stage1["stage2_protocol_v2_shortlist"] != [64, 24, 40, 56]:
        raise AssertionError("unexpected Stage-2 shortlist")

    profiles = {}
    for raw in stage1["profiles"]:
        row = dict(raw)
        # The frozen wire bridge requires strings; floats/JSON numbers can round u64.
        if not isinstance(row["d_max"], str) or not row["d_max"].isdigit():
            raise AssertionError("d_max must be an exact decimal string")
        if (row["first_uncertified_d"] is not None and
                (not isinstance(row["first_uncertified_d"], str) or
                 not row["first_uncertified_d"].isdigit())):
            raise AssertionError("first_uncertified_d must be a decimal string")
        row["d_max"] = int(row["d_max"])
        if row["first_uncertified_d"] is not None:
            row["first_uncertified_d"] = int(row["first_uncertified_d"])
        profiles[row["stored_levels"]] = row
    if set(profiles) != {24, 32, 40, 48, 56, 64}:
        raise AssertionError("frozen Stage-1 profile set")

    log_paths = sorted(logs_dir.glob("worker-*.txt"))
    if len(log_paths) != len(WORKERS):
        raise AssertionError(
            f"expected {len(WORKERS)} worker logs, found {len(log_paths)}"
        )

    head_paths = sorted(logs_dir.glob("head-*.txt"))
    if len(head_paths) != len(WORKERS):
        raise AssertionError("incomplete per-worker source HEAD provenance")
    for worker in WORKERS:
        source = logs_dir / f"head-{worker}.txt"
        if not source.is_file() or source.read_text(encoding="utf-8").strip() != expected_head:
            raise AssertionError(f"worker={worker}: source HEAD mismatch")

    samples = []
    pass_workers: set[int] = set()
    for path in log_paths:
        for line in path.read_text(encoding="utf-8").splitlines():
            row = parse_sample(line)
            if row is not None:
                samples.append(row)
            prefix = "STRICT_COMPACT_OPT_A_STAGE2_WORKER_PASS worker="
            if line.startswith(prefix):
                pass_workers.add(int(line[len(prefix) :]))

    if pass_workers != set(WORKERS):
        raise AssertionError(f"incomplete worker PASS set: {sorted(pass_workers)}")

    expected_count = len(WORKERS) * len(J_VALUES) * len(ROUNDS)
    if len(samples) != expected_count:
        raise AssertionError(
            f"expected {expected_count} raw samples, got {len(samples)}"
        )

    seen: set[tuple[int, int, int]] = set()
    for row in samples:
        key = (row["worker"], row["stored_levels"], row["round"])
        if key in seen:
            raise AssertionError(f"duplicate raw sample {key}")
        seen.add(key)

    worker_rows = []
    viability: dict[int, list[bool]] = {levels: [] for levels in J_VALUES}

    for worker in WORKERS:
        medians: dict[int, dict[str, float]] = {}
        for levels in J_VALUES:
            selected = [
                row
                for row in samples
                if row["worker"] == worker and row["stored_levels"] == levels
            ]
            if len(selected) != len(ROUNDS):
                raise AssertionError(
                    f"incomplete samples worker={worker} J={levels}"
                )
            medians[levels] = {
                metric: statistics.median(row[metric] for row in selected)
                for metric in (
                    "update_ns_per_token",
                    "estimate_ns",
                    "xor_ns_per_kib",
                )
            }

        control = medians[CONTROL_J]
        for levels in J_VALUES:
            current = medians[levels]
            update_reg = regression(
                current["update_ns_per_token"],
                control["update_ns_per_token"],
            )
            estimate_reg = regression(
                current["estimate_ns"], control["estimate_ns"]
            )
            xor_reg = regression(
                current["xor_ns_per_kib"], control["xor_ns_per_kib"]
            )
            passes = (
                update_reg <= MAX_UPDATE_REGRESSION
                and estimate_reg <= MAX_ESTIMATE_REGRESSION
                and xor_reg <= MAX_XOR_REGRESSION
            )
            viability[levels].append(passes)
            worker_rows.append(
                {
                    "worker": worker,
                    "stored_levels": levels,
                    **current,
                    "update_regression_vs_j64": update_reg,
                    "estimate_regression_vs_j64": estimate_reg,
                    "xor_regression_vs_j64": xor_reg,
                    "viability_pass": passes,
                }
            )

    candidate_rows = []
    for levels in J_VALUES:
        passes_all = all(viability[levels])
        profile = profiles[levels]
        selected_workers = [
            row for row in worker_rows if row["stored_levels"] == levels
        ]
        candidate_rows.append(
            {
                "stored_levels": levels,
                "state_bytes": profile["state_bytes"],
                "d_max": profile["d_max"],
                "full_domain": profile["d_max"] == FULL_DOMAIN,
                "all_workers_viability_pass": passes_all,
                "worker_median_update_ns": statistics.median(
                    row["update_ns_per_token"] for row in selected_workers
                ),
                "worker_median_estimate_ns": statistics.median(
                    row["estimate_ns"] for row in selected_workers
                ),
                "worker_median_xor_ns_per_kib": statistics.median(
                    row["xor_ns_per_kib"] for row in selected_workers
                ),
                "worst_worker_update_regression": max(
                    row["update_regression_vs_j64"] for row in selected_workers
                ),
                "worst_worker_estimate_regression": max(
                    row["estimate_regression_vs_j64"] for row in selected_workers
                ),
                "worst_worker_xor_regression": max(
                    row["xor_regression_vs_j64"] for row in selected_workers
                ),
            }
        )

    viable_smaller = [
        row
        for row in candidate_rows
        if row["stored_levels"] < CONTROL_J
        and row["all_workers_viability_pass"]
    ]
    if not viable_smaller:
        decision = "KEEP_J64"
        full_range_min_j = CONTROL_J
    else:
        decision = "RANGE_FRONTIER_PASS"
        full_range = [
            row
            for row in candidate_rows
            if row["full_domain"] and row["all_workers_viability_pass"]
        ]
        full_range_min_j = min(
            row["stored_levels"] for row in full_range
        ) if full_range else CONTROL_J

    full_range_profile = next(
        row for row in candidate_rows if row["stored_levels"] == full_range_min_j
    )
    state_reduction = 1.0 - (
        full_range_profile["state_bytes"] / profiles[CONTROL_J]["state_bytes"]
    )

    return {
        "format": "deltameter.strict-compact-opt-a-stage2.v1",
        "validated_source_head": expected_head,
        "raw_sample_count": len(samples),
        "workers": list(WORKERS),
        "rounds_per_worker_profile": len(ROUNDS),
        "layout": "level",
        "gates": {
            "max_update_regression": MAX_UPDATE_REGRESSION,
            "max_estimate_regression": MAX_ESTIMATE_REGRESSION,
            "max_xor_regression": MAX_XOR_REGRESSION,
        },
        "worker_medians": worker_rows,
        "candidates": candidate_rows,
        "full_range_min_viable_j": full_range_min_j,
        "full_range_state_reduction_vs_j64": state_reduction,
        "decision": decision,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage1", type=Path, required=True)
    parser.add_argument("--logs-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--head", required=True, help="exact checked-out source SHA")
    args = parser.parse_args()

    payload = build_payload(args.stage1, args.logs_dir, args.head)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    for row in payload["candidates"]:
        print(
            "STRICT_COMPACT_OPT_A_STAGE2_CANDIDATE "
            f"J={row['stored_levels']} state_bytes={row['state_bytes']} "
            f"d_max={row['d_max']} pass={row['all_workers_viability_pass']} "
            f"update_ns={row['worker_median_update_ns']:.6f} "
            f"estimate_ns={row['worker_median_estimate_ns']:.3f} "
            f"xor_ns_per_kib={row['worker_median_xor_ns_per_kib']:.6f} "
            f"worst_update_reg={row['worst_worker_update_regression']:.6f} "
            f"worst_estimate_reg={row['worst_worker_estimate_regression']:.6f} "
            f"worst_xor_reg={row['worst_worker_xor_regression']:.6f}"
        )

    print(
        "STRICT_COMPACT_OPT_A_STAGE2_PASS "
        f"decision={payload['decision']} "
        f"full_range_min_j={payload['full_range_min_viable_j']} "
        f"state_reduction={payload['full_range_state_reduction_vs_j64']:.6f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
