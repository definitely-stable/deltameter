#!/usr/bin/env python3
"""Strict 3000 source-anchored two-owner ABA/first-ACK crash records."""
import argparse
import hashlib
import json
import re
from pathlib import Path
HASH="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
FIELDS=set("worker lane rep generation N op source_wal_bytes wire_bytes retry_bytes receiver_crash exact".split())
def parse(line):
    if not line.startswith("B1B1B1A_SAMPLE "):return None
    fields=[part.split("=",1) for part in line[len("B1B1B1A_SAMPLE "):].split()]
    row=dict(fields)
    assert len(fields)==len(row)==len(FIELDS) and set(row)==FIELDS
    assert row["op"] in ("insert","delete")
    for key in FIELDS-{"op"}:
        assert re.fullmatch(r"[0-9]+",row[key])
        row[key]=int(row[key])
    w,l,rep,g=(row[k] for k in ("worker","lane","rep","generation"))
    assert 1<=w<=5 and l in (0,1) and rep in (0,1,2) and 2<=g<=101
    assert row["N"]==(256 if l==0 else 65536)
    assert row["op"]==("insert" if g%2==0 else "delete")
    assert row["source_wal_bytes"]==2*(g-1)*88
    assert row["wire_bytes"]==(340 if g==51 else 356)
    assert row["retry_bytes"]==(356 if g==51 else 0)
    assert row["receiver_crash"]==int(g==51)
    assert row["exact"]==1
    return row
def run(root,sha,certificate):
    assert re.fullmatch(r"[0-9a-f]{40}",sha)
    assert hashlib.sha256(certificate.read_bytes()).hexdigest()==HASH
    sample=("B1B1B1A_SAMPLE worker=1 lane=0 rep=0 generation=51 N=256 op=delete "
            "source_wal_bytes=8800 wire_bytes=340 retry_bytes=356 receiver_crash=1 exact=1")
    assert parse(sample)
    for bad in (sample.replace("wire_bytes=340","wire_bytes=999"),
                sample.replace("exact=1","exact=0"),
                sample.replace("op=delete","op=insert"),
                sample.replace("receiver_crash=1","receiver_crash=0")):
        try:parse(bad)
        except AssertionError:continue
        raise AssertionError("forged evidence accepted")
    results={}
    for worker in range(1,6):
        assert (root/f"head-{worker}.txt").read_text().strip()==sha
        assert (root/f"rust-{worker}.txt").exists()
        lines=(root/f"worker-{worker}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1A_WORKER_PASS worker={worker} records=600 actual_receiver_sigkill=6 negative_groups=6")==1
        count=0
        for line in lines:
            row=parse(line)
            if row is None:continue
            assert row["worker"]==worker
            key=(worker,row["lane"],row["rep"],row["generation"])
            assert key not in results
            results[key]=row
            count+=1
        assert count==600
    assert len(results)==3000
    lanes=[]
    for lane,n in enumerate((256,65536)):
        items=[v for v in results.values() if v["lane"]==lane]
        assert len(items)==1500
        kills=sum(v["receiver_crash"] for v in items)
        assert kills==15
        assert sum(v["op"]=="insert" for v in items)==750
        assert sum(v["op"]=="delete" for v in items)==750
        lanes.append({"N":n,"generations":1500,"receiver_sigkills":kills,
            "physical_event_bytes":sum(v["wire_bytes"] for v in items),
            "physical_replay_bytes":sum(v["retry_bytes"] for v in items),
            "source_wal_bytes_per_owner_after_100_events":8800})
    return {"schema":"deltameter.b1b1b1-b1a.aba-receipt.v1","source_sha":sha,
      "certificate_sha256":HASH,"workers":5,"records":3000,
      "actual_receiver_sigkills_after_first_ack":30,
      "public_api_changes":False,"malicious_sender_authenticated":False,
      "power_cut_tested":False,"product_go":False,"lanes":lanes,
      "verdict":"B1B1B1_B1A_ABA_RECEIPT_CHAIN_RESEARCH_ACCEPT_NO_PRODUCT_GO"}
def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    x=p.parse_args()
    result=run(x.input,x.head,x.certificate)
    x.out.parent.mkdir(parents=True,exist_ok=True)
    x.out.write_text(json.dumps(result,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B1A_AGGREGATE_PASS records={result['records']} verdict={result['verdict']}")
    for lane in result["lanes"]:
        print(f"B1B1B1A_COST N={lane['N']} generations={lane['generations']} receiver_sigkills={lane['receiver_sigkills']} event_bytes={lane['physical_event_bytes']} replay_bytes={lane['physical_replay_bytes']}")
if __name__=="__main__":main()
