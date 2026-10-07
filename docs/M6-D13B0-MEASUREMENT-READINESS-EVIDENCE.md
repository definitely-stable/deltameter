# M6-D13-B0 — measurement-readiness evidence

Issue #54; parents #19/#14. Baseline: merged D13-A protocol v2 at
`7b5bbdbc7b278c58b85d1e3732d68ac70ee371df`.

Status: **MEASUREMENT_READY candidate; no performance/system verdict.**

Canonical measurement-source head:
`de0c57531c0a186e160f8c4617a1c7878181dc56`.

## What B0 proves

B0 makes the later D13-B system comparison measurable without allowing Python
controller time, hidden rebuilds, missing memory fields or transport-policy drift to
decide the result.

The frozen source manifest covers 13 protocol/worker/validator/workflow sources and
is checked fail-closed before the readiness inventory.

Hosted readiness run #25 / `37646837382`:

- frozen-source verification: **PASS**, 13 files;
- Rust direct/D11 partial controls: **PASS**;
- pinned RIBLT pull/stream controls: **PASS**;
- persistent session chains: **CHAIN_READINESS_PASS**, 1,776 rows;
- sparse 1M scaling: **SCALING_READINESS_PASS**, 14 rows;
- performance decision: **NOT_MEASURED**.

Artifacts:

~~~text
core
  id      11494018879
  sha256  4c8880ac2649e131ef6ddae5ee27af824d80f2f62ece83d1c0a3e64550a1dbce

scaling
  id      11494114098
  sha256  b644ca4b6fb5984a11e104b8d6da0c080f8fe327334a0d07b99f2a9b101d1076
~~~

The scaling inventory covers `N=1,048,576`, `d=128/512/1024`, direct exact,
D11 exact fallback, pinned RIBLT pull and stream-lower-bound lanes, plus natural
RIBLT exhaustion.

## Deterministic staged D11 false-candidate witness

The first persistent-chain implementation intentionally rejected every exact
fallback when the eventual session difference was `d<=8`. Hosted CI exposed that
this gate was too strong.

Frozen witness:

~~~text
N = 1024
true d = 8
removed = 0x1, 0x2, 0x3, 0x4
inserted =
  0x80000000007a3910
  0x80000000007a3911
  0x80000000007a3912
  0x80000000007a3913
~~~

At the first staged attempt `k=1`:

~~~text
maintained A sketch == fresh rebuild
maintained B sketch == fresh rebuild
maintained decoder == fresh decoder
provisional candidate produced
provisional candidate != exact symmetric difference
stage = k=1 < true d=8
~~~

Therefore this is **not maintained-state corruption** and not an in-capacity
`k=8` D11 decoder failure. It is a concrete early-stage over-capacity false
candidate admitted by the empirical one-extra-syndrome guard.

This is consistent with the existing contract: one extra syndrome is an empirical
guard, not an adversarial theorem. Earlier zero-false-success inventories remain
valid only for their finite matrices; they are not universal guarantees.

B0 now records `false_candidate` separately. A fallback with eventual `d<=8` is
valid only when fresh rebuild reproduces the same early-stage provisional candidate,
independent verification rejects it, and all failed work plus exact fallback remain
charged. Any other unexplained fallback remains INVALID.

The witness is a permanent regression in the hosted Rust readiness control.

## Measurement boundaries now available

Rust reports non-overlapping native phases for:

- exact source update;
- D11 sketch update;
- direct serialization/apply;
- prefix materialization;
- locator/decode;
- candidate exact/sketch apply;
- verification preparation;
- exact fallback serialization/apply.

Pinned Go RIBLT reports native import, coded-symbol production/decode, candidate
application, verification preparation and fallback phases.

Both sides retain source length/capacity, candidate capacity, process CPU ticks with
`CLK_TCK`, and hosted Linux RSS/HWM evidence. Python remains orchestration only.

Persistent chains cover:

~~~text
updates/session = 0, 1, 8, 64
sessions/build  = 1, 10, 100
~~~

D11 state is retained between sessions and checked against fresh rebuilds after each
session. RIBLT source reimport/reset costs remain explicit.

## Verdict

**MEASUREMENT_READY** once final exact-head repository CI closes on the
documentation/evidence head.

This verdict authorizes only the predeclared D13-B hosted comparison. It does not
select direct exact, D11 or Rateless IBLT, and it does not authorize a public
ExactSmallDelta API, Coverage::Exact, snapshot change, unsafe/FFI/SIMD or another
algebraic decoder optimization.

Production/public ExactSmallDelta remains **NO-GO**.
