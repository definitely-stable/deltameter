# Decision record 0001 — pre-implementation baseline

Status: **accepted and updated through M6-C evidence**  
Updated: 2026-10-06 after accepting the bit-equivalent Energy sign-mask optimization.

## Context

DeltaMeter needs a small, composable estimator for symmetric-difference size. The project is maintained by one developer and uses ordinary public GitHub-hosted CI.

The main architectural risk is not lack of infrastructure. It is accidentally turning an asymptotic or empirical observation into a public mathematical guarantee.

## Decisions

### D1 — Primary mathematical model

For true sets:

```text
d = |A △ B| = ||A XOR B||_0
```

over GF(2).

The integer F2 representation remains a valid reference path, not the sole framing.

### D2 — First implementation backend

Implement **EnergyDeltaMeter first**.

Reason:

- simple finite-sample proof path;
- exact specialized variance under explicit assumptions;
- incremental energy statistic;
- useful reference for later Parity validation.

### D3 — Q1 strict-Parity gate

For v0, **NO-GO for strict Parity `Coverage::Proven`**.

Parity may expose:

- point estimate;
- asymptotic metadata;
- empirical/calibrated diagnostics if clearly labelled.

Parity may not expose a proven high-confidence capacity API until a concrete finite-sample theorem exists.

### D4 — Published F-PCSA and set-specialized parity are separate

The first Parity implementation must reproduce the **published finite-field construction**:

```text
FIELDMAP[i,j] in F
h(v) chooses row/level with published probability mass
g(v) is uniform over F
row statistic = highest non-zero level
```

Classical PCSA first-1/bitmap models are intuition only, not the proof object.

A proposed specialization with `g(v)=1` / `cell ^= 1` is a separate research algorithm, tentatively named `ParityPcsaSetV1`.

It inherits no published `1.638/sqrt(m)` claim automatically.

### D5 — Exact small-d recovery

Not in v0.

PinSketch/Minisketch is recorded as a strong future candidate, but no FFI/BCH decoder is added before Energy/Parity measurements show a real product need.

### D6 — Threat model

v0 supports the ordinary oblivious-input randomized-algorithm model.

A secret seed is not required for that theorem model.

Chosen-input-after-seed and adaptive-query robustness are separate future work.

### D7 — Randomness

For Energy, keep the proof contract explicit:

```text
bucket family: pairwise-uniform collisions
sign family:   4-wise independent signs
families:      independent/domain-separated
```

Do not weaken the sign requirement merely to simplify implementation.

Parity randomness remains unfrozen beyond reproduction requirements until its strict-tail path is known.

No cryptographic dependency is required by default.

### D8 — Hash implementation

Do not approve a hash family by brand name alone.

“Universal hash”, collision bounds, PRNG quality and exact k-wise independence are different properties.

The implementation must choose the smallest explicit construction that matches the estimator proof.

### D9 — Serialization

Do not freeze a persisted wire format during bootstrap.

First freeze the estimator state and configuration.

When persistence is later added, it must include at least:

- algorithm/version;
- domain;
- dimensions;
- randomness/hash-suite identity;
- seed/config identity;
- canonical endian-independent encoding.

Rust's default `Hash` is never a persisted identity.

### D10 — Rust shape

- one crate;
- concrete types before broad traits;
- `u64` keys first;
- safe scalar implementation first;
- stable Rust;
- no nightly SIMD requirement.

### D11 — Verification

Use:

- executable mathematics;
- deterministic reference vectors;
- property tests;
- exact small-case enumeration where useful;
- theorem-derived configuration tests.

Monte Carlo is diagnostic, not proof of `1e-6` or `1e-9` failure probabilities.

### D12 — CI

Use public GitHub-hosted runners normally.

Two layers:

```text
PR:
    fast deterministic checks

research/manual/main:
    wider finite-d grids
    moderate Monte Carlo diagnostics
    profile regeneration
    benchmark artifacts
```

The public repository removes quota pressure, but it does not make billion-trial rare-event simulation a mathematically useful proof strategy.

### D13 — M3 Parity public contract

M3 may expose the published GF(2)-F-PCSA reproduction through an experimental public wrapper.

Contract:

~~~text
Parity estimate:
    point estimate
    + Coverage::Asymptotic { relative_standard_error }

Parity capacity:
    no Coverage::Proven
    no recommended_capacity
~~~

Built-in Parity profiles are engineering memory/RSE scales, not strict epsilon/delta profiles.

The public u64 seed is a reproducibility input for the deterministic pseudo-oracle. It is not a cryptographic key and is not claimed to instantiate the paper's ideal random oracle.

Energy/Parity memory comparisons must keep their guarantee mismatch explicit.

### D14 — Formal verification and FFI

Not v0 requirements.

No Verus/Alerus/Creusot gate and no Minisketch FFI unless a concrete later need justifies the maintenance surface.

### D15 — Stop the published-W_i strict-Parity track

The post-M3 research round closes the current attempt to make the published W_i F-PCSA estimator a strict finite-sample backend.

Reasons:

- the exact full-state fixed-d law is exponential at target m,J;
- W_i is not sufficient for d;
- Poissonization gives a clean exact law but no practical certified fixed-d bridge;
- stochastic monotonicity of the full published statistic remains unresolved;
- no practical 1e-6/1e-9 one-sided inversion exists;
- Energy already supplies a strict backend.

This is a research stop gate, not an impossibility theorem for all GF(2)-linear estimators.

### D16 — ParityLevelCounts is research-only

A possible follow-up statistic is:

~~~text
S_j = popcount(FIELDMAP[:,j])
~~~

It reuses the mergeable FIELDMAP state and has exact one-level finite laws.

It remains research-only until all of the following exist:

- certified fixed-d tail/inversion;
- outward-rounded numerical evaluation for target deltas;
- useful width/power;
- complete memory/update/query accounting.

No public type and no Coverage::Proven are added now.

### D17 — No hybrid REPLACE decision from secondary reports

Simple Set Sketching, Minisketch/PinSketch, IBLT-derived estimators and related candidates remain separate future audits.

In particular:

- high-probability exact recovery is not automatically unconditional Coverage::Exact;
- exact mean/variance plus an asymptotic chi-square limit is not a finite-sample confidence theorem;
- alternative-backend memory numbers must be compared against the generated Energy profiles, including all amplification tables.

Historical post-M3 next step was M4; M4/M5 are now complete and the current sequence is M6 below.

### D18 — Freeze a root-only v0 API

M4 freezes the supported external surface at the crate root.

The published F-PCSA reproduction remains internal research machinery behind ParityDeltaMeter. Raw Parity packed words and Energy's internal row slice are not public contracts.

Public enums/result records that may grow are marked #[non_exhaustive] before the freeze. A compile-level integration test exercises the supported root surface.

### D19 — Freeze balanced v0 defaults without conflating guarantees

The v0 defaults are:

~~~text
EnergyProfile::DEFAULT:
    relative error = 10%
    failure probability <= 1e-6
    strict finite-sample Proven contract,
    subject to the explicit independent-uniform randomness precondition

ParityProfile::DEFAULT:
    Standard
    m = 256
    J = 64
    asymptotic RSE ~= 10.2375%
    experimental Asymptotic contract
~~~

The similar nominal error scale does not make the guarantees equivalent.

### D20 — Do not add a batch API in v0

M4 does not freeze add_many, toggle_many, iterator ingestion or a batch trait.

The current scalar implementations do not expose a measured batch-specific amortization opportunity. Moving a caller loop behind a new method would enlarge the compatibility surface without changing the algorithm.

Revisit batching only with a concrete implementation advantage such as vectorized hashing, parallel lane processing or another measured amortization mechanism.

### D21 — Performance changes require paired evidence

The PR benchmark runs base and head sequentially on the same GitHub-hosted runner and records update/merge/query medians.

Accepted:

- Parity J=64 query fast path;
- allocation-free direct accumulation of the published row statistic.

Rejected and reverted:

- Energy non-mutating merge rewrite;
- Energy stack-scratch median query.

Those Energy candidates did not show a stable paired benefit. Hosted-runner benchmark numbers remain diagnostic, not product promises.

### D22 — M4 does not freeze serialization

API/state stabilization is not evidence that persistence is required.

M4 deliberately avoids a raw-state export or wire format. Persisted serialization, if needed, requires a separate compatibility/versioning decision after M4.

### D23 — Add canonical interchange after the API freeze

After M4, the missing product capability is cross-boundary sketch transfer.

Reason:

- if both full source sets are already co-resident, exact comparison is often available;
- a compact mergeable sketch is most useful across process, host, storage or time boundaries;
- therefore persistence is justified by the core use case rather than by serialization convenience.

M5 uses a dependency-free canonical v1 envelope with:

- explicit version, backend and u64-set domain tags;
- little-endian fixed-width integers;
- exact payload lengths;
- CRC32C accidental-corruption detection;
- self-contained backend configuration;
- primary state only, with derived caches recomputed on decode.

Energy serializes its exact hash coefficients because dimensions alone are insufficient to continue or compare the sketch. Parity serializes seed plus shape because that deterministically reconstructs its pseudo-oracle.

A serialized Energy profile marker cannot self-certify the theorem's randomness assumption. The ordinary decoder rejects Proven snapshots; restoring Proven coverage requires an explicitly named decoder whose caller accepts the uniform-row provenance precondition.

CRC32C is not authentication. No crypto, serde, compression, file or network abstraction is added.

## Immediate implementation sequence

```text
M0 research bootstrap
-> M1 EnergyDeltaMeter
-> M2 published F-PCSA reproduction
-> experimental ParityDeltaMeter
-> post-M3 strict-Parity stop gate
-> performance/API freeze
-> canonical sketch interchange v1
-> optional ParityLevelCounts / exact-small-d research
```

Strict Parity theory is no longer a blocker for beginning the useful Rust crate.

The post-M3 Gemini/Qwen/DeepSeek round strengthens the NO-GO decision for the published W_i strict track. Exact Poissonized and truncation results are retained as research artifacts; alternative backends require separate audits.


## M6 sequencing decision (2026-10-06)

Choose the bounded estimator-assisted value experiment before adopting ExactSmallDelta/hybrid. This does not establish economic superiority. Snapshot-plus-full-transfer is an overhead control; a useful decision policy and exact completion oracle are required. Keep the strict published-W_i NO-GO and all M5 compatibility/provenance rules.

The [M6 critical audit/design](../M6-RECONCILIATION-AND-OPTIMIZATION.md) and parent #14 supersede stale post-M3 next-step text only; historical research findings are unchanged. M6-0 (#15) begins with snapshot compatibility/evidence. After M6-0, M6-A and the read-only/source-audit part of M6-D may proceed in parallel; production ExactSmallDelta/hybrid adoption remains gated on M6-A product evidence. M6-B/C/D are tracked in #17–#19.


### D24 — Generic snapshot-admission workflow is NO-GO

M6-A tested a bounded two-process estimator-assisted admission workflow with an exact completion oracle.

The measured 8192-element deterministic matrix produced:

~~~text
direct exact total bytes       = 722,356
snapshot control total bytes   = 1,134,900
snapshot admission total bytes = 937,896
~~~

The snapshot-admission arm was therefore +29.838% application bytes versus direct exact for the fixed matrix.

For equal-size 8192-element sets:

~~~text
direct exact exchange = 65,668 bytes
snapshot-only reject  = 37,504 bytes
~~~

so byte-only break-even requires a reject fraction above 57.111%. The measured matrix rejected 3/11 comparisons.

Cold one-shot use is also NO-GO: constructing the custom Energy state on each side costs roughly two orders of magnitude more local CPU time than the in-memory exact-transfer control.

This does not prove that estimator-assisted admission is universally useless. It preserves a conditional future case for already-maintained sketches and a named reject-heavy or downstream-expensive workload. No such product workload is established today.

Therefore:

- do not add a public network/admission API;
- do not call an estimator path exact reconciliation;
- continue the independent exact-lane audit;
- prioritize measured Energy update/build optimization;
- keep any future admission policy workload-specific and explicit about provenance/error budgets.

Canonical evidence: docs/M6-A-EVIDENCE.md.


### D25 — Accept bit-equivalent Energy sign-mask caching

M6-C replaces the cubic sign hash's per-row field multiplications with an exact GF(2)-linear functional cache.

For fixed c:

~~~text
mask(c)[i] = L(c * x^i)
L(c*y) = parity(mask(c) & y)
~~~

where L extracts bit zero in the existing polynomial basis.

The cubic sign remains exactly:

~~~text
L(c0)
xor parity(mask(c1) & x)
xor parity(mask(c2) & x^2)
xor parity(mask(c3) & x^3)
~~~

x^2 and x^3 are computed once per key. Bucket hashing is unchanged.

The accepted mask builder walks c*x^i with the existing multiply-by-x reduction recurrence rather than performing 64 full gf64_mul calls per mask.

Paired hosted evidence on the final measured implementation shows roughly 68–71% lower Energy update latency. Construction/decode overhead amortizes after about 1–1.4 updates. Difference/query controls show no material regression.

The cache is private derived meter state (24R bytes), is never serialized, and is cloned/reused by difference.

No coefficient, field polynomial, profile, randomness/provenance, public API, snapshot-v1 byte or Coverage contract changes.

Canonical evidence: docs/M6-C-EVIDENCE.md.


### D26 — ExactSmallDelta Phase 1 selects a private PinSketch64 lab reference

M6-D audited PinSketch/Minisketch, Simple Set Sketching, classical and Rateless IBLT, and the 2025–2026 CertainSync, Self-Sizing IBLT and XYZ-Sketch frontier.

The production decision is **NO-GO** in this slice.

Reasons:

- M6-A already rejects a generic public estimator-assisted reconciliation workflow;
- direct libminisketch adoption requires C/C++ FFI and an unsafe/toolchain boundary that the crate currently forbids;
- Minisketch can return a plausible wrong decode when the true difference exceeds the admitted bound;
- successful sketch decode is not independent final verification;
- probabilistic peel constructions do not automatically satisfy an exact-result contract;
- the newest 2026 constructions are too fresh to treat as mature dependencies without reproduction.

The strongest lab candidate is PinSketch/Minisketch because recovery is deterministic when the actual difference count is within the configured capacity.

For DeltaMeter's exact u64 set domain, a collision-free PinSketch64 mapping is:

~~~text
key != 0 -> identical GF(2^64) field element
key == 0 -> one separate XOR-composable presence bit
~~~

This avoids truncation and avoids hashing collisions.

The next permitted implementation is a private pure-Rust reference with small fixed capacities, exact-oracle verification, over-capacity adversarial inventory and no public API/snapshot integration.

Do not add Coverage::Exact. Decoder false-success, identifier mapping, final verification and any Energy estimate failure remain separate failure-budget terms.

Canonical audit: docs/M6-D-EXACT-LANE-AUDIT.md.


### D27 — Retain guarded PinSketch64 only as a private reference

M6-D1 independently implemented the PinSketch/Minisketch odd-syndrome path in pure stable Rust for small audited limits.

The reference uses exact nonzero u64 -> GF(2^64) identity mapping and one out-of-band XOR-composable zero bit. No hashing/truncation is used.

The first design used `stored_capacity == max_elements`. Hosted over-capacity inventory found **306 false successful decodes in 896 deterministic cases**. That design is rejected.

The corrected lab design separates the two quantities:

~~~text
stored_capacity = max_elements + 1
decode_limit    = max_elements
~~~

The extra syndrome is used as an algebraic consistency guard after candidate recovery.

The final hosted inventory recorded **0 false successes in 896 guarded deterministic cases**, including weights beyond the simple BCH minimum-distance exclusion region. This is empirical evidence only; DeltaMeter does not adopt a 2^-64 adversarial theorem from that observation.

For max_elements 1/2/4/8 the guarded lab envelope is 32/40/56/88 bytes. On the hosted EPYC 7763 reference decoder, full-capacity decode ranges from ~8.9 us at d=1 to ~15.36 ms at d=8. Cold construction for an 8192-key source ranges from ~1.62 ms to ~5.47 ms.

Therefore:

- keep the implementation private and lab-only;
- require explicit decode limits in every call;
- reject the unguarded configuration;
- do not add Coverage::Exact, snapshot-v1 integration or a public ExactSmallDelta API;
- do not claim the guarded empirical result is independent final verification;
- prefer the next experiment to test nested guarded syndrome prefixes rather than Energy-first sizing.

Canonical evidence: docs/M6-D1-PINSKETCH64-EVIDENCE.md.


### D28 — Guarded incremental PinSketch64 prefixes are a lab GO, not a production protocol

M6-D2 evaluates the D1 odd-syndrome family as a nested progressive stream with the frozen stage schedule:

~~~text
decode limit        1   2   4   8
stored syndromes    2   3   5   9
~~~

The receiver starts with the smallest guarded prefix and receives only the new suffix words after a failed attempt.

Hosted evidence on the measured implementation records:

- all d <= 8 workloads recover the exact oracle;
- sampled d > 8 workloads reject;
- false-success count in the frozen matrix is zero;
- incremental cumulative syndrome bytes equal the ideal fixed-known-k payload;
- retry savings versus naive full-prefix resend are about 40–53%;
- repeated reference decode CPU is the dominant retry tax.

For d=8, the maintained-state experiment uses 73 application bytes but about 89.6 ms cumulative reference decode CPU versus about 38.7 ms for one-shot fixed guarded decode. The direct raw-u64 baseline is ~65.5 KiB and ~8.7 us local merge-scan CPU.

Therefore the primitive is useful only in named maintained-state bandwidth/RTT regimes; it does not justify a generic public reconciliation API.

Rateless IBLT remains the external unknown-d comparator. Its ~1.35d headline is a coded-symbol count, not directly a byte count; the pinned official implementation's coded symbol carries Symbol + u64 Hash + int64 Count.

Decision:

- retain nested guarded prefixes as a private lab primitive;
- keep the D1 exact-u64 mapping and one-guard research semantics;
- do not reintroduce Energy-first sizing for this small-d path;
- do not add snapshot-v1/public API/Coverage::Exact;
- next optimize cumulative retry CPU by decoder-state reuse without changing the D2 byte/RTT protocol.

Canonical evidence: docs/M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md.


### D29 — Incremental Berlekamp–Massey reuse is not the decoder bottleneck

M6-D3 freezes the D2 communication protocol and continues Berlekamp–Massey state across guarded prefix growth instead of recomputing it from the first syndrome after every retry.

The incremental state is algebraically equivalent to fresh BM at every audited stage. Locator polynomials, D1 candidates and the exact oracle agree for the complete d<=8 matrix; d=9/10/16 remains rejected.

Hosted phase decomposition shows that this correctness-preserving reuse does not materially reduce end-to-end decoder CPU:

~~~text
d=2  -0.279%
d=3  +0.113%
d=4  +0.519%
d=5  +0.505%
d=8  +0.566%
~~~

At d=8, incremental BM itself is about 74 us, roughly 0.07% of ~106.9 ms candidate decoder CPU. Root factorization plus candidate verification consumes essentially all remaining time.

Fresh even-syndrome reconstruction is only about 1.3–1.4 us at the largest audited stages, so a standalone even-syndrome cache cannot materially change the result and is skipped.

Decision:

- retain incremental BM only as correctness/diagnostic evidence;
- reject Candidate A as a performance optimization;
- skip the proposed even-syndrome cache as independently negligible;
- keep D2 bytes, RTTs, guard semantics and exact-u64 mapping frozen;
- move the next experiment to safe-Rust root factorization, beginning with characteristic-2 polynomial squaring and monic reduction inside the trace loop;
- do not add unsafe CLMUL/SIMD, randomized splitting or multiple root-finding changes in the same slice.

Canonical evidence: docs/M6-D3-INCREMENTAL-BM-EVIDENCE.md.


### D30 — Characteristic-2 trace squaring materially reduces private decoder CPU

**ACCEPT D4 Candidate A for private research; production/public ExactSmallDelta remains NO-GO.**

D4 preserves the D2 1/2/4/8 protocol, 17/25/41/73 bytes, one guard syndrome,
exact u64/zero semantics and deterministic root splitting. Only Frobenius square/mod
uses coefficient squares and monic reduction; verification remains in both arms.

Canonical implementation `cf6434d4007825b8fc05ebc9b064ce4860d4442b`, hosted run
37578505762: d=8 complete decoder 48.328 -> 6.575 ms, paired reduction 86.384%
(three per-run medians 86.263–86.513%). d=2..5 reductions are 88.2–93.6%.
Correctness and all strict CI gates pass; false-success is zero in the specified
repeated deterministic matrix, an empirical result rather than a guard theorem.

Root+verify still accounts for ~98.7% of the d=8 phase sum. The next single candidate
is deterministic safe-Rust quadratic specialization, gated first on degree-specific
factor and verification profiling. Do not start BM/even-syndrome caching, field
multiplication or SIMD work. If the quadratic share is immaterial, stop and evaluate
the maintained-state system end to end.

Canonical evidence: [M6-D4 trace square](../M6-D4-TRACE-SQUARE-EVIDENCE.md), including
raw runs, source hashes, exact-head CI and measurement limitations. D1/D2/D3 files,
public API, snapshot-v1 and Coverage are unchanged.


### D31 — Degree-two root work is material enough for one isolated solver experiment

M6-D5 profiles the accepted D4 root path without changing its factorization algorithm.

The profiler assigns non-overlapping local self time to recursive factor frames by
current polynomial degree, measures candidate verification separately, and keeps the
accepted D4 decoder as an uninstrumented control.

Canonical hosted evidence on AMD EPYC 9V45, Rust 1.99.0:

~~~text
d=8 accepted D4 control       ~7.137 ms
d=8 factor wall               ~7.125 ms
d=8 degree-two self work      ~1.964 ms
d=8 degree >=3 self work      ~5.130 ms
d=8 verification              ~0.0039 ms
~~~

Degree-two work is therefore about 27.6% of factor wall and 27.5% of accepted D4
control latency at d=8. It is also material at d=2/3/4. Verification is ~0.055%
at d=8 and is not the next optimization target.

Decision:

- D5 remains profile-only;
- authorize one separate deterministic safe-Rust quadratic solver experiment;
- keep all degree >=3 splitting, D4 trace-square, D2 bytes/RTTs and guard semantics frozen;
- do not expect another order-of-magnitude gain: the measured d=8 removable upper
  bound is roughly one quarter of current latency;
- reject the quadratic candidate if paired hosted evidence does not produce a
  reproducible material end-to-end reduction;
- do not bundle field multiplication, SIMD/CLMUL, randomized splitting or protocol changes.

Canonical evidence: docs/M6-D5-ROOT-PROFILE-EVIDENCE.md.


### D32 — Deterministic quadratic specialization is accepted, but its gain is factor-tree dependent

M6-D6 replaces only degree-two factorization in the accepted D4 root path.

The solver rewrites a monic quadratic x^2 + a*x + b into the Artin-Schreier equation
y^2 + y = b/a^2 and solves the fixed GF(2)-linear map deterministically in safe Rust.
Degree >=3 factorization remains the accepted D4 deterministic trace split.

Correctness agrees with the frozen D1/D4 controls, the exact oracle and the existing
failure/guard semantics across d=0/1/2/3/4/5/8 and rejected d=9/10/16 workloads.

Multi-corpus hosted evidence is required because the amount of degree-two work
depends on the deterministic factor tree.

Canonical evidence records:

~~~text
d=2 aggregate reduction  ~39.6%
d=4 aggregate reduction  ~37.5%
d=8 aggregate reduction  ~15.3%

d=8 corpus medians:
  ~3.7%
  ~27.1%
  ~15.3%
~~~

The matching D5-style corpus realizes ~27.1%, close to D5's ~27.5% measured
degree-two opportunity, validating the profiling gate while proving that the
opportunity is not uniform across corpora.

The per-solve GF(2) Gaussian implementation costs roughly 10.2 us and is not the
next bottleneck for the larger maintained-state workloads.

Decision:

- accept the deterministic quadratic solver for the private research decoder;
- do not claim a universal fixed speedup;
- keep degree >=3 D4 trace splitting, D2 bytes/RTTs, field representation and guard
  semantics frozen;
- do not optimize Artin-Schreier matrix setup yet;
- profile the post-D6 residual factor work across multiple corpora before selecting
  another root specialization;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D6-QUADRATIC-EVIDENCE.md.
