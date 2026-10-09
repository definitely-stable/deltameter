#!/usr/bin/env python3
"""Fail-closed 3,240-record hosted two-owner near-full threshold matrix."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

TS = (32, 64, 128)
BS = tuple(range(6, 14))
RATIOS = ((0, 1), (1, 4), (1, 2), (3, 4), (9, 10),
          (99, 100), (1, 1), (101, 100), (11, 10))
FIELDS = {"worker", "scenario", "ratio", "seed", "T", "d",
          "b", "m", "cutoff", "odd", "safe", "false_safe",
          "owner_bytes", "struct_bytes", "key_bytes", "heap_allocations"}


def d_for(t: int, fraction: tuple[int, int]) -> int:
    num, den = fraction
    return (t*num + den - 1)//den if num > den else t*num//den


def read_table(path: Path) -> dict[tuple[int, int], int]:
    lines = path.read_text(encoding="ascii").splitlines()
    assert lines[0] == "# deltameter.guard-b2a-nearfull-cutoffs.v1"
    values = {}
    for line in lines[1:]:
        if line.startswith("#"):
            continue
        cols = line.split()
        assert len(cols) == 4
        t, b, m, c = map(int, cols)
        assert t in TS and b in BS and m == (1 << b) - 1
        assert -1 <= c <= t
        key = (t, b)
        assert key not in values
        values[key] = c
    assert len(values) == len(TS)*len(BS) == 24
    return values


def parse_record(line: str, cert: dict) -> dict | None:
    if not line.startswith("B2A_SAMPLE "):
        return None
    parts = [v.split("=", 1) for v in line[len("B2A_SAMPLE "):].split()]
    result = dict(parts)
    assert len(parts) == len(result) == len(FIELDS) and set(result) == FIELDS
    for k, s in result.items():
        assert re.fullmatch(r"-?[0-9]+", s), (k, s)
        result[k] = int(s)
    w, scenario, ratio, seed, t, d, b = (
        result["worker"], result["scenario"], result["ratio"],
        result["seed"], result["T"], result["d"], result["b"])
    assert 1 <= w <= 5 and 0 <= scenario < len(TS)
    assert 0 <= ratio < len(RATIOS) and 0 <= seed < 3 and b in BS
    assert t == TS[scenario] and d == d_for(t, RATIOS[ratio])
    assert (d > t) == (RATIOS[ratio][0] > RATIOS[ratio][1])
    assert result["m"] == (1 << b) - 1
    assert result["cutoff"] == cert[(t, b)]
    assert 0 <= result["odd"] <= min(d, result["m"])
    assert result["owner_bytes"] == 8 * ((result["m"] + 63)//64)
    assert 0 < result["struct_bytes"] <= 256
    assert result["key_bytes"] == 32 and result["heap_allocations"] == 1
    assert result["safe"] == int(result["cutoff"] >= 0
                                 and result["odd"] <= result["cutoff"])
    assert result["false_safe"] == int(bool(result["safe"]) and d > t)
    if 0 <= d <= result["cutoff"]:
        assert result["safe"] == 1, "pointwise S<=d guarantee violated"
    return result


def self_test(table: dict) -> None:
    good = ("B2A_SAMPLE worker=1 scenario=0 ratio=0 seed=0 "
            "T=32 d=0 b=6 m=63 cutoff=5 odd=0 safe=1 "
            "false_safe=0 owner_bytes=8 struct_bytes=64 "
            "key_bytes=32 heap_allocations=1")
    # Use actual certificate c: avoid assuming any concrete cutoff aside from bounds.
    c = table[(32, 6)]
    good = good.replace("cutoff=5", f"cutoff={c}").replace("safe=1", f"safe={int(c >= 0)}")
    assert parse_record(good, table) is not None
    for corrupt in [
        good.replace(f"cutoff={c}", f"cutoff={c + 1}"),
        good.replace("owner_bytes=8", "owner_bytes=512"),
        good.replace("key_bytes=32", "key_bytes=0"),
        good.replace("ratio=0", "ratio=8"),
        good.replace("odd=0", "odd=99"),
        good.replace("safe="+str(int(c >= 0)), "safe=2"),
    ]:
        try:
            parse_record(corrupt, table)
        except (AssertionError, KeyError, ValueError):
            continue
        raise AssertionError("mutated record accepted")


def aggregate(root: Path, path: Path, sha: str) -> dict:
    assert re.fullmatch(r"[0-9a-f]{40}", sha)
    cert = read_table(path)
    self_test(cert)
    rows = {}
    for worker in range(1, 6):
        raw = (root / f"worker-{worker}.txt").read_text(encoding="utf-8").splitlines()
        assert raw.count(f"DELTAGUARD_B2A_WORKER_PASS worker={worker}") == 1
        assert (root / f"head-{worker}.txt").read_text().strip() == sha
        count = 0
        for line in raw:
            rec = parse_record(line, cert)
            if rec is None:
                continue
            assert rec["worker"] == worker
            key = (worker, rec["scenario"], rec["ratio"], rec["seed"], rec["b"])
            assert key not in rows
            rows[key] = rec
            count += 1
        assert count == 3 * 9 * 3 * 8 == 648
    assert len(rows) == 5*3*9*3*8 == 3240
    groups = []
    for ti, t in enumerate(TS):
        for ri, fraction in enumerate(RATIOS):
            for b in BS:
                selected = [rows[(w, ti, ri, seed, b)]
                            for w in range(1, 6) for seed in range(3)]
                assert len(selected) == 15
                groups.append({
                    "T": t,
                    "d": d_for(t, fraction),
                    "ratio": f"{fraction[0]}/{fraction[1]}",
                    "b": b,
                    "cutoff": cert[(t,b)],
                    "owner_bytes": selected[0]["owner_bytes"],
                    "safe": sum(v["safe"] for v in selected),
                    "samples": 15,
                    "false_safe_observed": sum(v["false_safe"] for v in selected),
                })
    focus = next(x for x in groups if x["T"] == 64
                 and x["d"] == 48 and x["b"] == 11)
    assert focus["cutoff"] >= 48 and focus["owner_bytes"] == 256
    assert focus["safe"] == 15, "frozen d=.75T proof utility gate failed"
    return {
        "format": "deltameter.guard-b2a-matrix.v1",
        "source_sha": sha,
        "cutoffs_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "all_rows": len(rows),
        "five_hosted_workers": True,
        "public_key_fixture_not_random_monte_carlo": True,
        "total_false_safe_observed": sum(v["false_safe"] for v in rows.values()),
        "groups": groups,
        "verdict": "B2A_PHYSICAL_3240_PASS_UTILITY_CANDIDATE",
        "scope": "RESEARCH_ONLY_NO_PRODUCT_GO_NO_ADAPTIVE_PRF_SECURITY",
    }


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--input", type=Path, required=True)
    p.add_argument("--cutoffs", type=Path, required=True)
    p.add_argument("--head", required=True)
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args()
    result = aggregate(args.input, args.cutoffs, args.head)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, sort_keys=True, indent=2)+"\n")
    print(f"B2A_AGGREGATE_PASS rows={result['all_rows']} verdict={result['verdict']}")
    for rec in result["groups"]:
        if rec["T"] in (32,64) and rec["b"] in (8,10,11,12):
            if rec["ratio"] in ("1/4","3/4","9/10"):
                print(f"B2A_POWER T={rec['T']} d={rec['d']} b={rec['b']} "
                      f"cutoff={rec['cutoff']} safe={rec['safe']}/15")


if __name__ == "__main__":
    main()
