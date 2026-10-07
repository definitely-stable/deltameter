#!/usr/bin/env python3
"""Fail closed if any frozen M6-D13-B measurement source changes."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).with_name("frozen-sources.json")


def git_blob_sha1(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


def main() -> int:
    expected = json.loads(MANIFEST.read_text())
    if not expected:
        raise SystemExit("empty D13-B frozen source manifest")
    for rel, expected_sha in expected.items():
        path = ROOT / rel
        if not path.is_file():
            raise SystemExit(f"missing frozen source: {rel}")
        actual = git_blob_sha1(path)
        if actual != expected_sha:
            raise SystemExit(
                f"frozen source changed: {rel}: expected {expected_sha}, got {actual}"
            )
    print(f"D13B_FROZEN_SOURCES_PASS files={len(expected)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
