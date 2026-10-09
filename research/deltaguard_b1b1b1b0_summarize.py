#!/usr/bin/env python3
"""Fail-closed source-head/fixture bound B1-B1B1-B0 Linux WAL/restart evidence.

ACK and source generation are file-reopened. SIGKILL != power loss.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path

CASES=("before_wal","after_write_before_sync","after_wal_sync_before_commit",
       "after_commit_before_send","after_send_before_ack",
       "after_receiver_sync_before_ack","after_ack_before_owner_sync")
NS=(256,65536)
CUTS="c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
KEYS=set("worker lane rep case_idx case N initial_bytes stage_bytes recovery_bytes source_gen ack_gen receiver_gen replayed dedup exact".split())

def parse(line:str)->dict|None:
    if not line.startswith("B1B1B1B0_SAMPLE "):return None
    pairs=[p.split("=",1) for p in line[len("B1B1B1B0_SAMPLE "):].split()]
    rec=dict(pairs)
    assert len(pairs)==len(KEYS)==len(rec) and set(rec)==KEYS
    assert rec["case"] in CASES
    for k in KEYS-{"case"}:
        assert re.fullmatch(r"[0-9]+",rec[k]),(k,rec[k])
        rec[k]=int(rec[k])
    w,l,rep,case=(rec[k] for k in ("worker","lane","rep","case_idx"))
    assert w in range(1,6) and l in range(2) and rep in range(3) and case in range(7)
    assert rec["case"]==CASES[case]
    n=NS[l]
    assert rec["N"]==n
    assert rec["initial_bytes"]==242+16*n
    assert rec["stage_bytes"]==([4,4,4,4,150,150,184][case])
    assert rec["recovery_bytes"]==((290+16*n) if case<3 else 308)
    generation=1 if case<3 else 2
    assert rec["source_gen"]==rec["ack_gen"]==rec["receiver_gen"]==generation
    assert rec["replayed"]==int(case in (3,4))
    assert rec["dedup"]==int(case in (5,6))
    assert rec["exact"]==1
    return rec

def self_test():
    base=("B1B1B1B0_SAMPLE worker=1 lane=0 rep=0 case_idx=4 "
          "case=after_send_before_ack N=256 initial_bytes=4338 stage_bytes=150 "
          "recovery_bytes=308 source_gen=2 ack_gen=2 receiver_gen=2 "
          "replayed=1 dedup=0 exact=1")
    assert parse(base)
    for bad in (base.replace("recovery_bytes=308","recovery_bytes=309"),
                base.replace("replayed=1","replayed=0"),
                base.replace("ack_gen=2","ack_gen=1"),
                base.replace("case_idx=4","case_idx=5"),
                base.replace("exact=1","exact=0")):
        try: parse(bad)
        except (AssertionError,ValueError,KeyError):continue
        raise AssertionError("accepted mutated evidence")
def aggregate(folder:Path,head:str,cutoff:Path)->dict:
    assert re.fullmatch(r"[0-9a-f]{40}",head)
    assert hashlib.sha256(cutoff.read_bytes()).hexdigest()==CUTS
    self_test()
    records={}
    for w in range(1,6):
        assert (folder/f"head-{w}.txt").read_text().strip()==head
        assert (folder/f"rust-{w}.txt").exists()
        lines=(folder/f"worker-{w}.txt").read_text().splitlines()
        assert lines.count(f"DELTAGUARD_B1B1B1B0_WORKER_PASS worker={w} records=42 negative=5")==1
        rows=0
        for line in lines:
            item=parse(line)
            if item is None:continue
            assert item["worker"]==w
            k=(w,item["lane"],item["rep"],item["case_idx"])
            assert k not in records
            records[k]=item
            rows+=1
        assert rows==42
    assert len(records)==210
    totals=[]
    for lane,n in enumerate(NS):
        values=[v for v in records.values() if v["lane"]==lane]
        assert len(values)==105
        assert sum(v["case_idx"]>=3 for v in values)==60
        totals.append({
            "N":n,"probes":105,
            "early_sigkill_uncommitted":sum(v["case_idx"]<3 for v in values),
            "durable_committed_replayed":sum(v["replayed"] for v in values),
            "receiver_disk_deduplicated":sum(v["dedup"] for v in values),
            "source_bootstrap_total_bytes":sum(v["initial_bytes"] for v in values),
            "stage_tcp_total_bytes":sum(v["stage_bytes"] for v in values),
            "recovery_tcp_total_bytes":sum(v["recovery_bytes"] for v in values),
            "cold_source_gen1_reconnect_bytes":290+16*n,
            "committed_delta_gen2_reconnect_bytes":308,
        })
    return {
        "schema":"deltameter.b1b1b1b0.durable-two-owner-prefix-restart.v1",
        "source_sha":head,"certificate_sha256":CUTS,"workers":5,
        "source_bound_probes":210,"two_owner_sigkill_source_processes":420,
        "durable_linux_sync_all_and_directory_sync":True,
        "source_authority_from_private_persisted_files_only_on_reboot":True,
        "receiver_separate_os_subprocess_reads_synced_receiver_bin":True,
        "sigkill_does_not_simulate_power_failure":True,
        "no_multi_generation_or_delete":True,
        "no_malicious_sender_authentication":True,
        "no_system_product_go":True,
        "lanes":totals,
        "verdict":"B1B1B1B0_DURABLE_PREFIX_REPLAY_RESEARCH_ACCEPT_NO_PRODUCT_GO",
    }
def main():
    p=argparse.ArgumentParser()
    p.add_argument("--input",type=Path,required=True)
    p.add_argument("--head",required=True)
    p.add_argument("--certificate",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    data=aggregate(a.input,a.head,a.certificate)
    a.out.parent.mkdir(parents=True,exist_ok=True)
    a.out.write_text(json.dumps(data,indent=2,sort_keys=True)+"\n")
    print(f"B1B1B1B0_AGGREGATE_PASS records={data['source_bound_probes']} verdict={data['verdict']}")
    for v in data["lanes"]:
        print(f"B1B1B1B0_REPLAY N={v['N']} cases={v['probes']} "
              f"replay={v['durable_committed_replayed']} dedup={v['receiver_disk_deduplicated']} "
              f"bootstrap_bytes={v['source_bootstrap_total_bytes']} "
              f"recovery_bytes={v['recovery_tcp_total_bytes']}")
if __name__=="__main__":main()
