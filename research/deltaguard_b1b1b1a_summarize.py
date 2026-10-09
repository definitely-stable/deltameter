#!/usr/bin/env python3
"""Fail-closed 390 physical TCP fault probes / two-owner source FULL reset.

No malicious-peer authentication or crash-consistent incremental recovery.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import re
from pathlib import Path

CASES = ("disconnect", "short_header", "short_body", "checksum", "owner",
         "epoch", "generation", "key", "noncanonical", "membership",
         "oversize", "ackloss", "sigkill")
NS = (256, 65536)
CUTS = "c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935"
FIELDS = set("worker lane rep fault_idx case N d bootstrap_bytes fault_bytes recovery_bytes ackloss_once rss_healthy_max cpu_healthy_ticks exact".split())
FAULT_BYTES = {
    "disconnect":1,"short_header":17,"short_body":68,
    "checksum":90,"owner":90,"epoch":90,"generation":90,
    "key":90,"noncanonical":99,"membership":90,
    "oversize":65,"ackloss":163,"sigkill":69,
}

def parse(line: str) -> dict | None:
    if not line.startswith("B1B1B1A_PROBE "):
        return None
    tokens = [item.split("=", 1) for item in line[14:].split()]
    row = dict(tokens)
    assert len(tokens) == len(FIELDS) == len(row) and set(row) == FIELDS
    assert row["case"] in CASES
    for k in FIELDS - {"case"}:
        assert re.fullmatch(r"[0-9]+", row[k]), (k, row[k])
        row[k] = int(row[k])
    assert row["worker"] in range(1,6)
    assert row["lane"] in (0,1)
    assert row["rep"] in (0,1,2)
    assert row["fault_idx"] == CASES.index(row["case"])
    assert row["N"] == NS[row["lane"]]
    assert row["d"] == 48
    boot = 242 + 16 * row["N"]
    assert row["bootstrap_bytes"] == boot
    assert row["recovery_bytes"] == boot + 16
    assert row["fault_bytes"] == FAULT_BYTES[row["case"]]
    assert row["ackloss_once"] == (1 if row["case"] == "ackloss" else 0)
    assert row["rss_healthy_max"] > 0
    assert row["cpu_healthy_ticks"] >= 0
    assert row["exact"] == 1
    return row

def self_test() -> None:
    example = ("B1B1B1A_PROBE worker=1 lane=0 rep=0 fault_idx=3 case=checksum "
               "N=256 d=48 bootstrap_bytes=4338 fault_bytes=90 recovery_bytes=4354 "
               "ackloss_once=0 rss_healthy_max=1024 cpu_healthy_ticks=1 exact=1")
    assert parse(example)
    for bad in (
        example.replace("fault_bytes=90","fault_bytes=89"),
        example.replace("fault_idx=3","fault_idx=2"),
        example.replace("ackloss_once=0","ackloss_once=1"),
        example.replace("exact=1","exact=0"),
        example.replace("recovery_bytes=4354","recovery_bytes=0"),
        example.replace("case=checksum","case=notfound"),
    ):
        try:
            parse(bad)
        except (AssertionError, ValueError, KeyError):
            continue
        raise AssertionError("mutated protocol evidence accepted")

def aggregate(root: Path, sha: str, certificate: Path) -> dict:
    assert re.fullmatch(r"[0-9a-f]{40}", sha)
    assert hashlib.sha256(certificate.read_bytes()).hexdigest() == CUTS
    self_test()
    records = {}
    for worker in range(1,6):
        assert (root/f"head-{worker}.txt").read_text().strip() == sha
        assert (root/f"rust-{worker}.txt").exists()
        lines = (root/f"worker-{worker}.txt").read_text().splitlines()
        assert lines.count(
            f"DELTAGUARD_B1B1B1A_WORKER_PASS worker={worker} probes=78 resets=78 ackloss=6 sigkill=6"
        ) == 1
        count = 0
        for line in lines:
            row = parse(line)
            if row is None:
                continue
            assert row["worker"] == worker
            key = (worker,row["lane"],row["rep"],row["fault_idx"])
            assert key not in records, "duplicate source-bound physical probe"
            records[key] = row
            count += 1
        assert count == 78, (worker,count)
    assert len(records) == 390
    summary = []
    for lane,n in enumerate(NS):
        subset = [r for r in records.values() if r["lane"] == lane]
        assert len(subset) == 195
        summary.append({
            "N":n,
            "source_bootstrap_bytes_per_case":242+16*n,
            "physically_resynced_full_bytes_per_case":258+16*n,
            "source_case_count":195,
            "sum_full_resync_bytes":sum(r["recovery_bytes"] for r in subset),
            "sum_fault_bytes":sum(r["fault_bytes"] for r in subset),
            "ackloss_exactly_once_count":sum(r["ackloss_once"] for r in subset),
            "real_sigkill_count":sum(r["case"]=="sigkill" for r in subset),
            "worker_case_counts":[sum(r["worker"]==w for r in subset) for w in range(1,6)],
        })
        assert summary[-1]["ackloss_exactly_once_count"] == 15
        assert summary[-1]["real_sigkill_count"] == 15
        assert summary[-1]["worker_case_counts"] == [39]*5
    return {
        "schema":"deltameter.b1b1b1a.full-reset-faults.v1",
        "source_sha":sha,
        "certificate_sha256":CUTS,
        "workers":5,"physical_source_probes":390,
        "true_os_sigkill_cases":30,
        "ackloss_true_replay_once_cases":30,
        "full_two_owner_tcp_resets":390,
        "source_rebuild_is_fixture_deterministic_not_crash_durable":True,
        "owner_integrity_tag_is_not_authenticated":True,
        "incremental_crash_journal_tested":False,
        "public_product_go":False,
        "scenarios":summary,
        "verdict":"B1B1B1A_PHYSICAL_FULL_RESET_FAULTS_PASS_NO_PRODUCT_GO",
    }

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--input",type=Path,required=True)
    ap.add_argument("--head",required=True)
    ap.add_argument("--certificate",type=Path,required=True)
    ap.add_argument("--out",type=Path,required=True)
    args = ap.parse_args()
    report=aggregate(args.input,args.head,args.certificate)
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(report,sort_keys=True,indent=2)+"\n")
    print(f"B1B1B1A_AGGREGATE_PASS records={report['physical_source_probes']} "
          f"verdict={report['verdict']}")
    for x in report["scenarios"]:
        print(f"B1B1B1A_COST N={x['N']} bootstrap={x['source_bootstrap_bytes_per_case']} "
              f"recovery={x['physically_resynced_full_bytes_per_case']} "
              f"case_count={x['source_case_count']} "
              f"fault_bytes={x['sum_fault_bytes']} "
              f"ackloss={x['ackloss_exactly_once_count']} "
              f"sigkill={x['real_sigkill_count']}")
if __name__ == "__main__":
    main()
