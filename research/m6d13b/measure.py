#!/usr/bin/env python3
"""M6-D13-B hosted system-comparison controller.

Python orchestrates only. All algorithm timings come from the frozen native B0
workers; subprocess/controller elapsed time is intentionally absent.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "m6d13b0"))

from model import CONTRACT, d11_wire, direct_wire, require, riblt_wire  # noqa: E402
from readiness import (  # noqa: E402
    parse_record,
    validate_check,
    validate_memory,
    validate_ready,
    validate_riblt_check,
    validate_riblt_ready,
    validate_riblt_sync,
    validate_riblt_update,
    validate_sync,
    validate_update,
)


MASK64 = (1 << 64) - 1
ARMS = ["direct", "d11", "riblt_pull", "riblt_stream_lb"]


class Worker:
    def __init__(self, path: str):
        self.process = subprocess.Popen(
            [str(Path(path).resolve())],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def request(self, line: str) -> str:
        if self.process.poll() is not None:
            raise RuntimeError("worker exited")
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(line + "\n")
        self.process.stdin.flush()
        result = self.process.stdout.readline()
        if not result:
            raise RuntimeError("worker EOF")
        return result.rstrip("\n")

    def close(self) -> None:
        if self.process.poll() is None:
            assert self.process.stdin is not None
            self.process.stdin.write("quit\n")
            self.process.stdin.flush()
            self.process.stdin.close()
        self.process.wait(timeout=30)


def validate_rust_pair_ready(
    line: str, mode: str, left_len: int, right_len: int
) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "ready", "not Rust pair-ready record")
    for key in (
        "source_build_ns",
        "sketch_build_ns",
        "clk_tck",
        "source_len",
        "source_a_cap",
        "source_b_cap",
        "sketch_payload_bytes",
    ):
        require(key in fields, f"missing Rust pair-ready {key}")
    require(fields["clk_tck"] > 0, "invalid Rust pair CLK_TCK")
    require(fields["source_len"] == left_len, "Rust pair left length")
    require(fields["source_a_cap"] >= left_len, "Rust pair A capacity")
    require(fields["source_b_cap"] >= right_len, "Rust pair B capacity")
    require(
        fields["sketch_payload_bytes"] == (146 if mode == "d11" else 0),
        "Rust pair sketch payload",
    )
    validate_memory(fields)
    return fields


def validate_go_pair_ready(
    line: str, left_len: int, right_len: int
) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "ready", "not RIBLT pair-ready record")
    for key in (
        "source_build_ns",
        "clk_tck",
        "source_len",
        "source_a_cap",
        "source_b_cap",
        "runtime_alloc_bytes",
        "runtime_heap_sys_bytes",
    ):
        require(key in fields, f"missing RIBLT pair-ready {key}")
    require(fields["clk_tck"] > 0, "invalid RIBLT pair CLK_TCK")
    require(fields["source_len"] == left_len, "RIBLT pair left length")
    require(fields["source_a_cap"] >= left_len, "RIBLT pair A capacity")
    require(fields["source_b_cap"] >= right_len, "RIBLT pair B capacity")
    require(
        fields["runtime_heap_sys_bytes"] >= fields["runtime_alloc_bytes"],
        "RIBLT pair heap accounting",
    )
    validate_memory(fields)
    return fields


def key_text(keys: list[int]) -> str:
    return ",".join(f"{key:x}" for key in keys) if keys else "-"


def splitmix64(value: int) -> int:
    z = (value + 0x9E3779B97F4A7C15) & MASK64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
    return (z ^ (z >> 31)) & MASK64


def next_insert(current: set[int], seed: int, counter: int) -> tuple[int, int]:
    while True:
        value = splitmix64(seed + counter) | (1 << 63)
        counter += 1
        if value not in current:
            return value, counter


def build_schedule(n: int, updates: int, sessions: int, seed: int) -> list[list[tuple[str, int]]]:
    current = set(range(1, n + 1))
    counter = 0
    result: list[list[tuple[str, int]]] = []
    for session in range(sessions):
        ops: list[tuple[str, int]] = []
        if updates == 1:
            if session % 2 == 0:
                value, counter = next_insert(current, seed, counter)
                current.add(value)
                ops.append(("insert", value))
            else:
                value = min(current)
                current.remove(value)
                ops.append(("delete", value))
        else:
            removes = updates // 2
            adds = updates - removes
            for value in sorted(current)[:removes]:
                current.remove(value)
                ops.append(("delete", value))
            for _ in range(adds):
                value, counter = next_insert(current, seed, counter)
                current.add(value)
                ops.append(("insert", value))
        if len(ops) != updates:
            raise ValueError("schedule update count")
        result.append(ops)
    return result


def make_pair(n: int, d: int, seed: int) -> tuple[list[int], list[int]]:
    if d < 0 or d > 2 * n:
        raise ValueError("invalid d")
    right = list(range(1, n + 1))
    left = set(right)
    removes = d // 2
    adds = d - removes
    for value in right[:removes]:
        left.remove(value)
    counter = 0
    for _ in range(adds):
        value, counter = next_insert(left, seed, counter)
        left.add(value)
    out = sorted(left)
    if len(set(out) ^ set(right)) != d:
        raise ValueError("pair d mismatch")
    return out, right


def rust_arm(
    worker_path: str,
    arm: str,
    initial_left: list[int],
    initial_right: list[int],
    schedules: list[list[tuple[str, int]]] | None,
    warmups: int,
    worker_id: int,
    process_id: int,
    kind: str,
    n: int,
    u_or_d: int,
) -> list[dict[str, object]]:
    mode = "direct" if arm == "direct" else "d11"
    worker = Worker(worker_path)
    rows: list[dict[str, object]] = []
    try:
        ready_line = worker.request(
            f"init_pair {mode} {key_text(initial_left)} {key_text(initial_right)}"
        )
        if schedules is None:
            ready = validate_rust_pair_ready(
                ready_line, mode, len(initial_left), len(initial_right)
            )
        else:
            ready = validate_ready(ready_line, mode)
        build_ns = ready["source_build_ns"] + ready["sketch_build_ns"]

        session_schedules = schedules if schedules is not None else [[]]
        for session, ops in enumerate(session_schedules):
            update_ns = 0
            for operation, key in ops:
                update = validate_update(
                    worker.request(f"update {operation} {key:x}"),
                    mode,
                    True,
                )
                update_ns += update["native_total_ns"]

            sync = validate_sync(worker.request("sync"), mode)
            validate_check(worker.request("check"))

            if mode == "direct":
                application_bytes, rounds = direct_wire(sync)
                false_candidate = 0
            else:
                application_bytes, rounds = d11_wire(sync)
                false_candidate = sync["false_candidate"]
                if u_or_d <= 8 and sync["fallback"]:
                    if false_candidate != 1:
                        raise ValueError(
                            f"unexplained d<=8 D11 fallback kind={kind} n={n} "
                            f"d={u_or_d} worker={worker_id} process={process_id}"
                        )
                if u_or_d > 8 and sync["fallback"] != 1:
                    raise ValueError("over-capacity D11 completed without fallback")

            if session < warmups:
                continue

            rows.append(
                {
                    "kind": kind,
                    "worker": worker_id,
                    "process": process_id,
                    "arm": arm,
                    "n": n,
                    "u_or_d": u_or_d,
                    "session": session - warmups,
                    "build_ns": build_ns,
                    "update_native_ns": update_ns,
                    "sync_native_ns": sync["native_total_ns"],
                    "session_native_ns": update_ns + sync["native_total_ns"],
                    "application_bytes": application_bytes,
                    "rounds": rounds,
                    "fallback": sync["fallback"],
                    "false_candidate": false_candidate,
                    "final_k": sync["final_k"],
                    "vmrss_bytes": sync["vmrss_bytes"],
                    "vmhwm_bytes": sync["vmhwm_bytes"],
                    "candidate_capacity": sync["candidate_capacity"],
                }
            )
        return rows
    finally:
        worker.close()


def riblt_arm(
    worker_path: str,
    arm: str,
    initial_left: list[int],
    initial_right: list[int],
    schedules: list[list[tuple[str, int]]] | None,
    warmups: int,
    worker_id: int,
    process_id: int,
    kind: str,
    n: int,
    u_or_d: int,
) -> list[dict[str, object]]:
    lane = "pull" if arm == "riblt_pull" else "stream"
    worker = Worker(worker_path)
    rows: list[dict[str, object]] = []
    try:
        ready_line = worker.request(
            f"init_pair {key_text(initial_left)} {key_text(initial_right)}"
        )
        if schedules is None:
            ready = validate_go_pair_ready(
                ready_line, len(initial_left), len(initial_right)
            )
        else:
            ready = validate_riblt_ready(ready_line)
        build_ns = ready["source_build_ns"]

        session_schedules = schedules if schedules is not None else [[]]
        for session, ops in enumerate(session_schedules):
            update_ns = 0
            for operation, key in ops:
                update = validate_riblt_update(
                    worker.request(f"update {operation} {key:x}"),
                    True,
                )
                update_ns += update["native_total_ns"]

            sync = validate_riblt_sync(
                worker.request(f"sync {lane} {CONTRACT['riblt_limit_cells']}"),
                lane,
            )
            validate_riblt_check(worker.request("check"))
            application_bytes, rounds, fallback_kind = riblt_wire(sync, lane)

            if session < warmups:
                continue

            rows.append(
                {
                    "kind": kind,
                    "worker": worker_id,
                    "process": process_id,
                    "arm": arm,
                    "n": n,
                    "u_or_d": u_or_d,
                    "session": session - warmups,
                    "build_ns": build_ns,
                    "update_native_ns": update_ns,
                    "sync_native_ns": sync["native_total_ns"],
                    "session_native_ns": update_ns + sync["native_total_ns"],
                    "application_bytes": application_bytes,
                    "rounds": rounds,
                    "fallback": sync["fallback"],
                    "fallback_kind": fallback_kind,
                    "cells": sync["cells"],
                    "batches": sync["batches"],
                    "vmrss_bytes": sync["vmrss_bytes"],
                    "vmhwm_bytes": sync["vmhwm_bytes"],
                    "runtime_alloc_bytes": sync["runtime_alloc_bytes"],
                    "runtime_heap_sys_bytes": sync["runtime_heap_sys_bytes"],
                    "candidate_capacity": sync["remote_cap"] + sync["local_cap"],
                }
            )
        return rows
    finally:
        worker.close()


def run_arm(
    rust_worker: str,
    riblt_worker: str,
    arm: str,
    left: list[int],
    right: list[int],
    schedules: list[list[tuple[str, int]]] | None,
    warmups: int,
    worker_id: int,
    process_id: int,
    kind: str,
    n: int,
    u_or_d: int,
) -> list[dict[str, object]]:
    if arm in ("direct", "d11"):
        return rust_arm(
            rust_worker, arm, left, right, schedules, warmups,
            worker_id, process_id, kind, n, u_or_d
        )
    return riblt_arm(
        riblt_worker, arm, left, right, schedules, warmups,
        worker_id, process_id, kind, n, u_or_d
    )


def rotate_arms(worker_id: int, process_id: int) -> list[str]:
    shift = (worker_id + process_id) % len(ARMS)
    return ARMS[shift:] + ARMS[:shift]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-worker", required=True)
    parser.add_argument("--riblt-worker", required=True)
    parser.add_argument("--worker-id", type=int, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    if args.worker_id not in range(1, CONTRACT["hosted_workers"] + 1):
        raise SystemExit("invalid worker id")

    rows: list[dict[str, object]] = []
    for process_id in range(CONTRACT["processes_per_worker"]):
        order = rotate_arms(args.worker_id, process_id)

        for n in CONTRACT["core"]["n"]:
            base = list(range(1, n + 1))
            for updates in CONTRACT["core"]["updates_per_session"]:
                total_sessions = CONTRACT["warmup_sessions"] + CONTRACT["measured_sessions"]
                seed = (
                    (args.worker_id << 52)
                    ^ (process_id << 44)
                    ^ (n << 8)
                    ^ updates
                    ^ 0xD13B000000000000
                ) & MASK64
                schedules = build_schedule(n, updates, total_sessions, seed)
                for arm in order:
                    rows.extend(
                        run_arm(
                            args.rust_worker, args.riblt_worker, arm,
                            base, base, schedules, CONTRACT["warmup_sessions"],
                            args.worker_id, process_id, "core", n, updates,
                        )
                    )

        boundary_n = CONTRACT["boundary"]["n"]
        for d in CONTRACT["boundary"]["d"]:
            seed = (
                0xB0A0000000000000
                ^ (args.worker_id << 40)
                ^ (process_id << 32)
                ^ d
            ) & MASK64
            left, right = make_pair(boundary_n, d, seed)
            for arm in order:
                rows.extend(
                    run_arm(
                        args.rust_worker, args.riblt_worker, arm,
                        left, right, None, 0,
                        args.worker_id, process_id, "boundary", boundary_n, d,
                    )
                )

        scale_n = CONTRACT["scaling"]["n"]
        for d in CONTRACT["scaling"]["d"]:
            seed = (
                0x5CA1E00000000000
                ^ (args.worker_id << 40)
                ^ (process_id << 32)
                ^ d
            ) & MASK64
            left, right = make_pair(scale_n, d, seed)
            for arm in order:
                rows.extend(
                    run_arm(
                        args.rust_worker, args.riblt_worker, arm,
                        left, right, None, 0,
                        args.worker_id, process_id, "scaling", scale_n, d,
                    )
                )

    expected_core = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["core"]["n"])
        * len(CONTRACT["core"]["updates_per_session"])
        * len(ARMS)
        * CONTRACT["measured_sessions"]
    )
    expected_boundary = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["boundary"]["d"])
        * len(ARMS)
    )
    expected_scaling = (
        CONTRACT["processes_per_worker"]
        * len(CONTRACT["scaling"]["d"])
        * len(ARMS)
    )
    if len(rows) != expected_core + expected_boundary + expected_scaling:
        raise ValueError(
            f"incomplete worker rows {len(rows)} expected "
            f"{expected_core + expected_boundary + expected_scaling}"
        )

    report = {
        "format": "deltameter.m6d13b.raw.v1",
        "worker": args.worker_id,
        "rows": rows,
        "performance_decision": "RAW_ONLY",
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"D13B_RAW_PASS worker={args.worker_id} rows={len(rows)} "
        "performance_decision=RAW_ONLY"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
