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


### D33 — Residual degree dominance is unstable; optimize the common trace path instead

M6-D7 profiles the accepted D6 decoder across five deterministic corpora without
changing the factorization algorithm.

After degree-two specialization, no single higher degree remains a stable global
bottleneck.

For exact d=8 workloads:

~~~text
degree 3 share: median ~28.5%, range ~5.7-71.5%
degree 4 share: median ~29.2%, range ~13.6-82.3%
degree 5 share: median ~8.2%, range ~6.8-23.4%
degree 8 share: median ~3.8%, range ~3.1-10.7%
~~~

The dominant degree flips between degree 3 and degree 4 depending on the factor tree.

However, summing the common degree>=3 trace phase shows a stable cross-corpus
bottleneck. For d=8, trace self time is roughly 63-83% of accepted D6 control
latency across all five corpora. GCD is secondary and more variable; successful
division and candidate verification are small.

Decision:

- do not start an isolated cubic solver;
- do not start an isolated quartic solver;
- do not optimize D6 Artin-Schreier setup;
- keep D6 quadratic specialization and all D2 protocol semantics frozen;
- authorize one general safe-Rust trace-path experiment, beginning with a
  bit-equivalent GF(2^64) scalar-squaring specialization only;
- do not bundle generic multiplication, GCD/division, SIMD/CLMUL, unsafe or field
  representation changes in that experiment.

Canonical evidence: docs/M6-D7-RESIDUAL-ROOT-PROFILE-EVIDENCE.md.


### D34 — Dedicated scalar GF(2^64) squaring is accepted for the private research decoder

M6-D8 tests the shared trace-path primitive selected by D7.

The candidate replaces only `gf64_mul(x, x)` used for coefficient squaring inside
degree>=3 trace square/mod with a safe-Rust carryless bit-spread square followed by
reduction in the frozen polynomial basis. D6 quadratic roots, generic field
multiplication, GCD/division, protocol bytes/RTTs and verification remain unchanged.

A quantitative gate was frozen before the first performance run: every d=8 corpus
must improve by at least 5%, every process median must stay positive, aggregate d=8
must improve by at least 5%, and d=3/4/5 may not regress by more than 2%.

Canonical hosted evidence at the accepted implementation head reports:

~~~text
d=8 D4     13.043%
d=8 D5     14.213%
d=8 D6     16.984%
d=8 D7a    13.144%
d=8 D7b    14.940%
aggregate  14.263%
~~~

Correctness matches the scalar reference on all 64 basis vectors plus deterministic
full-width vectors, and the complete decoder matches accepted D6 across all stages
and corpora. d=9/10/16 remain reject; false-success total is zero in the repeated
deterministic matrix.

Decision:

- accept D8 scalar squaring for private research;
- do not claim a production performance promise;
- do not change the field representation or generic multiplication;
- do not start cubic/quartic specialization;
- re-profile residual CPU after D8 before authorizing anything else;
- if no remaining common decoder phase is materially large, stop algebraic
  micro-optimization and proceed to maintained-state system comparison;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D8-GF64-SQUARE-EVIDENCE.md.


### D35 — Trace remains the only common material bottleneck after D8

M6-D9 re-profiles the accepted D8 decoder without changing its algorithm.

The decision threshold was frozen before measurement: a phase must account for at
least 25% of accepted D8 d=8 latency in every one of the five deterministic corpora
to remain a common material bottleneck. Profiler validity additionally requires at
most +5% median d=8 wall overhead per corpus.

Hosted evidence passes the validity gate with only ~0.5-1.0% profiler overhead.

For d=8:

~~~text
trace         median 62.208%, range 57.545-80.590%
GCD           median 35.486%, range  8.088-39.307%
division      median  0.820%, range  0.554- 2.773%
quadratic     median  0.772%, range  0.632- 3.003%
verification  median  0.088%, range  0.079- 0.358%
~~~

Only trace clears the common-material gate.

Trace operation counts show 2.5-3.1 generic reduction multiplications per
coefficient-square operation across the five d=8 corpora. This is not timing
attribution.

Decision:

- keep D9 profiling-only;
- authorize one narrower trace-internal measurement slice;
- do not preselect generic multiplication, reduction, GCD, cubic/quartic roots,
  SIMD/CLMUL, tables or caches;
- if the narrower measurement finds no independently material trace component, stop
  algebraic decoder micro-optimization and move to maintained-state system comparison;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D9-POST-D8-PROFILE-EVIDENCE.md.


### D36 — Polynomial reduction is the stable post-D8 trace-internal bottleneck

M6-D10 uses offline replay of exact operands collected from accepted D8
factorization. No per-operation timers are inserted into the decoder.

The replay decomposition validity interval was frozen at 70-130%. Observed additive
closure is 97.829-99.387% across the five d=8 corpora.

Full square/mod consumes 99.753-99.931% of full trace replay. Its internal
decomposition is:

~~~text
modulus preparation   1.929-2.842%
square-build          5.043-6.719%
net reduction        88.299-92.428%
trace accumulation    0.537-0.708% of full trace
~~~

The predeclared selector required net polynomial reduction to exceed 50% of
square/mod in every corpus. It clears that threshold by a wide margin.

Decision:

- keep D10 measurement-only;
- authorize exactly one safe-Rust bit-equivalent polynomial-reduction experiment;
- preserve D8 scalar square, factor tree, D6 quadratic solver, GCD/division, field
  representation, protocol and verification;
- do not broaden this into a generic GF(2^64) rewrite;
- if the isolated reduction experiment fails the end-to-end gate, stop algebraic
  decoder micro-optimization and move to maintained-state system comparison;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D10-TRACE-INTERNAL-EVIDENCE.md.


### D37 — M6-B keeps B2/B3 separate; direct Energy decode is the next performance experiment

The M6-B deep-research audit reconciles two external reports with the actual
snapshot-v1 implementation and B1 hosted evidence.

Accepted corrections:

- the current decoder does not allocate arbitrary Energy primary state from a short
  malformed payload; exact counter bytes are checked before meter construction;
- B2 is exact-shape/resource hardening, not a newly discovered critical DoS fix;
- B2 and B3 remain separate so performance attribution is preserved;
- B3 is widened to direct decoded-state construction: capacity-only counters and
  one-pass checked row-energy accumulation instead of zero-initialize, overwrite and
  rescan;
- the default Energy B3 logical opportunity is removal of 753,664 bytes of
  destination-counter zero-store/post-read touches for `B=2048,R=23`; this is not
  a physical memory-bus claim;
- the external "~10.7% residual CRC" and "33-40% memory traffic" values are not
  adopted as exact evidence because they depend on assumed standalone rates and
  cache/allocator behavior;
- slicing-by-4/8 and encode CRC fusion are deferred until residual hosted
  attribution selects them;
- decode CRC fusion remains NO-GO for current v1;
- no new snapshot-core wire-size cap is introduced.

Correct benchmark fixtures are `B=512,R=9` for Energy-small (37,340 bytes) and
`rows=17,levels=13` for the 76-byte padded Parity case.

Canonical audit: docs/M6-B-CODEC-DECODER-RESEARCH-AUDIT.md.  
Execution plan: docs/superpowers/plans/2026-10-07-m6b-post-research.md.


### D38 — Fixed-constant trace reduction is accepted for private research

M6-D11 follows D10's measured reduction selector and changes only multiplication
inside trace polynomial reduction.

A factor-frame-local plan represents each fixed modulus coefficient with 16
positional nibble tables. Generic multiplication outside trace reduction, D8 scalar
squaring, D6 quadratic roots, GCD/division, field representation, protocol and
verification remain unchanged.

The performance threshold was frozen before measurement:

~~~text
every d=8 corpus >= 10%
aggregate d=8 >= 15%
every d=8 process median > 0%
d=3/4/5 corpus medians >= -2%
~~~

Canonical hosted evidence reports:

~~~text
D4 d=8    51.614%
D5 d=8    45.916%
D6 d=8    66.087%
D7a d=8   51.228%
D7b d=8   47.342%
aggregate 51.166%
~~~

d=3 improves ~12.7-12.8%, d=4 ~48.8-50.3% and d=5 ~37.7-46.2%.
Correctness remains bit-equivalent/end-to-end exact on the frozen matrix.

Decision:

- accept the fixed-constant reduction plan for the private research decoder;
- explicitly drop parent-frame plans before recursive factorization;
- retain the table-memory trade-off as private research only;
- do not generalize this into a global GF(2^64) multiplier or public state;
- re-profile accepted D11 before choosing another algebraic optimization;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D11-FIXED-REDUCTION-EVIDENCE.md.


### D39 — B2 exact Energy shape preflight is accepted as resource hardening

After B1's table-driven CRC32C was accepted and merged, M6-B2 moved full Energy
backend-shape validation ahead of row allocation.

The candidate preserves the checksum-before-backend boundary and the public
`ProvenanceRequired` precedence, reuses existing Energy dimension rules and adds no
snapshot-core transport cap.

Valid-CRC malformed fixtures now exercise backend validation directly. Parity code
requires no structural rewrite because it already checks config-derived packed-state
bytes before allocating its state vector.

Canonical hosted measurement at implementation head
`dfa859508f3be81e5e5c7a7d9683aa2aa6138e63` reports valid Energy decode movement
between -0.097% and +0.190%, i.e. noise-scale. A dedicated 229,364-byte valid-CRC
shape-rejection fixture improves by 3.544%.

Decision:

- ACCEPT B2 for stronger allocation/resource invariants and test coverage;
- do not claim a newly fixed arbitrary-primary-allocation vulnerability;
- do not claim a valid-decode speedup;
- keep outer transport admission separate;
- proceed next to one isolated B3 direct Energy decoded-state construction
  experiment;
- keep further CRC slicing/fusion and sign-mask fusion deferred.

Canonical evidence: docs/M6-B2-STRUCTURAL-VALIDATION-EVIDENCE.md.


### D40 — Replicated post-D11 residuals stop algebraic specialization

M6-D12 profiles the complete accepted D11 staged decoder rather than assuming that
the prior trace bottleneck survived a ~50% end-to-end reduction.

The original selector required a phase to account for at least 25% of accepted D11
d=8 latency in every one of five deterministic corpora. Exact-head reruns placed the
D6-derived GCD share on both sides of that threshold, so neither outcome was selected
post hoc.

Before collecting more data, D12 froze a replication closure: five independent
GitHub-hosted workers, three profiler processes per worker, four balanced samples per
process, and paired phase/control shares. The 25% threshold itself did not move.

All workers pass logical/correctness gates and the +5% profiler-overhead ceiling.

Across the 25 worker/corpus medians:

~~~text
prefix        0.017- 0.092%
locator       3.422-17.951%
validation    0.003- 0.017%
plan build    0.363- 2.716%
trace        18.034-36.786%
GCD          23.574-74.713%
division      1.120- 8.434%
quadratic     1.367- 9.453%
verification  0.161- 1.113%
~~~

The D6-derived GCD worker medians are 23.657%, 24.750%, 24.575%, 23.574% and
23.583%. Therefore GCD is not a replication-stable common-material phase.

Decision:

- stop algebraic decoder micro-optimization after accepted D11;
- do not start GCD, trace, cubic/quartic, generic field-multiply, D11 table-layout,
  SIMD/CLMUL or factor-cache work without new system-level evidence;
- move M6-D to a maintained-state system comparison of accepted private D11
  PinSketch, direct exact reconciliation and a rateless-style comparator;
- include maintained state cost, memory, bytes, retries/RTTs, decode CPU and final
  verification in that comparison;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D12-POST-D11-PROFILE-EVIDENCE.md.


### D41 — Direct one-pass Energy snapshot decode is NO-GO

M6-B3 evaluates the direct decoded-state construction selected after B2.

The candidate parses counters into capacity-only storage and accumulates row energy
during parsing, eliminating one explicit counter zero-fill and the later
`recompute_energies` counter scan from the source-level path.

The predeclared gate required at least 3% improvement in every Energy-default decode
lane. Hosted paired evidence instead reports:

~~~text
default: +2.452% .. +2.682% regression
small:   +2.912% .. +3.237% regression
~~~

Unchanged Parity decode guardrails remain near neutral, so the consistent Energy-only
regression is attributed to the candidate rather than a single runner outlier.

Decision:

- B3 is NO-GO and its production code is reverted;
- do not infer latency from the source-level logical-touch reduction;
- do not substitute an unplanned two-pass/fusion variant after observing the result;
- M6-B #17 is complete with B1 ACCEPT / B2 ACCEPT / B3 NO-GO;
- skip the optional residual CRC selector for this milestone;
- require a new measured bottleneck before any future slicing-by-N, CRC fusion,
  hardware specialization or decoded-state candidate.

Canonical evidence: docs/M6-B3-DIRECT-DECODE-EVIDENCE.md.


### D42 — D13-A protocol v2 is the system-comparison foundation

D13-A corrects the system contract before performance measurement.

Decision:

- a validated full target-list transfer is terminal for direct exact and exact
  fallback; it is not sent back a second time merely to imitate sketch verification;
- provisional D11 candidates require the independent reverse-list verification
  boundary;
- pinned RIBLT `pull_pow2` is retained as a concrete transport comparator but
  cannot by itself reject the rateless architecture;
- D13-B performance remains blocked on native CPU/memory accounting, persistent
  multi-session state, a streaming rateless comparator and larger scaling evidence;
- no public/production ExactSmallDelta decision follows.

Canonical protocol: docs/M6-D13A-SYSTEM-PROTOCOL.md.


### D43 — D13-B0 is measurement-ready; staged guard false positives are charged

D13-B0 establishes a native measurement substrate for the system comparison.

Hosted readiness at measurement-source head
`de0c57531c0a186e160f8c4617a1c7878181dc56` passes frozen-source validation,
1,776 persistent-chain rows and a 14-row sparse one-million-key scaling inventory.
No performance winner is measured in B0.

B0 also finds a deterministic staged D11 witness with true `d=8`: at `k=1`,
maintained and fresh sketches produce the same provisional but incorrect candidate.
The independent verifier rejects it and protocol-v2 exact fallback completes
correctly.

Decision:

- classify the witness as an early-stage over-capacity guard false candidate, not
  maintained-state corruption or a sufficient-capacity k=8 decoder failure;
- preserve the existing statement that the one-extra-syndrome guard is empirical,
  not a theorem;
- record and charge `false_candidate` separately in D13-B;
- allow d<=8 fallback only when fresh rebuild reproduces the same earlier-stage
  provisional candidate and independent verification rejects it;
- any other unexplained d<=8 fallback remains INVALID;
- freeze the measurement sources before D13-B;
- authorize only the predeclared hosted system comparison after final exact-head CI;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D13B0-MEASUREMENT-READINESS-EVIDENCE.md.


### D44 — M6-D13-B stops the current reconciliation-backend product lane

D13-B measures the complete maintained-state system contract after D13-B0 declared
the measurement substrate ready.

The valid source head is
`3161119129bb033c39ad5c403ca46de14b7664b9`. Hosted run
`37651049535` contributes five independent workers, three replicate families per
worker, 3,900 raw observations and 360 named
`(N,U,sessions/build,RTT,bandwidth)` cells.

The frozen gate requires at least 10% modeled total-cost reduction versus direct
exact on every worker for the same named cell.

Result:

~~~text
D11 qualifying cells                 0 / 360
RIBLT stream-lower-bound cells       0 / 360

best D11 cell:
  N=65,536 U=1 S=100 RTT=0 bandwidth=1 Mbps
  worst-worker improvement          -0.0424%
  median-worker improvement         -0.0356%

best optimistic stream_lb cell:
  same named cell
  worst-worker improvement          -0.4352%
  median-worker improvement         -0.2978%
~~~

The result is explained by the exact-verification contract, not by a single noisy
decoder benchmark. Direct sends the canonical target list once. A successful
sketch/rateless candidate still sends a full canonical list for independent
verification, in addition to sketch/coded-symbol bytes and extra feedback RTTs.

The hosted boundary confirms that cost directly. At N=65,536,d=8, direct uses
524,392 application bytes and one RTT; D11 uses 524,849 bytes and five RTTs. D11
native compute is also ~2.889 ms versus ~0.363 ms direct. At d=9 D11 safely falls
back in 15/15 observations but remains more expensive.

At N=1,048,576 direct native synchronization is ~3.2-3.3 ms on the sparse scaling
guard. D11 is ~9.7-10.4 ms with expected exact fallback. The pinned optimistic
RIBLT stream lane is ~3.34-5.33 s and about 260+ MiB median HWM because the upstream
lifecycle reimports retained source state each session.

Decision:

- record **STOP_SYSTEM_PRODUCT** for the current M6 reconciliation-backend program;
- do not productionize D11/ExactSmallDelta from M6;
- do not add a public reconciliation/network API from this evidence;
- do not pivot M6 to the pinned RIBLT implementation;
- keep D12 STOP_ALGEBRAIC_MICRO_OPT binding;
- retain D11 and accepted D1-D11 optimizations as private research/reference code;
- direct exact remains the system baseline under the current exact-verification
  contract;
- a compact probabilistic/digest verification tier or genuinely maintained rateless
  implementation would be a materially new protocol and requires a new program;
- production/public ExactSmallDelta remains NO-GO.

Canonical evidence: docs/M6-D13B-SYSTEM-EVIDENCE.md.


### D45 — STRICT-COMPACT OPT-A selects a 28 KiB full-range research baseline

OPT-A varies only the stored level count J and physical packing while preserving
the accepted m=4096, delta=1e-6, delta/64 Q32 thresholds and keyed-BLAKE3 oracle
contract.

The exact continuous range frontier shows:

~~~text
J=24  12 KiB  d_max=84,315,082,377
J=32  16 KiB  d_max=21,584,661,088,732
J=40  20 KiB  d_max=5,525,673,238,715,561
J=48  24 KiB  d_max=1,414,572,349,111,183,676
J=56  28 KiB  d_max=2^64-1
J=64  32 KiB  d_max=2^64-1
~~~

J=64's accepted certificate uses no certifying level above 52, so levels 57..64
are unnecessary for the current full-domain q95 width contract.

Physical-layout screening finds keyed-BLAKE3 update cost essentially unchanged
between row-major and level-major layouts, while level-major level-count extraction
is much faster and makes each level one contiguous 512-byte chunk.

Canonical Stage-2 run `37756954797` at source
`cfa7e48a4784146573073c6992ed9536f24b1d24` contributes five independent hosted
workers and 160 raw measurements. Every candidate passes the predeclared viability
limits.

For J=56 versus J=64, worst-worker regressions are only:

~~~text
update     +0.105%
estimate   +3.670%
XOR/KiB    +2.648%
~~~

Decision:

- record **RANGE_FRONTIER_PASS**;
- use J=56 / 28 KiB / LEVEL_MAJOR as the preferred full-u64 research baseline for
  OPT-B/OPT-C;
- retain J=64 only as a regression/control profile;
- retain the exact J frontier as range-specific profile evidence;
- do not select a public default yet;
- keep #69 public profile/snapshot freeze blocked on OPT-B/OPT-C;
- no alpha reallocation, generalized ladder or new oracle is authorized by OPT-A.

Canonical evidence: docs/research/STRICT-COMPACT-OPT-A-EVIDENCE.md.


### D46 — OPT-A refinement confirms J=52 / 26 KiB full-range research profile

The separately preregistered post-Stage-2 J51/J52 refinement preserves D45's
historical J matrix and five-worker J56 result. Exact hosted certification
(run 37801885016, source a0ba405aaa045db4490cde28b248a0739eb09f9c)
reproduces [4096,2^64-1] for J52 using the unchanged certified Q32 table,
alpha=delta/64 and keyed-BLAKE3 computational input assumptions. The packed
J52 LEVEL_MAJOR bitmap is 26,624 bytes, saving 18.75% relative to J64 and
7.14% relative to J56, and passes all five paired performance workers.

J51's current certificate stops at d=11,316,578,792,889,469,415, which does
NOT prove any impossibility/lower bound for alternate certificate designs.

Decision: **J52_REFINEMENT_PASS**; prefer J=52 as the full-u64 OPT-B/OPT-C
research baseline, retain J64 control and the J56 historical candidate;
no public default, snapshot, alpha or oracle changes. #69 remains blocked on
#73/#74 product-shape evidence.

Canonical addendum: docs/research/STRICT-COMPACT-OPT-A-REFINEMENT-EVIDENCE.md.


### D47 — OPT-B keeps the derived odd-count cache under five-worker gate

On preregistered source 2f8dc09199e376d3c6019ebb0d529d3879b15eeb,
GitHub-hosted five-worker run 37807795968 (1680 complete raw observations)
measures J52 / LEVEL_MAJOR maintained u16 counts against an **allocation-free**
bitmap scan control. All workers pass frozen update <=+10%, estimate >=5x
speedup, and >=10% end-to-end gain on one shared realistic mixed lane:
observed worst update +2.670%, minimum estimate 13.563x, minimum mixed-100
gain +15.22% (also mixed-1/mixed-10 pass).

Decision: **KEEP_CACHE** as derived internal research state (2*J bytes,
J52=104 B), always rebuild after XOR canonical merge, never serialize cache;
continue #74 progressive transfer. The public profile/snapshot gate #69 stays
blocked until #74 closes. Source-specific hosted timings are not API promises.

Evidence: docs/research/STRICT-COMPACT-OPT-B-EVIDENCE.md.


### D48 — OPT-C C2 transfer accounting, not public acceptance

Run 37812390251, source 3994b6e0e74ef51d4a50245a883c2e25939d9d55:
10,560 validated hosted observations. Full bitmap: 26,750 application
bytes. Naive push-all: 27,838 bytes. A T-informed one-level bounded
prefix: 655 bytes in named synthetic workload examples.

Verdict: C2_PHYSICAL_ACCOUNTING_PASS and MODELED_C3_CANDIDATE only.
Network latency is analytical, not observed; a partial parity state
does not replace full interchange. A simple scalar result is a
separate comparator for trusted threshold-only use cases.
No public API/snapshot change; #69 stays blocked until C3.

Evidence: docs/research/STRICT-COMPACT-OPT-C-C2-EVIDENCE.md.


### D49 — OPT-C C3-A loopback transport and XOR functional foundation

Issue #80, measured source `0968adec45d14261f950b3f8d6a9690c27407424`,
five hosted workers, run 37815243109. Complete source-attested results:
1,440 measured localhost TCP request/response trials, 360 two-party
level XOR/extension oracle observations. The loopback pace/RTT simulator
measures actual socket round trips under application sleeps, NOT WAN
network quality or kernel shaping. All five workers passed the frozen
foundation checks. The T-informed bounded prefix qualifies in the
restricted controlled 10Mbps RTT<=50ms matrix; interactive4 does not.

Threshold-only trusted sender U transfer is 24B, beating a 655B
one-level prefix. Independent two-party level XOR reuse is functionally
distinct, but has so far been tested as a **local oracle and logical
byte model**, not a real two-party TCP lifecycle.

Decision: **C3_A_TRANSPORT_FOUNDATION_PASS, no public progressive
approval**. C3-B must measure a full real two-party retained-state
lifecycle with rounds, cumulative application bytes, timing and
generation changes before #80/#74 can close. #69 remains blocked.
Evidence: docs/research/STRICT-COMPACT-OPT-C3A-EVIDENCE.md.

### D50 — request-first dual-TCP partial reuse fails frozen RTT gate

PR #82, source d0480354e372d1d81d0e03263cd375467b680031,
GitHub-hosted run 37816319786: five workers / 1,440 genuine
two-source TCP checkpoints PASS all correctness, provenance,
application byte and full-XOR reconstruction checks. One level
from each sender costs 1,344B, versus concurrent dual-full 53,500B,
but requires an extra feedback RTT. At 10Mbps, application RTT
0/10/50ms, the original no-p95-latency-regression gate qualifies
**0/3** scenarios on all workers. Full fallback 55,778B also costs
4.26% more bytes than dual-full. The strict product verdict is
**C3B_TWO_PARTY_NO_GO** for the request-first interactive transport.

Keep the data and gates unchanged. A new pre-registered sender-first
fixed T-centered level probe is a separate C3-C protocol hypothesis
that can avoid the initial feedback RTT. No public API, key or
snapshot-v1 freeze. Evidence:
docs/research/STRICT-COMPACT-OPT-C3B-EVIDENCE.md.


### D51 — sender-first two-party prefix passes narrow transport gate; no general public progressive API

PR #83 source 9db84aecd04a93ad72cd603453bb6970d1fb42fa,
GitHub-hosted five-worker run 37817271128 with 1,440 complete
two-source TCP checkpoints. Unlike D50's request-first protocol
(which remains NO-GO), each sender eagerly includes its fixed
T-centered first level after the initial T request, avoiding an extra
round trip. The unchanged p95 bytes / p95 time gate passes all
3 named d/T scenarios on all workers at controlled app-paced
10Mbps RTT=0/10/50ms.

Two senders' one-level bound costs 1,310B vs complete 53,500B.
Full retained fallback costs 55,744B (4.19% more than complete).
A trusted one-sender scalar result dominates threshold-only
workloads; two independent sender states need receiver-side XOR.
The result is scoped to a *single immutable compatible-source
threshold-bound application*, **not** WAN, authentication,
full-state compression, or unconditional statistical guarantees.

Decision: **C3C_CONTROLLED_TWO_PARTY_PASS / GENERAL_PUBLIC_PROGRESSIVE_NO_GO**.
Leave sender-first as optional future application-specific research;
move #69 forward with J52 LEVEL_MAJOR + derived cached counters,
while key provenance/authentication/snapshot lifecycle remain
explicit public acceptance conditions.

Evidence: docs/research/STRICT-COMPACT-OPT-C3C-EVIDENCE.md.

### D52 — DeltaGuard G0 actual k-level state and small-T utility boundary

Issue #84 / PR #87, measured source c9036ac68ab13656115a57c970049a696009ab7a,
hosted run 37876830842: 5/5 workers, exact source-attested 1,200
deterministic two-owner parity observations and checksum-pinned unchanged Q32
m4096/alpha=1e-6/64 passed all algebraic/selected-state invariants.
Actual per-owner bitmap payload k=1/2/4/8 is 512/1024/2048/4096B
plus separately costed owner, key and allocator. SAFE iff U_selected<=T,
with ideal-oracle one-sided guarantee inherited (NOT a new theorem).
Observed useful at d=.75T among 15 deterministic fixed-key ranges:
T64 0/15 any k; T4096 10/15 any k;
T65536 10/15 (k1), 14/15 (k2), 15/15 (k4/8);
T1048576 11/15 (k1), 14/15 (k2/4/8).
Zero false-safe in empirical samples does NOT demonstrate probability 1e-6
nor security for public deterministic test key or adaptive input.

Decision **G0_FOUNDATION_PASS_EMPIRICAL_POWER_CANDIDATE**, only for
existing fixed-m4096/known-Q32 research model.
**NO GUARD PRODUCT FREEZE**: small T shows poor utility; further
m/alpha/level optimization needs fresh exact finite-sample
certificates (issue #85), key adaptive-query model issue #86, and
system-cost comparator. Evidence:
docs/research/DELTAGUARD-G0-EVIDENCE.md.

### D53 — G1-A certified per-k alpha, but no general small-T product

Issue #85, DeltaGuard #84, research PR #88.
Frozen 1/2/4 selected-level prior allocation
`alpha_j=1/(1,000,000*k)` at unchanged m4096 and G0
physical two-owner XOR parity state.
Three regenerated Q32 lookup tables have independent
exact-rational/Taylor and integer-likelihood certificates
under original fixed-input ideal-oracle theorem. Control
delta/64 Q32 hash unchanged. 900 paired raw source-attested
observations from five GitHub-hosted workers, run
37877960458, plus certified table artifact.
At T4096,d=.75T, k1 SAFE improves 10/15 → 14/15;
k2/k4 11/15 → 14/15. T32 and T64 at d=.25T
remain 0/15 across k1/2/4; T256,d=.75T remains 0/15.
Exact monotone-table minimum endpoint is 59/61/64 for
k1/2/4, making T32 SAFE **impossible in THIS method**
under any state. This does not prove information-theoretic
impossibility of a different decision algorithm.

Verdict **G1A_CERT_PASS /
G1A_ALGEBRA_PASS_EMPIRICAL_UTILITY_CANDIDATE**;
**NO G1A SMALL-T PRODUCT / PUBLIC API FREEZE**.
A selected k/T must be fixed before observing input.
The preallocated delta/k guarantee is not an arbitrary
adaptive-T or repeated-profile union guarantee; keyed BLAKE3
assumption computational, independent of certified ideal tail.
No new theorem novelty claim. Next: G1-B exact fixed-d
lazy Ehrenfest/odd-occupancy CDF inversion and independent
finite T+1 safety, with conditional product power gate;
issue #86 security boundary. Evidence:
docs/research/DELTAGUARD-G1A-EVIDENCE.md.

### D54 — DeltaGuard G1-B1 exact lazy-Ehrenfest threshold and unit-coefficient pilot

Issue #89; research PR #90; 1,800-row GitHub-hosted
two-owner XOR matrix and 24 exact fixed-d cutoff certificates.
The G1-B1 ideal-oracle recurrence for m=4096 and a
predeclared single level j has integer weights:
`n_(d+1)[s]=(D-m)n_d[s]+(m-s+1)n_d[s-1]+(s+1)n_d[s+1]`,
D=m*2^(j+1) for original random coefficient, D=m*2^j
for a new separately domain-separated unit coefficient `g(v)=1`.
The stochastic kernel is monotone because
`(m+1)/D<=1`, so `P_d(S<=c)` is maximized for d>T
at d=T+1. Cutoffs accepted by an exact integer
`10^6 * CDF_numerator(T+1,c) <= D^(T+1)` comparison;
162 independent complete small-bitmask probability oracle cases.

For j=1, T=32: original ALWAYS_UNKNOWN (-1 cutoff),
unit accepts S<=3. At T64: original S<=1, unit S<=13.
At d=T/4 ideal one-shot power: T32 old=0, unit=0.364215;
T64 old=0.063661, unit=0.997956 (integer-truncated ppm).
At d=.75T unit power T32≈0.000145 and T64≈0.001189;
**no near-threshold high-power claim**.
Five GitHub-hosted workers, 1,800 source-pinned deterministic
two-owner records, correctly maintain 512B per owner and
fail closed on cross-profile/level XOR; sampled T32,d8
unit SAFE 7/15; T64,d16 unit SAFE 15/15. Published test
key is NOT random-secret Monte Carlo or real PRF security.

Verdict **G1B_EXACT_CHAIN_CERT_PASS** and
**G1B_B1_XOR_PASS_EMPIRICAL_UNIT_CANDIDATE**, in the original
strict NONADAPTIVE ideal-oracle model only.
**NO PUBLIC GUARD PRODUCT FREEZE; NO G1B-COMBINED LEVEL CERT**,
no snapshot identity/wire interop between original and unit.
Classical lazy Ehrenfest state/count calculus, NOT a new theorem.
Do not infer real-world network/CPU/RSS performance or repeated
adaptive query security. Next evaluate separately certified joint
levels or direct exact comparator; issue #86 key/adaptive threat.
Evidence docs/research/DELTAGUARD-G1B-EVIDENCE.md.

### D55 — G1-B2-A near-full single-level GF(2) guard is a strong scoped research candidate

Parent DeltaGuard #89, PR #91; preregistered
`docs/research/DELTAGUARD-G1B2A-PROTOCOL.md` before running experiments.
New profile uses m=2^b-1 physical parity rows and D=2^b
equiprobable keyed BLAKE3 slots, exactly one no-toggle dummy
slot. This profile is **key-domain-separated** and incompatible with
J52/G0 and G1B original/unit XOR states. It is NOT a new theorem:
lazy Ehrenfest chain and Odd Sketch XOR construction are prior art.

Independent arbitrary-precision integer certificate for each
T in {32,64,128}, b in {6..13}:
`n_(d+1)[s]=(D-m)n_d[s]+(m-s+1)n_d[s-1]+(s+1)n_d[s+1]`.
Since `(m+1)/D=1`, the chain is stochastically monotone
from zero, so `P_d(S<=c)` for any d>T is bounded by the
exact integer law at d=T+1. Cutoffs are accepted by
`1,000,000*sum_{s<=c}n_(T+1)[s]<=D^(T+1)`.
Twenty-four full profiles and 39 independent exact full-bitmask
oracle cases passed the hosted proof check. This is a
**single predeclared T,b,nonadaptive ideal-oracle** one-sided
1e-6 guarantee, NOT unconditional keyed-PRF/adaptive security.

Source `68b2e2c3d894f3716c79f41919d840449b3a774b`,
[run 37889355518](https://github.com/definitely-stable/deltameter/actions/runs/37889355518):
five GitHub-hosted two-owner physical XOR workers and fail-closed
aggregate passed **3,240** uniquely keyed deterministic source
records. Each owner holds a genuine single-level Box bitmap;
direct-keyed per-token bitmap reference and cross-b/T/key errors
passed. Physical payload 8..1024B, Rust owner object 64B
including derived-key 32B, plus allocator overhead.

Crucial **T64,d48,b11**: bitmap 256B/owner,
cutoff S<=48. Since S<=d pointwise, SAFE is **guaranteed**
for all inputs with true d<=48, and false SAFE remains
ideal-probability <=1e-6 for all d>64.
Named fixture 15/15 SAFE; T32,d24,b12 512B/owner
cutoff=24 also 15/15. Near boundary T64,d57,
both b11 and b12 samples returned 0/15 SAFE,
so **no near-threshold/high-precision product claim**.

Decision:
- **B2A_EXACT_CERT_PASS / B2A_PHYSICAL_3240_PASS_UTILITY_CANDIDATE**;
- rank B2A single-level above a complex correlated joint-level
  G1-B2 proof until a realistic two-owner cost/power comparison;
- **NO G1-B2-A PUBLIC API OR SNAPSHOT GO**; B2A remains
  a specialized optional research candidate;
- next preregister end-to-end source-maintenance, CPU, heap,
  network and RTT vs direct exact, trusted scalar,
  G1B1 and J52, including repeated sessions/fallback;
- #86 computational PRF/attacker/key model and #69 public
  product/key/snapshot lifecycle remain independent blockers.

Canonical evidence:
`docs/research/DELTAGUARD-G1B2A-EVIDENCE.md`.

### D56 — G1-B2-B0 retained-source guard-only byte model passes, resolved near-T stops

PR #93, issue #92. Protocol was frozen BEFORE first B0 runs,
`docs/research/DELTAGUARD-G1B2B0-PROTOCOL.md`.
A real Rust physical retained-state comparison uses two independently
maintained and actually sorted canonical u64 source lists for
N={256,4096,65536}, d={0,16,48,57,64,65},
sessions={1,10,100}, three fixture repetitions, B2A b11/b12,
G1B unit j1 and full J52 baseline. Every sample independently
rechecks exact `|A△B|` and XOR parity after source-maintained
incremental updates.

Five GitHub-hosted worker matrices and strict source-/certificate-
pinned aggregation passed **3,240/3,240** physical raw records;
first completed CI run 37896624499 source
`a7fa51c3f2e9da2da9f7b32ccb019e40d26cd839`.
Regenerated B2A proof certificate is byte-identical
(SHA256 `c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`).
Rust initial/update/XOR/query native wall-clock timings are recorded;
network application bytes and modeled RTT/bandwidth **are NOT TCP**.

At predeclared T64,d48, b11 c48 means the independently maintained
two-owner 256B-per-owner guard ALWAYS SAFE since S<=d<=c.
With 64B/model peer headers: modeled single-round **640B**
versus 4,368B direct canonical source transfer at N256, and
1,048,848B at N65536, session10; 15/15 per N fixture
observations. G1B unit j1 returns 0/15 SAFE at the named N256,
d48 lane and needs a full exact fallback for resolved decisions.

At d57 the b11 and b12 guards return UNKNOWN in the named
fixtures. Forced resolution adds 24B request, a second RTT and the
full direct exact lists: b11 N256 costs 5,040B vs direct 4,376B;
N65536 1,049,520B vs direct 1,048,856B.
Thus **STOP_RESOLVED_NEAR_T**, and ZERO claim that UNKNOWN proves
d>T, or that a one-sender 24B scalar is available to isolated
two-source XOR receiver. J52 is a physical size/update baseline
only here, not an unverified Q32 classifier.

Decision: **B2B0_PHYSICAL_SOURCE_PASS_GUARD_SCOPED_MODEL_CANDIDATE**.
Preserve the useful true two-owner precommitted threshold niche,
but **NO SYSTEM_PRODUCT_GO / PUBLIC API / SNAPSHOT change**.
B2-B1 must physically implement/run sender-first independent
two-peer localhost TCP and measure p95 RTT/bandwidth plus
corruption/session identity; explicit #86 keyed/adaptive and
#69 persistence/key lifecycle gates remain open.
No originality claim for classic bitmap XOR/Markov chain.
Canonical: `docs/research/DELTAGUARD-G1B2B0-EVIDENCE.md`.

### D57 — B1-A two-physical-sender TCP frame correctness; no measured system verdict

Parent #92, research PR #95; B1-B protocol preregistered before
any B1 TCP run. Exact test-code source
`e63608dc3cc870bb45b835012ff1cc1e28103609`,
five GitHub-hosted workers SUCCESS in
[run 37899112626](https://github.com/definitely-stable/deltameter/actions/runs/37899112626):
75 actual two-connection localhost TCP transactions
(15/worker including corrupted frame, duplicate owner, truncated body).
Source u64 lists are physically independent per sender actor;
receiver guard payload is XOR'd and compared with independent per-token
rehashing of exact A triangle B. Version/key-ID/owner/profile/epoch/
sequence/checksum/canonical payload errors fail closed.
N256,T64,d48 guard = 640 application bytes vs full direct 4224B;
d57/65 gave UNKNOWN on all five public-fixture source pairs.
This is not an independent statistical trial for delta=1e-6.

Decision: **B1_A_FRAME_CORRECTNESS_PASS (RESEARCH ONLY)**. Source
threads reside in one process and use a public fixture/key ID and
unkeyed corruption tag; no malicious-peer authentication is claimed.
No application bandwidth/RTT p95, true session churn, receiver-retained
exact delta replication, retransmission/ACK, fully resolved fallback,
or total memory/RSS product gate was executed here. NO SYSTEM_PRODUCT_GO,
NO public API or Snapshot v1. Next B1-B must compare same-topology
retained exact, count initial sync plus delta event traffic across
1/10/100 sessions, and run the frozen five-worker real TCP p95 test.
Security #86 and public lifecycle #69 remain independent blockers.
Canonical: docs/research/DELTAGUARD-G1B2B1A-EVIDENCE.md.

### D58 — B1-B0 physical retained exact and resolved fallback: query cadence is a product parameter

Parent #92; PR #96. Preregistered B1-B0 protocol before evidence.
Reproducible code source `f885785749aa607aac5173ffb8755153bc26a8d0`,
hosted [run 37902855683](https://github.com/definitely-stable/deltameter/actions/runs/37902855683):
five workers, exactly 360 source-bound records and PASS aggregator.
Each retains independent A/B sorted lists and B2A11 XOR bitmap,
physically streams every generation delta to a separate exact
receiver (99 events/owner), physically batches the same deltas for
S1/10/100 sparse checkpoint queries, sends a B2A guard pair on
EVERY generation, and uses a second real TCP request/complete full
list transaction for guard UNKNOWN. Exact B2A c48 certificate unchanged.
Full exact source agreement on every generation; independent bit-oracle
XOR per checkpoint. Atomic two-pass batch-validation explicitly prevents
partial receiver mutations on malformed later event.

**N256,d48,S100** actual cumulative application bytes:
B2A dense guard 64000, retained online exact 18678,
batched sparse exact 6262, B2A sparse guard 1920.
**N65536,d48,S100**: guard dense 64000, retained exact online
1063158 and batched 1050742, guard sparse 1920.
So no single "guard always best" conclusion; it depends on N,
cold/warm receiver bootstrap, query cadence, and permitted batching.
For N256,d57,S100, UNKNOWN and mandatory *physical* full fallback
costs 6506B vs direct full 5816B; maintain STOP_RESOLVED_NEAR_T.
The earlier B0 single 24B hypothetical request is NOT the real
B1-B0 two-owner fallback (two 24B requests plus two 1B hellos).

Decision: **B1B0_RETAINED_PHYSICAL_EXACT_FALLBACK_PASS,
RESEARCH ONLY / NO SYSTEM_PRODUCT_GO**. All byte counts are real
application TCP counts on localhost; owners are separate threads,
not separate processes or WAN. Current checksum/key fixture is not
hostile sender authentication; no real RTT/bandwidth paced p95,
retries/ACK, crash-safe generations, or RSS HWM.
Next B1-B1 must measure those with named 5-worker paired scenarios,
and compare a *warm pre-cached* exact receiver separately from cold
bootstrap. Keep #92 open; #86 and #69 block public semantics.
Canonical evidence: docs/research/DELTAGUARD-G1B2B1B0-EVIDENCE.md.

### D59 — B1-B1-A two OS sender processes and app-paced cost frontier; mixed p95 cannot decide product

Parent #97/#92, PR #98, frozen preregistration
`docs/research/DELTAGUARD-B1B1A-PROTOCOL.md` at
`cb62948b8a1101e96dba1ac50941305f8cd13627`
before first B1-B1-A timing. First completely valid
[run 37911404072](https://github.com/definitely-stable/deltameter/actions/runs/37911404072),
source `5a91d358cb9801130bd4804e225eacd28c9403ed`:
24 pinned B2A exact cutoff profiles, 300/300 complete verified
records, 5 hosted independent worker CI jobs and strict aggregator PASS.

This is **ACTUAL two distinct sender OS processes**, each on
its own localhost TCP socket. Real 25B hello, 24B requests, 64B
response frames and (when UNKNOWN) second two-owner full source
request, per-owner app-paced transfer, injected server response
delay, Linux **pre-response sampled** per-child VmHWM and build/update elapsed-wall diagnostics; late serialization/transfer memory peaks are NOT captured by the child hello and need separate post-transfer measurement.
It does NOT reproduce shared-wire bandwidth, network RTT, hostile
sender authentication or durable source daemon lifecycle.

At N256,d48,S10 (one named query), guard 738B, warm exact 338B
plus **previously physically delivered** 4322B source bootstrap.
At N256,d48,S100, guard 738B, warm exact 1958B after the
same 4322B warm bootstrap. N65536,d48,S10 guard 738B
vs warm exact query 338B after 1,048,802B physically provisioned
bootstrap. N256,d57,S10 resolved real fallback 5162B versus
direct full 4474B: **STOP_RESOLVED_NEAR_T** remains.

**Critical fairness check:** guard/full query `wall_ns` includes
source child spawning + first build, whereas warm exact `wall_ns`
begins after physically completed bootstrap with connected senders.
Comparing their reported 'p95' is invalid as a product latency
verdict. Per-worker maximum of THREE repeats is a weak p95 estimator
even on comparable states. Maintain separate setup/query times.
The cumulative dense-vs-sparse 100-generation byte crossover from
D58 is separate evidence, not contradicted by the B1-B1-A
single-query timings.

Decision: **B1B1A_PHYSICAL_TWO_PROCESS_COST_FOUNDATION_PASS_NO_PRODUCT_GO**.
Next #97 B1-B1B must have stateful sender-daemon persistent warm
source and repeated same-lifecycle hot/hot timing, real ACK/retry
and fail-closed reconnect, more samples, RSS separation and
accounted full cold bootstrap; #86 and #69 still public blockers.
No change to canonical snapshot or public Rust API.
Evidence: docs/research/DELTAGUARD-B1B1A-EVIDENCE.md.
