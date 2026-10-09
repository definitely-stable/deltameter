# DeltaGuard B1-B1-A — two sender OS processes, paced localhost TCP

Parent [#97](https://github.com/definitely-stable/deltameter/issues/97),
[#92](https://github.com/definitely-stable/deltameter/issues/92);
research [PR #98](https://github.com/definitely-stable/deltameter/pull/98).
[Frozen protocol](DELTAGUARD-B1B1A-PROTOCOL.md) was committed at
`cb62948b8a1101e96dba1ac50941305f8cd13627` BEFORE any B1-B1-A
measurement. Prior B1B0/D58 one-sided B2A exact certificate unchanged.

## Scope and reproducible evidence

**B1B1A_PHYSICAL_TWO_PROCESS_COST_FOUNDATION_PASS_NO_PRODUCT_GO**.

First fully valid five-worker hosted
[run 37911404072](https://github.com/definitely-stable/deltameter/actions/runs/37911404072),
source `5a91d358cb9801130bd4804e225eacd28c9403ed`.
All five separately hosted GitHub workers passed and exact source-
and cutoff-bound fail-closed Python aggregator passed exactly **300/300
records**. Each record is 1 of 4 preregistered lanes, 3 repeats,
5 service modes, across 5 workers. The 24 B2A cutoffs were
independently regenerated and their byte-exact cert SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`
pinned. The CI branch HEAD must be cited separately after any
follow-up doc commits.

Two sender **OS child processes** connect over independent loopback
TCP sockets to the parent receiver process. Source owner A/B
initial canonical inventories and B2A bitmaps are physically
constructed independently, with shared per-generation token updates.
The receiver verifies source truth, B2A XOR against per-token keyed
rehash, exact cold and physically provisioned warm two-list receiver,
canonical sequenced events, and explicit two-sender resolved fallback.
The physical record counts both 25B child hellos and 24B receiver
requests per sender, complete B1-A 64B headers and all payload.
Source child build/update CPU and `/proc/self/status` VmHWM
are collected along with parent peak RSS.

The app delays in this run are **not Internet RTT**: a child sleeps
a predeclared response delay per request. Per-owner byte pacing
uses sleeps before sending bounded chunks, not Linux qdisc
or a single contended bottleneck. Child processes share the SAME
GitHub-hosted machine, not separate physical hosts; fixtures and
checksum are public and not malicious-peer authentication.

## Real one-query physical byte results

All listed bytes physically passed over sender TCP sockets and
include receiver requests, sender hello, headers and payloads.
`warm_exact` additionally paid (and records) a real prior
two-owner source sync, but query bytes below correctly exclude
that prior provisioning.

| Lane / T64 | Guard-only query | Warm exact query | Warm exact prior sync | Resolved guard query |
|---|---:|---:|---:|---:|
| N256,d48,S10; 10Mbps, delay 0ms | 738B | 338B | 4,322B | 738B (SAFE) |
| N256,d48,S100; 10Mbps, delay 10ms | 738B | 1,958B | 4,322B | 738B (SAFE) |
| N256,d57,S10; 10Mbps, delay 50ms | 738B UNKNOWN | 338B | 4,330B | **5,162B** fallback |
| N65536,d48,S10; 100Mbps, delay 10ms | 738B | 338B | 1,048,802B | 738B (SAFE) |

`S` is the requested source generation number, **not the count of
guard queries in the B1B1A single-query trial**. For frequency /
cumulative 100-session query contracts, use predecessor D58 and
the separate future persistent daemon benchmark. Under this
physical 25B-hello / 24B-request frame protocol, warm exact
at S10 needs **338B**, 54% fewer bytes than the 738B guard,
even for large N if prior source synchronization is real.
At S100 warm exact needs **1,958B**, so 738B guard transfers
about 62.3% fewer query bytes, conditional on one query after
99 source updates and an already maintained guard.
Cold exact must additionally transfer both full initial inventories
and validate sequenced deltas before it can claim a warm query.

N256,d57,S10 resolved guard performs a second physical request
to each source and transfers the complete source lists:
**5,162B**, compared to DIRECT_FULL's **4,474B**,
so preserve **STOP_RESOLVED_NEAR_T**, even before additional
feedback-induced delay.

## Latency evidence and critical comparability caveat

The raw logs record wall-clock times (ns), child source-build/
incremental-update elapsed wall-clock diagnostics, owner **pre-transfer** RSS and parent RSS,
with per-worker *3-sample nearest-rank p95 = MAXIMUM*.
This is numerically correct but statistically weak.

For example, N256,d48,S10 at delay 0ms, per-worker
three-trial maximum guard time was about 1.33–1.75ms;
the corresponding warm exact query was about 0.21–0.25ms.
**These numbers MUST NOT be used to conclude warm exact
has lower p95 in an equivalent persistent deployment**:
guard timing begins BEFORE spawning sender OS processes and before
their initial source/bitmap build; the warm exact query timer
begins AFTER actual receiver-source bootstrap on already connected
live senders. These have different process-lifecycle boundaries.
`total_ns` reports the full warm setup duration including that
bootstrap separately. Fair hot/hot and cold/cold p95 require the
next protocol slice and persistent sender loops. Mixed timing
contracts are diagnostics, never a product winner.

The same caution applies across workers. Five separate hosted
VM workers and three repeats per profile are not a calibrated
host-independent p95 latency confidence interval. Unkeyed fixture
checksums and immutable epoch identifiers do not establish
hostile/adaptive sketch security.

## Decision and next work

This slice **ACCEPTS actual two-process transport, source-bound
pairing, calibrated app-pacing code, physical bytes and scope-
limited empirical 3-repeat latency diagnostics**.

It explicitly **REJECTS SYSTEM_PRODUCT_GO**. Before closure of
#97/#92, implement:
- truly persistent sender processes across multiple consecutive
  requests, with comparable hot/hot source/guard/exact lifecycle;
- paired warm/hot equal-timed queries, sufficient repeat count for
  meaningful p95, server build/update and receiver-side RSS HWM
  separated, cold-init amortization across query horizon;
- physical ACK/retry, peer disconnect and crash/restart/rollback,
  duplicated/skipped generations and hostile frame tests with
  fail-closed receiver mutation;
- fuller frozen 1/10/100Mbps and 0/10/50ms application-emulated
  lane selection, within GitHub-hosted CI budgets;
- independent #86 security model and #69 API/persistence gate.

A source-derived ideal-oracle <=1e-6 nonadaptive false-SAFE
certificate cannot be converted into malicious/adaptive coverage
from TCP fixture test outcomes. No public API or Snapshot v1 changed.
