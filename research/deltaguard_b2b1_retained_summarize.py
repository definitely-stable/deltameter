#!/usr/bin/env python3
"""Fail-closed physical five-worker B1-B0 retained exact / fallback aggregate.

Only byte counts were physically observed. This is NOT timed p95/RTT evidence.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path

NS = (256, 65536)
DS = (16, 48, 57, 65)
SESSIONS = (1, 10, 100)
FIELDS = {
    "worker", "ni", "di", "rep", "N", "d", "S", "odd", "safe",
    "source_a", "source_b", "build_ns", "bootstrap_ns", "delta_ns",
    "bootstrap_bytes", "retained_exact_bytes", "retained_batched_bytes",
    "guard_sparse_bytes", "guard_dense_bytes", "direct_bytes",
    "fallback_bytes", "resolved_bytes", "generations", "queries_sparse",
}
CUTS_SHA = "c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"


def parse(line: str) -> dict | None:
    if not line.startswith("B1B0_SAMPLE "):
        return None
    parts = [x.split("=", 1) for x in line[len("B1B0_SAMPLE "):].split()]
    rec = dict(parts)
    assert len(parts) == len(rec) == len(FIELDS) and set(rec) == FIELDS
    assert all(re.fullmatch(r"[0-9]+", value) for value in rec.values())
    x = {k: int(v) for k, v in rec.items()}
    w, ni, di, rep, n, d, s = (x[k] for k in
                              ("worker", "ni", "di", "rep", "N", "d", "S"))
    assert 1 <= w <= 5 and ni in range(len(NS)) and di in range(len(DS))
    assert 0 <= rep < 3 and s in SESSIONS
    assert n == NS[ni] and d == DS[di]
    assert x["source_a"] == n + s-1
    assert x["source_b"] == n + d % 2 + s-1
    assert x["generations"] == s-1
    assert x["queries_sparse"] == SESSIONS.index(s)+1
    assert x["safe"] in (0, 1) and x["safe"] == int(x["odd"] <= 48)
    assert 0 <= x["odd"] <= d
    if d <= 48:
        assert x["safe"] == 1, "pointwise guaranteed SAFE violated"
    init = 128+8*(2*n+d % 2)
    direct = 128+8*(x["source_a"]+x["source_b"])
    assert x["bootstrap_bytes"] == init
    assert x["retained_exact_bytes"] == init+146*(s-1)
    assert x["retained_batched_bytes"] == init+128*(x["queries_sparse"]-1)+18*(s-1)
    assert x["guard_sparse_bytes"] == 640*x["queries_sparse"]
    assert x["guard_dense_bytes"] == 640*s
    assert x["direct_bytes"] == direct
    assert x["fallback_bytes"] == (0 if x["safe"] else direct+50)
    assert x["resolved_bytes"] == 640+x["fallback_bytes"]
    if not x["safe"]:
        assert x["resolved_bytes"] > direct
    return x


def self_test() -> None:
    sample = (
        "B1B0_SAMPLE worker=1 ni=0 di=1 rep=0 N=256 d=48 S=1 odd=46 safe=1 "
        "source_a=256 source_b=256 build_ns=7 bootstrap_ns=8 delta_ns=0 "
        "bootstrap_bytes=4224 retained_exact_bytes=4224 retained_batched_bytes=4224 "
        "guard_sparse_bytes=640 guard_dense_bytes=640 direct_bytes=4224 "
        "fallback_bytes=0 resolved_bytes=640 generations=0 queries_sparse=1"
    )
    assert parse(sample)
    for bad in (
        sample.replace("safe=1", "safe=0"),
        sample.replace("retained_exact_bytes=4224", "retained_exact_bytes=0"),
        sample.replace("retained_batched_bytes=4224", "retained_batched_bytes=4225"),
        sample.replace("guard_dense_bytes=640", "guard_dense_bytes=0"),
        sample.replace("source_a=256", "source_a=255"),
        sample.replace("di=1", "di=5"),
        sample.replace("odd=46", "odd=60"),
        sample.replace("fallback_bytes=0", "fallback_bytes=100"),
        sample.replace("queries_sparse=1", "queries_sparse=2"),
    ):
        try:
            parse(bad)
        except (AssertionError, KeyError, ValueError):
            continue
        raise AssertionError("mutated source/model record accepted")


def aggregate(root: Path, source_sha: str, certificate: Path) -> dict:
    assert re.fullmatch("[0-9a-f]{40}", source_sha)
    assert hashlib.sha256(certificate.read_bytes()).hexdigest() == CUTS_SHA
    self_test()
    rows = {}
    for worker in range(1, 6):
        assert (root / f"head-{worker}.txt").read_text().strip() == source_sha
        raw = (root / f"worker-{worker}.txt").read_text().splitlines()
        assert raw.count(
            f"DELTAGUARD_B1B0_WORKER_PASS worker={worker} samples=72"
        ) == 1
        count = 0
        for line in raw:
            rec = parse(line)
            if rec is None:
                continue
            assert rec["worker"] == worker
            key = (worker, rec["ni"], rec["di"], rec["rep"], rec["S"])
            assert key not in rows, "duplicate sample"
            rows[key] = rec
            count += 1
        assert count == 72
    assert len(rows) == 5*2*4*3*3 == 360
    summary = []
    for ni, n in enumerate(NS):
        for di, d in enumerate(DS):
            for s in SESSIONS:
                group = [rows[w, ni, di, rep, s] for w in range(1, 6)
                         for rep in range(3)]
                assert len(group) == 15
                first = group[0]
                summary.append({
                    "N": n, "d": d, "session": s,
                    "guard_safe": sum(v["safe"] for v in group),
                    "trials": 15,
                    "retained_streaming_exact_bytes": first["retained_exact_bytes"],
                    "retained_batched_exact_bytes": first["retained_batched_bytes"],
                    "guard_sparse_bytes": first["guard_sparse_bytes"],
                    "guard_dense_bytes": first["guard_dense_bytes"],
                    "direct_full_bytes": first["direct_bytes"],
                    "resolved_bytes_if_unknown": first["direct_bytes"]+690,
                    "cold_source_bootstrap_bytes": first["bootstrap_bytes"],
                })
    for n in NS:
        for s in SESSIONS:
            v = next(x for x in summary if x["N"] == n and
                     x["d"] == 48 and x["session"] == s)
            assert v["guard_safe"] == 15
    false_safe = sum(int(v["safe"] and v["d"] > 64) for v in rows.values())
    return {
        "schema": "deltameter.b1b0-physical-retained-exact.v1",
        "source_sha": source_sha,
        "certificate_sha256": CUTS_SHA,
        "workers": 5,
        "samples": 360,
        "physical_sockets_used": True,
        "source_threads_share_process": True,
        "physical_application_bytes_only": True,
        "p95_transport_pass": False,
        "keyed_prf_adaptive_security_proven": False,
        "false_safe_fixture_observed": false_safe,
        "scope": "B1B0_CORRECTNESS_ONLY_NO_SYSTEM_PRODUCT_GO",
        "comparisons": summary,
        "verdict": "B1B0_RETAINED_PHYSICAL_EXACT_FALLBACK_PASS",
    }


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--input", type=Path, required=True)
    p.add_argument("--head", required=True)
    p.add_argument("--certificate", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args()
    doc = aggregate(args.input, args.head, args.certificate)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(doc, sort_keys=True, indent=2)+"\n")
    print(f"B1B0_AGGREGATE_PASS samples={doc['samples']} "
          f"verdict={doc['verdict']}")
    for v in doc["comparisons"]:
        if v["N"] == 256 and v["session"] == 100 and v["d"] in (48, 57):
            print("B1B0_COST " + " ".join(f"{k}={value}" for k,value in v.items()))


if __name__ == "__main__":
    main()
