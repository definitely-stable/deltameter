# M6-C — Bit-equivalent Energy sign hashing plan

Status: complete in PR #22; ACCEPT. Issue: #18. Parent: #14.

## Goal

Replace the cubic sign hash's per-row GF(2^64) multiplications with a bit-equivalent linear-functional cache while preserving every observable Energy result and snapshot-v1 byte.

## Algebra

Let L(y) be bit zero of y in the existing polynomial basis.

For fixed c, multiplication y -> c*y is GF(2)-linear. Define:

~~~text
mask(c)[i] = L(c * 2^i)
~~~

Then for every y:

~~~text
L(c*y) = parity(mask(c) & y)
~~~

The existing cubic sign polynomial is:

~~~text
p(x) = c0 + c1*x + c2*x^2 + c3*x^3
sign(x) = +1 when L(p(x))=0, else -1
~~~

Therefore the exact same sign bit is:

~~~text
L(c0)
xor parity(mask(c1) & x)
xor parity(mask(c2) & x^2)
xor parity(mask(c3) & x^3)
~~~

Compute x^2 and x^3 once per key, outside the row loop.

For B>1:

- baseline: one bucket GF multiply + three sign GF multiplies per row = 4R;
- candidate: one bucket GF multiply per row + two shared powers = R+2.

For the default R=23 profile this is 92 -> 25 GF multiplications per update. This is an operation count, not a throughput claim.

## Cache placement

Do not alter EnergyRowHash or EnergyConfig layout.

EnergyDeltaMeter receives a private Box<[[u64;3]]> derived sign-mask cache:

- 24R raw bytes, 552 bytes for R=23;
- built from the existing c1/c2/c3 coefficients;
- never serialized;
- rebuilt by ordinary construction/decode;
- cloned/reused by difference so merge does not repay mask construction.

Snapshot v1 remains byte-identical.

## Correctness gates

Before performance acceptance:

1. prove/test L(c*y)=parity(mask(c)&y);
2. compare fast sign against the existing Horner oracle for:
   - zero;
   - one;
   - high-bit;
   - all-ones;
   - deterministic full-width coefficients and keys;
3. cover every Energy profile shape;
4. preserve signed-difference behavior;
5. preserve failed-update atomicity;
6. preserve post-decode continuation;
7. run snapshot-v1 golden vectors on base and head;
8. preserve public API and theorem/randomness contract.

The old Horner sign implementation remains test-only as the differential oracle.

## Performance gates

Paired GitHub-hosted base/head evidence must measure separately:

- EnergyDeltaMeter::new construction;
- snapshot decode, which rebuilds masks;
- update steady state;
- difference/merge;
- query as an unchanged control.

Use identical injected benchmark/compatibility sources, pinned SHAs, same runner and alternating AB/BA rounds.

Retain the candidate only if:

- update has a repeatable material improvement;
- construction/decode overhead is explicitly reported;
- merge/query do not materially regress;
- amortization is favorable for a realistic number of updates.

No absolute CI latency threshold.

## Non-goals

- no snapshot format change;
- no coefficient/GF polynomial/profile change;
- no unsafe/SIMD/nightly;
- no batch API;
- no new dependency;
- no change to Coverage::Proven assumptions.


## Completion record

M6-C completed on 2026-10-06. Canonical evidence: [M6-C evidence](../../M6-C-EVIDENCE.md).

Final measured implementation head: `9952eea2760ca07cf2075cfde457a39b477930dc`.

Final hosted paired run: `m6c-performance #6 / 37516957320`.

Verdict: **ACCEPT**. Energy update latency improves by about 68–71%; optimized cache construction amortizes after roughly 1–1.4 updates; snapshot-v1 bytes and public/theorem contracts are unchanged.
