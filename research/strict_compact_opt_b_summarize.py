#!/usr/bin/env python3
"""Fail-closed five-worker benchmark aggregation for STRICT-COMPACT OPT-B."""
from __future__ import annotations
import argparse
import json
import math
import re
import statistics
from pathlib import Path

WORKERS = range(1, 6)
J_VALUES = (24, 52, 64)
MODES = ("scan", "cache")
LANES = ("update", "query", "merge", "mixed_1", "mixed_10", "mixed_100", "mixed_1000")
ROUNDS = range(8)
PREFIX = "STRICT_COMPACT_OPT_B_SAMPLE "
FULL_SAMPLES = 5 * len(J_VALUES) * len(MODES) * len(LANES) * len(ROUNDS)
Q32_SHA256 = "634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a"

def parse(line: str) -> dict | None:
    if not line.startswith(PREFIX):
        return None
    pairs = [x.split("=", 1) for x in line[len(PREFIX):].split()]
    item = dict(pairs)
    assert len(item) == 8 and set(item) == {
        "worker", "round", "J", "mode", "lane", "state_bytes", "cache_bytes", "ns"
    }, "malformed sample"
    item["worker"] = int(item["worker"])
    item["round"] = int(item["round"])
    item["J"] = int(item["J"])
    item["state_bytes"] = int(item["state_bytes"])
    item["cache_bytes"] = int(item["cache_bytes"])
    item["ns"] = float(item["ns"])
    assert item["worker"] in WORKERS and item["round"] in ROUNDS
    assert item["J"] in J_VALUES and item["mode"] in MODES and item["lane"] in LANES
    assert item["state_bytes"] == 512 * item["J"]
    assert item["cache_bytes"] == (2 * item["J"] if item["mode"] == "cache" else 0)
    assert math.isfinite(item["ns"]) and item["ns"] > 0
    return item

def aggregate(log_dir: Path, head: str) -> dict:
    assert re.fullmatch(r"[a-f0-9]{40}", head), "invalid expected HEAD"
    samples: dict[tuple[int, int, str, str, int], dict] = {}
    for worker in WORKERS:
        file = log_dir / f"worker-{worker}.txt"
        source = log_dir / f"head-{worker}.txt"
        assert file.is_file() and source.is_file(), f"missing worker {worker}"
        assert source.read_text(encoding="utf-8").strip() == head, "provenance mismatch"
        lines = file.read_text(encoding="utf-8").splitlines()
        assert lines.count("STRICT_COMPACT_OPT_B_CORRECTNESS_PASS") == 1
        assert lines.count(f"STRICT_COMPACT_OPT_B_WORKER_PASS worker={worker}") == 1
        count = 0
        for line in lines:
            row = parse(line)
            if row is None:
                continue
            assert row["worker"] == worker, f"foreign worker in {file.name}"
            key = (worker, row["J"], row["mode"], row["lane"], row["round"])
            assert key not in samples, f"duplicate sample {key}"
            samples[key] = row
            count += 1
        assert count == len(J_VALUES) * len(MODES) * len(LANES) * len(ROUNDS)
    assert len(samples) == FULL_SAMPLES
    medians = {}
    for w in WORKERS:
        for j in J_VALUES:
            for mode in MODES:
                for lane in LANES:
                    medians[(w,j,mode,lane)] = statistics.median(
                        samples[(w,j,mode,lane,round_no)]["ns"] for round_no in ROUNDS
                    )
    workers = []
    for w in WORKERS:
        ratios = {}
        for lane in LANES:
            s = medians[(w,52,"scan",lane)]
            c = medians[(w,52,"cache",lane)]
            ratios[lane] = {
                "scan_ns": s, "cache_ns": c,
                "cache_over_scan": c / s,
                "scan_over_cache": s / c,
                "improvement": (s - c) / s,
            }
        update = ratios["update"]["cache_over_scan"] <= 1.10
        query = ratios["query"]["scan_over_cache"] >= 5.0
        workers.append({
            "worker": w,
            "lanes": ratios,
            "update_pass": update,
            "query_pass": query,
        })
    qualifying = [
        lane for lane in ("mixed_1", "mixed_10", "mixed_100")
        if all(row["lanes"][lane]["improvement"] >= 0.10 for row in workers)
    ]
    go = all(w["update_pass"] and w["query_pass"] for w in workers) and bool(qualifying)
    other = []
    for j in J_VALUES:
        for lane in LANES:
            other.append({
                "J": j, "lane": lane,
                "worker_median_scan_ns": statistics.median(
                    medians[(w,j,"scan",lane)] for w in WORKERS),
                "worker_median_cache_ns": statistics.median(
                    medians[(w,j,"cache",lane)] for w in WORKERS),
            })
    return {
        "format": "deltameter.strict-compact-opt-b.v1",
        "source_head": head,
        "j_primary": 52,
        "q32_sha256": Q32_SHA256,
        "sample_count": FULL_SAMPLES,
        "workers": workers,
        "small_and_control_profiles": other,
        "frozen_gates": {
            "max_update_regression": 0.10,
            "min_query_speedup": 5.0,
            "min_mixed_workload_improvement": 0.10,
            "qualified_lanes": ["mixed_1", "mixed_10", "mixed_100"],
        },
        "mixed_lanes_all_worker_pass": qualifying,
        "decision": "KEEP_CACHE" if go else "DROP_CACHE",
    }

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--logs-dir", type=Path, required=True)
    ap.add_argument("--head", required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    x = aggregate(args.logs_dir, args.head)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(x, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"STRICT_COMPACT_OPT_B_EVIDENCE_PASS decision={x['decision']} samples={x['sample_count']}")
    for row in x["workers"]:
        print(
            f"OPT_B_WORKER worker={row['worker']} update_pass={row['update_pass']} "
            f"query_pass={row['query_pass']} "
            f"update_ratio={row['lanes']['update']['cache_over_scan']:.5f} "
            f"query_speedup={row['lanes']['query']['scan_over_cache']:.3f} "
            f"mixed_1_gain={row['lanes']['mixed_1']['improvement']:.4f} "
            f"mixed_10_gain={row['lanes']['mixed_10']['improvement']:.4f} "
            f"mixed_100_gain={row['lanes']['mixed_100']['improvement']:.4f}"
        )

if __name__ == "__main__":
    main()
