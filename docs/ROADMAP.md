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

Wrap the reproduced algorithm in the small public API.

Deliverables:

- explicit `Coverage::Asymptotic`/experimental status;
- merge compatibility checks;
- state/memory/update benchmarks;
- comparison with Energy.

Optional research:

- exact small finite enumeration/DP on the actual FIELDMAP state or a proved sufficient statistic;
- finite-J truncation-budget checker;
- fixed-d analysis;
- Poissonized law only if the exact category independence is derived;
- de-Poissonization only after its theorem hypotheses and forward-difference bounds are verified;
- one-sided test inversion with monotonicity checks.

Strict-Parity theory is **not** a v0 release blocker.

## M4 — Performance and API freeze

After Energy and experimental Parity exist:

- measure update/merge/query throughput;
- choose default profile(s);
- decide whether batch APIs matter;
- optimize only demonstrated bottlenecks;
- freeze the small public API.

Only then decide whether persisted serialization is actually required.

## Post-v0 candidates

Only after evidence:

- strict finite-sample Parity;
- `ParityPcsaSetV1` with `g(v)=1`;
- ExactSmallDelta via Minisketch/PinSketch or a pure-Rust alternative;
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
