#!/usr/bin/env python3
"""G1B: fail-closed 900-row 5-worker exact-cutoff compatibility/power analysis."""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path

TS = (32,64,256)
NUM = (0,2,6,8,9)
PROFILES = ("original","unit")
LEVELS = (1,2,3,4)
FIELDS = {"worker","scenario","ratio","seed","T","d","profile","level",
          "cutoff","odd","safe","false_safe","bitmap_bytes"}

def cutoffs(path: Path) -> dict[tuple[int,str,int],int]:
    lines=path.read_text(encoding="ascii").splitlines()
    assert lines[0]=="# format deltameter.guard-g1b-cutoff.v1"
    data={}
    for line in lines:
        if line.startswith("#"):
            continue
        words=line.split()
        assert len(words)==4
        t,profile,j,c=words
        key=(int(t),profile,int(j))
        assert key not in data and key[0] in TS and profile in PROFILES
        assert key[2] in LEVELS
        data[key]=int(c)
    assert len(data)==24
    return data

def parse(line:str,cuts:dict)->dict|None:
    if not line.startswith("G1B_SAMPLE "):
        return None
    parts=[p.split("=",1) for p in line[len("G1B_SAMPLE "):].split()]
    rec=dict(parts)
    assert len(parts)==len(FIELDS)==len(rec) and set(rec)==FIELDS
    assert rec["profile"] in PROFILES
    for key in FIELDS-{"profile"}:
        assert re.fullmatch(r"-?[0-9]+",rec[key]),(key,rec[key])
        rec[key]=int(rec[key])
    w,s,q,seed,t,d,p,j=(rec["worker"],rec["scenario"],rec["ratio"],
                       rec["seed"],rec["T"],rec["d"],rec["profile"],rec["level"])
    assert w in range(1,6) and s in range(len(TS)) and q in range(len(NUM))
    assert seed in range(3) and j in LEVELS and t==TS[s] and d==t*NUM[q]//8
    assert rec["cutoff"]==cuts[(t,p,j)]
    assert rec["bitmap_bytes"]==512
    assert 0<=rec["odd"]<=min(d,4096)
    assert rec["safe"]==int(rec["cutoff"]>=0 and rec["odd"]<=rec["cutoff"])
    assert rec["false_safe"]==int(bool(rec["safe"]) and d>t)
    return rec

def run(root:Path, source:str,cutoff_path:Path)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",source)
    cuts=cutoffs(cutoff_path)
    rows={}
    for w in range(1,6):
        lines=(root/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_G1B_WORKER_PASS worker={w}")==1
        assert (root/f"head-{w}.txt").read_text().strip()==source
        local=0
        for line in lines:
            rec=parse(line,cuts)
            if rec is None:continue
            assert rec["worker"]==w
            key=(w,rec["scenario"],rec["ratio"],rec["seed"],rec["profile"],rec["level"])
            assert key not in rows
            rows[key]=rec
            local+=1
        assert local==3*5*3*2*4, (w,local)
    assert len(rows)==900
    counts=[]
    for profile in PROFILES:
        for tix,t in enumerate(TS):
            for q,num in enumerate(NUM):
                for j in LEVELS:
                    group=[rows[(w,tix,q,seed,profile,j)]
                           for w in range(1,6) for seed in range(3)]
                    assert len(group)==15
                    counts.append({"T":t,"d_ratio_num":num,
                                   "d_ratio_den":8,"profile":profile,"level":j,
                                   "cutoff":cuts[(t,profile,j)],
                                   "safe":sum(x["safe"] for x in group),
                                   "false_safe_observed":sum(x["false_safe"] for x in group),
                                   "samples":15})
    qualifying=[x for x in counts if x["profile"]=="unit" and (
        (x["T"]==32 and x["d_ratio_num"]==2 and x["safe"]>=5)
        or (x["T"]==64 and x["d_ratio_num"]==2 and x["safe"]>=12))]
    return {
        "format":"deltameter.guard-g1b-exact-power.v1",
        "source_sha":source,"raw_rows":len(rows),"five_workers":True,
        "cutoff_sha256":hashlib.sha256(cutoff_path.read_bytes()).hexdigest(),
        "fixed_public_test_key_not_statistical_proof":True,
        "physical_owner_bitmap_bytes":512,
        "total_false_safe_observed":sum(x["false_safe"] for x in rows.values()),
        "profiles_power":counts,
        "qualifying_unit_profiles":qualifying,
        "verdict":("G1B_B1_XOR_PASS_EMPIRICAL_UNIT_CANDIDATE" if qualifying
                   else "G1B_B1_XOR_PASS_LOW_POWER"),
        "public_contract":"NOT_AUTHORIZED",
        "note":"All 900 records are deterministic source- and certificate-bound correctness/utility observations, not random-key Monte Carlo.",
    }

def main()->None:
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--cutoffs",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    report=run(a.input,a.head,a.cutoffs)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(report,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print("G1B_B1_AGGREGATE_PASS rows=900 result="+report["verdict"])
    for row in report["profiles_power"]:
        if row["T"] in (32,64) and row["d_ratio_num"] in (2,6):
            print(f"G1B_POWER T={row['T']} d_over_t={row['d_ratio_num']}/8 "
                  f"profile={row['profile']} j={row['level']} "
                  f"cutoff={row['cutoff']} safe={row['safe']}/15")
if __name__=="__main__":
    main()
