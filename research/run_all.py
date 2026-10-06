#!/usr/bin/env python3
"""Run the small DeltaMeter research suite and write JSON artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from energy_profiles import build_payload as build_energy_payload
from fpcsa_exact_small import build_payload as build_fpcsa_exact_payload
from parity_full_exact_small import build_payload as build_full_exact_payload
from parity_level_moments import build_grid as build_parity_grid
from parity_poissonized import build_payload as build_poissonized_payload
from parity_truncation_budget import build_payload as build_truncation_payload


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
    fpcsa_exact_path = args.out / "fpcsa-exact-small.json"
    parity_full_exact_path = args.out / "parity-full-exact-small.json"
    parity_poissonized_path = args.out / "parity-poissonized.json"
    parity_truncation_path = args.out / "parity-truncation-budget.json"

    write_json(energy_path, build_energy_payload())
    write_json(parity_path, build_parity_grid())
    write_json(fpcsa_exact_path, build_fpcsa_exact_payload())
    write_json(parity_full_exact_path, build_full_exact_payload())
    write_json(parity_poissonized_path, build_poissonized_payload())
    write_json(parity_truncation_path, build_truncation_payload())

    print(f"wrote {energy_path}")
    print(f"wrote {parity_path}")
    print(f"wrote {fpcsa_exact_path}")
    print(f"wrote {parity_full_exact_path}")
    print(f"wrote {parity_poissonized_path}")
    print(f"wrote {parity_truncation_path}")


if __name__ == "__main__":
    main()
