#!/usr/bin/env python3
"""Fail-closed B1-B1-A two-child-process TCP physical-pacing evidence.

Actual bytes/wall times are source-bound; three repeats only, no product p95 GO.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path
from statistics import median

LANES=((256,48,10,10,0),(256,48,100,10,10),
       (256,57,10,10,50),(65536,48,10,100,10))
MODES=("guard","full","cold_exact","warm_exact","resolved")
CUTS="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
FIELDS=set("worker lane rep N d S mbps delay mode bytes warm_setup_bytes wall_ns total_ns rss_parent rss_owner1 rss_owner2 source_build_ns source_update_ns odd safe fallback exact_verified".split())

def check_record(line:str)->dict|None:
    if not line.startswith("B1B1A_SAMPLE "):return None
    pieces=[t.split("=",1) for t in line[len("B1B1A_SAMPLE "):].split()]
    rec=dict(pieces)
    assert len(pieces)==len(FIELDS)==len(rec) and set(rec)==FIELDS
    assert rec["mode"] in MODES
    for k in FIELDS-{"mode"}:
        assert re.fullmatch(r"[0-9]+",rec[k]),(k,rec[k])
        rec[k]=int(rec[k])
    w,l,rep=(rec[k] for k in ("worker","lane","rep"))
    assert w in range(1,6) and l in range(4) and rep in range(3)
    n,d,s,mbps,delay=LANES[l]
    assert (rec["N"],rec["d"],rec["S"],rec["mbps"],rec["delay"])==(n,d,s,mbps,delay)
    assert rec["exact_verified"]==1
    assert rec["bytes"]>0 and rec["wall_ns"]>0 and rec["total_ns"]>0
    assert all(rec[k]>0 for k in ("rss_parent","rss_owner1","rss_owner2"))
    assert rec["source_build_ns"]>=0 and rec["source_update_ns"]>=0
    full=128+8*(2*(n+s-1)+d%2)
    first=128+8*(2*n+d%2)
    warm_setup=50+48+first
    warm_query=48+128+18*(s-1)
    mode=rec["mode"]
    if mode in ("guard","resolved"):
        assert 0<=rec["odd"]<=d
        assert rec["safe"]==int(rec["odd"]<=48)
        if d<=48:assert rec["safe"]==1
        assert rec["warm_setup_bytes"]==0
        expected_fallback=int(mode=="resolved" and rec["safe"]==0)
        assert rec["fallback"]==expected_fallback
        expected=50+48+640+(48+full if expected_fallback else 0)
        assert rec["bytes"]==expected
    else:
        assert rec["safe"]==2 and rec["odd"]==0 and rec["fallback"]==0
        if mode=="full":
            assert rec["bytes"]==50+48+full and rec["warm_setup_bytes"]==0
        if mode=="cold_exact":
            assert rec["bytes"]==warm_setup+warm_query
            assert rec["warm_setup_bytes"]==0
        if mode=="warm_exact":
            assert rec["bytes"]==warm_query
            assert rec["warm_setup_bytes"]==warm_setup
            assert rec["wall_ns"]<=rec["total_ns"]
    return rec

def self_test():
    a=("B1B1A_SAMPLE worker=1 lane=0 rep=0 N=256 d=48 S=10 mbps=10 delay=0 "
       "mode=guard bytes=738 warm_setup_bytes=0 wall_ns=10 total_ns=15 "
       "rss_parent=1024 rss_owner1=2048 rss_owner2=2048 source_build_ns=1 "
       "source_update_ns=5 odd=44 safe=1 fallback=0 exact_verified=1")
    assert check_record(a)
    for bad in (a.replace("bytes=738","bytes=739"),
                a.replace("warm_setup_bytes=0","warm_setup_bytes=1"),
                a.replace("safe=1","safe=0"),a.replace("lane=0","lane=7"),
                a.replace("exact_verified=1","exact_verified=0")):
        try:check_record(bad)
        except (AssertionError,ValueError,KeyError):continue
        raise AssertionError("bad case accepted")

def summary(root:Path,head:str,cert:Path)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    assert hashlib.sha256(cert.read_bytes()).hexdigest()==CUTS
    self_test()
    records={}
    for w in range(1,6):
        assert (root/f"head-{w}.txt").read_text().strip()==head
        assert (root/f"rust-{w}.txt").exists()
        lines=(root/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1A_WORKER_PASS worker={w} records=60")==1
        count=0
        for line in lines:
            x=check_record(line)
            if x is None:continue
            assert x["worker"]==w
            key=(w,x["lane"],x["rep"],x["mode"])
            assert key not in records
            records[key]=x
            count+=1
        assert count==60
    assert len(records)==300
    res=[]
    for l,conditions in enumerate(LANES):
        for mode in MODES:
            by_worker=[]
            for w in range(1,6):
                rows=[records[w,l,r,mode] for r in range(3)]
                lat=sorted(x["wall_ns"] for x in rows)
                by_worker.append({
                    "worker":w, "n":3,
                    "latency_p50_ns":int(median(lat)),
                    "latency_nearest_rank_p95_ns":lat[-1],
                    "latency_p95_reliability":"LOW_N_EQ_3_MAXIMUM",
                    "bytes":rows[0]["bytes"],
                    "bootstrap_bytes":rows[0]["warm_setup_bytes"],
                    "max_owner_peak_rss":max(max(x["rss_owner1"],x["rss_owner2"]) for x in rows),
                    "false_safe_fixture":sum(int(x["safe"]==1 and x["d"]>64) for x in rows),
                    "unknown":sum(int(x["safe"]==0) for x in rows),
                })
                assert len({(x["bytes"],x["warm_setup_bytes"]) for x in rows})==1,(
                    "inconsistent mode bytes",l,mode,w)
            res.append({"lane":l,"N":conditions[0],"d":conditions[1],
                "S":conditions[2],"Mbps_per_sender":conditions[3],
                "injected_response_delay_ms":conditions[4],
                "mode":mode,"workers":by_worker})
    return {
        "format":"deltameter.b1b1a.two-os-sender-processes.v1",
        "source_sha":head,"cutoff_sha256":CUTS,
        "physical_tcp":True,"distinct_sender_os_processes":True,
        "application_pacing_per_owner_not_shared_network":True,
        "injected_delay_is_not_true_network_rtt":True,
        "receiver_warm_cache_provisioned_over_physical_tcp":True,
        "ack_retry_tested":False,"malicious_peer_authentication_proven":False,
        "records":300,"workers":5,"observed":res,
        "verdict":"B1B1A_PHYSICAL_TWO_PROCESS_COST_FOUNDATION_PASS_NO_PRODUCT_GO",
    }

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    report=summary(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(report,indent=2,sort_keys=True)+"\n")
    print(f"B1B1A_AGGREGATE_PASS records={report['records']} verdict={report['verdict']}")
    for x in report["observed"]:
        if x["lane"] in (0,1,2) and x["mode"] in ("guard","warm_exact","resolved"):
            print(f"B1B1A_PHYSICAL lane={x['lane']} mode={x['mode']} bytes={x['workers'][0]['bytes']} "
                  f"bootstrap={x['workers'][0]['bootstrap_bytes']} "
                  f"worker_p95_ns={[k['latency_nearest_rank_p95_ns'] for k in x['workers']]}")
if __name__=="__main__":main()
