#!/usr/bin/env python3
"""B1-B1B1-B1B0: fail closed 3000 signed-source-head two-owner WAL records.

Public proof identity and hash receipts are NOT malicious-peer signatures.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path

CUT="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
FIELDS=set("worker lane rep gen N mode tcp retry wal_bytes written sync checkpoint crash exact".split())
def parse(line):
    if not line.startswith("B1B1B1B1B0_SAMPLE "):return None
    tokens=[v.split("=",1) for v in line[len("B1B1B1B1B0_SAMPLE "):].split()]
    rec=dict(tokens)
    assert len(tokens)==len(rec)==len(FIELDS) and set(rec)==FIELDS
    assert rec["mode"] in ("insert","delete")
    for k in FIELDS-{"mode"}:
        assert re.fullmatch(r"[0-9]+",rec[k]),(k,rec[k])
        rec[k]=int(rec[k])
    w,l,rep,g=(rec[k] for k in ("worker","lane","rep","gen"))
    assert w in range(1,6) and l in (0,1) and rep in range(3) and g in range(2,102)
    n=(256,65536)[l]
    assert rec["N"]==n
    assert rec["mode"]==("insert" if g%2==0 else "delete")
    assert rec["tcp"]==(340 if g==51 else 356)
    assert rec["retry"]==(356 if g==51 else 0)
    assert rec["crash"]==int(g==51)
    assert rec["wal_bytes"]==(0 if g in (51,101) else ((g-1)%50)*256)
    checkpoint=336+128+16*n if g in (51,101) else 0
    assert rec["checkpoint"]==checkpoint
    assert rec["written"]==304+checkpoint
    assert rec["sync"]==(7 if checkpoint else 3)
    assert rec["exact"]==1
    return rec
def verify(root,sha,cutfile):
    assert re.fullmatch(r"[0-9a-f]{40}",sha)
    assert hashlib.sha256(cutfile.read_bytes()).hexdigest()==CUT
    test=("B1B1B1B1B0_SAMPLE worker=1 lane=0 rep=0 gen=51 N=256 "
          "mode=delete tcp=340 retry=356 wal_bytes=0 written=4864 sync=7 "
          "checkpoint=4560 crash=1 exact=1")
    assert parse(test)
    for bad in (test.replace("written=4864","written=4865"),
                test.replace("checkpoint=4560","checkpoint=0"),
                test.replace("wal_bytes=0","wal_bytes=256"),
                test.replace("crash=1","crash=0"),
                test.replace("exact=1","exact=0")):
        try:parse(bad)
        except (AssertionError,ValueError,KeyError):continue
        raise AssertionError("invalid record accepted")
    records={}
    for w in range(1,6):
        assert (root/f"head-{w}.txt").read_text().strip()==sha
        assert (root/f"rust-{w}.txt").exists()
        lines=(root/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1B1B0_WORKER_PASS worker={w} records=600 actual_receiver_sigkill=6 negative_groups=6")==1
        count=0
        for line in lines:
            rec=parse(line)
            if rec is None:continue
            assert rec["worker"]==w
            key=(w,rec["lane"],rec["rep"],rec["gen"])
            assert key not in records
            records[key]=rec
            count+=1
        assert count==600
    assert len(records)==3000
    lanes=[]
    for lane,n in enumerate((256,65536)):
        rows=[v for v in records.values() if v["lane"]==lane]
        assert len(rows)==1500 and sum(v["crash"] for v in rows)==15
        assert sum(v["mode"]=="insert" for v in rows)==750
        assert sum(v["mode"]=="delete" for v in rows)==750
        logical=sum(v["written"] for v in rows)
        file_size=336+128+16*n
        expected=15*(100*304+2*file_size)
        assert logical==expected
        lanes.append({"N":n,"records":1500,
            "receiver_sigkill_after_first_ack":15,
            "receiver_logical_written_bytes":logical,
            "receiver_fsync_calls":sum(v["sync"] for v in rows),
            "checkpoint_bytes":sum(v["checkpoint"] for v in rows),
            "physical_tcp_first_attempt_bytes":sum(v["tcp"] for v in rows),
            "physical_tcp_identical_replay_bytes":sum(v["retry"] for v in rows),
            "max_receiver_wal_bytes":max(v["wal_bytes"] for v in rows),
            "per_100_gen_logical_written":100*304+2*file_size,
            "per_100_gen_initial_checkpoint_excluded":file_size,
            "legacy_per_100_gen_full_snapshot_write_bytes":
                 454400 if n==256 else 104902400,
        })
        assert lanes[-1]["max_receiver_wal_bytes"]==49*256
    return {"schema":"deltameter.b1b1b1-b1b0.atomic-receiver-wal.v1",
        "source_sha":sha,"certificate_sha256":CUT,"workers":5,"records":3000,
        "actual_receiver_os_sigkill":30,
        "preregistered_checkpoint_every":50,
        "txn_size_bytes":256,"commit_marker_size_bytes":48,
        "source_journals_private_and_fsynced":True,
        "receiver_2_owner_transaction_committed_before_ack":True,
        "power_loss_or_torn_fsync_tested":False,
        "malicious_sender_authenticated":False,
        "fair_durable_exact_v_guard_latency_tested":False,
        "system_product_go":False,
        "lanes":lanes,
        "verdict":"B1B1B1_B1B0_INCREMENTAL_RECEIVER_RESEARCH_ACCEPT_NO_PRODUCT_GO"}
def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    result=verify(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(result,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B1B1B0_AGGREGATE_PASS records={result['records']} verdict={result['verdict']}")
    for row in result["lanes"]:
        print(f"B1B1B1B1B0_COST N={row['N']} records={row['records']} bytes={row['receiver_logical_written_bytes']} maxwal={row['max_receiver_wal_bytes']} checkpoint={row['checkpoint_bytes']} tcp={row['physical_tcp_first_attempt_bytes']} replay={row['physical_tcp_identical_replay_bytes']}")
if __name__=="__main__":main()
