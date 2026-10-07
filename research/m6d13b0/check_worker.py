#!/usr/bin/env python3
"""Hosted integration check for the persistent Rust B0 readiness worker."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from readiness import (
    validate_check,
    validate_diagnose,
    validate_ready,
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
        self.process.wait(timeout=5)


def key_text(keys: list[int]) -> str:
    return ",".join(f"{key:x}" for key in keys) if keys else "-"


def main() -> int:
    if len(sys.argv) != 3:
        raise SystemExit("usage: check_worker.py WORKER OUT.json")

    worker = Worker(sys.argv[1])
    report: dict[str, object] = {"format": "deltameter.m6d13b0.rust-partial.v1"}
    try:
        base = [1, 3, 5, 7]

        ready = worker.request(f"init direct {key_text(base)}")
        report["direct_ready"] = validate_ready(ready, "direct")

        rejected = worker.request("update insert 3")
        report["direct_reject"] = validate_update(rejected, "direct", False)

        updated = worker.request("update insert 9")
        report["direct_update"] = validate_update(updated, "direct", True)

        synced = worker.request("sync")
        report["direct_sync"] = validate_sync(synced, "direct")
        report["direct_check"] = validate_check(worker.request("check"))

        ready = worker.request(f"init d11 {key_text(base)}")
        report["d11_ready"] = validate_ready(ready, "d11")

        rejected = worker.request("update delete 2")
        report["d11_reject"] = validate_update(rejected, "d11", False)

        updated = worker.request("update insert 9")
        report["d11_update_d1"] = validate_update(updated, "d11", True)
        d1 = validate_sync(worker.request("sync"), "d11")
        if d1["fallback"] != 0 or d1["final_k"] != 1:
            raise ValueError("d=1 did not complete at k=1 without fallback")
        report["d11_sync_d1"] = d1
        report["d11_check_d1"] = validate_check(worker.request("check"))

        for key in [11, 13, 15, 17, 19, 21, 23, 25, 27]:
            validate_update(worker.request(f"update insert {key:x}"), "d11", True)
        d9 = validate_sync(worker.request("sync"), "d11")
        if d9["fallback"] != 1:
            raise ValueError("d=9 did not exercise exact fallback")
        report["d11_sync_d9"] = d9
        report["d11_check_d9"] = validate_check(worker.request("check"))

        # Deterministic B0 witness: true d=8, but the k=1 guarded prefix admits a
        # provisional false candidate. Maintained and fresh sketches/decoders agree,
        # proving this is an over-capacity guard event rather than persistence drift.
        witness_base = list(range(1, 1025))
        witness_start = (1 << 63) + 8_010_000
        witness_target = witness_base[4:] + list(range(witness_start, witness_start + 4))
        ready = worker.request(
            f"init_pair d11 {key_text(witness_target)} {key_text(witness_base)}"
        )
        report["d11_false_candidate_ready"] = validate_ready(ready, "d11")
        diagnose = validate_diagnose(worker.request("diagnose"))
        if not (
            diagnose["exact_d"] == 8
            and diagnose["a_rebuild"] == 1
            and diagnose["b_rebuild"] == 1
            and diagnose["maintained_decoded"] == 1
            and diagnose["maintained_exact"] == 0
            and diagnose["maintained_k"] == 1
            and diagnose["fresh_decoded"] == 1
            and diagnose["fresh_exact"] == 0
            and diagnose["fresh_k"] == 1
            and diagnose["candidate_equal"] == 1
        ):
            raise ValueError(f"D11 false-candidate witness changed: {diagnose}")
        report["d11_false_candidate_diagnose"] = diagnose
        witness_sync = validate_sync(worker.request("sync"), "d11")
        if witness_sync["fallback"] != 1 or witness_sync["false_candidate"] != 1:
            raise ValueError("D11 false-candidate witness did not charge exact fallback")
        report["d11_false_candidate_sync"] = witness_sync
        report["d11_false_candidate_check"] = validate_check(worker.request("check"))

        report["decision"] = "MEASUREMENT_PARTIAL_PASS"
        Path(sys.argv[2]).write_text(json.dumps(report, indent=2) + "\n")
        print("MEASUREMENT_PARTIAL_PASS")
        return 0
    finally:
        worker.close()


if __name__ == "__main__":
    raise SystemExit(main())
