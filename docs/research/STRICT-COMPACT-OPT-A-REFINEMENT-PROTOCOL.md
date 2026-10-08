# STRICT-COMPACT OPT-A refinement: J=51/52 full-range boundary

Issue #72 / PR #76; parent #70; public gate #69.

Status: **PRE-REGISTERED PROTOCOL; NO REFINEMENT RESULTS YET**.

## Why a separate lane

The accepted J=64 interval proof and OPT-A Stage 1 show that no selected width witness uses
a level above 52. Thus the same existing witnesses are legal for J=52; a packed
52-level profile would occupy exactly 4096*52/8 = 26,624 B = 26 KiB, compared
with J=56 (28 KiB) and J=64 (32 KiB). The published J=56 verdict is a
*minimum of the original tested matrix*, not a proof of minimal J.

The original frozen Stage 1/2 datasets, thresholds, profile matrix, artifacts
and original verdict must not be overwritten or silently reinterpreted.

## Statistical design

Hold fixed m=4096, alpha per level=delta/64, delta=1e-6, the certified Q32
table hash 634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a,
keyed-BLAKE3 v1, u64 tokens, q95 U/d <=1.5 on [4096,2^64-1], and
upper domain ceiling 2^64 represented by u128.

Use the **existing exact-rational interval verifier** on J=51 and J=52.
J=52 must cover every integer d from 4096 through 2^64-1 without gaps; any
missing witness is a fatal invalid result, not a reason to approximate.
J=51 is exploratory: a certificate success is admissible, but a failure to
certify is NOT a mathematical lower bound on the required J. The interval
construction's float comparisons rank proposals only; the pass boundary is exact.

Serialize all values in the d-domain as decimal strings, including 2^64,
rather than JavaScript/JSON floating-point numbers. Validate their round trip.
Retain the J=64 accepted witness set and distinguish theorem inheritance from
an independent J=52 certificate reproduction.

## Physical and correctness conditions

LEVEL_MAJOR, contiguous 512 B per level, exact packed size m*J/8.
Retain the exact keyed hash mapping and Q32 table.
Run existing logical-cell, XOR merge, double-toggle, cross-layout equivalence,
and estimate consistency checks on J=51/J=52, plus deterministic seeded
mixed-update stress, vector invariance, and no per-toggle storage growth.
No production, public API or snapshot change.

## Frozen hosted performance comparison

GitHub-hosted five-worker paired, rotated order. Profiles: J=52 candidate,
J=56 current full-range candidate, J=64 original control. Two warmups,
eight raw measured rounds per profile per worker. Record update ns/token,
estimate ns, XOR ns/KiB, and exact physical state bytes.

A J=52 implementation is viable only if on **every worker** relative to J=64:
update regression <=15%, estimate regression <=25%, XOR ns/KiB regression <=25%.
These are the original Stage-2 *viability* thresholds, not performance-win targets.
Also report J=52 vs J=56 without adding a post-hoc numerical gate.

For each worker require matching source commit SHA, complete 3*8 observations,
unique worker/profile/round keys, known finite positive timing values,
full worker PASS marker and exact state bytes. Aggregation fails closed.
Keep head/toolchain/Q32 digest and raw samples as artifacts.

## Decisions

- `J52_REFINEMENT_PASS`: independent certificate and all five workers pass.
  Promote J=52 / 26 KiB as the **preferred full-u64 research baseline**;
  document that J<52 is not proved impossible.
- `RETAIN_J56`: J=52 proof passes but physical correctness/performance fails.
- `INVALID`: certificate, oracle, provenance, data completeness or CI gaps.
  Do not replace the previously accepted J=56 verdict with unverified numbers.

No merge of PR #76, no #73 implementation and no public freeze until evidence
is inspected and recorded. #73/#74 may use J=56 as control before refinement
but must choose their primary profile only after this gate.
