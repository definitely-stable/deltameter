#!/usr/bin/env python3
"""Fail-closed paired five-worker OPT-A J52 refinement summary."""
from __future__ import annotations
import argparse
import json
import math
import re
import statistics
from pathlib import Path

WORKERS = (1, 2, 3, 4, 5)
PROFILES = (52, 56, 64)
ROUNDS = range(8)
FIELDS = ("worker", "round", "J", "layout", "state_bytes",
          "update_ns_per_token", "estimate_ns", "xor_ns_per_kib")
METRICS = ("update_ns_per_token", "estimate_ns", "xor_ns_per_kib")
GATES = (0.15, 0.25, 0.25)
PREFIX = "STRICT_COMPACT_OPT_A_REFINE_SAMPLE "

def parse(line: str) -> dict | None:
    if not line.startswith(PREFIX):
        return None
    parts = [s.split("=", 1) for s in line[len(PREFIX):].split()]
    row = dict(parts)
    assert len(row) == len(FIELDS) and set(row) == set(FIELDS)
    for key in ("worker", "round", "J", "state_bytes"):
        row[key] = int(row[key])
    for key in METRICS:
        row[key] = float(row[key])
        assert math.isfinite(row[key]) and row[key] > 0
    assert row["worker"] in WORKERS and row["round"] in ROUNDS
    assert row["J"] in PROFILES and row["layout"] == "level"
    assert row["state_bytes"] == 4096 * row["J"] // 8
    return row

def run(log_dir: Path, expected_head: str, cert_path: Path) -> dict:
    assert re.fullmatch(r"[0-9a-f]{40}", expected_head)
    cert = json.loads(cert_path.read_text(encoding="utf-8"))
    assert cert["decision"] == "J52_EXACT_CERT_PASS"
    assert cert["table_sha256"] == (
        "634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a")
    candidates = {row["J"]: row for row in cert["profiles"]}
    assert set(candidates) == {51, 52}
    assert candidates[52]["covers_full_domain"] and candidates[52]["state_bytes"] == 26624
    assert int(candidates[52]["d_max"]) == (1 << 64) - 1
    assert int(cert["domain_cardinality"]) == 1 << 64

    samples = {}
    for worker in WORKERS:
        log = log_dir / f"worker-{worker}.txt"
        sha = log_dir / f"head-{worker}.txt"
        assert log.exists() and sha.exists(), f"missing provenance for worker={worker}"
        assert sha.read_text(encoding="utf-8").strip() == expected_head
        data = log.read_text(encoding="utf-8").splitlines()
        assert data.count(f"STRICT_COMPACT_OPT_A_REFINE_WORKER_PASS worker={worker}") == 1
        n = 0
        for line in data:
            row = parse(line)
            if row is None:
                continue
            assert row["worker"] == worker
            key = (worker, row["J"], row["round"])
            assert key not in samples, f"duplicate {key}"
            samples[key] = row
            n += 1
        assert n == len(PROFILES) * len(ROUNDS), f"worker={worker}: incomplete {n}"

    assert len(samples) == len(WORKERS) * len(PROFILES) * len(ROUNDS)
    summaries = []
    for worker in WORKERS:
        med = {}
        for j in PROFILES:
            med[j] = {
                metric: statistics.median(samples[(worker, j, round_no)][metric]
                                          for round_no in ROUNDS)
                for metric in METRICS
            }
        ctrl = med[64]
        for j in PROFILES:
            ratios = [med[j][metric] / ctrl[metric] - 1 for metric in METRICS]
            summaries.append({
                "worker": worker, "J": j, "state_bytes": 4096*j//8,
                **med[j],
                "regressions_vs_J64": dict(zip(METRICS, ratios)),
                "pass": all(reg <= gate for reg, gate in zip(ratios, GATES)),
            })
    j52 = [row for row in summaries if row["J"] == 52]
    assert len(j52) == len(WORKERS)
    decision = "J52_REFINEMENT_PASS" if all(x["pass"] for x in j52) else "RETAIN_J56"
    result = {
        "format": "deltameter.strict-compact-opt-a-refinement.v1",
        "head": expected_head,
        "workers": list(WORKERS),
        "profiles": list(PROFILES),
        "sample_count": len(samples),
        "gate": dict(zip(METRICS, GATES)),
        "worker_medians": summaries,
        "decision": decision,
        "J52_state_reduction_vs_J64": 1 - 52/64,
        "J52_state_reduction_vs_J56": 1 - 52/56,
    }
    return result

def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--logs-dir", type=Path, required=True)
    p.add_argument("--cert", type=Path, required=True)
    p.add_argument("--head", required=True)
    p.add_argument("--out", type=Path, required=True)
    a = p.parse_args()
    result = run(a.logs_dir, a.head, a.cert)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("STRICT_COMPACT_OPT_A_REFINE_SUMMARY " + result["decision"])
    for row in result["worker_medians"]:
        if row["J"] == 52:
            print(f"J52_WORKER worker={row['worker']} pass={row['pass']} "
                  f"update_ns={row['update_ns_per_token']:.3f} "
                  f"estimate_ns={row['estimate_ns']:.3f} "
                  f"xor_ns_per_kib={row['xor_ns_per_kib']:.3f}")

if __name__ == "__main__":
    main()
