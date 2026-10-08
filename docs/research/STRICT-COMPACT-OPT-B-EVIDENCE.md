# STRICT-COMPACT OPT-B — maintained odd-count cache evidence

Issue #73, parent #70, prerequisite merged PR #76.
Protocol: [STRICT-COMPACT-OPT-B-PROTOCOL.md](STRICT-COMPACT-OPT-B-PROTOCOL.md).

**Decision: KEEP_CACHE (research-only; no public/snapshot freeze).**

## Test conditions

Canonical measured source:
`2f8dc09199e376d3c6019ebb0d529d3879b15eeb`.

GitHub-hosted five-worker workflow:
[37807795968](https://github.com/definitely-stable/deltameter/actions/runs/37807795968)
— all five workers and fail-closed aggregator SUCCESS.

- 5 workers * 8 rounds * 3 J profiles * 2 implementations * 7 workloads =
  **1,680 complete raw timing observations**;
- J=52 LEVEL_MAJOR / 26 KiB packed bitmap primary;
  J=24 bounded-range scaling guard and J=64 original full-range control;
- same keyed BLAKE3 mapping, delta/64 Q32 table, u128 upper-bound arithmetic;
- **allocation-free SCAN control** uses stack counts and contiguous `count_ones`
  in each 64-word level, no per-query heap Vec;
- CACHE stores an additional exact derived u16 count for each level;
- identical pre-population, rotated order, two warmups, eight raw measurements;
- source HEAD per worker, exact J/variant/lane/round completeness and timing
  sanity checked before applying the frozen threshold;
- deterministic seeded/cancellation/XOR+rebuild and inference equivalence
  are required and passed by every worker.

### J=52 hosted per-worker results

| Worker | update cache/scan | query scan/cache | mixed-1 gain | mixed-10 gain | mixed-100 gain |
|---:|---:|---:|---:|---:|---:|
| 1 | 1.02374 | 14.080x | 89.02% | 64.89% | 16.15% |
| 2 | 1.02670 | 13.974x | 88.87% | 64.79% | 15.99% |
| 3 | 1.01779 | 13.563x | 88.81% | 63.01% | 15.22% |
| 4 | 1.02583 | 15.391x | 90.78% | 72.39% | 18.59% |
| 5 | 1.02601 | 18.137x | 92.09% | 74.17% | 23.86% |

Here mixed-k gain is (SCAN time - CACHE time) / SCAN time, for the entire
real workload that includes keyed token updates AND estimates every k updates.
Measurements are paired worker medians; results are not generalized
cross-hardware guarantees.

Pre-registered KEEP_CACHE requirements for **every** worker:

1. update regression <=10%: observed worst **+2.670%**, PASS;
2. query speedup >=5x: observed minimum **13.563x**, PASS;
3. one common realistic mixed lane from k=1/10/100 improves >=10% on every
   worker: **all three lanes** satisfy this condition (k=100 minimum +15.22%), PASS.

All gate requirements pass. Mixed-1000 was measured but cannot be the sole
qualifying product lane.

## Evidence/artifact provenance

GitHub artifact ZIP SHA-256 digests:

| Kind | ID | sha256 |
|---|---:|---|
| summary | 11564150419 | 14e52bdbea2ea888c859033ec0510568a953060ad4276a951ac831a81b6a7d50 |
| worker 1 | 11563906344 | 292a2bf66cd4972f866ce7f8535f77fb0494ce5f3519a07239965d7dfffe68de |
| worker 2 | 11562934948 | e7c4b31ff45094516d2d13ca9039b033ce7f583ccb16676966c38464206921ec |
| worker 3 | 11562649841 | 735f9cba8d81c643e4f5a78690b0f90efb02be243ed492aeeb14d87a8d18194d |
| worker 4 | 11563443528 | f6b98655fed8b28af1124a349c516bf80c499293efd1f007e8fbf6b86f3eaee6 |
| worker 5 | 11564045318 | ae5541eb554269397ecc19fd81f24512a95c0f6001c24438d1e8414e234e213e |

The workflow summary JSON contains all per-worker median ratios and the
J=24/64 guard-profile results, including the separately measured XOR merge +
rebuild lane and mixed-1000 lane. Unlike the update/query/mixed workload
gate, merge results have no frozen hard cutoff: this cannot be silently
presented as proof of uniformly faster merges.

## Semantics and memory accounting

Cache update occurs ONLY on an actual coefficient=1, in-range stored-bit XOR.
The exact per-level count is in [0,4096] and fits u16.

Cache cannot be linearly merged: merging canonical bitmaps by XOR requires
popcount rebuild from the resulting bitmaps. Cache is DERIVED and never
serialized, so it cannot change the canonical mathematical state,
snapshot representation, inference theorem or oracle contract.

At J=52 the logical bitmap is 26,624 B and derived cache payload is
104 B (0.391% extra logical parity-state bytes). The Rust owner, allocator
metadata and oracle key are separate and must not be omitted from any
later *total per-instance allocation* claim.

## Product decision

- **KEEP_CACHE** for subsequent research/product-shape design at J=52.
- No public API, no persistence wire change, no new confidence theorem.
- Use the existing frozen Q32 and keyed-BLAKE3 computational contract.
- For #74 progressive transfer, transmit canonical bitmap chunks only;
  derived counts can be rebuilt from received level chunks and are never
  implicitly transmitted.
- #69 public profile/snapshot remains blocked pending the separate OPT-C gate.
