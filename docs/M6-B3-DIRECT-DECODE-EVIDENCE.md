# M6-B3 — direct Energy decode evidence

Status: **NO-GO; candidate not retained. M6-B implementation work complete.**  
Issue: #17. Parent: #14. PR: #51.  
Evidence date: 2026-10-07.

## Decision

B3 tested one isolated direct decoded-state construction candidate after accepted B1
and B2.

The candidate removed the explicit zero-initialization/overwrite/recompute shape from
source code by parsing counters into capacity-only storage and accumulating row
energies during parsing.

The predeclared performance gate failed decisively. Every measured Energy decode
lane regressed, while unchanged Parity/encode guardrails stayed near neutral.

**Verdict: NO-GO.** The B2 production decoder is retained.

No second B3 implementation is substituted after seeing the result. That preserves
the one-candidate-at-a-time rule and the predeclared stop condition.

## Canonical provenance

Base main:

`099cceb43e2d76a4b2b0890ecefc155965511cee`

Measured candidate head:

`c3ca57023f08a9e275ebca9962fce45d74597503`

Snapshot-performance run:

`37611779326`

Artifact:

- name: `snapshot-perf-37611779326-1`
- artifact ID: `11477203889`
- SHA256: `218df523eb92f0fa1289d80c8d3fa7bf654aea5b5a926506e1c1b8da24b2e014`

The paired run used exact base/head SHAs above and the existing four-round balanced
AB/BA protocol.

The final evidence-only PR head was synchronized onto current main
`a7492c8999cbe9c446f515d5a60ffe7b5bc4ba99` after independent M6-D12 merged.
That synchronization does not alter the measured B3 candidate or its base/head
performance evidence; it only replays the retained documentation and regression
test on the newer main.

## Candidate

After B2 exact structural preflight and config reconstruction, the candidate:

- allocated counters with `Vec::with_capacity(B*R)` rather than a zero-filled
  `vec![0; B*R]`;
- allocated row energies with `Vec::with_capacity(R)`;
- read counters in snapshot row-major order;
- pushed each counter directly;
- accumulated `sum(c^2)` for the current row using checked i128 addition;
- constructed the final private meter only after all counters/energies were valid;
- derived accepted M6-C sign masks exactly once;
- allocated pending-update scratch exactly once.

The individual i64 square remains representable in i128; only the row sum can
overflow.

A new valid-CRC B=2/R=1 fixture with two `i64::MIN` counters verifies that row
energy overflow remains `SnapshotError::InvalidPayload`.

## Predeclared gate

The experiment required:

- at least 3% lower decode latency in **each** Energy-default content lane;
- no Energy-small decode regression greater than 2%;
- no systematic >3% regression in unchanged Parity/encode guardrails.

A rerun was permitted only for one isolated lane accompanied by comparable
guardrail drift.

The observed result does not qualify for that rerun exception.

## Hosted result

Energy default decode:

~~~text
empty       +2.682%
full-width  +2.501%
sequential  +2.452%
~~~

Energy small decode:

~~~text
empty       +2.912%
full-width  +2.924%
sequential  +3.237%
~~~

Every Energy decode lane regressed in the same direction.

Unchanged decode guardrails were near neutral:

~~~text
Parity padded:   -0.149% .. +0.545%
Parity standard: +0.143% .. +0.505%
~~~

Encode guardrails were also approximately neutral, aside from small noise-scale
movements.

B2's malformed structural rejection lane changed by -0.153%, also neutral.

This pattern is not consistent with a single noisy Energy sample. The candidate
specifically regresses the operation it changes.

## Interpretation

The exact microarchitectural cause was not measured and is therefore not asserted.

A plausible explanation is that fusing checked row-energy accumulation into the
counter parse loop introduces a serial dependency chain on the row accumulator and
prevents the separate recomputation pass from benefiting from a tighter compiler/
cache execution pattern. This remains a hypothesis, not evidence.

What is established is simpler: removing source-level logical memory touches did
not translate into lower end-to-end decode latency on the hosted runner.

The source-level 736 KiB default-shape logical-touch opportunity from the research
audit therefore must not be treated as a performance proxy.

## Correctness

The candidate preserved:

- snapshot-v1 byte compatibility;
- B2 checksum-before-backend ordering;
- B2 ProvenanceRequired precedence;
- exact counter order;
- row-major energy summation semantics;
- valid-CRC energy-overflow rejection;
- existing continuation and round-trip tests;
- M6-C sign-mask semantics.

Correctness is insufficient for retention because the explicit performance gate
failed.

## B3 verdict

**NO-GO. Revert the candidate and retain B2.**

Do not replace B3 with an unplanned two-pass variant, sign-mask fusion or CRC change
inside the same decision slice.

## M6-B completion

The three issue #17 candidates now have explicit outcomes:

~~~text
B1 table-driven scalar CRC32C     ACCEPT
B2 exact structural preflight     ACCEPT
B3 direct one-pass Energy decode  NO-GO
~~~

The optional residual CRC selector is **SKIPPED for this milestone**. No current
measurement demonstrates a remaining codec bottleneck that justifies extending #17
with slicing-by-N, CRC fusion, hardware specialization or another allocation
candidate.

Any future codec optimization requires a new profile/evidence trigger and a separate
decision slice.

M6-B can close after this NO-GO record is merged.
