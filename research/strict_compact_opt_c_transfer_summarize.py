#!/usr/bin/env python3
"""OPT-C C2: fail-closed five-worker physical frame evidence + modeled network grid."""
from __future__ import annotations

import argparse
import json
import math
import re
import statistics
from collections import defaultdict
from pathlib import Path

WORKERS = range(1, 6)
SCENARIOS = ((4096, 8192), (65536, 131072), (1048576, 2097152))
SEEDS = range(32)
ORDERS = ("asc", "desc", "center")
MODES = (("pushall", 0), ("bounded", 0), ("elide", 0),
         ("interactive", 1), ("interactive", 2), ("interactive", 4),
         ("interactive", 8))
ROWS = 4096
J = 52
COMMON = 96 + 8 + 18
FULL_BYTES = COMMON + 26624 + 4
FRAME = 533
REQUEST = 17
FIELDS = {"worker", "scenario", "seed", "order", "mode", "batch",
          "d", "T", "bytes", "frames", "requests", "selected", "useful",
          "bound", "copy_ns", "prep_ns", "process_ns"}
TOTAL = 5 * len(SCENARIOS) * len(SEEDS) * (1 + len(ORDERS)*len(MODES))
NETWORK = tuple((mbps, rtt) for mbps in (1, 10, 100) for rtt in (0, 10, 50, 150))

def nearest_rank(values: list[float], probability: float) -> float:
    assert values
    return sorted(values)[math.ceil(len(values)*probability)-1]

def parse(line: str) -> dict | None:
    if not line.startswith("C2_SAMPLE "):
        return None
    pairs = [part.split("=", 1) for part in line[len("C2_SAMPLE "):].split()]
    obj = dict(pairs)
    assert len(obj) == len(FIELDS) and set(obj) == FIELDS, "malformed record"
    for field in FIELDS-{"order", "mode"}:
        assert re.fullmatch(r"[0-9]+", obj[field]), f"non-decimal {field}"
        obj[field] = int(obj[field])
    assert obj["worker"] in WORKERS and obj["scenario"] in range(3)
    assert obj["seed"] in SEEDS and obj["useful"] in (0,1)
    assert (obj["d"],obj["T"]) == SCENARIOS[obj["scenario"]]
    assert obj["copy_ns"] >= 0 and obj["prep_ns"] >= 0
    assert obj["process_ns"] > 0
    assert obj["bytes"] >= COMMON and obj["frames"] <= J and obj["selected"] <= J
    assert 0 <= obj["bound"] <= 1 << 64
    assert obj["useful"] == (obj["bound"] <= obj["T"])
    if obj["mode"] == "complete":
        assert obj["order"] == "none" and obj["batch"] == 0
        assert obj["bytes"] == FULL_BYTES and obj["frames"] == 0
        assert obj["requests"] == 1 and obj["selected"] == J
    else:
        assert obj["order"] in ORDERS and (obj["mode"],obj["batch"]) in MODES
        assert obj["requests"] >= 1 and obj["selected"] >= 1
        if obj["mode"] == "pushall":
            assert obj["selected"] == J and obj["frames"] == J
        if obj["mode"] == "bounded":
            assert obj["frames"] == obj["selected"]
        if obj["mode"] == "elide":
            assert obj["frames"] <= obj["selected"]
        if obj["mode"] == "interactive":
            assert obj["frames"] == obj["selected"]
            assert obj["requests"] == math.ceil(obj["selected"]/obj["batch"])
        else:
            assert obj["requests"] == 1
        extra = obj["requests"]*REQUEST if obj["mode"] == "interactive" else 0
        assert obj["bytes"] == COMMON + FRAME*obj["frames"] + extra
    return obj

def mode_name(item: dict) -> str:
    if item["mode"] == "complete":
        return "complete"
    return item["order"] + "/" + item["mode"] + (
        "_" + str(item["batch"]) if item["mode"] == "interactive" else "")

def model_us(item: dict, mbps: int, rtt_ms: int) -> float:
    # Measured CPU components + *idealized* serialization and RTT on lossless
    # 1-hop link. NOT an actual remote network timing observation.
    return ((item["copy_ns"] + item["prep_ns"] + item["process_ns"])/1000.0
            + item["bytes"] * 8 / mbps
            + item["requests"] * rtt_ms * 1000)

def summarize(logdir: Path, expected_head: str) -> dict:
    assert re.fullmatch(r"[a-f0-9]{40}", expected_head)
    values: dict[tuple[int,int,int,str],dict] = {}
    for worker in WORKERS:
        file = logdir/f"worker-{worker}.txt"
        head = logdir/f"head-{worker}.txt"
        assert file.is_file() and head.is_file()
        assert head.read_text(encoding="utf-8").strip() == expected_head
        lines = file.read_text(encoding="utf-8").splitlines()
        assert lines.count(f"STRICT_COMPACT_OPT_C_C2_WORKER_PASS worker={worker}") == 1
        count = 0
        for line in lines:
            row = parse(line)
            if row is None:
                continue
            assert row["worker"] == worker
            key = (worker,row["scenario"],row["seed"],mode_name(row))
            assert key not in values, f"duplicate {key}"
            values[key] = row
            count += 1
        assert count == len(SCENARIOS)*len(SEEDS)*(1+len(ORDERS)*len(MODES)), (
            worker, count
        )
    assert len(values) == TOTAL
    all_modes = ("complete",) + tuple(
        order+"/"+mode+("_"+str(batch) if mode=="interactive" else "")
        for order in ORDERS for mode,batch in MODES
    )
    summary = []
    for worker in WORKERS:
        for scenario,(d,t) in enumerate(SCENARIOS):
            full = [values[(worker,scenario,seed,"complete")] for seed in SEEDS]
            for mode in all_modes:
                items = [values[(worker,scenario,seed,mode)] for seed in SEEDS]
                for i in range(len(items)):
                    assert items[i]["bound"] >= full[i]["bound"], "false improved partial bound"
                    if mode == "complete":
                        assert items[i]["bound"] == full[i]["bound"]
                metrics = {
                    "worker":worker, "scenario":scenario, "d":d,"T":t,"mode":mode,
                    "sample_count":len(items),
                    "p50_bytes":nearest_rank([x["bytes"] for x in items],.5),
                    "p95_bytes":nearest_rank([x["bytes"] for x in items],.95),
                    "max_bytes":max(x["bytes"] for x in items),
                    "min_bytes":min(x["bytes"] for x in items),
                    "p95_frames":nearest_rank([x["frames"] for x in items],.95),
                    "p95_requests":nearest_rank([x["requests"] for x in items],.95),
                    "useful_count":sum(x["useful"] for x in items),
                    "p95_saving_vs_complete": 1-nearest_rank(
                        [x["bytes"] for x in items],.95)/FULL_BYTES,
                    "ideal_link_grid":{},
                }
                for mbps,rtt in NETWORK:
                    # Pairwise model includes actual measured sender copy,
                    # full frame processing, sender Q32 selection and receiver scan.
                    latency = [model_us(x,mbps,rtt) for x in items]
                    metrics["ideal_link_grid"][f"{mbps}Mbps_{rtt}ms"] = {
                        "p50_us":nearest_rank(latency,.5),
                        "p95_us":nearest_rank(latency,.95),
                    }
                summary.append(metrics)
    # Require the SAME protocol/order in >=2 scenarios, across ALL workers;
    # a protocol that fails to reach U<=T for >5% of cases is ineligible.
    candidates=[]
    for mode in all_modes[1:]:
        passing = []
        for scenario in range(3):
            data=[next(s for s in summary if s["worker"]==w
                       and s["scenario"]==scenario and s["mode"]==mode)
                  for w in WORKERS]
            baseline=[next(s for s in summary if s["worker"]==w
                       and s["scenario"]==scenario and s["mode"]=="complete")
                      for w in WORKERS]
            valid = all(x["useful_count"]>=31 for x in data)
            bytes_ok = all(x["p95_saving_vs_complete"]>=.25 for x in data)
            time_ok = all(
                x["ideal_link_grid"][f"10Mbps_{rtt}ms"]["p95_us"]
                <= b["ideal_link_grid"][f"10Mbps_{rtt}ms"]["p95_us"]
                for x,b in zip(data,baseline) for rtt in (0,10,50)
            )
            if valid and bytes_ok and time_ok:
                passing.append(scenario)
        if len(passing)>=2:
            candidates.append({"mode":mode,"scenarios":passing})
    return {
        "format":"deltameter.strict-compact-opt-c-c2-model.v1",
        "source_head":expected_head,
        "samples":TOTAL,"per_worker_samples":TOTAL//5,
        "physical_complete_bytes":FULL_BYTES,
        "physical_pushall_bytes":COMMON+52*533,
        "workers":list(WORKERS),
        "network_model":"lossless serial-link + measured CPU; NOT measured RTT",
        "model_grid":[{"mbps":m,"rtt_ms":r} for m,r in NETWORK],
        "per_worker_scenario_protocol":summary,
        "modeled_qualifying_candidates":candidates,
        "preliminary_decision":("MODELED_C3_CANDIDATE" if candidates else "MODELED_C3_NO_GO"),
        "real_world_progressive_verdict":"NOT_TESTED",
    }

def main() -> None:
    p=argparse.ArgumentParser()
    p.add_argument("--logs-dir",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    out=summarize(a.logs_dir,a.head)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(out,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(f"STRICT_COMPACT_OPT_C_C2_EVIDENCE_PASS samples={out['samples']} "
          f"preliminary={out['preliminary_decision']}")
    print(f"MODELED_C2_QUALIFYING_MODES {len(out['modeled_qualifying_candidates'])}")
    for candidate in out["modeled_qualifying_candidates"][:12]:
        print(f"MODELED_C2_CANDIDATE mode={candidate['mode']} scenarios={candidate['scenarios']}")
    for s in out["per_worker_scenario_protocol"]:
        if s["worker"]==1 and s["mode"] in ("complete","center/bounded",
                                            "center/elide","center/interactive_4",
                                            "center/pushall"):
            print(f"C2_SCENARIO scenario={s['scenario']} mode={s['mode']} "
                  f"p95_bytes={s['p95_bytes']} useful={s['useful_count']}/32 "
                  f"saving={s['p95_saving_vs_complete']:.4f}")

if __name__=="__main__":
    main()
