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
- [x] Link roadmap and canonical M6 document.

## Task 2: protocol and accounting

Files: `research/m6d13a/protocol.py`, `test_protocol.py`.
Interfaces: `Frame.encode/decode`, `Session.exchange/verify`, exact list codec,
`run_session(a,b,lane,worker)` -> trace plus final set; `validate_report` fail-closed.
- [x] Tests first: framing/identity/replay/length, list canonicality, exact direct
  2 RTTs, 8/9 fallback accounting, false-candidate final verification, budget caps.
- [x] Run Python unittest, observe missing implementation; implement, rerun.

## Task 3: real comparator workers

Files: `examples/m6d13a_worker.rs`, `research/m6d13a/riblt_worker.go`.
Private line-oriented control channel is harness I/O, not simulated network wire.
Rust commands initialize maintained sketches, emit prefix, and decode received
prefix against B. Assert membership-change replay equals rebuild. Go uses the
pinned real encoder/decoder, 24-byte cells; consumes full batches and keeps state.
- [x] Add integration checks for known differences, zero/high bits, d=8/9,
  no oracle-based success, exact fallback and post-session equality.
- [ ] Hosted compile/format/Clippy/tests and real pinned external integration.

## Task 4: complete inventory and provenance

Files: `research/m6d13a/run.py`, `.github/workflows/m6d13a.yml`.
- [x] Generate frozen matrix and run three arms; retain per-frame direction,
  kind/parameter/length/hash; CPU and allocated peak remain null.
- [x] Reject missing/extra/duplicate rows, mismatched protocol/source hashes,
  inconsistent counters and wrong verdict; test artifact mutations.
- [x] Upload report, environment, hashes and raw traces on hosted CI.
- [ ] Final review, PR, exact-head CI; no merge or performance claim implied.

## Execution/review record

Protocol-only remote commit: `ace62153caab424d97e23894916dabd47cb0e6ec`.
PR #53. No performance runs were made.

- Python tests: initial import failure, implementation, 9 passing tests.
- Independent review: cross-session replay and invalid verification transitions
  reproduced; regression tests failed before fixes, then all 12 tests passed.
- Also bind trace SHA256 to reconstructed full frames, retain small sketch payloads,
  require unique session IDs, and retain failed-verification payloads for replay.
- First hosted Rust failure was rustfmt only; exact formatter changes applied.
- Dedicated workflow compiles both real workers and runs upstream Go tests;
  ordinary `rust` workflow owns formatting, strict Clippy and complete Rust tests.
- Maintained duplicate-insert/missing-delete API is not exposed in this foundation;
  replay uses validated set membership differences. Update-rate/session-count
  performance grid remains a D13-B instrumentation requirement, not completed here.
- Local checkout has no push credentials; GitHub API publishes commit trees and
  fast-forward updates with an expected-head lease. Remote trees match local work.

- First complete hosted foundation inventory passed: run `37625532197`, 1,350
  rows. It predates review hardening and is not the final acceptance run.
- Added a further fail-closed gate: in-capacity D11 must reach the expected
  k without hiding decoder breakage behind fallback (regression RED -> GREEN).
  Local foundation suite: 13 tests passed; final hosted HEAD pending.


## Protocol v2 review amendment

Formal review identified two winner-changing risks before any timing, so the
foundation contract is amended before D13-B:

- direct exact and exact fallback now terminate on receipt of the validated
  canonical A list; they do not pay a redundant second O(N) reverse-list transfer;
- reverse-list VERIFY/ACK remains mandatory only for provisional sketch candidates;
- the external Rateless IBLT lane is explicitly named `riblt_pull_pow2`; it is a
  transport control, not a faithful substitute for upstream stream-until-stop;
- an architecture-level rateless verdict is blocked until D13-B0 adds a separately
  frozen streaming comparator;
- D13-B0 must also add native/non-overlapping CPU accounting, tracked capacity/RSS,
  genuine persistent multi-session D11 state, and a scaling extension at
  N=1,048,576 with d=128/512/1024 plus natural exhaustion.

The foundation correctness grid remains intentionally bounded to N<=65,536 and
d<=64. No performance conclusion is permitted from that boundary. Protocol v2
requires a fresh untimed exact-head inventory before merge.
