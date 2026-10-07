# M6-D13-B0 implementation plan

Issue #54. Baseline main: `7b5bbdbc7b278c58b85d1e3732d68ac70ee371df`.
Contract: `docs/M6-D13B0-MEASUREMENT-READINESS.md`.

## Goal

Produce a trustworthy measurement substrate for D13-B. Do not measure/select a
winner in this slice.

## Slice 1 — freeze and validator

- freeze this contract and machine-readable readiness JSON before timing code;
- fail closed on missing phases, overlapping totals, missing memory fields, changed
  source pins or incomplete session chains;
- preserve D13-A v2 exact/fallback semantics.

## Slice 2 — persistent Rust worker

Create a new private B0 worker; do not modify historical D11 support modules.

Implement:
- sorted unique exact A/B state;
- optional maintained capacity-9 D11 state;
- membership-validated insert/delete with atomic rejection;
- deterministic persistent session chain;
- D11 staged decode using accepted D11;
- exact direct/fallback merge application;
- fresh-rebuild oracle after every D11 session;
- native non-overlapping phase elapsed metrics;
- Vec len/capacity plus Linux RSS/HWM checkpoints.

## Slice 3 — pinned Go comparator readiness

Build the B0 adapter inside the exact upstream pin.

Implement both:
- existing pull_pow2 semantics;
- stream-lower-bound semantics that produces/consumes one real coded symbol at a
  time until Decoded().

Measure imports, production and decode inside Go, not Python. Record runtime memory
and process RSS/HWM. Reimport source state per session because upstream forbids set
mutation after stream consumption.

## Slice 4 — readiness matrices

Run:
- persistent U=0,1,8,64;
- sessions/build=1,10,100, using a bounded representative N set for full readiness;
- D13-A 8/9 boundary controls;
- sparse scaling N=1,048,576 with d=128/512/1024;
- natural RIBLT exhaustion/fallback.

The full Cartesian future performance grid is intentionally deferred.

## Slice 5 — evidence closure

- validate phase sums and source/state equivalence;
- retain raw per-session metrics and memory checkpoints;
- freeze measurement source hashes only after implementation review;
- exact-head GitHub-hosted CI;
- record MEASUREMENT_READY or INVALID.

No D13-B performance run starts automatically. No public API/Coverage/snapshot
change, unsafe, FFI, SIMD or new Rust runtime dependency.
