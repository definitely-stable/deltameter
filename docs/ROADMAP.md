# Roadmap

The roadmap is intentionally linear. One developer should be able to understand the whole repository without an orchestration layer.

## M0 — Research bootstrap

Status: complete; merged in PR #1.

Deliverables:

- structured research corpus;
- source audits and decision record;
- dependency-free research scripts;
- one GitHub Actions research workflow;
- reproducible Energy profile derivation;
- reproducible finite-(d) published-F-PCSA moment checks;
- explicit Q1 v0 decision.

Acceptance:

- documentation distinguishes proven, asymptotic, empirical and open claims;
- contradictory research inputs are archived and reconciled;
- Q1 does not falsely claim a strict Parity theorem;
- scripts run with stock Python 3;
- workflow produces reproducible artifacts.

## M1 — Energy reference implementation

Status: complete; merged in PR #3.

Build one Rust crate, not a workspace.

Deliverables:

- `EnergyDeltaMeter`;
- explicit pairwise bucket + 4-wise sign randomness;
- theorem-profile construction from 6R caller-supplied independent uniform u64 words, with no RNG dependency;
- incremental energy maintenance;
- compatible source-sketch difference/subtract;
- theorem-derived profiles;
- deterministic and property tests.

Acceptance:

- implementation matches the documented estimator;
- profile generator and Rust configuration agree;
- algebraic merge/difference properties pass;
- no unsafe code required;
- no crypto dependency.

## M2 — Published F-PCSA reproduction

Status: complete; merged in PR #5.

Do not start with the custom g(v)=1 variant and do not substitute a classical-PCSA first-1-position model.

Deliverables:

- faithful FIELDMAP[i,j] finite-field state;
- published h row/level distribution;
- published uniform field coefficient g;
- rightmost-nonzero row statistic;
- explicit finite level policy J;
- packed state;
- XOR composition;
- finite-d moment regression where analytically justified;
- asymptotic/middle-range reproduction.

Acceptance:

- observed behavior is consistent with the published construction;
- the implementation documents the paper's middle-range limitation;
- published and custom set-specialized variants are not conflated;
- universe size N, row count m, Hamming weight d and level bound J are not overloaded;
- no strict capacity claim.

## M3 — Experimental ParityDeltaMeter

Status: complete in PR #7.

Wrap the reproduced algorithm in the small public API.

Deliverables:

- explicit `Coverage::Asymptotic { relative_standard_error }` status;
- `ParityConfig` + `ParityDeltaMeter` wrapper over M2;
- Compact/Standard/Accurate experimental profiles;
- exact configuration compatibility for XOR merge;
- visible finite-J truncation outcomes;
- stable-Rust state/update/merge/query benchmark harness;
- documented memory/guarantee comparison with Energy.

Post-M3 research status:

- exact tiny full-state enumeration/DP: implemented for proof/regression-sized cases;
- W_i non-sufficiency: established by a finite counterexample;
- finite-J truncation budget: implemented, with signal and state-distortion events separated;
- Poissonized cell/row law: derived and implemented as a research artifact;
- fixed-d de-Poissonization/inversion: no practical certified bridge;
- full published-statistic monotonicity: unresolved;
- strict published-W_i path: **STOP / NO-GO**.

A narrow `ParityLevelCounts` research track may continue post-v0, but it is not a release blocker and has no public `Coverage::Proven` contract.

## M4 — Performance and API freeze

Status: complete; merged in PR #11.

Deliverables:

- paired base/head stable-Rust benchmark for update/merge/query on the same GitHub-hosted runner;
- explicit v0 defaults:
  - Energy = 10% relative error / failure probability <= 1e-6;
  - experimental Parity = Standard (m=256, J=64);
- no batch API in v0: current scalar backends expose no measured batch-specific amortization opportunity;
- demonstrated Parity query optimization for J=64:
  - no highest-level Vec allocation in estimate;
  - one u64 highest-set-bit operation per row instead of scanning 64 levels;
- rejected Energy merge and query micro-optimizations when paired measurements did not show a stable benefit;
- root-only supported public API; published F-PCSA reproduction remains internal research machinery;
- raw Parity packed words are not public, so an in-memory layout is not accidentally frozen as a wire format;
- extensible public enums/result records marked non-exhaustive before the freeze;
- integration compile contract for the supported root API;
- rustdoc warnings denied in CI;
- persisted serialization deliberately remains unfrozen.

Acceptance:

- cargo fmt --all -- --check;
- cargo clippy --all-targets -- -D warnings;
- cargo test --all-targets;
- RUSTDOCFLAGS="-D warnings" cargo doc --no-deps;
- release-mode examples compile;
- GitHub-hosted paired performance workflow succeeds and uploads evidence;
- Energy remains the only Coverage::Proven backend;
- Parity remains Coverage::Asymptotic;
- no unsafe, SIMD, nightly, new runtime dependency or self-hosted runner requirement.

See docs/M4-PERFORMANCE-API-FREEZE.md for the freeze record and performance interpretation.

Only after M4 is merged should a separate requirement decide whether persisted serialization is actually needed.

## M5 — Canonical sketch interchange v1

Status: complete; merged in PR #13.

Rationale:

- when both full sets are already co-resident, exact comparison is often available;
- the mergeable-sketch use case becomes materially useful across process, host, storage or time boundaries;
- therefore a minimal self-contained interchange representation is a product requirement after the M4 API freeze.

Deliverables:

- versioned canonical envelope shared by Energy and Parity;
- little-endian fixed-width integers;
- exact backend and length validation;
- CRC32C accidental-corruption detection;
- self-contained Energy hash-row configuration plus primary counters;
- Energy derived energies recomputed on decode rather than persisted;
- self-contained Parity shape/seed plus packed GF(2) state;
- zero-padding canonicality for the final Parity word;
- public encode_snapshot/decode_snapshot methods and SnapshotError;
- fixed byte vectors and fail-closed malformed-input tests;
- no serde, file/network helper, compression, crypto, unsafe or new runtime dependency.

The exact format and compatibility rules are in docs/M5-INTERCHANGE-V1.md.

## M6 — Evidence-gated workflow and compatible optimization

Status: M6-0 complete; merged in PR #20. Parent issue: #14.

[Critical audit and design](M6-RECONCILIATION-AND-OPTIMIZATION.md) records the research constraints, corrections to the initial proposal and acceptance gates.

- M6-0 / #15: snapshot compatibility fixtures, serialization evidence, single-buffer candidate.
- M6-A / #16: complete in PR #21; estimator-assisted admission is NO-GO as a generic/public workflow on the measured matrix (+29.838% application bytes vs direct exact).
- M6-B / #17: independently measured CRC/decoder resource candidates.
- M6-C / #18: complete in PR #22; bit-equivalent Energy sign hashing ACCEPT, with about 68–71% lower update latency and negligible amortized setup after the optimized mask builder.
- M6-D / #19: D1 guarded PinSketch64 and D2 nested-prefix LAB-GO are complete. D3 / #28 proves incremental Berlekamp–Massey reuse is correct but performance NO-GO; root factorization/verification dominates decoder CPU. Production ExactSmallDelta/public reconciliation remains NO-GO. Next research target is isolated characteristic-2 trace/root-factor specialization.

Snapshot plus an unchanged full-list exchange is an overhead control, not a demonstrated benefit. M6 may conclude that a proposed workflow is not worthwhile. No public networking API or strict Parity work is implied.

[First-slice implementation plan](superpowers/plans/2026-10-06-m6-foundation.md). M6-0 measured verdict: [evidence](M6-0-EVIDENCE.md). M6-A measured verdict: [NO-GO evidence](M6-A-EVIDENCE.md). M6-C measured verdict: [ACCEPT evidence](M6-C-EVIDENCE.md). M6-D Phase-1 decision: [exact-lane audit](M6-D-EXACT-LANE-AUDIT.md). D1 measured verdict: [PinSketch64 lab evidence](M6-D1-PINSKETCH64-EVIDENCE.md). D2 measured verdict: [incremental prefix evidence](M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md). D3 measured verdict: [incremental BM NO-GO evidence](M6-D3-INCREMENTAL-BM-EVIDENCE.md).

## Post-v0 candidates

Only after evidence:

- `ParityLevelCounts` strict finite-sample research, only if certified fixed-d tails become practical;
- `ParityPcsaSetV1` with `g(v)=1`;
- ExactSmallDelta: Phase-1 audit selects a private pure-Rust PinSketch64 reference for lab evaluation; no production backend or public API until the reference passes exact-domain, over-capacity, verification and performance gates;
- SIMD/unsafe specialization;
- formal verification of a narrow proof/code boundary.

## Working rules

- Prefer one PR per coherent milestone slice.
- Avoid framework-first changes.
- Preserve historical research, but keep one canonical decision layer.
- Public GitHub-hosted runners may be used normally.
- CI simplicity matters more than artificial runner-minute minimization.
- No benchmark number becomes a promise until reproduced on named hardware.
