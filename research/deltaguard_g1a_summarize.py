#!/usr/bin/env python3
"""Strict five-worker G1-A aggregation; empirical power is not an error proof."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

TS = (32, 64, 256, 4096)
NUM = (0, 2, 6, 8, 9)
KS = (1, 2, 4)
SEEDS = range(3)
FIELDS = {
    "worker", "scenario", "ratio", "seed", "T", "d", "k",
    "owner_bytes", "old_bound", "new_bound", "old_safe", "new_safe",
    "false_safe",
}


def read_row(line: str) -> dict | None:
    if not line.startswith("G1A_SAMPLE "):
        return None
    parts = [pair.split("=", 1) for pair in line[len("G1A_SAMPLE "):].split()]
    row = dict(parts)
    assert len(parts) == len(FIELDS) == len(row) and set(row) == FIELDS
    for field in FIELDS:
        assert re.fullmatch("[0-9]+", row[field]), (field, row[field])
        row[field] = int(row[field])
    w, s, n, seed, k = (
        row["worker"], row["scenario"], row["ratio"], row["seed"], row["k"]
    )
    assert 1 <= w <= 5 and s in range(4) and n in range(5) and seed in SEEDS
    assert k in KS and row["T"] == TS[s] and row["d"] == TS[s] * NUM[n] // 8
    assert row["owner_bytes"] == 512 * k
    assert 0 <= row["new_bound"] <= row["old_bound"] <= (1 << 64)
    assert row["old_safe"] == int(row["old_bound"] <= row["T"])
    assert row["new_safe"] == int(row["new_bound"] <= row["T"])
    assert row["false_safe"] == int(row["new_safe"] and row["d"] > row["T"])
    assert row["new_safe"] >= row["old_safe"]
    return row


def summarize(root: Path, head: str) -> dict:
    assert re.fullmatch("[0-9a-f]{40}", head)
    data = {}
    for w in range(1, 6):
        assert (root / f"head-{w}.txt").read_text().strip() == head
        lines = (root / f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_G1A_WORKER_PASS worker={w}") == 1
        seen = 0
        for line in lines:
            row = read_row(line)
            if row is None:
                continue
            assert row["worker"] == w
            key = (w, row["scenario"], row["ratio"], row["seed"], row["k"])
            assert key not in data
            data[key] = row
            seen += 1
        assert seen == 4 * 5 * 3 * 3
    assert len(data) == 900

    power = []
    for s, t in enumerate(TS):
        for n, numerator in enumerate(NUM):
            for k in KS:
                selected = [
                    data[(w, s, n, seed, k)]
                    for w in range(1, 6)
                    for seed in SEEDS
                ]
                assert len(selected) == 15
                power.append({
                    "T": t, "d_over_t_numerator": numerator,
                    "d_over_t_denominator": 8, "k": k,
                    "old_safe": sum(z["old_safe"] for z in selected),
                    "new_safe": sum(z["new_safe"] for z in selected),
                    "observed_false_safe": sum(z["false_safe"] for z in selected),
                    "observations": len(selected),
                    "per_owner_bitmap_bytes": 512 * k,
                })
    threshold64 = [z for z in power if z["T"] == 64 and z["d_over_t_numerator"] == 2]
    threshold4096 = [z for z in power if z["T"] == 4096 and z["d_over_t_numerator"] == 6]
    eligible = any(z["new_safe"] >= 12 for z in threshold64 + threshold4096)
    bad = sum(z["false_safe"] for z in data.values())
    return {
        "format": "deltameter.g1a-alpha-utility.v1",
        "source_head": head,
        "raw_paired_records": 900,
        "old_vs_new_logical_outcomes": 1800,
        "workers": 5,
        "false_safe_observed": bad,
        "fixed_public_test_key_not_monte_carlo": True,
        "mathematical_certification": "separate exact Q32 generator/certifier artifacts",
        "power": power,
        "candidate_lanes": [
            {"T": z["T"], "d_ratio": f"{z['d_over_t_numerator']}/8",
             "k": z["k"], "new_safe": z["new_safe"], "old_safe": z["old_safe"]}
            for z in threshold64 + threshold4096 if z["new_safe"] >= 12
        ],
        "decision": (
            "G1A_ALGEBRA_PASS_EMPIRICAL_UTILITY_CANDIDATE" if eligible
            else "G1A_ALGEBRA_PASS_LOW_POWER"
        ),
        "product_status": "NOT_AUTHORIZED",
        "keyed_security_status": "NOT_PROVEN",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.input, args.head)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, sort_keys=True, indent=2) + "\n")
    print("DELTAGUARD_G1A_UTILITY_PASS rows=900 decision=" + result["decision"])
    for row in result["power"]:
        if row["d_over_t_numerator"] in (2, 6):
            print(
                f"G1A_POWER T={row['T']} d_over_t={row['d_over_t_numerator']}/8 "
                f"k={row['k']} old={row['old_safe']}/15 new={row['new_safe']}/15 "
                f"false_safe_observed={row['observed_false_safe']}"
            )
    if result["false_safe_observed"]:
        print("WARNING: empirical false-safe requires audit; not a statistical proof.")


if __name__ == "__main__":
    main()
