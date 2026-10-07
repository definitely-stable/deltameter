# M6-D13-A implementation plan

Goal: executable, evidence-gated system protocol foundation; no timing conclusions.
Architecture: Python standard-library transport/accounting harness, private Rust
D11 worker, pinned external Go RIBLT worker. No crate dependency or production edits.
Spec: `docs/M6-D13A-SYSTEM-PROTOCOL.md`; issue #52, parents #19/#14.
Execution: native implementation in isolated task checkout, hosted Rust/Go checks.

## Constraints and review focus

Frozen D1/D4/D6/D8/D11 modules unchanged; all restrictions in the spec apply.
Review: oracle leakage into stage selection; undercounted batch/verification;
post-stream RIBLT update misuse; hidden d>8 false success; omitted/duplicate rows.
No local Rust/Go installation is available; hosted CI is the compiler/test gate.

## Task 1: freeze contract before implementation

- [x] Audit main, #14/#19, roadmap, M6 and D2/D11/D12 evidence and upstream pin.
- [x] Write protocol, JSON contract, this plan; create issue and commit separately.
- [ ] Link roadmap and canonical M6 document.

## Task 2: protocol and accounting

Files: `research/m6d13a/protocol.py`, `test_protocol.py`.
Interfaces: `Frame.encode/decode`, `Session.exchange/verify`, exact list codec,
`run_session(a,b,lane,worker)` -> trace plus final set; `validate_report` fail-closed.
- [ ] Tests first: framing/identity/replay/length, list canonicality, exact direct
  2 RTTs, 8/9 fallback accounting, false-candidate final verification, budget caps.
- [ ] Run Python unittest, observe missing implementation; implement, rerun.

## Task 3: real comparator workers

Files: `examples/m6d13a_worker.rs`, `research/m6d13a/riblt_worker.go`.
Private line-oriented control channel is harness I/O, not simulated network wire.
Rust commands initialize maintained sketches, emit prefix, and decode received
prefix against B. Assert membership-change replay equals rebuild. Go uses the
pinned real encoder/decoder, 24-byte cells; consumes full batches and keeps state.
- [ ] Add integration checks for known differences, zero/high bits, d=8/9,
  no oracle-based success, exact fallback and post-session equality.
- [ ] Hosted compile/format/Clippy/tests and real pinned external integration.

## Task 4: complete inventory and provenance

Files: `research/m6d13a/run.py`, `.github/workflows/m6d13a.yml`.
- [ ] Generate frozen matrix and run three arms; retain per-frame direction,
  kind/parameter/length/hash; CPU and allocated peak remain null.
- [ ] Reject missing/extra/duplicate rows, mismatched protocol/source hashes,
  inconsistent counters and wrong verdict; test artifact mutations.
- [ ] Upload report, environment, hashes and raw traces on hosted CI.
- [ ] Final review, PR, exact-head CI; no merge or performance claim implied.
