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

Status: next implementation milestone after issue #8 research reconciliation.

After Energy and experimental Parity exist:

- measure update/merge/query throughput;
- choose default profile(s);
- decide whether batch APIs matter;
- optimize only demonstrated bottlenecks;
- freeze the small public API.

Only then decide whether persisted serialization is actually required.

## Post-v0 candidates

Only after evidence:

- `ParityLevelCounts` strict finite-sample research, only if certified fixed-d tails become practical;
- `ParityPcsaSetV1` with `g(v)=1`;
- ExactSmallDelta via Minisketch/PinSketch, Simple Set Sketching, IBLT-derived methods or a pure-Rust alternative, only after a separate primary-source and failure-semantics audit;
- persisted wire format;
- SIMD/unsafe specialization;
- formal verification of a narrow proof/code boundary.

## Working rules

- Prefer one PR per coherent milestone slice.
- Avoid framework-first changes.
- Preserve historical research, but keep one canonical decision layer.
- Public GitHub-hosted runners may be used normally.
- CI simplicity matters more than artificial runner-minute minimization.
- No benchmark number becomes a promise until reproduced on named hardware.
