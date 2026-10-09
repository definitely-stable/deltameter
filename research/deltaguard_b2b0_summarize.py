#!/usr/bin/env python3
"""Fail closed on every retained-state B0 physical and modeled-wire record.

No model outputs here may be called physical TCP or WAN measurements.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
import statistics
from pathlib import Path

NS = (256,4096,65536)
DS = (0,16,48,57,64,65)
SESSIONS = (1,10,100)
PROFILES = ("b11","b12","unit1","j52")
FIELDS = {"worker","ni","di","rep","N","d","session","profile",
          "owner_bytes","struct_bytes","init_ns","inc_ns","query_ns",
          "direct_build_ns","direct_compare_ns","odd","cutoff","answer",
          "safe","direct_bytes","guard_bytes","resolved_bytes",
          "sources_a","sources_b","model_header","model_request"}
BYTES = {"b11":256,"b12":512,"unit1":512,"j52":26624}
CUTOFFS = {"b11":48,"b12":52,"unit1":13,"j52":-2}
BPS=(1,10,100)
RTTS=(0,10,50)


def read_table(path: Path)->dict:
    lines=path.read_text(encoding="ascii").splitlines()
    assert lines[0]=="# deltameter.guard-b2a-nearfull-cutoffs.v1"
    profiles={}
    for line in lines[1:]:
        if line.startswith("#"):continue
        a=line.split()
        assert len(a)==4
        t,b,m,c=map(int,a)
        assert b in range(6,14) and t in (32,64,128) and m==2**b-1
        assert (t,b) not in profiles
        profiles[t,b]=c
    assert len(profiles)==24
    assert profiles[64,11]==48 and profiles[64,12]==52
    return profiles


def parse(line:str)->dict|None:
    if not line.startswith("B0_SAMPLE "):return None
    parts=[v.split("=",1) for v in line[len("B0_SAMPLE "):].split()]
    rec=dict(parts)
    assert len(parts)==len(FIELDS)==len(rec) and set(rec)==FIELDS
    assert rec["profile"] in PROFILES and rec["answer"] in ("safe","unknown","na")
    for key in FIELDS-{"profile","answer"}:
        assert re.fullmatch(r"[0-9]+|-1|-2",rec[key]),(key,rec[key])
        rec[key]=int(rec[key])
    w,ni,di,rep,n,d,s,p=(rec[x] for x in
                          ("worker","ni","di","rep","N","d","session","profile"))
    assert 1<=w<=5 and ni in range(3) and di in range(6)
    assert rep in range(3) and s in SESSIONS
    assert n==NS[ni] and d==DS[di]
    assert rec["sources_a"]==n+s-1
    assert rec["sources_b"]==n+d%2+s-1
    assert rec["owner_bytes"]==BYTES[p]
    assert 0 < rec["struct_bytes"] <= 256
    for key in ("init_ns","inc_ns","query_ns","direct_build_ns",
                "direct_compare_ns"):
        assert rec[key]>=0
    assert rec["cutoff"]==CUTOFFS[p]
    assert rec["model_header"]==64 and rec["model_request"]==24
    assert rec["safe"] in (0,1)
    expected_direct=128+8*(rec["sources_a"]+rec["sources_b"])
    expected_guard=2*(64+BYTES[p])
    assert rec["direct_bytes"]==expected_direct
    assert rec["guard_bytes"]==expected_guard
    assert rec["resolved_bytes"]==expected_guard+(0 if rec["safe"] else 24+expected_direct)
    assert 0<=rec["odd"]<=min(d,2**11-1 if p=="b11" else
                              2**12-1 if p=="b12" else
                              4096) if p!="j52" else rec["odd"]==0
    assert rec["safe"]==int(p!="j52" and rec["odd"]<=rec["cutoff"])
    assert rec["answer"]==("na" if p=="j52" else "safe" if rec["safe"] else "unknown")
    assert not rec["safe"] or d<=64  # deterministic fixture sanity only
    if p!="j52" and d<=CUTOFFS[p]:
        assert rec["safe"]==1
    if not rec["safe"]:
        assert rec["resolved_bytes"]>rec["direct_bytes"]
    return rec


def self_test()->None:
    x=("B0_SAMPLE worker=1 ni=0 di=2 rep=0 N=256 d=48 session=10 "
       "profile=b11 owner_bytes=256 struct_bytes=64 init_ns=123 "
       "inc_ns=9 query_ns=8 direct_build_ns=6 direct_compare_ns=7 "
       "odd=45 cutoff=48 answer=safe safe=1 direct_bytes=4368 "
       "guard_bytes=640 resolved_bytes=640 sources_a=265 sources_b=265 "
       "model_header=64 model_request=24")
    # N256, session10 => |A|=|B|=265, 128+530*8=4368.
    assert parse(x)
    for bad in (x.replace("owner_bytes=256","owner_bytes=512"),
                x.replace("sources_a=265","sources_a=266"),
                x.replace("resolved_bytes=640","resolved_bytes=700"),
                x.replace("cutoff=48","cutoff=47"),
                x.replace("answer=safe","answer=unknown"),
                x.replace("direct_bytes=4368","direct_bytes=128"),
                x.replace("session=10","session=1"),
                x.replace("ni=0","ni=3")):
        try:parse(bad)
        except (AssertionError,KeyError,ValueError):
            continue
        raise AssertionError("mutated model record accepted")


def quantile_ns(values:list[int],p:int)->int:
    assert values
    seq=sorted(values)
    return seq[((len(seq)-1)*p+99)//100]


def aggregate(root:Path,cert:Path,head:str)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    read_table(cert)
    self_test()
    unique={}
    for w in range(1,6):
        assert (root/f"head-{w}.txt").read_text().strip()==head
        raw=(root/f"worker-{w}.txt").read_text().splitlines()
        assert raw.count(f"DELTAGUARD_B2B0_WORKER_PASS worker={w} records=648")==1
        count=0
        for line in raw:
            rec=parse(line)
            if rec is None:continue
            assert rec["worker"]==w
            key=(w,rec["ni"],rec["di"],rec["rep"],rec["session"],rec["profile"])
            assert key not in unique
            unique[key]=rec
            count+=1
        assert count==648,(w,count)
    assert len(unique)==3240
    for w in range(1,6):
        for ni in range(3):
            for di in range(6):
                for rep in range(3):
                    for s in SESSIONS:
                        base=None
                        for p in PROFILES:
                            row=unique[w,ni,di,rep,s,p]
                            same=(row["sources_a"],row["sources_b"],row["direct_bytes"])
                            if base is None:base=same
                            else:assert same==base
    summaries=[]
    for ni,n in enumerate(NS):
        for di,d in enumerate(DS):
            for s in SESSIONS:
                for p in PROFILES:
                    rs=[unique[w,ni,di,rep,s,p]
                        for w in range(1,6) for rep in range(3)]
                    safe=sum(x["safe"] for x in rs)
                    summaries.append({
                      "N":n,"d":d,"session":s,"profile":p,
                      "safe":safe,"samples":len(rs),
                      "guard_bytes":rs[0]["guard_bytes"],
                      "direct_bytes":rs[0]["direct_bytes"],
                      "resolved_bytes_p50":int(statistics.median(x["resolved_bytes"] for x in rs)),
                      "update_p50_ns":int(statistics.median(x["inc_ns"] for x in rs)),
                      "update_p95_ns":quantile_ns([x["inc_ns"] for x in rs],95),
                      "build_p50_ns":int(statistics.median(x["init_ns"] for x in rs)),
                      "query_p95_ns":quantile_ns([x["query_ns"] for x in rs],95),
                      "worst_worker_safe":min(sum(unique[w,ni,di,rep,s,p]["safe"]
                         for rep in range(3)) for w in range(1,6)),
                    })
    for n in NS:
        point=next(x for x in summaries if x["N"]==n and x["d"]==48
                   and x["session"]==10 and x["profile"]=="b11")
        assert point["safe"]==15 and point["worst_worker_safe"]==3
        assert point["guard_bytes"]*10 <= point["direct_bytes"]*9
    # Byte model only: one round, single serial bottleneck, *not TCP*.
    model=[]
    for n in NS:
        r=next(x for x in summaries if x["N"]==n and x["d"]==48
               and x["session"]==10 and x["profile"]=="b11")
        for rtt in RTTS:
            for mbps in BPS:
                model.append({
                  "N":n,"d":48,"session":10,"profile":"b11","RTT_ms":rtt,
                  "bandwidth_Mbps":mbps,
                  "direct_model_ms":rtt+r["direct_bytes"]*8/(mbps*1000),
                  "guard_model_ms":rtt+r["guard_bytes"]*8/(mbps*1000),
                  "transport":"ANALYTIC_SINGLE_SERIAL_LINK_ONLY_NOT_TCP",
                })
    return {
      "format":"deltameter.guard-b2b0-retained-system.v1",
      "source_sha":head,
      "cutoff_sha256":hashlib.sha256(cert.read_bytes()).hexdigest(),
      "rows":len(unique),"workers":5,"profiles":list(PROFILES),
      "originality":"NONE_ASSERTED",
      "fixture_false_safe_zero_is_not_proof":True,
      "physical_io_or_wan_measured":False,
      "scope":"RESEARCH_FOUNDATION_ONLY_NO_SYSTEM_PRODUCT_GO",
      "modeled_wire_header_bytes_per_peer":64,
      "modeled_fallback_request_bytes":24,
      "model_summaries":summaries,"model_latencies":model,
      "verdict":"B2B0_PHYSICAL_SOURCE_PASS_GUARD_SCOPED_MODEL_CANDIDATE",
    }


def main()->None:
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--cutoffs",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    report=aggregate(a.input,a.cutoffs,a.head)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(report,indent=2,sort_keys=True)+"\n")
    print(f"B2B0_AGGREGATE_PASS rows={report['rows']} verdict={report['verdict']}")
    for rec in report["model_summaries"]:
        if rec["N"] in (256,65536) and rec["d"] in (48,57,65) and rec["session"]==10:
            print(f"B2B0_COST N={rec['N']} d={rec['d']} session=10 "
                  f"profile={rec['profile']} safe={rec['safe']}/15 "
                  f"guard_bytes={rec['guard_bytes']} direct_bytes={rec['direct_bytes']} "
                  f"resolved_p50_bytes={rec['resolved_bytes_p50']} "
                  f"build_p50_ns={rec['build_p50_ns']} query_p95_ns={rec['query_p95_ns']}")


if __name__=="__main__":main()
