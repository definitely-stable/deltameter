#!/usr/bin/env python3
"""Fail-closed G0 fixed-m small parity guard identity/power; no statistical proof from samples."""
from __future__ import annotations
import argparse
import json
import re
from pathlib import Path

WORKERS=range(1,6)
TS=(64,4096,65536,1048576)
NUM=(0,2,6,8,9)
KS=(1,2,4,8)
SEEDS=range(3)
FIELDS={"worker","scenario","ratio","seed","T","d","k","bytes","pair_bytes",
        "full_bound","guard_bound","safe","false_safe"}
ROWS=5*len(TS)*len(NUM)*len(SEEDS)*len(KS)

def parse(line:str)->dict|None:
    if not line.startswith("GUARD_G0_SAMPLE "):return None
    parts=[pair.split("=",1) for pair in line[len("GUARD_G0_SAMPLE "):].split()]
    row=dict(parts)
    assert len(parts)==len(FIELDS)==len(row) and set(row)==FIELDS
    for field in FIELDS:
        assert re.fullmatch(r"[0-9]+",row[field]),(field,row[field])
        row[field]=int(row[field])
    w,s,q,seed,k=row["worker"],row["scenario"],row["ratio"],row["seed"],row["k"]
    assert w in WORKERS and s in range(len(TS)) and q in range(len(NUM))
    assert seed in SEEDS and k in KS
    assert row["T"]==TS[s] and row["d"]==TS[s]*NUM[q]//8
    assert row["bytes"]==512*k and row["pair_bytes"]==1024*k
    assert 0<=row["full_bound"]<=row["guard_bound"]<=1<<64
    assert row["safe"]==int(row["guard_bound"]<=row["T"])
    assert row["false_safe"]==int(bool(row["safe"]) and row["d"]>row["T"])
    return row

def aggregate(root:Path,head:str)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    seen={}
    for w in WORKERS:
        f=root/f"worker-{w}.txt"
        src=root/f"head-{w}.txt"
        assert f.is_file() and src.is_file()
        assert src.read_text(encoding="utf-8").strip()==head
        lines=f.read_text(encoding="utf-8").splitlines()
        assert lines.count(f"DELTAGUARD_G0_WORKER_PASS worker={w}")==1
        local=0
        for line in lines:
            row=parse(line)
            if row is None: continue
            assert row["worker"]==w
            key=(w,row["scenario"],row["ratio"],row["seed"],row["k"])
            assert key not in seen,key
            seen[key]=row
            local+=1
        assert local==len(TS)*len(NUM)*len(SEEDS)*len(KS),(w,local)
    assert len(seen)==ROWS,len(seen)
    for w in WORKERS:
        for s in range(len(TS)):
            for q in range(len(NUM)):
                for seed in SEEDS:
                    data=[seen[(w,s,q,seed,k)] for k in KS]
                    baseline=data[0]["full_bound"]
                    assert all(z["full_bound"]==baseline for z in data)
                    # The selected k-level subsets are nested in the frozen T-only order.
                    assert all(data[i]["guard_bound"]>=data[i+1]["guard_bound"]
                               for i in range(len(data)-1))
    points=[]
    for s,t in enumerate(TS):
        for q,numer in enumerate(NUM):
            for k in KS:
                rows=[seen[(w,s,q,seed,k)] for w in WORKERS for seed in SEEDS]
                points.append({"T":t,"ratio_num":numer,"ratio_den":8,"k":k,
                               "physical_owner_payload_bytes":512*k,
                               "two_owner_payload_bytes":1024*k,
                               "samples":len(rows),
                               "safe_count":sum(x["safe"] for x in rows),
                               "false_safe_observed_count":sum(x["false_safe"] for x in rows),
                               "ceiling_count":sum(x["guard_bound"]==(1<<64) for x in rows)})
    near=[z for z in points if z["ratio_num"]==6]
    go=any(z["safe_count"]>=12 for z in near)
    return {
        "format":"deltameter.guard-g0.fixed-m4096.v1",
        "source_head":head,
        "proof_boundary":"C0 simultaneous ideal-oracle confidence inherited; no adaptive-token guarantee",
        "hash_test_key":"public deterministic test-only fixture; not a secret",
        "table_sha256":"634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a",
        "raw_rows":len(seen),"workers":5,
        "thresholds":list(TS),"ratios_numerators":list(NUM),
        "ratio_denominator":8,"seeds_per_worker":3,
        "k_values":list(KS),
        "detailed_power":points,
        "observed_false_safe_total":sum(x["false_safe"] for x in seen.values()),
        "qualifying_empirical_75percent_power_lanes":[
            {"T":z["T"],"k":z["k"],"useful":z["safe_count"],"out_of":z["samples"]}
            for z in near if z["safe_count"]>=12],
        "result":("G0_FOUNDATION_PASS_EMPIRICAL_POWER_CANDIDATE" if go
                  else "G0_FOUNDATION_PASS_WITH_LOW_POWER"),
        "public_result":"NOT_AUTHORIZED",
        "note":"15 deterministic fixed-key input variations per cell; never infer 1e-6 rare-event reliability from these samples."
    }

def main()->None:
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    report=aggregate(a.input,a.head)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(report,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(f"DELTAGUARD_G0_EVIDENCE_PASS samples={report['raw_rows']} result={report['result']} false_safe_observed={report['observed_false_safe_total']}")
    for row in report["detailed_power"]:
        if row["ratio_num"] in (2,6,8,9):
            print(f"GUARD_G0_POWER T={row['T']} ratio={row['ratio_num']}/8 k={row['k']} safe={row['safe_count']}/{row['samples']} ceiling={row['ceiling_count']}/{row['samples']}")

if __name__=="__main__":
    main()
