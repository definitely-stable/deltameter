# Roadmap

The roadmap is intentionally linear. One developer should be able to understand the whole repository without an orchestration layer.

## M0 — Research bootstrap

Status: this branch.

Deliverables:

- mathematical/design documentation;
- dependency-free research scripts;
- one GitHub Actions research workflow;
- reproducible Energy profile derivation;
- reproducible finite-(d) Parity moment calculations.

Acceptance:

- scripts run with stock Python 3;
- no third-party Python packages;
- workflow produces research artifacts;
- documentation distinguishes proven, asymptotic and open claims.

## M1 — Energy reference implementation

Build one Rust crate, not a workspace.

Deliverables:

- `EnergyDeltaMeter`;
- deterministic configured randomness satisfying the stated proof assumptions;
- incremental energy maintenance;
- merge/subtract compatibility checks;
- theorem-derived profiles;
- unit/property tests.

Acceptance:

- unbiasedness/variance regression matches the research model;
- profile generator and Rust configuration agree;
- no unsafe code required;
- no crypto dependency.

## M2 — Parity reproduction

Deliverables:

- published finite-field PCSA reproduction;
- packed state;
- XOR merge;
- finite-(d) moment regression;
- middle-range statistical reproduction.

Acceptance:

- observed bias/variance are consistent with the chosen published construction;
- the code never labels asymptotic coverage as proven.

## M3 — Strict Parity decision

Research only what is necessary to answer one question:

> Can Parity provide a useful finite-sample one-sided capacity bound with practical memory?

Possible outcomes:

- **GO:** implement strict profiles;
- **NO-GO:** keep Parity as point-estimate fast path and use Energy for strict capacity;
- **REPLACE:** test a different GF(2) estimator only if evidence shows a real improvement.

This milestone is where small-d exact recovery may be reconsidered. It is not a bootstrap dependency.

## M4 — Performance and API freeze

After both main backends exist:

- measure update/merge/query throughput;
- choose default profile(s);
- decide whether batch APIs matter;
- optimize only proven bottlenecks;
- freeze the small public API;
- only then design serialization if persisted sketches are actually required.

## Working rules

- Prefer one PR per coherent milestone slice.
- Avoid “framework first” changes.
- No benchmark number becomes a promise until reproduced on named hardware.
- No background security subsystem without a concrete caller.
- Public GitHub-hosted runners may be used normally; CI simplicity matters more than runner-minute minimization.
