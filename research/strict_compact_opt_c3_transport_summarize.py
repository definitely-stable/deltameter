#!/usr/bin/env python3
"""Fail-closed hosted C3-A TCP/functional evidence; no WAN/product verdict."""
from __future__ import annotations

import argparse
import json
import math
import re
from pathlib import Path

WORKERS = range(1, 6)
SCENARIOS = ((4096,8192),(65536,131072),(1048576,2097152))
SEEDS = range(8)
MODES = ("scalar","complete","bounded","interactive4")
RTTS = (0,10,50)
C3_FIELDS = {"worker","scenario","seed","d","T","mode","mbps","rtt_ms",
             "bytes","requests","bound","useful","elapsed_ns","copy_ns"}
XOR_FIELDS = {"worker","scenario","seed","levels","bytes","bound",
              "useful","full_bound"}
TOTAL_TCP = len(WORKERS)*len(SCENARIOS)*len(SEEDS)*len(MODES)*len(RTTS)
TOTAL_XOR = len(WORKERS)*len(SCENARIOS)*len(SEEDS)*3

def p95(vals: list[int]) -> int:
    assert vals
    return sorted(vals)[math.ceil(len(vals)*.95)-1]

def decode_record(line: str, prefix: str, fields: set[str]) -> dict | None:
    if not line.startswith(prefix):
        return None
    items = [x.split("=",1) for x in line[len(prefix):].split()]
    data = dict(items)
    assert len(items) == len(fields) == len(data) and set(data) == fields
    for key in fields-{"mode"}:
        assert re.fullmatch(r"[0-9]+", data[key]), f"invalid {key}"
        data[key] = int(data[key])
    return data

def parse_tcp(row: dict) -> None:
    assert row["worker"] in WORKERS and row["scenario"] in range(3)
    assert row["seed"] in SEEDS and row["mode"] in MODES and row["rtt_ms"] in RTTS
    assert row["mbps"] == 10
    assert (row["d"],row["T"]) == SCENARIOS[row["scenario"]]
    assert row["requests"] >= 1 and row["useful"] in (0,1)
    assert 0 <= row["bound"] <= 1<<64 and row["useful"] == (row["bound"]<=row["T"])
    assert row["elapsed_ns"] > 0 and row["copy_ns"] >= 0
    assert row["elapsed_ns"] >= row["copy_ns"]
    if row["mode"] == "scalar":
        assert row["bytes"] == 24 and row["requests"] == 1
    if row["mode"] == "complete":
        assert row["bytes"] == 26750 and row["requests"] == 1
    if row["mode"] == "bounded":
        assert row["bytes"] >= 655 and (row["bytes"]-122)%533 == 0
        assert row["bytes"]<=122+52*533 and row["requests"] == 1
    if row["mode"] == "interactive4":
        n = row["requests"]-1
        assert 1 <= n <= 13
        assert row["bytes"] == 122 + n*17 + n*4*533 or (
            n == 13 and row["bytes"] == 122+n*17+52*533
        )

def parse_xor(row: dict) -> None:
    assert row["worker"] in WORKERS and row["scenario"] in range(3)
    assert row["seed"] in SEEDS and row["levels"] in (1,4,52)
    assert row["bytes"] == 2*(122+row["levels"]*533)
    assert row["useful"] in (0,1) and 0 <= row["full_bound"] <= row["bound"] <= 1<<64
    assert row["useful"] == (row["bound"]<=SCENARIOS[row["scenario"]][1])
    if row["levels"] == 52:
        assert row["bound"] == row["full_bound"]

def aggregate(path: Path, head: str) -> dict:
    assert re.fullmatch(r"[a-f0-9]{40}",head)
    tcp, reuse = {}, {}
    for w in WORKERS:
        f=path/f"worker-{w}.txt"
        p=path/f"head-{w}.txt"
        assert f.is_file() and p.is_file()
        assert p.read_text(encoding="utf-8").strip() == head
        lines=f.read_text(encoding="utf-8").splitlines()
        assert lines.count(f"STRICT_COMPACT_OPT_C3A_WORKER_PASS worker={w}") == 1
        tc=0
        xr=0
        for line in lines:
            row=decode_record(line,"C3_TCP_SAMPLE ",C3_FIELDS)
            if row is not None:
                parse_tcp(row)
                assert row["worker"] == w
                key=(w,row["scenario"],row["seed"],row["mode"],row["rtt_ms"])
                assert key not in tcp
                tcp[key]=row
                tc+=1
                continue
            row=decode_record(line,"C3_XOR_REUSE ",XOR_FIELDS)
            if row is not None:
                parse_xor(row)
                assert row["worker"] == w
                key=(w,row["scenario"],row["seed"],row["levels"])
                assert key not in reuse
                reuse[key]=row
                xr+=1
        assert tc == len(SCENARIOS)*len(SEEDS)*len(MODES)*len(RTTS), (w,tc)
        assert xr == len(SCENARIOS)*len(SEEDS)*3, (w,xr)
    assert len(tcp) == TOTAL_TCP and len(reuse) == TOTAL_XOR
    metrics=[]
    gates=[]
    for w in WORKERS:
        for scenario in range(3):
            for rtt in RTTS:
                base=[tcp[(w,scenario,s,"complete",rtt)] for s in SEEDS]
                scalar=[tcp[(w,scenario,s,"scalar",rtt)] for s in SEEDS]
                assert all(x["bound"]==y["bound"] for x,y in zip(base,scalar))
                for mode in MODES:
                    rows=[tcp[(w,scenario,s,mode,rtt)] for s in SEEDS]
                    for sample,full in zip(rows,base):
                        assert sample["bound"]>=full["bound"]
                    metrics.append({
                        "worker":w,"scenario":scenario,"rtt_ms":rtt,"mode":mode,
                        "samples":len(rows),"useful":sum(x["useful"] for x in rows),
                        "p95_bytes":p95([x["bytes"] for x in rows]),
                        "p95_elapsed_ns":p95([x["elapsed_ns"] for x in rows]),
                        "min_elapsed_ns":min(x["elapsed_ns"] for x in rows),
                        "max_elapsed_ns":max(x["elapsed_ns"] for x in rows),
                        "p95_requests":p95([x["requests"] for x in rows]),
                    })
            for levels in (1,4,52):
                rows=[reuse[(w,scenario,s,levels)] for s in SEEDS]
                gates.append({"worker":w,"scenario":scenario,"reused_levels":levels,
                              "useful":sum(x["useful"] for x in rows),
                              "p95_two_party_bytes":p95([x["bytes"] for x in rows])})
    # Functionally matched: no scalar substitution into XOR or full-interchange.
    # C3-A is controlled localhost emulation, therefore NOT definitive C3.
    controlled=[]
    for mode in ("bounded","interactive4"):
        accepted=[]
        for scenario in range(3):
            ok=True
            for worker in WORKERS:
                for rtt in RTTS:
                    current=next(m for m in metrics if m["worker"]==worker and
                                 m["scenario"]==scenario and m["mode"]==mode and m["rtt_ms"]==rtt)
                    full=next(m for m in metrics if m["worker"]==worker and
                              m["scenario"]==scenario and m["mode"]=="complete" and m["rtt_ms"]==rtt)
                    ok &= current["useful"]>=7
                    ok &= current["p95_bytes"] <= int(full["p95_bytes"]*.75)
                    ok &= current["p95_elapsed_ns"] <= full["p95_elapsed_ns"]
            if ok:
                accepted.append(scenario)
        controlled.append({"mode":mode,"scenarios_passing_all_workers":accepted})
    return {
        "format":"deltameter.strict-compact-c3a-tcp.v1",
        "source_head":head,
        "tcp_samples":TOTAL_TCP,"xor_reuse_samples":TOTAL_XOR,
        "transport":"real loopback TCP with application-layer RTT/rate sleep; not WAN",
        "bandwidth_emulation_mbps":10,"rtt_emulation_ms":list(RTTS),
        "threshold_only_fair_scalar_bytes":24,
        "full_interchange_bytes":26750,
        "per_worker_protocol":metrics,
        "local_two_party_reuse_oracle":gates,
        "controlled_emulation_gate":controlled,
        "decision":"C3_A_TRANSPORT_FOUNDATION_PASS",
        "final_product_verdict":"NOT_DETERMINED",
    }

def main() -> None:
    p=argparse.ArgumentParser()
    p.add_argument("--logs-dir",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    args=p.parse_args()
    output=aggregate(args.logs_dir,args.head)
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(output,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(f"C3A_EVIDENCE_PASS tcp={output['tcp_samples']} xor={output['xor_reuse_samples']} decision={output['decision']}")
    for result in output["controlled_emulation_gate"]:
        print(f"C3A_GATE mode={result['mode']} scenarios={result['scenarios_passing_all_workers']}")
    for metric in output["per_worker_protocol"]:
        if metric["worker"]==1 and metric["rtt_ms"]==10:
            print(f"C3A_SAMPLE_SUMMARY scenario={metric['scenario']} mode={metric['mode']} p95_bytes={metric['p95_bytes']} p95_elapsed_ns={metric['p95_elapsed_ns']} useful={metric['useful']}/8")
    for row in output["local_two_party_reuse_oracle"]:
        if row["worker"]==1:
            print(f"C3A_XOR_SUMMARY scenario={row['scenario']} levels={row['reused_levels']} bytes={row['p95_two_party_bytes']} useful={row['useful']}/8")

if __name__=="__main__":
    main()
