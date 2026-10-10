#!/usr/bin/env python3
"""Independent frozen B1-B1B1-B1B1A 150 OS SIGKILL WAL cuts and 600 read pairs."""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import re
from pathlib import Path

PROOF="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
CASES=("partial_wal","full_wal_fsync","commit_synced",
       "commit_conflict","checkpoint_synced")
NS=(256,65536)
CKPT={256:4560,65536:1049040}
CA_FIELDS=set("worker lane rep case_idx case N gen before after repair wire wal exact".split())
RD_FIELDS=set("worker lane rep sample N cold_ns hot_ns logical_read exact".split())
def record(line,prefix,fields):
    if not line.startswith(prefix):return None
    items=[v.split("=",1) for v in line[len(prefix):].split()]
    row=dict(items)
    assert len(items)==len(row)==len(fields) and set(row)==fields
    for k in fields-{"case"}:
        assert re.fullmatch(r"[0-9]+",row[k]),(k,row[k])
        row[k]=int(row[k])
    return row
def case_parse(line):
    row=record(line,"B1B1B1_B1B1A_CASE ",CA_FIELDS)
    if row is None:return None
    assert row["worker"] in range(1,6) and row["lane"] in (0,1)
    assert row["rep"] in range(3) and row["case_idx"] in range(5)
    i=row["case_idx"]
    assert row["case"]==CASES[i]
    assert row["N"]==NS[row["lane"]]
    assert row["gen"]==(51 if i==4 else 2)
    assert row["before"]==(1 if i<=1 else (51 if i==4 else 2))
    assert row["after"]==row["gen"]
    assert row["repair"]==int(i in (0,1,4))
    assert row["wire"]==356
    assert row["wal"]==(0 if i==4 else 256)
    assert row["exact"]==1
    return row
def read_parse(line):
    row=record(line,"B1B1B1_B1B1A_READ ",RD_FIELDS)
    if row is None:return None
    assert row["worker"] in range(1,6) and row["lane"] in (0,1)
    assert row["rep"] in range(3) and row["sample"] in range(20)
    assert row["N"]==NS[row["lane"]]
    assert row["logical_read"]==CKPT[row["N"]]+48
    assert row["cold_ns"]>=0 and row["hot_ns"]>=0 and row["exact"]==1
    return row
def self_test():
    example=("B1B1B1_B1B1A_CASE worker=1 lane=0 rep=0 case_idx=4 "
             "case=checkpoint_synced N=256 gen=51 before=51 after=51 repair=1 "
             "wire=356 wal=0 exact=1")
    assert case_parse(example)
    for changed in ("wire=355","repair=0","wal=256","before=50","exact=0"):
        initial=changed.split("=")[0]
        v=re.search(r"\b"+initial+r"=[^ ]+",example).group()
        try:case_parse(example.replace(v,changed))
        except AssertionError:continue
        raise AssertionError("mutated/partial physical crash evidence accepted")
    rd=("B1B1B1_B1B1A_READ worker=1 lane=1 rep=2 sample=19 "
        "N=65536 cold_ns=1 hot_ns=0 logical_read=1049088 exact=1")
    assert read_parse(rd)
    for changed in ("logical_read=1","sample=20","exact=0"):
        first=changed.split("=")[0]
        v=re.search(r"\b"+first+r"=[^ ]+",rd).group()
        try:read_parse(rd.replace(v,changed))
        except AssertionError:continue
        raise AssertionError("mutated read sample accepted")
def p95(ns):
    vals=sorted(ns)
    assert len(vals)==20
    return vals[math.ceil(.95*len(vals))-1]
def aggregate(folder:Path,head:str,cutoff:Path):
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    assert hashlib.sha256(cutoff.read_bytes()).hexdigest()==PROOF
    self_test()
    cases={};reads={}
    for w in range(1,6):
        assert (folder/f"head-{w}.txt").read_text().strip()==head
        assert (folder/f"rust-{w}.txt").exists()
        lines=(folder/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1_B1B1A_WORKER_PASS worker={w} cases=30 repairs=18 reads=120 real_sigkill=30")==1
        nc=nr=0
        for line in lines:
            c=case_parse(line)
            if c is not None:
                assert c["worker"]==w
                key=(w,c["lane"],c["rep"],c["case_idx"])
                assert key not in cases
                cases[key]=c;nc+=1
            read=read_parse(line)
            if read is not None:
                assert read["worker"]==w
                key=(w,read["lane"],read["rep"],read["sample"])
                assert key not in reads
                reads[key]=read;nr+=1
        assert nc==30 and nr==120,(w,nc,nr)
    assert len(cases)==150 and len(reads)==600
    lanes=[]
    for lane,n in enumerate(NS):
        cc=[x for x in cases.values() if x["lane"]==lane]
        rr=[x for x in reads.values() if x["lane"]==lane]
        assert len(cc)==75 and len(rr)==300
        assert sum(v["repair"] for v in cc)==45
        assert sum(v["case_idx"]==4 for v in cc)==15
        audits=[]
        for w in range(1,6):
            for rep in range(3):
                v=[reads[(w,lane,rep,i)] for i in range(20)]
                audits.append({
                    "worker":w,"rep":rep,
                    "cold_reload_p95_ns":p95([q["cold_ns"] for q in v]),
                    "hot_materialized_access_p95_ns":p95([q["hot_ns"] for q in v]),
                    "cold_logical_file_bytes_per_load":CKPT[n]+48,
                })
        lanes.append({
            "N":n,"crash_cases":len(cc),"true_os_sigkill":len(cc),
            "repaired_uncommitted_or_stale_wal":45,
            "checkpoint_after_rename_cutpoints":15,
            "read_audit_pairs":len(rr),
            "physical_event_replay_tcp_bytes":len(cc)*356,
            "physical_wal_bytes_removed_by_repair_estimated":
                15*(80+256+50*256),
            "read_logical_bytes_per_reload":CKPT[n]+48,
            "read_p95_by_worker_fixture":audits,
        })
    return {
        "schema":"deltameter.b1b1b1-b1b1a.crashcuts-read.v1",
        "source_sha":head,"certificate_sha256":PROOF,
        "workers":5,"physical_sigkill_cases":150,
        "canonical_wal_repairs":90,"paired_read_observations":600,
        "power_cut_or_torn_fsync_proven":False,
        "malicious_peer_authentication":False,
        "fair_durable_exact_vs_guard_measured":False,
        "product_go":False,
        "lanes":lanes,
        "verdict":"B1B1B1_B1B1A_CRASHCUT_TAIL_REPAIR_RESEARCH_ACCEPT_NO_PRODUCT_GO",
    }
def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    x=aggregate(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(x,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B1_B1B1A_AGGREGATE_PASS cuts={x['physical_sigkill_cases']} repairs={x['canonical_wal_repairs']} reads={x['paired_read_observations']} verdict={x['verdict']}")
    for z in x["lanes"]:
        print(f"B1B1B1_B1B1A_COST N={z['N']} SIGKILL={z['true_os_sigkill']} repairs={z['repaired_uncommitted_or_stale_wal']} reads={z['read_audit_pairs']} bytes_replayed={z['physical_event_replay_tcp_bytes']} logical_bytes_per_reload={z['read_logical_bytes_per_reload']}")
if __name__=="__main__":main()
