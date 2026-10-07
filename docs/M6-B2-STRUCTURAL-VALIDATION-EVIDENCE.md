# M6-B2 — structural validation evidence

Status: **ACCEPT for snapshot-v1 resource hardening; no valid-decode speedup claim.**  
Issue: #17. Parent: #14. PR: #49.  
Evidence date: 2026-10-07.

## Decision

B2 moves complete Energy backend shape validation ahead of row allocation without
changing snapshot-v1 bytes, the checksum-first boundary, theorem assumptions or
transport policy.

The candidate is accepted because it makes the decoder's allocation invariant
stronger and easier to review, exercises malformed backend payloads with recomputed
valid CRC32C, preserves the important error precedence, and shows no material
valid-input regression.

This is **not** a fix for an arbitrary short-header primary-allocation vulnerability.
The pre-B2 decoder already bounded row allocation by available row bytes and checked
exact counter bytes before primary meter construction.

## Canonical provenance

Base main:

`703f8a39a7e4c7183197707fa5a1ed2d557bfe17`

Measured implementation head:

`dfa859508f3be81e5e5c7a7d9683aa2aa6138e63`

Hosted gates:

- Rust `37610509378` — SUCCESS
- snapshot-performance `37610509347` — SUCCESS
- M4 performance `37610509532` — SUCCESS
- M6-A `37610509500` — SUCCESS
- M6-C `37610509495` — SUCCESS

Snapshot-performance artifact:

- name: `snapshot-perf-37610509347-1`
- artifact ID: `11477831160`
- SHA256: `9e167ad7468433a9803923eda695570319ac6292c0b78d70a77702101372c55c`

## Implementation

After the fixed 16-byte Energy metadata is read, B2 now:

1. preserves `ProvenanceRequired` before backend-shape validation when the public
   decoder sees a Proven snapshot without the explicit provenance assumption;
2. validates custom/proven profile metadata and dimension counts;
3. computes checked row bytes and verifies the complete row block is present;
4. computes checked `buckets * tables * sizeof(i64)`;
5. requires the exact full backend payload length;
6. only then allocates and parses hash rows.

The explicit row-block lower-bound check remains before counter-size arithmetic.
That preserves the old fail-closed behavior for short row blocks instead of turning
them into an earlier arithmetic-overflow result.

The existing Energy dimension rules are factored into a count-based private helper
and reused by both normal configuration construction and decode preflight. No
arbitrary snapshot-core `B_max` or `R_max` is added.

Parity production code is unchanged because it already validates config-derived
packed-state bytes before allocating the state vector.

## Malformed-input coverage

New integration fixtures mutate canonical snapshots and then recompute CRC32C with
an independent bitwise reference implementation.

Coverage includes:

- Energy counter-shape mismatch after changing bucket count;
- Energy row-block mismatch after changing table count;
- invalid custom profile tags;
- Proven shape mismatch while preserving public `ProvenanceRequired` precedence;
- Parity packed-state mismatch from a valid config shape requiring more words.

Tests first demonstrate that an unrepaired mutation returns `ChecksumMismatch`;
after CRC recomputation the same input reaches backend validation and fails with the
expected backend error.

No accepted snapshot bytes are changed.

## Hosted measurement

The existing paired AB/BA harness remains the valid-input guardrail. B2 adds one
diagnostic valid-CRC rejection lane with 4095 Energy rows and a mismatched bucket
count. The malformed snapshot is 229,364 bytes.

Valid Energy decode is effectively neutral:

~~~text
energy-default:
  empty       +0.065%
  full-width  +0.052%
  sequential  +0.033%

energy-small:
  empty       +0.190%
  full-width  +0.102%
  sequential  -0.097%
~~~

These values are noise-scale and do not support a positive speed claim.

The malformed structural-rejection lane changes from about 565.3 us to 545.3 us,
a paired median reduction of **3.544%** across four balanced rounds.

Parity/encode lanes remain guardrails and show only noise-scale movement.

## Error precedence

Observable precedence intentionally retained:

- envelope/version/domain/flags/declared-length checks precede CRC;
- CRC precedes backend parsing;
- public decoding of a Proven Energy snapshot without the explicit provenance
  assumption returns `ProvenanceRequired` before Energy shape validation;
- backend structural failures remain `InvalidPayload` where they were already
  represented as such.

No new public `SnapshotError` variant is introduced.

## Verdict

**ACCEPT B2.**

Reason:

- clearer allocation-free shape preflight;
- valid-CRC malformed-input coverage now verifies the intended backend boundary;
- no format/API/theorem change;
- no material valid-decode regression;
- modest diagnostic rejection improvement;
- no dependency, unsafe, SIMD, FFI or transport-policy expansion.

Next M6-B slice: **B3 direct Energy decoded-state construction**, separately
measured. It must not be bundled with further CRC work or sign-mask fusion.
