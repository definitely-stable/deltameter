# STRICT-COMPACT-OPT-A — range/layout protocol

Issue: #72. Parent: #70. Baseline main:
`862579643fb44bfd3df3b65a863bfdc90b998611`.

Status: **FROZEN v2 BEFORE STAGE-2 PERFORMANCE EVIDENCE**.

## Question

Can STRICT-COMPACT materially reduce per-instance state below the accepted J=64
32 KiB baseline without weakening the existing statistical/oracle contract, and
which physical layout should be retained for the next progressive-transfer slice?

## Frozen statistical contract

Keep unchanged:

~~~text
rows m                      4096
ideal statistical delta     1e-6
per-level alpha table       delta/64
q95 width ratio target      1.5
declared useful d minimum   4096
domain cardinality          2^64
runtime Q32 table SHA-256   634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a
oracle                      keyed BLAKE3 v1 research contract
~~~

For active J<=64 the reused table remains conservative because only J of the
64 pre-budgeted level tests are used:

`sum(alpha_used) = J * delta/64 <= delta`.

No alpha reallocation is allowed in OPT-A.

## J matrix

~~~text
J = 24, 32, 40, 48, 56, 64
~~~

Expected packed instance bytes:

~~~text
J=24  12,288
J=32  16,384
J=40  20,480
J=48  24,576
J=56  28,672
J=64  32,768
~~~

The shared 16 KiB Q32 lookup table is reported separately and is not counted as
per-instance state.

## Mathematical frontier

The exact-rational STRICT-COMPACT-002 proof boundary is retained.

For each J:

1. certifying levels are restricted to 1..=J;
2. start at d=4096;
3. find the first integer d that cannot be certified at q95 U/d<=1.5;
4. define d_max as the preceding integer;
5. report a hole-free exact interval cover for [4096,d_max];
6. record the first failing d when finite;
7. J=64 must reproduce the accepted full-range result through 2^64-1.

Binary64 may rank candidate levels but may not decide certificate validity.

## Physical layouts

Every candidate must allocate exactly:

`ceil(m*J/64)` u64 words.

Two layouts are frozen.

### ROW_MAJOR

~~~text
bit = row*J + (level-1)
~~~

### LEVEL_MAJOR

~~~text
bit = (level-1)*m + row
~~~

Both represent exactly the same logical m×J parity matrix.

For m=4096 LEVEL_MAJOR makes one level exactly 64 contiguous u64 words = 512 B.

## Oracle and update semantics

Reuse STRICT-COMPACT-003 exactly:

- keyed BLAKE3;
- row from output word 0;
- level from trailing-zero distribution of output word 1;
- coefficient bit from output word 2;
- level>J is an idle/truncation event;
- coefficient=0 is idle;
- stored event XORs exactly one cell.

No new oracle mapping is allowed in OPT-A.

## Correctness gates

For every J and both layouts:

- allocated byte count equals m*J/8 exactly;
- double-toggle cancels;
- row-major and level-major represent identical logical cells after the same stream;
- level counts are identical;
- same-layout XOR merge equals combined-stream state;
- cross-layout extracted level counts agree after equivalent streams;
- integer lookup upper bound agrees across layouts;
- upper bound remains <=2^64 and uses u128;
- no per-toggle allocation;
- J=64 oracle/reference estimate vectors remain compatible with STRICT-COMPACT-004.

Any failure => OPT-A INVALID.

## Stage 1 hosted screen

One GitHub-hosted worker.

For every J/layout pair:

- populate with a deterministic unique-token stream;
- benchmark update;
- benchmark level-count extraction + lookup;
- benchmark XOR word merge separately.

Timing protocol:

- fixed deterministic corpus;
- 2 warmup rounds;
- 8 measured rounds;
- report median;
- no candidate decision from a single individual timing sample.

Stage 1 is for Pareto elimination only.

Metrics:

~~~text
update_ns_per_token
estimate_ns
xor_ns_per_kib
state_bytes
certified_d_max
~~~

## Stage 1 shortlist rule

Retain at most five distinct J values using this deterministic order:

1. J=64 full-domain control;
2. the smallest J with any nontrivial certified prefix;
3. the smallest J whose certified d_max reaches at least 2^32-1;
4. the smallest J whose certified d_max reaches at least 2^48-1;
5. the smallest J whose certified d_max reaches 2^64-1.

Skip duplicates or an unmet tier; do not replace them post-hoc with another profile.

### v2 amendment provenance

Stage 1 completed before any Stage-2 timing and revealed that J=56 preserves the
full u64 declared range while J=64 was the only full-range tier in the original
shortlist rule. Omitting the smallest full-range profile would make Stage 2 unable
to answer the stated product question.

Therefore v2 adds item 5 above before Stage-2 measurement. No Stage-2 observation
exists at amendment time. Frozen Stage-2 J values from the canonical Stage-1
artifact are `64,24,40,56`; all use LEVEL_MAJOR under the already-frozen layout
selection rule.

For each retained J prefer LEVEL_MAJOR. Select ROW_MAJOR instead only if its median
update ns/token is at least 10% lower **and** LEVEL_MAJOR is not at least 20% better
on either estimate latency or XOR ns/KiB.

This is only a shortlist rule, not a public-layout decision.

## Stage 2 confirmation

Only the J=64 control plus at most three shortlisted candidates advance.

Run five independent GitHub-hosted workers with deterministic benchmark-order
rotation and raw per-round evidence.

A candidate is implementation-viable if on every worker:

- state size matches the mathematical profile;
- correctness gates pass;
- update regression versus J=64 control is <=15%;
- estimate latency does not regress >25%;
- XOR merge throughput does not regress >25%.

These are viability limits, not product-win thresholds.

## OPT-A decision

### RANGE_FRONTIER_PASS

At least one J<64:

- has a hole-free certified useful range;
- physically realizes the advertised smaller state;
- passes five-worker implementation viability.

### KEEP_J64

No smaller J survives both range utility and implementation viability.

### INVALID

Any proof hole, table/oracle drift, layout mismatch, incomplete raw evidence or
post-hoc gate change.

## Non-goals

- no public API;
- no snapshot format;
- no alpha optimization;
- no rows/m optimization;
- no cached odd-count state;
- no progressive wire format;
- no generalized ladder;
- no new hash/oracle;
- no production dependency change.
