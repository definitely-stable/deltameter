#!/usr/bin/env python3
"""Fail closed if any frozen D13-B0 measurement source changes."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).with_name("frozen-sources.json")


def git_blob_sha1(path: Path) -> str:
    data = path.read_bytes()
    header = f"blob {len(data)}\0".encode()
    return hashlib.sha1(header + data).hexdigest()


def main() -> int:
    expected = json.loads(MANIFEST.read_text())
    if not expected:
        raise SystemExit("empty frozen source manifest")
    actual = {}
    for rel, sha in expected.items():
        path = ROOT / rel
        if not path.is_file():
            raise SystemExit(f"missing frozen source: {rel}")
        actual_sha = git_blob_sha1(path)
        actual[rel] = actual_sha
        if actual_sha != sha:
            raise SystemExit(
                f"frozen source changed: {rel}: expected {sha}, got {actual_sha}"
            )
    print(f"FROZEN_SOURCES_PASS files={len(actual)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
