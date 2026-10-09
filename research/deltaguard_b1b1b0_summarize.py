#!/usr/bin/env python3
"""Strict 5-hosted-worker, 1000 hot/hot source-bound physical TCP records.

Research only. No calibrated WAN, malicious-peer authenticity or PRODUCT_GO.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import re
from pathlib import Path

LANES=((256,48,10,0),(256,57,10,10),(65536,48,100,10))
MODES=("guard","retained_delta","direct_full","resolved_guard")
CUTS="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
FIELDS=set("worker lane generation N d mbps delay mode bytes wall_ns odd safe fallback exact".split())
BOOT=set("worker lane N d bytes".split())
FINISH=set("worker lane records advance_bytes failure_bytes quit_bytes child_rss1 child_rss2 parent_rss".split())
MAGIC_END="DELTAGUARD_B1B1B0_WORKER_PASS"

def split(line:str,prefix:str,fields:set[str])->dict|None:
    if not line.startswith(prefix+" "):return None
    chunks=[p.split("=",1) for p in line[len(prefix)+1:].split()]
    result=dict(chunks)
    assert len(chunks)==len(result)==len(fields) and set(result)==fields
    for key,value in result.items():
        if key=="mode":
            assert value in MODES
        else:
            assert re.fullmatch(r"[0-9]+",value),(key,value)
            result[key]=int(value)
    return result

def check_sample(line:str)->dict|None:
    a=split(line,"B1B1B0_SAMPLE",FIELDS)
    if a is None:return None
    w,l,g=a["worker"],a["lane"],a["generation"]
    assert 1<=w<=5 and 0<=l<=2 and 2<=g<=21
    n,d,mbps,delay=LANES[l]
    assert (a["N"],a["d"],a["mbps"],a["delay"])==(n,d,mbps,delay)
    assert a["mode"] in (MODES if l==1 else MODES[:3])
    assert a["wall_ns"]>0 and a["exact"]==1
    full=224+8*(2*(n+g-1)+(d%2))
    guarded=736
    delta=242
    mode=a["mode"]
    assert a["fallback"] in (0,1)
    if mode in ("guard","resolved_guard"):
        assert 0<=a["odd"]<=d
        assert a["safe"]==int(a["odd"]<=48)
        if d<=48:assert a["safe"]==1
        expected_fallback=int(mode=="resolved_guard" and a["safe"]==0)
        assert a["fallback"]==expected_fallback
        assert a["bytes"]==guarded+expected_fallback*full
    else:
        assert a["odd"]==0 and a["safe"]==2 and a["fallback"]==0
        assert a["bytes"]==(delta if mode=="retained_delta" else full)
    return a

def check_boot(line:str)->dict|None:
    a=split(line,"B1B1B0_BOOT",BOOT)
    if a is None:return None
    assert a["worker"] in range(1,6) and a["lane"] in range(3)
    n,d,_,_=LANES[a["lane"]]
    assert (a["N"],a["d"])==(n,d)
    assert a["bytes"]==226+8*(2*n+d%2)
    return a

def check_finish(line:str)->dict|None:
    a=split(line,"B1B1B0_FINISH",FINISH)
    if a is None:return None
    assert a["worker"] in range(1,6) and a["lane"] in range(3)
    assert a["records"]==(80 if a["lane"]==1 else 60)
    assert a["advance_bytes"]==21*96
    assert a["failure_bytes"]==32+64+9+16+242
    assert a["quit_bytes"]==112
    assert all(a[k]>0 for k in ("child_rss1","child_rss2","parent_rss"))
    return a

def self_test():
    sample=("B1B1B0_SAMPLE worker=1 lane=0 generation=2 N=256 d=48 "
            "mbps=10 delay=0 mode=guard bytes=736 wall_ns=350 odd=46 "
            "safe=1 fallback=0 exact=1")
    assert check_sample(sample)
    for bad in (
        sample.replace("bytes=736","bytes=737"),
        sample.replace("safe=1","safe=0"),
        sample.replace("generation=2","generation=1"),
        sample.replace("exact=1","exact=0"),
        sample.replace("mode=guard","mode=resolved_guard"),
    ):
        try: check_sample(bad)
        except (AssertionError,KeyError,ValueError):continue
        raise AssertionError("bad sample was accepted")

def aggregate(root:Path,head:str,certificate:Path)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    assert hashlib.sha256(certificate.read_bytes()).hexdigest()==CUTS
    self_test()
    samples={}
    boots={}
    ends={}
    for w in range(1,6):
        assert (root/f"head-{w}.txt").read_text().strip()==head
        assert (root/f"rust-{w}.txt").exists()
        lines=(root/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"{MAGIC_END} worker={w} records=200 retry_tests=3")==1
        counts=[0,0,0]
        for line in lines:
            s=check_sample(line)
            if s is not None:
                assert s["worker"]==w
                key=(w,s["lane"],s["generation"],s["mode"])
                assert key not in samples
                samples[key]=s
                counts[0]+=1
                continue
            b=check_boot(line)
            if b is not None:
                assert b["worker"]==w
                key=(w,b["lane"])
                assert key not in boots
                boots[key]=b
                counts[1]+=1
                continue
            e=check_finish(line)
            if e is not None:
                assert e["worker"]==w
                key=(w,e["lane"])
                assert key not in ends
                ends[key]=e
                counts[2]+=1
        assert counts==[200,3,3],(w,counts)
    assert len(samples)==1000 and len(boots)==15 and len(ends)==15
    findings=[]
    for lane,(n,d,mbps,delay) in enumerate(LANES):
        for mode in (MODES if lane==1 else MODES[:3]):
            workers=[]
            for w in range(1,6):
                rows=[samples[(w,lane,g,mode)] for g in range(2,22)]
                assert len(rows)==20
                ts=sorted(r["wall_ns"] for r in rows)
                assert all(v>0 for v in ts)
                workers.append({
                    "worker":w,"n":20,
                    "p50_ns":int((ts[9]+ts[10])/2),
                    "p95_nearest_rank_ns":ts[18],
                    "p95_scope":"EMPIRICAL_N20_NOT_CONFIDENCE_BOUND",
                    "mean_ns":sum(ts)//20,
                    "total_bytes":sum(x["bytes"] for x in rows),
                    "min_bytes":min(x["bytes"] for x in rows),
                    "max_bytes":max(x["bytes"] for x in rows),
                    "unknown_count":sum(int(x["safe"]==0) for x in rows),
                    "fallback_count":sum(x["fallback"] for x in rows),
                    "child1_post_query_rss":ends[(w,lane)]["child_rss1"],
                    "child2_post_query_rss":ends[(w,lane)]["child_rss2"],
                    "parent_rss":ends[(w,lane)]["parent_rss"],
                    "source_bootstrap_bytes":boots[(w,lane)]["bytes"],
                    "common_advance_bytes":ends[(w,lane)]["advance_bytes"],
                    "negative_retry_bytes":ends[(w,lane)]["failure_bytes"],
                    "graceful_exit_telemetry_bytes":ends[(w,lane)]["quit_bytes"],
                })
            findings.append({"lane":lane,"N":n,"d":d,"mbps_per_sender":mbps,
                "injected_delay_ms":delay,"mode":mode,"workers":workers})
    return {
        "schema":"deltameter.b1b1b0.persistent-hot-hot.v1",
        "source_sha":head,
        "certificate_sha256":CUTS,
        "workers":5,"records":1000,"sessions":15,
        "independent_sender_os_processes":True,
        "persistent_source_generation_1_to_22":True,
        "physical_ack_and_nack_with_one_checksum_replay":True,
        "crash_restart_reconnection_tested":False,
        "full_cpu_clock_tested":False,
        "product_p95_acceptance":False,
        "injected_application_delay_is_wan_rtt":False,
        "comparison":findings,
        "verdict":"B1B1B0_PERSISTENT_HOT_HOT_CORRECTNESS_ACCEPT_NO_PRODUCT_GO",
    }

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    result=aggregate(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(result,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B0_AGGREGATE_PASS records={result['records']} verdict={result['verdict']}")
    for x in result["comparison"]:
        if x["lane"] in (0,1):
            print(f"B1B1B0_HOT lane={x['lane']} mode={x['mode']} "
                  f"worker_p95_ns={[w['p95_nearest_rank_ns'] for w in x['workers']]} "
                  f"worker_total_bytes={[w['total_bytes'] for w in x['workers']]}")
if __name__=="__main__":main()
