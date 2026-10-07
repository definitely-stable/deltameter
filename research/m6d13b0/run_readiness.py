#!/usr/bin/env python3
"""M6-D13-B0 persistent-chain and sparse-scaling readiness inventory."""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from readiness import (
    CONTRACT,
    validate_check,
    validate_diagnose,
    validate_ready,
    validate_riblt_check,
    validate_riblt_ready,
    validate_riblt_sync,
    validate_riblt_update,
    validate_sync,
    validate_update,
)


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
        self.process.wait(timeout=10)


def key_text(keys: list[int]) -> str:
    return ",".join(f"{key:x}" for key in keys) if keys else "-"


def base_keys(n: int) -> list[int]:
    return list(range(1, n + 1))


def operations(current: set[int], updates: int, session: int, next_new: int):
    result: list[tuple[str, int]] = []
    if updates == 0:
        return result, next_new

    if updates == 1:
        if session % 2 == 0:
            while next_new in current:
                next_new += 1
            result.append(("insert", next_new))
            current.add(next_new)
            next_new += 1
        else:
            key = min(current)
            result.append(("delete", key))
            current.remove(key)
        return result, next_new

    removes = updates // 2
    adds = updates - removes
    for key in sorted(current)[:removes]:
        result.append(("delete", key))
        current.remove(key)
    for _ in range(adds):
        while next_new in current:
            next_new += 1
        result.append(("insert", next_new))
        current.add(next_new)
        next_new += 1
    return result, next_new


def run_rust_chain(worker: Worker, mode: str, updates: int, sessions: int):
    base = base_keys(CONTRACT["chain_n"])
    validate_ready(worker.request(f"init {mode} {key_text(base)}"), mode)
    oracle = set(base)
    next_new = (1 << 63) + updates * 1_000_000 + sessions * 10_000
    rows = []

    for session in range(sessions):
        ops, next_new = operations(oracle, updates, session, next_new)
        if len(ops) != updates:
            raise ValueError("wrong update count")
        for operation, key in ops:
            validate_update(
                worker.request(f"update {operation} {key:x}"),
                mode,
                True,
            )

        diagnose = None
        if mode == "d11":
            diagnose = validate_diagnose(worker.request("diagnose"))
            if diagnose["exact_d"] != updates:
                raise ValueError(
                    f"unexpected D11 exact_d={diagnose['exact_d']} updates={updates} "
                    f"sessions={sessions} session={session}"
                )

        sync = validate_sync(worker.request("sync"), mode)
        if mode == "direct":
            if sync["fallback"] != 0:
                raise ValueError("direct fallback")
        elif updates <= 8:
            if sync["fallback"] != 0:
                # A false candidate at an earlier stage k<d is an over-capacity
                # guard event, not an in-capacity decoder failure. Protocol v2
                # verifies it independently and charges an exact fallback.
                # Any other fallback for d<=8 remains a hard readiness failure.
                if not (
                    sync["false_candidate"] == 1
                    and diagnose["maintained_decoded"] == 1
                    and diagnose["maintained_exact"] == 0
                    and diagnose["fresh_decoded"] == 1
                    and diagnose["fresh_exact"] == 0
                    and diagnose["maintained_k"] < updates
                ):
                    raise ValueError(
                        "unexplained in-capacity D11 fallback "
                        f"updates={updates} sessions={sessions} session={session} "
                        f"sync={sync} diagnose={diagnose}"
                    )
            elif sync["false_candidate"] != 0:
                raise ValueError("D11 false-candidate flag without fallback")
        else:
            if sync["fallback"] != 1:
                raise ValueError("over-capacity D11 did not fall back")
        validate_check(worker.request("check"))
        rows.append(
            {
                "arm": mode,
                "updates": updates,
                "sessions": sessions,
                "session": session,
                "diagnose": diagnose,
                "metrics": sync,
            }
        )
    return rows


def run_riblt_chain(worker: Worker, lane: str, updates: int, sessions: int):
    base = base_keys(CONTRACT["chain_n"])
    validate_riblt_ready(worker.request(f"init {key_text(base)}"))
    oracle = set(base)
    next_new = (1 << 62) + updates * 1_000_000 + sessions * 10_000
    rows = []

    for session in range(sessions):
        ops, next_new = operations(oracle, updates, session, next_new)
        if len(ops) != updates:
            raise ValueError("wrong RIBLT update count")
        for operation, key in ops:
            validate_riblt_update(
                worker.request(f"update {operation} {key:x}"),
                True,
            )

        sync = validate_riblt_sync(worker.request(f"sync {lane} 1024"), lane)
        if sync["fallback"] != 0:
            raise ValueError(f"{lane} unexpectedly fell back for d={updates}")
        validate_riblt_check(worker.request("check"))
        rows.append(
            {
                "arm": f"riblt_{lane}",
                "updates": updates,
                "sessions": sessions,
                "session": session,
                "metrics": sync,
            }
        )
    return rows


def balanced_pair(n: int, d: int):
    if d % 2:
        raise ValueError("scaling d must be even")
    base = base_keys(n)
    half = d // 2
    right = base
    left = base[half:] + list(range(n + 1, n + half + 1))
    left.sort()
    if len(left) != n or len(set(left) ^ set(right)) != d:
        raise ValueError("bad scaling pair")
    return left, right


def run_scaling(rust: Worker, riblt: Worker):
    rows = []
    spec = CONTRACT["scaling"][0]
    n = spec["n"]
    for d in spec["d"]:
        left, right = balanced_pair(n, d)
        left_text, right_text = key_text(left), key_text(right)

        validate_ready(
            rust.request(f"init_pair direct {left_text} {right_text}"),
            "direct",
        )
        direct = validate_sync(rust.request("sync"), "direct")
        validate_check(rust.request("check"))
        rows.append({"arm": "direct", "n": n, "d": d, "metrics": direct})

        validate_ready(
            rust.request(f"init_pair d11 {left_text} {right_text}"),
            "d11",
        )
        d11 = validate_sync(rust.request("sync"), "d11")
        if d11["fallback"] != 1:
            raise ValueError("scaling D11 must use exact fallback")
        validate_check(rust.request("check"))
        rows.append({"arm": "d11", "n": n, "d": d, "metrics": d11})

        for lane in ("pull", "stream"):
            validate_riblt_ready(
                riblt.request(f"init_pair {left_text} {right_text}")
            )
            result = validate_riblt_sync(
                riblt.request(f"sync {lane} 1024"),
                lane,
            )
            validate_riblt_check(riblt.request("check"))
            rows.append(
                {"arm": f"riblt_{lane}", "n": n, "d": d, "metrics": result}
            )

    d = spec["natural_exhaustion_d"]
    left, right = balanced_pair(n, d)
    left_text, right_text = key_text(left), key_text(right)
    for lane in ("pull", "stream"):
        validate_riblt_ready(riblt.request(f"init_pair {left_text} {right_text}"))
        result = validate_riblt_sync(riblt.request(f"sync {lane} 1024"), lane)
        if result["fallback"] != 1:
            raise ValueError(f"natural {lane} exhaustion did not fall back")
        validate_riblt_check(riblt.request("check"))
        rows.append(
            {
                "arm": f"riblt_{lane}",
                "n": n,
                "d": d,
                "control": "natural_exhaustion",
                "metrics": result,
            }
        )
    return rows


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-worker", required=True)
    parser.add_argument("--riblt-worker", required=True)
    parser.add_argument("--mode", choices=("chains", "scaling"), required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    rust = Worker(args.rust_worker)
    riblt = Worker(args.riblt_worker)
    try:
        if args.mode == "chains":
            rows = []
            for updates in CONTRACT["updates_per_session"]:
                for sessions in CONTRACT["sessions_per_build"]:
                    rows.extend(run_rust_chain(rust, "direct", updates, sessions))
                    rows.extend(run_rust_chain(rust, "d11", updates, sessions))
                    rows.extend(run_riblt_chain(riblt, "pull", updates, sessions))
                    rows.extend(run_riblt_chain(riblt, "stream", updates, sessions))
            decision = "CHAIN_READINESS_PASS"
        else:
            rows = run_scaling(rust, riblt)
            decision = "SCALING_READINESS_PASS"

        report = {
            "format": "deltameter.m6d13b0.inventory.v1",
            "mode": args.mode,
            "rows": rows,
            "decision": decision,
            "performance_decision": "NOT_MEASURED",
        }
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(report, indent=2) + "\n")
        print(f"{decision} rows={len(rows)} performance_decision=NOT_MEASURED")
        return 0
    finally:
        rust.close()
        riblt.close()


if __name__ == "__main__":
    raise SystemExit(main())
