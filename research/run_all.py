#!/usr/bin/env python3
"""Run the small DeltaMeter research suite and write JSON artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from energy_profiles import build_payload as build_energy_payload
from parity_level_moments import build_grid as build_parity_grid


def write_json(path: Path, payload: dict) -> None:
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("research-out"))
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)

    energy_path = args.out / "energy-profiles.json"
    parity_path = args.out / "parity-level-moments.json"

    write_json(energy_path, build_energy_payload())
    write_json(parity_path, build_parity_grid())

    print(f"wrote {energy_path}")
    print(f"wrote {parity_path}")


if __name__ == "__main__":
    main()
