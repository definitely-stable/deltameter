# STRICT-COMPACT — canonical next-work execution plan

Umbrella: #70  
Parent research program: #59  
Public-boundary gate: #69  
Execution issues: #72 OPT-A, #73 OPT-B, #74 OPT-C

Status: **canonical execution order before public profile/snapshot freeze**.

## Executive decision

The project has already proved enough mathematics to stop asking whether
STRICT-COMPACT is feasible.

The next question is product shape.

The correct critical path is:

~~~text
OPT-A  range-aware J + physical layout
  |
  +--> OPT-B maintained odd-count cache
  |
  +--> OPT-C progressive level-prefix transfer
  |
  +--> #69 public profile/snapshot decision
~~~

Generalized ladder, DeltaGuard and oracle alternatives remain later research lanes.

Do not start public state/profile serialization before OPT-A and OPT-C close.

## Accepted baseline

The current accepted private candidate is:

~~~text
m                           4096 rows
J                           64 levels
instance parity state       32,768 B
shared certified Q32 table  16,384 B
ideal statistical delta     <= 1e-6
useful width contract       q95 U/d <= 1.5
useful d range              [4096, 2^64-1]
domain ceiling              2^64 (u128)
oracle                      keyed BLAKE3 v1
oracle update               ~96.5 ns/token median hosted evidence
private API                 GO_PRIVATE_API_FREEZE
~~~

The 16 KiB Q32 table is shared/static and must be reported separately from
per-instance state in every optimization result.

---

# Phase 1 — OPT-A: state/range frontier and physical layout

Issue: #72.

This is the only immediate implementation priority.

## A0 — parameterize the existing certificate without changing evidence

The current range certifier hardcodes:

~~~text
LEVELS = 64
~~~

while the runtime Q32 table itself is normalized by observed count and was built
for the conservative per-level budget:

~~~text
alpha = delta / 64.
~~~

First refactor the research tooling so active stored levels are an explicit
parameter while retaining the **same committed Q32 table**.

Required rule:

~~~text
active J <= 64
used alpha mass = J * delta/64 <= delta
~~~

Therefore the first J-frontier remains conservative without generating a new
table.

Files expected to change:

- `research/strict_compact_range_certify.py`
- new `research/strict_compact_j_frontier.py`
- new tests/reference JSON as needed.

The existing J=64 artifact must reproduce byte-for-byte equivalent verdict fields
or an explicitly reviewed schema-equivalent result.

This is the regression anchor.

## A1 — exact continuous range frontier

Evaluate:

~~~text
J = 24, 32, 40, 48, 56, 64
m = 4096
delta = 1e-6
q95 width target = 1.5
d_min = 4096
~~~

For each J, restrict certifying levels to 1..=J and construct a hole-free
continuous certificate:

~~~text
[4096, d_max(J)]
~~~

using the same exact-rational proof boundary as STRICT-COMPACT-002.

Binary64 remains candidate-ranking only.

Required Pareto output:

| J | state bytes | d_max | exact intervals | levels used | ceiling transition |
|---:|---:|---:|---:|---|---|

No candidate is called "default" in OPT-A.

## A2 — physical state layout must actually shrink

This is a critical implementation requirement.

The current J=64 private prototype stores:

~~~text
words.len() = rows = 4096
one u64 per row
~~~

which is naturally 32 KiB.

Simply changing a constant to J=32 would still consume 32 KiB unless the bitmap is
physically repacked.

For each J prototype an exact logical m×J bit state with:

~~~text
packed words = ceil(m*J/64)
state bytes  = ceil(m*J/8)
~~~

and verify the advertised sizes:

~~~text
J=24  12 KiB
J=32  16 KiB
J=40  20 KiB
J=48  24 KiB
J=56  28 KiB
J=64  32 KiB
~~~

## A3 — compare row-major and level-major layouts

Do not freeze the current row-major shape by inertia.

### Compact row-major

Logical bit:

~~~text
bit = row*J + (level-1)
~~~

Advantages:
- close to current F-PCSA generic packing;
- row-local logical representation.

Disadvantages:
- individual levels are scattered across the bitmap;
- progressive level transfer requires gather/repack.

### Level-major

Logical bit:

~~~text
bit = (level-1)*m + row
~~~

For m=4096:

~~~text
one level = 4096 bits = 64 u64 = 512 B
~~~

Advantages:
- each level is one contiguous 512-byte chunk;
- natural progressive/prefix transfer;
- natural trailing-level omission;
- level count scan is contiguous;
- same total memory.

Potential cost:
- different update address arithmetic/cache behavior.

### Required lab

Implement both layouts research-only for the J matrix.

Verify:
- identical logical cell state under the same keyed oracle stream;
- repeated toggle cancellation;
- XOR merge equivalence;
- levels >J are idle/truncated;
- same integer upper bound from extracted level counts;
- no allocation per toggle.

Measure:
- update ns/token;
- estimate extraction ns;
- XOR merge throughput;
- state bytes.

The preferred layout is selected by **system/product evidence**, not only update
microbenchmark latency.

Default design bias entering the experiment: level-major is preferable if its
update cost is not materially worse, because it directly enables OPT-C.

## A4 — measurement escalation

Do not immediately spend a large five-worker matrix on all twelve
(J × layout) combinations.

Stage 1:
- one hosted worker;
- all J/layout combinations;
- correctness + coarse performance;
- eliminate clearly dominated layouts/profiles.

Stage 2:
- five hosted workers;
- J=64 control plus at most three Pareto candidates;
- rotate benchmark order;
- retain raw observations.

Predeclare performance validity before Stage 2.

State/range is the primary objective; a small update regression may be acceptable
for a 2x state reduction, but it must be explicit.

## A5 — OPT-A verdict

Allowed outcomes:

### RANGE_FRONTIER_PASS

At least one J<64 produces a real smaller packed implementation and a meaningful
continuous certified range.

### KEEP_J64

Smaller J values lose too much useful range or create unacceptable implementation
cost.

### INVALID

Any certificate hole, table drift, logical-layout mismatch, oracle mapping drift or
missing measurement invalidates the slice.

OPT-A selects a **Pareto shortlist**, not yet a public default.

---

# Phase 2 — OPT-B: maintained odd-count cache

Issue: #73.

Start implementation only after OPT-A identifies the retained physical layout(s).

The theorem does not depend on this cache.

## B0 — cache semantics

Maintain:

~~~text
odd_counts[J]: u16
~~~

because each count lies in 0..=4096.

Memory:

~~~text
2*J bytes
J=32 -> 64 B
J=64 -> 128 B
~~~

On update:

~~~text
old = bit
bit ^= 1
if old == 0: count += 1
else:        count -= 1
~~~

The cache must be exactly rebuildable from canonical parity state.

## B1 — merge rule

Counts are **not linearly mergeable by arithmetic addition**.

For XOR merge:

1. XOR canonical packed words;
2. rebuild counts from the merged bitmap.

Measure rebuild cost and charge it to merge.

Do not invent a second merge algebra for the cache.

## B2 — serialization rule

Default decision entering the experiment:

**cache is derived and non-serialized**.

Rationale:
- mathematical/canonical state remains one parity bitmap;
- avoids snapshot invariants for redundant data;
- decode/rebuild remains deterministic.

Only reopen this if evidence shows rebuild is a real product bottleneck.

## B3 — paired performance protocol

Compare scan-based vs cached estimate with the same oracle/profile.

Measure:

- update ns/token;
- single estimate ns;
- merge + required rebuild;
- mixed workloads:
  - estimate every 1 update;
  - every 10;
  - every 100;
  - every 1000.

Suggested acceptance gates to freeze before timing:

- update regression <=10% on every hosted worker;
- query latency improvement >=5x on every hosted worker;
- total mixed-workload cost improves >=10% for at least one realistic
  estimate-every-1/10/100 lane on every worker.

If these gates prove inappropriate, change them **before** measurement, not after.

## B4 — verdict

### KEEP_CACHE

Measured product value exceeds update/memory cost.

### DROP_CACHE

The scan path is already cheap enough or update regression dominates.

Either result is acceptable. Cache is not required for public STRICT-COMPACT.

---

# Phase 3 — OPT-C: progressive/prefix transfer

Issue: #74.

Hard dependency: OPT-A.

This phase should use the retained physical layout. If level-major survives OPT-A,
progressive transfer becomes naturally zero-copy by level chunks.

## C0 — theorem note first

Before wire code, write a short proof note for adaptive prefix extension.

Given fixed predeclared per-level budgets:

~~~text
sum_j alpha_j <= delta
~~~

define event E that every per-level upper confidence statement holds.

Then:

~~~text
P(E) >= 1-delta.
~~~

On E, for **any** subset of received levels L:

~~~text
U_L = min_{j in L} U_j >= d.
~~~

Therefore choosing to request additional predeclared levels after observing earlier
bounds does not spend a second statistical error budget.

The stopping/request policy may be adaptive; the per-level statistics and budgets
must remain the predeclared ones.

Only after reviewing this note may docs use words such as "progressive" or
"anytime".

## C1 — representation candidates

Compare:

1. complete profile;
2. ordered one-shot prefix stream;
3. receiver-driven prefix extension;
4. trailing all-zero level elision.

Each level at m=4096 is 512 B in level-major form.

No snapshot-v1 modification.

## C2 — ordering

Do not assume natural numeric level order is optimal.

Evaluate:
- low-to-high;
- high-to-low;
- center-out / profile-specific order based on OPT-A range evidence.

A progressive representation must carry level identity if transfer order is not
canonical numeric order.

## C3 — product model

For named d regimes and network grid report:

- initial bytes;
- median/p95 bytes until the requested bound quality is reached;
- max bytes;
- number of round trips;
- RTT sensitivity;
- savings vs full profile.

Do not repeat the M6 mistake of optimizing bytes while hiding RTT.

## C4 — integrity/config binding

Research framing must include enough metadata to prevent accidental mixing of:

- profile/J;
- level index;
- oracle/key ID;
- table/version.

Secret key is never transmitted.

CRC/authentication design is research-only here; no public format freeze.

## C5 — verdict

### PROGRESSIVE_PASS

Typical transfer materially decreases without unacceptable RTT/API complexity.

### STREAM_ONLY

Incremental level chunks are useful, but receiver-driven request/response costs too
many RTTs; keep a one-shot ordered stream.

### DROP_PROGRESSIVE

Full fixed profile is simpler and system-level cost wins.

---

# Work allowed in parallel

## #69 public-boundary work

May continue in parallel on:
- computational coverage vocabulary;
- runtime BLAKE3 dependency policy;
- caller-managed secret semantics;
- zeroization policy decision;
- conceptual key/config binding.

Must **not** freeze:
- public memory profile;
- state physical layout;
- snapshot payload layout.

Those depend on OPT-A/C.

## Mathlab

Mathlab may independently investigate:
- finite independence;
- adaptive-token theorem;
- heterogeneous ladders;
- lower bounds;
- nested/prefix optimality.

No Mathlab result is required for OPT-A/B/C.

---

# Work explicitly deferred

Do not start yet:

- generalized `(pi_j,m_j,alpha_j)` ladder implementation;
- coefficient=1 redesign;
- DeltaGuard implementation;
- KMAC/HMAC/AES/SipHash oracle shootout;
- public STRICT-COMPACT source files;
- new persistent snapshot format;
- crate release work.

Reason: A-C can change the product shape cheaply while preserving the existing
theorem.

---

# Evidence discipline

Every slice gets:

1. protocol/plan frozen before measurement;
2. exact source HEAD;
3. raw hosted artifacts;
4. fail-closed completeness checks;
5. canonical evidence document;
6. issue/ROADMAP decision update.

No benchmark number becomes an API promise.

The mathematical certificate and the engineering performance experiment remain
separate evidence layers.

---

# Expected decision point after A-C

After OPT-A/B/C, issue #69 should receive one of three product inputs.

## PRODUCT_SHAPE_GO

We have:
- a Pareto-selected state/range profile;
- a chosen physical layout;
- a cache decision;
- a progressive/full-transfer decision.

Then public API/snapshot design can finally freeze.

## KEEP_32K_SIMPLE

Optimization does not materially improve the baseline. Freeze the existing simple
32 KiB J=64 profile rather than shipping unnecessary complexity.

## RESEARCH_AGAIN

A-C expose enough headroom that the generalized ladder is likely to materially
change the product. Open OPT-D/Mathlab OCC work before public freeze.

The default bias is **not** to choose RESEARCH_AGAIN unless A-C provide quantitative
evidence that another large improvement is plausible.
