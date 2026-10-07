# M6-D13-B implementation plan

Issue #56. Baseline `8b5421709965e88dfbb74cd61eb742321d738efa`.
Protocol: `docs/M6-D13B-SYSTEM-COMPARISON.md`.

## 1. Freeze before timing

- commit machine contract and human protocol;
- do not change B0 native workers or accepted D11 support modules;
- freeze Git blob hashes for all measurement sources before first hosted timing.

## 2. Controller and wire model

Implement a fail-closed Python controller that:
- generates deterministic persistent update chains;
- rotates arm order across five workers × three process families;
- records native build/update/sync/memory fields without controller timing;
- computes exact D13-A application bytes/RTTs;
- preserves all fallback/false-candidate observations;
- adds boundary and one-million-key scaling guards.

## 3. Hosted evidence

One workflow:
- matrix timing jobs worker=1..5;
- each builds the exact Rust B0 worker;
- each checks out pinned RIBLT and builds the exact B0 Go adapter;
- each emits one raw JSON artifact;
- aggregate job downloads all five artifacts, validates completeness and computes
  the frozen network/amortization grid.

No automatic benchmark threshold is enforced by GitHub timing itself; only the
predeclared summarizer decides the evidence label.

## 4. Closure

Retain raw artifacts, aggregate summary and exact-head run provenance. Record one of
the frozen D13-B outcomes. Update #19/ROADMAP/DECISIONS.

Do not implement public ExactSmallDelta, digest verification, snapshot-v2, unsafe,
FFI, SIMD or another decoder optimization in this slice.
