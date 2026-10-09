#!/usr/bin/env python3
"""G1A: generate Decimal-proposed and independently integer-certified alpha=delta/k tables.

The only exact probability result is inherited from the original per-level
de-Poissonization/KL bound. Keyed BLAKE3 is not proved a truly random oracle.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from decimal import Decimal
from pathlib import Path

from strict_compact_lookup_q32 import build_thresholds, table_text
from strict_compact_lookup_certify import certify, SENTINEL, LIKELIHOOD_TARGET

EXPECTED_BASELINE_SHA = (
    "634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a"
)
K_VALUES = (1, 2, 4)
DELTA_DENOMINATOR = 1_000_000


def require_rejection(table: list[int], target: int) -> None:
    """Independent exact checking must reject attempts to narrow or corrupt bound."""
    corrupt = table.copy()
    corrupt[0] = 1  # impossible overly narrow finite threshold
    try:
        certify(corrupt, target)
    except (AssertionError, ValueError):
        pass
    else:
        raise AssertionError("underestimated q32 accepted")
    corrupt = table.copy()
    corrupt[0] = SENTINEL  # invalid sentinel before finite thresholds
    try:
        certify(corrupt, target)
    except (AssertionError, ValueError):
        pass
    else:
        raise AssertionError("noncanonical sentinel accepted")
    corrupt = table.copy()
    corrupt[-1] = 1  # spurious finite value after sentinel
    try:
        certify(corrupt, target)
    except (AssertionError, ValueError):
        pass
    else:
        raise AssertionError("spurious finite threshold accepted")


def run(out_dir: Path) -> dict:
    out_dir.mkdir(parents=True, exist_ok=True)
    baseline = build_thresholds()
    baseline_bytes = table_text(baseline).encode("ascii")
    baseline_sha = hashlib.sha256(baseline_bytes).hexdigest()
    assert baseline_sha == EXPECTED_BASELINE_SHA, "default generator drift"
    (out_dir / "q32-base.txt").write_bytes(baseline_bytes)
    old = certify(baseline)
    assert old["likelihood_target_2_over_alpha"] == LIKELIHOOD_TARGET

    profiles = []
    manifest = [f"{baseline_sha}  q32-base.txt"]
    for k in K_VALUES:
        alpha = Decimal(1) / Decimal(DELTA_DENOMINATOR * k)
        target = 2 * DELTA_DENOMINATOR * k  # 2 / alpha exactly
        table = build_thresholds(alpha)
        assert len(table) == len(baseline) == 2048
        assert all(x <= y for x, y in zip(table, baseline)), "new q32 is wider"
        exact = certify(table, target=target)
        assert exact["likelihood_target_2_over_alpha"] == target
        assert exact["finite_entries"] + exact["sentinel_entries"] == 2048
        require_rejection(table, target)

        raw = table_text(table).encode("ascii")
        sha = hashlib.sha256(raw).hexdigest()
        file = f"q32-k{k}.txt"
        (out_dir / file).write_bytes(raw)
        manifest.append(f"{sha}  {file}")
        exact["table_sha256"] = sha
        exact["profile"] = f"fixed-prior-T k={k} alpha=1/{DELTA_DENOMINATOR*k}"
        profiles.append({
            "k": k,
            "alpha_numerator": 1,
            "alpha_denominator": DELTA_DENOMINATOR * k,
            "likelihood_target": target,
            "table_sha256": sha,
            "finite": exact["finite_entries"],
            "sentinel": exact["sentinel_entries"],
            "diff_from_old_counts": sum(x != y for x,y in zip(table, baseline)),
            "certificate_file": f"certificate-k{k}.json",
        })
        (out_dir / f"certificate-k{k}.json").write_text(
            json.dumps(exact, sort_keys=True, indent=2)+"\n", encoding="utf-8"
        )
        print(f"G1A_Q32_EXACT_CERT_PASS k={k} finite={exact['finite_entries']} "
              f"sentinels={exact['sentinel_entries']} sha256={sha}",flush=True)

    (out_dir / "sha256sum.txt").write_text("\n".join(manifest)+"\n",encoding="ascii")
    payload = {
        "format": "deltameter.g1a-alpha-recalibration.exact.v1",
        "m": 4096, "j_domain": 52, "delta_denominator": DELTA_DENOMINATOR,
        "baseline_table_sha256": baseline_sha,
        "old_alpha": "1/64000000",
        "profiles": profiles,
        "proof": "independent rational Taylor p lower bound, integer likelihood ratio, independent p=1/2 sentinel checks",
        "ideal_oracle_only": True,
        "adaptive_t_guarantee": "not proved",
        "release": "research-only",
    }
    (out_dir/"certificate-summary.json").write_text(
        json.dumps(payload,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    return payload


def main() -> None:
    a=argparse.ArgumentParser()
    a.add_argument("--out-dir",type=Path,required=True)
    v=a.parse_args()
    summary=run(v.out_dir)
    assert len(summary["profiles"])==3
    print("DELTAGUARD_G1A_CERT_PASS",flush=True)


if __name__=="__main__":
    main()
