#!/usr/bin/env python3
"""Fail-closed exact-output physical guard→FULL dominance certificate.

No magic FREE fallback and no inferred physical measurements for missing
N65536/1Mbps/Q10-Q100. Original 24-profile cutoff remains unchanged.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import re
from pathlib import Path

B2A="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
LANES=((256,48,100,0,100),(256,57,10,10,100),
       (256,48,1,50,100),(65536,48,100,0,10),
       (65536,57,100,10,10),(65536,57,1,50,1))
S_FIELDS=set("worker lane rep q N d mbps delay direct_bytes resolved_bytes direct_ns resolved_ns exact_local_ns S under exact".split())
P_FIELDS=set("worker lane rep q N d mbps delay direct_bytes resolved_bytes exact_hot_bytes cold_exact_bytes source_event_bytes receiver_write receiver_sync exact".split())
F_FIELDS=set("worker lane rep N d mbps delay Q direct_bytes resolved_bytes under direct_p95_ns resolved_p95_ns local_p95_ns source_build_ns source_ticks source_rss receiver_rss cold_exact_bytes source_event_bytes receiver_write receiver_sync quit_bytes exact".split())
def fields(line,prefix,required):
    if not line.startswith(prefix):return None
    pairs=[token.split("=",1) for token in line[len(prefix):].split()]
    d=dict(pairs)
    assert len(pairs)==len(d)==len(required) and set(d)==required
    for k,v in d.items():
        assert re.fullmatch("[0-9]+",v),(k,v)
        d[k]=int(v)
    return d
def check_common(v):
    w,i,rep=(v[k] for k in ("worker","lane","rep"))
    assert w in range(1,6) and i in range(6) and rep in range(3)
    n,d,mbps,delay,qmax=LANES[i]
    assert all(v[k]==value for k,value in zip(
        ("N","d","mbps","delay"),(n,d,mbps,delay),strict=True))
    assert v["exact"]==1
    return qmax
def sample(line):
    d=fields(line,"B1B1B1_B1B1B1A_SAMPLE ",S_FIELDS)
    if d is None:return None
    qmax=check_common(d)
    assert 1<=d["q"]<=qmax
    expected=240+16*d["N"]+8*(d["d"]%2)
    assert d["direct_bytes"]==expected
    assert d["resolved_bytes"]==expected+736
    assert d["S"]<=d["d"]
    assert d["under"]==int(d["S"]<=48)
    assert d["under"]==int(d["d"]==48), "frozen fixture guard outcome changed"
    return d
def prefix(line):
    d=fields(line,"B1B1B1_B1B1B1A_PREFIX ",P_FIELDS)
    if d is None:return None
    qmax=check_common(d)
    assert d["q"] in (1,10,100) and d["q"]<=qmax
    per=240+16*d["N"]+8*(d["d"]%2)
    assert d["direct_bytes"]==d["q"]*per
    assert d["resolved_bytes"]==d["q"]*(per+736)
    assert d["exact_hot_bytes"]==0
    assert d["cold_exact_bytes"]==226+16*d["N"]+8*(d["d"]%2)
    assert d["source_event_bytes"]==356
    assert d["receiver_write"]==1256+32*d["N"]+16*(d["d"]%2)
    assert d["receiver_sync"]==11
    return d
def p95(v):
    assert v
    return sorted(v)[math.ceil(len(v)*.95)-1]
def finish(line):
    d=fields(line,"B1B1B1_B1B1B1A_SESSION ",F_FIELDS)
    if d is None:return None
    qmax=check_common(d)
    assert d["Q"]==qmax
    assert d["direct_bytes"]==qmax*(240+16*d["N"]+8*(d["d"]%2))
    assert d["resolved_bytes"]==d["direct_bytes"]+736*qmax
    assert d["under"]==(qmax if d["d"]==48 else 0)
    assert d["cold_exact_bytes"]==226+16*d["N"]+8*(d["d"]%2)
    assert d["source_event_bytes"]==356
    assert d["receiver_write"]==1256+32*d["N"]+16*(d["d"]%2)
    assert d["receiver_sync"]==11
    assert d["quit_bytes"]==128 and d["source_rss"]>0 and d["receiver_rss"]>0
    return d
def selftest():
    s=("B1B1B1_B1B1B1A_SAMPLE worker=1 lane=0 rep=0 q=1 N=256 d=48 "
       "mbps=100 delay=0 direct_bytes=4336 resolved_bytes=5072 direct_ns=1 "
       "resolved_ns=2 exact_local_ns=1 S=48 under=1 exact=1")
    assert sample(s)
    for change in ("resolved_bytes=5071","direct_bytes=4335","under=0","exact=0",
                   "q=101","mbps=1"):
        k=change.split("=")[0]
        old=re.search(r"\b"+k+r"=\d+",s).group()
        try:sample(s.replace(old,change))
        except AssertionError:continue
        raise AssertionError("malformed evidence was accepted: "+change)
def analyze(root:Path,head:str,proof:Path):
    assert re.fullmatch("[0-9a-f]{40}",head)
    assert hashlib.sha256(proof.read_bytes()).hexdigest()==B2A
    selftest()
    samples={};prefixes={};finishes={}
    for w in range(1,6):
        assert (root/f"head-{w}.txt").read_text().strip()==head
        assert (root/f"rust-{w}.txt").exists()
        lines=(root/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1_B1B1B1A_WORKER_PASS worker={w} sessions=18 paired_queries=963 full_reconstruction=1")==1
        nc=np=nf=0
        for line in lines:
            s=sample(line)
            if s is not None:
                assert s["worker"]==w
                key=(w,s["lane"],s["rep"],s["q"])
                assert key not in samples
                samples[key]=s;nc+=1
            p=prefix(line)
            if p is not None:
                assert p["worker"]==w
                key=(w,p["lane"],p["rep"],p["q"])
                assert key not in prefixes
                prefixes[key]=p;np+=1
            f=finish(line)
            if f is not None:
                assert f["worker"]==w
                key=(w,f["lane"],f["rep"])
                assert key not in finishes
                finishes[key]=f;nf+=1
        assert (nc,np,nf)==(963,42,18),(w,nc,np,nf)
    assert (len(samples),len(prefixes),len(finishes))==(4815,210,90)
    result=[]
    for lane,(n,d,mbps,delay,qmax) in enumerate(LANES):
        relevant=[x for x in finishes.values() if x["lane"]==lane]
        assert len(relevant)==15
        for x in relevant:
            key=(x["worker"],lane,x["rep"])
            qs=[samples[key+(q,)] for q in range(1,qmax+1)]
            assert len(qs)==qmax
            assert x["direct_p95_ns"]==p95([v["direct_ns"] for v in qs])
            assert x["resolved_p95_ns"]==p95([v["resolved_ns"] for v in qs])
            assert x["local_p95_ns"]==p95([v["exact_local_ns"] for v in qs])
            for cut in (1,10,100):
                if cut>qmax:continue
                p=prefixes[key+(cut,)]
                assert p["direct_bytes"]==sum(v["direct_bytes"] for v in qs[:cut])
                assert p["resolved_bytes"]==sum(v["resolved_bytes"] for v in qs[:cut])
        result.append({
            "lane":lane,"N":n,"d":d,"mbps":mbps,"app_delay_ms":delay,"Q":qmax,
            "sessions":15,"paired_queries":15*qmax,
            "direct_exact_tcp_bytes":sum(x["direct_bytes"] for x in relevant),
            "guard_then_exact_tcp_bytes":sum(x["resolved_bytes"] for x in relevant),
            "additional_guard_tcp_bytes":15*qmax*736,
            "maintained_exact_cold_network_bytes":sum(x["cold_exact_bytes"] for x in relevant),
            "maintained_exact_event_tcp_bytes":15*356,
            "maintained_exact_receiver_logical_written_bytes":sum(x["receiver_write"] for x in relevant),
            "maintained_exact_receiver_sync_calls":15*11,
            "source_cpu_ticks_sum":sum(x["source_ticks"] for x in relevant),
            "source_build_wall_ns_sum":sum(x["source_build_ns"] for x in relevant),
            "max_source_vm_hwm_bytes":max(x["source_rss"] for x in relevant),
            "max_receiver_vm_hwm_bytes":max(x["receiver_rss"] for x in relevant),
            "guard_under_cutoff_samples":sum(x["under"] for x in relevant),
            "empirical_latency_rank_n":qmax,
            "direct_exact_p95_ns_by_worker_fixture":[{"worker":x["worker"],"rep":x["rep"],"value":x["direct_p95_ns"]} for x in relevant],
            "resolved_p95_ns_by_worker_fixture":[{"worker":x["worker"],"rep":x["rep"],"value":x["resolved_p95_ns"]} for x in relevant],
        })
    return {
        "schema":"deltameter.b1b1b1-b1b1b1a.exact-output-dominance.v1",
        "source_sha":head,"cutoffs_sha256":B2A,
        "github_hosted_workers":5,"sessions":90,"physical_exact_result_pairs":4815,
        "prefixes":210,"no_unmeasured_large_n_low_rate_q100":True,
        "guard_under_cutoff_certifies_exact_output":False,
        "product_go":False,
        "verdict":"B1B1B1_B1B1B1A_RESOLVED_EXACT_CAPABILITY_RESEARCH_ACCEPT_STOP_STATELESS_GUARD_FOR_EXACT_V1",
        "lanes":result}
def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    d=analyze(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(d,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B1_B1B1B1A_AGGREGATE_PASS sessions={d['sessions']} pairs={d['physical_exact_result_pairs']} prefixes={d['prefixes']} verdict={d['verdict']}")
    for l in d["lanes"]:
        print(f"B1B1B1_B1B1B1A_COST lane={l['lane']} N={l['N']} d={l['d']} mbps={l['mbps']} delay={l['app_delay_ms']} Q={l['Q']} direct={l['direct_exact_tcp_bytes']} resolved={l['guard_then_exact_tcp_bytes']} overhead={l['additional_guard_tcp_bytes']} cold={l['maintained_exact_cold_network_bytes']} receipts={l['maintained_exact_receiver_logical_written_bytes']}")
if __name__=="__main__": main()
