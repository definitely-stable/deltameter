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

Status: **complete**. Final system verdict: **STOP_SYSTEM_PRODUCT** in M6-D13-B / PR #57. Parent issue #14 is closed.

[Critical audit and design](M6-RECONCILIATION-AND-OPTIMIZATION.md) records the research constraints, corrections to the initial proposal and acceptance gates.

- M6-0 / #15: snapshot compatibility fixtures, serialization evidence, single-buffer candidate.
- M6-A / #16: complete in PR #21; estimator-assisted admission is NO-GO as a generic/public workflow on the measured matrix (+29.838% application bytes vs direct exact).
- M6-B / #17: complete. B1 table-driven scalar CRC32C ACCEPT in PR #40; B2 exact structural preflight ACCEPT in PR #49; B3 direct one-pass Energy decoded-state construction NO-GO in PR #51 because every Energy decode lane regressed by 2.45-3.24% while unchanged guardrails stayed near neutral. B3 code is not retained. The optional residual CRC selector is skipped for this milestone; future codec work requires a new measured trigger.
- M6-C / #18: complete in PR #22; bit-equivalent Energy sign hashing ACCEPT, with about 68–71% lower update latency and negligible amortized setup after the optimized mask builder.
- M6-D / #19: complete. D1-D11 build and optimize the private guarded PinSketch reference; D12 stops algebraic micro-optimization. D13-A/B0 freeze a fair system protocol and measurement substrate. D13-B / #56 then runs five independent hosted workers, 3,900 raw observations and 360 named network/amortization cells. **STOP_SYSTEM_PRODUCT**: D11 qualifies in 0/360 cells and even the optimistic RIBLT stream-lower-bound qualifies in 0/360 cells versus direct exact under the frozen >=10% every-worker gate. The best D11 cell is still ~0.04% worse on its worst worker; the best optimistic stream cell is ~0.44% worse. Production/public ExactSmallDelta remains NO-GO; retain D11 as private research/reference only. No further M6 reconciliation-backend work is authorized without a materially new protocol contract.

Snapshot plus an unchanged full-list exchange is an overhead control, not a demonstrated benefit. M6 may conclude that a proposed workflow is not worthwhile. No public networking API or strict Parity work is implied.

[First-slice implementation plan](superpowers/plans/2026-10-06-m6-foundation.md). M6-B: [research audit](M6-B-CODEC-DECODER-RESEARCH-AUDIT.md), [corrected post-research plan](superpowers/plans/2026-10-07-m6b-post-research.md), [B2 evidence](M6-B2-STRUCTURAL-VALIDATION-EVIDENCE.md), and [B3 NO-GO evidence](M6-B3-DIRECT-DECODE-EVIDENCE.md). M6-0 measured verdict: [evidence](M6-0-EVIDENCE.md). M6-A measured verdict: [NO-GO evidence](M6-A-EVIDENCE.md). M6-C measured verdict: [ACCEPT evidence](M6-C-EVIDENCE.md). M6-D Phase-1 decision: [exact-lane audit](M6-D-EXACT-LANE-AUDIT.md). D1 measured verdict: [PinSketch64 lab evidence](M6-D1-PINSKETCH64-EVIDENCE.md). D2 measured verdict: [incremental prefix evidence](M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md). D3 measured verdict: [incremental BM NO-GO evidence](M6-D3-INCREMENTAL-BM-EVIDENCE.md). D4 measured verdict: [trace-square ACCEPT evidence](M6-D4-TRACE-SQUARE-EVIDENCE.md). D5 profiling verdict: [root-profile evidence](M6-D5-ROOT-PROFILE-EVIDENCE.md). D6 measured verdict: [quadratic ACCEPT evidence](M6-D6-QUADRATIC-EVIDENCE.md). D7 profiling verdict: [post-quadratic residual evidence](M6-D7-RESIDUAL-ROOT-PROFILE-EVIDENCE.md). D8 measured verdict: [GF(2^64) scalar-square ACCEPT evidence](M6-D8-GF64-SQUARE-EVIDENCE.md). D9 profiling verdict: [post-D8 residual evidence](M6-D9-POST-D8-PROFILE-EVIDENCE.md). D10 measurement verdict: [trace-internal reduction selector evidence](M6-D10-TRACE-INTERNAL-EVIDENCE.md). D11 measured verdict: [fixed-constant reduction ACCEPT evidence](M6-D11-FIXED-REDUCTION-EVIDENCE.md). D12 profiling verdict: [post-D11 stop-gate evidence](M6-D12-POST-D11-PROFILE-EVIDENCE.md). D13-A protocol: [system comparison foundation](M6-D13A-SYSTEM-PROTOCOL.md). D13-B0 readiness: [measurement-readiness evidence](M6-D13B0-MEASUREMENT-READINESS-EVIDENCE.md). D13-B final system verdict: [STOP_SYSTEM_PRODUCT evidence](M6-D13B-SYSTEM-EVIDENCE.md).

## STRICT-COMPACT optimization before public freeze

Issue #70 defines the optimization program that follows the accepted 32 KiB
STRICT-COMPACT private candidate.

Public guarantee/key/snapshot semantics may continue under #69, but the first
public memory profile and interchange shape must not be frozen until the cheap
product-shape work closes:

1. range-aware stored-level frontier;
2. maintained odd-level counts;
3. progressive/prefix level transfer.

Only after those slices should the project decide whether to open the generalized
Certified Parity Ladder, DeltaGuard threshold primitive, or a broader oracle
portfolio.

Mathlab-facing theorem candidates — finite independence, adaptive tokens,
heterogeneous ladder optimization, state lower bounds and nested-prefix optimality
— are tracked as research transfers rather than production assumptions.

Canonical program:
[STRICT-COMPACT-OPT-001](research/STRICT-COMPACT-OPT-001-PROGRAM.md).

Canonical execution plan with child issues #72/#73/#74:
[STRICT-COMPACT-OPT-NEXT-WORK](research/STRICT-COMPACT-OPT-NEXT-WORK.md).

OPT-A / #72: **RANGE_FRONTIER_PASS** in PR #76. The exact frontier proves J=56
already preserves the full accepted u64 q95 range while reducing instance state
from 32 KiB to 28 KiB (-12.5%). Level-major is retained for subsequent research:
keyed update cost is effectively unchanged, estimate extraction is substantially
cheaper, and each level is a contiguous 512-byte chunk. J=24/32/40/48 remain valid
bounded-range frontier points. Canonical evidence:
[STRICT-COMPACT-OPT-A](research/STRICT-COMPACT-OPT-A-EVIDENCE.md).

OPT-A follow-up on PR #76: **J52_REFINEMENT_PASS** independently reproduces
full-u64 continuous certification at **J=52 / 26 KiB / LEVEL_MAJOR**. The
five-worker hosted refinement passed the frozen viability thresholds; J=51's
current certificate prefix ends at 11,316,578,792,889,469,415 (not a
mathematical impossibility for other constructions). Thus **J=52 supersedes
J=56 as the preferred full-u64 *research* baseline**, saving 18.75% versus
J=64. Original OPT-A Stage-1/2 evidence remains a valid historical frozen
matrix. Canonical addendum:
[STRICT-COMPACT-OPT-A-REFINEMENT](research/STRICT-COMPACT-OPT-A-REFINEMENT-EVIDENCE.md).

OPT-B / #73: five-worker research evidence gives **KEEP_CACHE** for the J52
LEVEL_MAJOR maintained per-level odd-count cache: +104 B derived non-serialized
payload, worst hosted update regression +2.670%, query speedup >=13.563x
on every worker and >=15.22% total mixed-100 gain on every worker versus
an allocation-free scan comparator. Evidence:
[STRICT-COMPACT-OPT-B](research/STRICT-COMPACT-OPT-B-EVIDENCE.md).
This is a private research/product decision, NOT a public API or snapshot change.

OPT-C / #74: C0/C1 frame and simultaneous-coverage foundation passed in
merged PR #78. C2 physical transport research collected **10,560** five-worker
observations on fixed d/T scenarios: unaided PUSH_ALL increases full transfer
26,750B to 27,838B; the T-informed one-level bounded prefix costs 655B
when the requested upper-bound criterion is satisfied. This is explicitly
**MODELED_C3_CANDIDATE**, not actual-network PROGRESSIVE_PASS and not
equivalent to transmitting a full reusable sketch. A trusted sender can
also return U alone much more cheaply. Canonical C2 evidence:
[STRICT-COMPACT-OPT-C-C2](research/STRICT-COMPACT-OPT-C-C2-EVIDENCE.md).

Next: #74 OPT-C progressive/prefix
transfer. Public memory profile/snapshot freeze remains blocked until those slices
close.

## Post-v0 candidates

Only after evidence:

- `ParityLevelCounts` strict finite-sample research, only if certified fixed-d tails become practical;
- `ParityPcsaSetV1` with `g(v)=1`;
- ExactSmallDelta: retain the private pure-Rust PinSketch64 reference for research/regression use only. M6-D13-B is STOP_SYSTEM_PRODUCT for the current exact-verification system contract; any production revisit requires a materially new protocol and fresh evidence;
- SIMD/unsafe specialization;
- formal verification of a narrow proof/code boundary.

## Working rules

- Prefer one PR per coherent milestone slice.
- Avoid framework-first changes.
- Preserve historical research, but keep one canonical decision layer.
- Public GitHub-hosted runners may be used normally.
- CI simplicity matters more than artificial runner-minute minimization.
- No benchmark number becomes a promise until reproduced on named hardware.


## M6-D13-A comparator/system foundation

Issue #52 freezes the next system comparison before measurements: direct exact,
maintained guarded D11 with exact fallback, and a pinned external Rateless IBLT
`pull_pow2` comparator. See [protocol](M6-D13A-SYSTEM-PROTOCOL.md) and
[plan](superpowers/plans/2026-10-07-m6d13a-system-foundation.md).
Protocol v2 makes a validated complete target-list transfer terminal for direct
and fallback; only provisional sketch candidates pay independent reverse-list
verification. The pull comparator alone cannot select/reject the rateless
architecture. D13-B timing is blocked on native CPU/peak-memory instrumentation,
persistent multi-session D11 state, a faithful RIBLT streaming lane and a larger
scaling extension. D13-A correctness/accounting acceptance is distinct from
performance or product acceptance. D12 STOP_ALGEBRAIC_MICRO_OPT and public
ExactSmallDelta NO-GO remain in force.
