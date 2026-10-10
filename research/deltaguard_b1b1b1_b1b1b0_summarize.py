#!/usr/bin/env python3
"""Fail-closed 1200 physically paired hot guard/full/exact queries.

Guard response S is NOT an exact symmetric difference or proof that d<=48.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import re
from pathlib import Path

SHA="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
S_FIELDS=set("worker ni di rep q N d mbps delay guard_bytes full_bytes hot_exact_bytes guard_ns full_ns hot_exact_ns odd under exact".split())
F_FIELDS=set("worker ni di rep N d cold_exact_bytes shared_source_event_bytes exact_receiver_write exact_receiver_sync guard_total full_total under_cutoff guard_p95_ns full_p95_ns hot_exact_p95_ns owner1_build_ns owner2_build_ns owner1_rss owner2_rss owner1_ticks owner2_ticks quit_bytes receiver_rss exact".split())
def read_record(line,prefix,keys):
    if not line.startswith(prefix):return None
    entries=[pair.split("=",1) for pair in line[len(prefix):].split()]
    r=dict(entries)
    assert len(entries)==len(keys)==len(r) and set(r)==keys
    for k in keys:
        assert re.fullmatch(r"[0-9]+",r[k]),(k,r[k])
        r[k]=int(r[k])
    return r
def sample(line):
    x=read_record(line,"B1B1B1_B1B1B0_SAMPLE ",S_FIELDS)
    if x is None:return None
    w,ni,di,rep,q=(x[k] for k in ("worker","ni","di","rep","q"))
    assert 1<=w<=5 and ni in (0,1) and di in (0,1) and rep in range(3) and 1<=q<=20
    n=(256,65536)[ni];d=(48,57)[di]
    assert x["N"]==n and x["d"]==d
    assert x["mbps"]==100 and x["delay"]==0
    assert x["guard_bytes"]==736 and x["full_bytes"]==240+16*n+8*(d%2)
    assert x["hot_exact_bytes"]==0 and x["odd"]<=d
    assert x["under"]==int(x["odd"]<=48)
    assert x["exact"]==1
    return x
def finish(line):
    x=read_record(line,"B1B1B1_B1B1B0_SESSION ",F_FIELDS)
    if x is None:return None
    assert 1<=x["worker"]<=5 and x["ni"] in (0,1) and x["di"] in (0,1) and x["rep"] in range(3)
    n=(256,65536)[x["ni"]];d=(48,57)[x["di"]]
    assert x["N"]==n and x["d"]==d
    assert x["cold_exact_bytes"]==226+16*n+8*(d%2)
    assert x["shared_source_event_bytes"]==356
    assert x["exact_receiver_write"]==1256+32*n+16*(d%2)
    assert x["exact_receiver_sync"]==11
    assert x["guard_total"]==14720
    assert x["full_total"]==20*(240+16*n+8*(d%2))
    assert 0<=x["under_cutoff"]<=20
    assert x["owner1_rss"]>0 and x["owner2_rss"]>0 and x["receiver_rss"]>0
    assert x["quit_bytes"]==128 and x["exact"]==1
    return x
def p95(values):
    assert len(values)==20
    return sorted(values)[math.ceil(.95*len(values))-1]
def self_test():
    line=("B1B1B1_B1B1B0_SAMPLE worker=1 ni=0 di=0 rep=0 q=1 N=256 "
          "d=48 mbps=100 delay=0 guard_bytes=736 full_bytes=4336 "
          "hot_exact_bytes=0 guard_ns=123 full_ns=456 hot_exact_ns=1 odd=48 under=1 exact=1")
    assert sample(line)
    for wrong in ("guard_bytes=735","under=0","d=57","exact=0",
                  "hot_exact_bytes=16","full_bytes=4335"):
        field=wrong.split("=")[0]
        before=re.search(r"\b"+field+r"=[0-9]+",line).group()
        try:sample(line.replace(before,wrong))
        except AssertionError:continue
        raise AssertionError("malformed evidence accepted")
def aggregate(folder:Path,head:str,certificate:Path)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    assert hashlib.sha256(certificate.read_bytes()).hexdigest()==SHA
    self_test()
    samples={};finals={}
    for worker in range(1,6):
        assert (folder/f"head-{worker}.txt").read_text().strip()==head
        assert (folder/f"rust-{worker}.txt").exists()
        lines=(folder/f"worker-{worker}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1_B1B1B0_WORKER_PASS worker={worker} sessions=12 queries=240")==1
        got_s=got_f=0
        for line in lines:
            s=sample(line)
            if s is not None:
                assert s["worker"]==worker
                key=(worker,s["ni"],s["di"],s["rep"],s["q"])
                assert key not in samples
                samples[key]=s;got_s+=1
            f=finish(line)
            if f is not None:
                assert f["worker"]==worker
                key=(worker,f["ni"],f["di"],f["rep"])
                assert key not in finals
                finals[key]=f;got_f+=1
        assert (got_s,got_f)==(240,12),(worker,got_s,got_f)
    assert len(samples)==1200 and len(finals)==60
    scenarios=[]
    for ni,n in enumerate((256,65536)):
        for di,d in enumerate((48,57)):
            rows=[v for v in finals.values() if v["ni"]==ni and v["di"]==di]
            assert len(rows)==15
            observed=[]
            for s in rows:
                key=(s["worker"],ni,di,s["rep"])
                qs=[samples[key+(q,)] for q in range(1,21)]
                assert s["guard_p95_ns"]==p95([v["guard_ns"] for v in qs])
                assert s["full_p95_ns"]==p95([v["full_ns"] for v in qs])
                assert s["hot_exact_p95_ns"]==p95([v["hot_exact_ns"] for v in qs])
                assert s["under_cutoff"]==sum(v["under"] for v in qs)
                observed.extend(qs)
            assert len(observed)==300
            scenarios.append({
                "N":n,"d":d,"session_count":15,"paired_queries":300,
                "source_build_wall_ns_total":sum(v["owner1_build_ns"]+v["owner2_build_ns"] for v in rows),
                "owner_proc_cpu_ticks_total":sum(v["owner1_ticks"]+v["owner2_ticks"] for v in rows),
                "source_vm_hwm_max_bytes":max(max(v["owner1_rss"],v["owner2_rss"]) for v in rows),
                "receiver_vm_hwm_max_bytes":max(v["receiver_rss"] for v in rows),
                "guard_odd_under_cutoff_count":sum(v["under"] for v in observed),
                "guard_query_wire_bytes":sum(v["guard_bytes"] for v in observed),
                "direct_full_query_wire_bytes":sum(v["full_bytes"] for v in observed),
                "maintained_exact_query_wire_bytes":0,
                "maintained_exact_cold_bootstrap_wire_bytes":sum(v["cold_exact_bytes"] for v in rows),
                "shared_source_update_wire_bytes":sum(v["shared_source_event_bytes"] for v in rows),
                "maintained_exact_receiver_logical_write_bytes":sum(v["exact_receiver_write"] for v in rows),
                "maintained_exact_receiver_sync_calls":sum(v["exact_receiver_sync"] for v in rows),
                "guard_p95_ns_by_worker_fixture":[{"worker":v["worker"],"rep":v["rep"],"p95":v["guard_p95_ns"]} for v in rows],
                "hot_exact_p95_ns_by_worker_fixture":[{"worker":v["worker"],"rep":v["rep"],"p95":v["hot_exact_p95_ns"]} for v in rows],
                "all_under_cutoff":all(v["under"]==1 for v in observed),
                "guard_only_supplies_exact_delta":False,
            })
    return {
        "schema":"deltameter.b1b1b1-b1b1b0.hot-contract.v1",
        "source_sha":head,"certificate_sha256":SHA,
        "workers":5,"sessions":60,"paired_queries":1200,
        "private_source_fsynced_wal":True,
        "receiver_atomic_exact_wal_fsynced":True,
        "source_processes_live_for_all_20_queries":True,
        "rate_mbps":100,"application_delay_ms":0,
        "real_wan_measured":False,
        "guard_odd_le_threshold_is_deterministic_certificate":False,
        "guard_only_exact_reconciliation_claim":False,
        "equivalent_service_contract":False,
        "system_product_go":False,
        "scenarios":scenarios,
        "verdict":"B1B1B1_B1B1B0_HOT_CONTRACT_COMPARISON_RESEARCH_ACCEPT_NO_PRODUCT_GO",
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
    print(f"B1B1B1_B1B1B0_AGGREGATE_PASS sessions={result['sessions']} queries={result['paired_queries']} verdict={result['verdict']}")
    for x in result["scenarios"]:
        print(f"B1B1B1_B1B1B0_COST N={x['N']} d={x['d']} sessions={x['session_count']} queries={x['paired_queries']} guard_bytes={x['guard_query_wire_bytes']} exact_bootstrap={x['maintained_exact_cold_bootstrap_wire_bytes']} exact_query_bytes=0 exact_receiver_write={x['maintained_exact_receiver_logical_write_bytes']} under={x['guard_odd_under_cutoff_count']}")
if __name__=="__main__":main()
