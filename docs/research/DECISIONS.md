# Decision record 0001 — pre-implementation baseline

Status: **accepted and updated through M6-A evidence**  
Updated: 2026-10-06 after the M6-A estimator-assisted workflow decision.

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
