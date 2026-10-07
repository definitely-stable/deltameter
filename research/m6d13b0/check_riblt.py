#!/usr/bin/env python3
"""Hosted integration check for the pinned B0 RIBLT adapter."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from readiness import (
    validate_riblt_check,
    validate_riblt_ready,
    validate_riblt_sync,
    validate_riblt_update,
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
            raise RuntimeError("RIBLT worker exited")
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(line + "\n")
        self.process.stdin.flush()
        result = self.process.stdout.readline()
        if not result:
            raise RuntimeError("RIBLT worker EOF")
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
        raise SystemExit("usage: check_riblt.py WORKER OUT.json")

    worker = Worker(sys.argv[1])
    report: dict[str, object] = {"format": "deltameter.m6d13b0.riblt-partial.v1"}
    base = [1, 3, 5, 7]

    try:
        report["stream_ready"] = validate_riblt_ready(
            worker.request(f"init {key_text(base)}")
        )
        report["stream_reject"] = validate_riblt_update(
            worker.request("update insert 3"), False
        )
        report["stream_update"] = validate_riblt_update(
            worker.request("update insert 9"), True
        )
        stream = validate_riblt_sync(worker.request("sync stream 1024"), "stream")
        if stream["fallback"] != 0:
            raise ValueError("d=1 stream unexpectedly fell back")
        report["stream_sync_d1"] = stream
        report["stream_check"] = validate_riblt_check(worker.request("check"))

        report["pull_ready"] = validate_riblt_ready(
            worker.request(f"init {key_text(base)}")
        )
        validate_riblt_update(worker.request("update insert 9"), True)
        pull = validate_riblt_sync(worker.request("sync pull 1024"), "pull")
        if pull["fallback"] != 0:
            raise ValueError("d=1 pull unexpectedly fell back")
        report["pull_sync_d1"] = pull
        report["pull_check"] = validate_riblt_check(worker.request("check"))

        report["fallback_ready"] = validate_riblt_ready(
            worker.request(f"init {key_text(base)}")
        )
        for key in [9, 11, 13, 15, 17, 19, 21, 23, 25]:
            validate_riblt_update(worker.request(f"update insert {key:x}"), True)
        exhausted = validate_riblt_sync(worker.request("sync stream 1"), "stream")
        if exhausted["fallback"] != 1:
            raise ValueError("forced stream cap did not exercise exact fallback")
        report["stream_forced_fallback"] = exhausted
        report["fallback_check"] = validate_riblt_check(worker.request("check"))

        report["decision"] = "RIBLT_PARTIAL_PASS"
        Path(sys.argv[2]).write_text(json.dumps(report, indent=2) + "\n")
        print("RIBLT_PARTIAL_PASS")
        return 0
    finally:
        worker.close()


if __name__ == "__main__":
    raise SystemExit(main())
