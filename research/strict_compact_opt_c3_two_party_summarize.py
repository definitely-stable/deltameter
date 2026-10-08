#!/usr/bin/env python3
"""C3-B five-worker two-party TCP lifecycle, strict provenance/completeness."""
from __future__ import annotations
import argparse
import json
import math
import re
from pathlib import Path

WORKERS=range(1,6)
SCENARIOS=((4096,8192),(65536,131072),(1048576,2097152))
SEEDS=range(8)
RTTS=(0,10,50)
STAGES=(("dual_full",52),("dual_reuse",1),("dual_reuse",4),("dual_reuse",52))
FIELDS={"worker","scenario","seed","d","T","mode","levels","mbps","rtt_ms",
        "bytes","requests","bound","useful","elapsed_ns"}
EXPECTED={("dual_full",52):(53500,2),("dual_reuse",1):(1344,4),
          ("dual_reuse",4):(4576,6),("dual_reuse",52):(55778,8)}
EXPECTED_COUNT=len(WORKERS)*len(SCENARIOS)*len(SEEDS)*len(RTTS)*len(STAGES)

def rank(values:list[int],p:float)->int:
    return sorted(values)[math.ceil(p*len(values))-1]

def parse(line:str)->dict|None:
    if not line.startswith("C3B_SAMPLE "):
        return None
    items=[x.split("=",1) for x in line[len("C3B_SAMPLE "):].split()]
    row=dict(items)
    assert len(row)==len(items)==len(FIELDS) and set(row)==FIELDS
    for f in FIELDS-{"mode"}:
        assert re.fullmatch(r"[0-9]+",row[f]), f
        row[f]=int(row[f])
    assert row["worker"] in WORKERS and row["scenario"] in range(3)
    assert row["seed"] in SEEDS and row["rtt_ms"] in RTTS and row["mbps"]==10
    assert (row["d"],row["T"])==SCENARIOS[row["scenario"]]
    assert (row["mode"],row["levels"]) in EXPECTED
    assert (row["bytes"],row["requests"])==EXPECTED[(row["mode"],row["levels"])]
    assert 0<=row["bound"]<=1<<64
    assert row["useful"]==int(row["bound"]<=row["T"])
    assert row["elapsed_ns"]>0
    return row

def summarize(directory:Path,head:str)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    obs={}
    for worker in WORKERS:
        log=directory/f"worker-{worker}.txt"
        src=directory/f"head-{worker}.txt"
        assert log.is_file() and src.is_file()
        assert src.read_text(encoding="utf-8").strip()==head
        lines=log.read_text(encoding="utf-8").splitlines()
        assert lines.count(f"STRICT_COMPACT_OPT_C3B_WORKER_PASS worker={worker}")==1
        count=0
        for line in lines:
            row=parse(line)
            if row is None:continue
            assert row["worker"]==worker
            key=(worker,row["scenario"],row["seed"],row["rtt_ms"],row["mode"],row["levels"])
            assert key not in obs
            obs[key]=row
            count+=1
        assert count==len(SCENARIOS)*len(SEEDS)*len(RTTS)*len(STAGES), (worker,count)
    assert len(obs)==EXPECTED_COUNT
    per_worker=[]
    for worker in WORKERS:
        for scenario in range(3):
            for rtt in RTTS:
                pairs={}
                for mode,levels in STAGES:
                    rows=[obs[(worker,scenario,seed,rtt,mode,levels)] for seed in SEEDS]
                    tag=mode+"_"+str(levels)
                    pairs[tag]={
                        "samples":len(rows),"useful":sum(x["useful"] for x in rows),
                        "p50_bytes":rank([x["bytes"] for x in rows],.5),
                        "p95_bytes":rank([x["bytes"] for x in rows],.95),
                        "p50_ns":rank([x["elapsed_ns"] for x in rows],.5),
                        "p95_ns":rank([x["elapsed_ns"] for x in rows],.95),
                    }
                for seed in SEEDS:
                    full=obs[(worker,scenario,seed,rtt,"dual_full",52)]["bound"]
                    last=1<<64
                    for levels in (1,4,52):
                        bound=obs[(worker,scenario,seed,rtt,"dual_reuse",levels)]["bound"]
                        assert full<=bound<=last, "nonmonotone or unsound partial bound"
                        last=bound
                    assert last==full, "final reconstructed XOR differs"
                per_worker.append({
                    "worker":worker,"scenario":scenario,"rtt_ms":rtt,"modes":pairs,
                })
    qualifying=[]
    for scenario in range(3):
        yes=True
        for worker in WORKERS:
            for rtt in RTTS:
                row=next(z for z in per_worker if
                         z["worker"]==worker and z["scenario"]==scenario
                         and z["rtt_ms"]==rtt)
                first=row["modes"]["dual_reuse_1"]
                full=row["modes"]["dual_full_52"]
                yes &= first["useful"]>=7 and first["p95_bytes"]<=.75*full["p95_bytes"]
                yes &= first["p95_ns"]<=full["p95_ns"]
        if yes:qualifying.append(scenario)
    return {
        "format":"deltameter.strict-compact-opt-c3b.v1",
        "source_head":head,"raw_checkpoints":len(obs),
        "transport":"two actual independent TCP streams, app-layer 10Mbps RTT pacing",
        "network_claim":"controlled localhost emulation only, no WAN claim",
        "per_worker_scenario_rtt":per_worker,
        "two_party_first_upper_bound_qualifying_scenarios":qualifying,
        "full_interchange_control_bytes":53500,
        "retained_level1_bytes":1344,"retained_level4_bytes":4576,
        "retained_full52_bytes":55778,
        "scoped_result":("C3B_CONTROLLED_TWO_PARTY_PASS" if len(qualifying)>=2
                          else "C3B_TWO_PARTY_NO_GO"),
        "public_progressive_api":"NOT_AUTHORIZED",
    }

def main()->None:
    p=argparse.ArgumentParser()
    p.add_argument("--logs-dir",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    x=summarize(a.logs_dir,a.head)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(x,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(f"C3B_EVIDENCE_PASS records={x['raw_checkpoints']} scoped={x['scoped_result']}")
    print(f"C3B_SCENARIOS {x['two_party_first_upper_bound_qualifying_scenarios']}")
    for row in x["per_worker_scenario_rtt"]:
        if row["worker"]==1 and row["rtt_ms"]==10:
            for k,v in row["modes"].items():
                print(f"C3B_SUMMARY scenario={row['scenario']} mode={k} p95_bytes={v['p95_bytes']} p95_ns={v['p95_ns']} useful={v['useful']}/8")

if __name__=="__main__":
    main()
