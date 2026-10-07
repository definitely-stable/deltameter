# M6-B3 — direct Energy decoded-state construction plan

Issue: #17. Parent: #14. Baseline: `099cceb43e2d76a4b2b0890ecefc155965511cee`.

## Objective

Remove redundant Energy snapshot-decode work after accepted B2 without changing
snapshot-v1 bytes, public semantics, theorem assumptions, checksum ordering or
sign-hash behavior.

B3 is one isolated candidate. No CRC, row/sign-mask fusion, transport policy or
format work is bundled.

## Current decode cost

After B2 has validated the complete payload shape, current Energy decode still:

1. constructs `Self::new(config)`;
2. zero-initializes the complete counter array;
3. zero-initializes the energy cache;
4. allocates pending-update scratch;
5. derives sign masks;
6. overwrites every zeroed counter from the snapshot;
7. scans every counter again in `recompute_energies`;
8. allocates a replacement energy cache.

For the default shape `B=2048, R=23`, the primary counter array is 47,104 i64
values = 376,832 bytes. At the source-level logical-touch model, direct decode can
remove one 376,832-byte zero-write and one 376,832-byte post-read: 753,664 bytes
(736 KiB) of redundant logical counter-state touches.

This is not a physical DRAM/cache-traffic claim.

## Candidate

Parse counters directly into capacity-only storage while accumulating each row's
energy in the same row-major order as the existing recomputation:

~~~text
counters = Vec::with_capacity(B * R)
energies = Vec::with_capacity(R)

for each table:
    energy = 0
    for each bucket:
        c = read_i64()
        counters.push(c)
        x = i128::from(c)
        energy = checked_add(energy, x * x)
    energies.push(energy)

construct the final private meter from validated config + counters + energies
derive the existing sign masks once
allocate pending scratch once
~~~

An individual i64 square is always representable in i128:

~~~text
max |c| = 2^63
c^2 <= 2^126 < 2^127
~~~

Only the row sum needs checked addition.

## Correctness invariants

- counter order is unchanged;
- energy accumulation order is identical to `recompute_energies`;
- row-sum overflow remains `SnapshotError::InvalidPayload`;
- no partially constructed public meter escapes on failure;
- custom/Proven configuration semantics are unchanged;
- `ProvenanceRequired` precedence remains B2 behavior;
- sign masks are derived by the accepted M6-C implementation and are not fused with parsing;
- pending scratch is still created with exactly one entry per table;
- accepted snapshots re-encode byte-for-byte identically;
- continued updates after decode remain equivalent.

## Required tests

Retain the entire snapshot-v1 compatibility suite and add one valid-CRC Energy
fixture whose counters overflow the cached row-energy sum.

For B=2/R=1, two `i64::MIN` counters each square to `2^126`; adding the second
term overflows signed i128. The new path must reject that snapshot with
`SnapshotError::InvalidPayload`, matching the old recomputation behavior.

## Measurement

Use the existing GitHub-hosted `snapshot-performance` paired AB/BA workflow.
The identical harness is injected into base/head.

Primary valid-decode lanes:

~~~text
Energy default: B=2048, R=23, 377980 bytes
Energy small:   B=512,  R=9,   37340 bytes
~~~

Parity and encode lanes are unchanged guardrails. B2's malformed rejection lane is
also retained as a guardrail.

## Predeclared acceptance gate

ACCEPT only if exact-head correctness is green and the first valid paired run shows:

- at least 3% lower decode latency in each of the three Energy-default content lanes;
- no Energy-small decode lane regresses by more than 2%;
- no systematic >3% regression appears in unchanged Parity or encode guardrails.

If one isolated lane violates a timing gate while unchanged guardrails show
comparable runner/process drift, rerun the same exact head once. After such a rerun,
accept only if the aggregate median Energy-default reduction across both attempts is
at least 3% and no repeated material regression is attributable to B3.

No absolute nanosecond SLA is created.

## Explicit non-goals

- further CRC slicing/fusion;
- sign-mask construction fusion;
- changing `EnergyConfig` public API;
- snapshot-v2;
- transport admission limits;
- unsafe, SIMD, FFI, nightly or new runtime dependencies;
- ExactSmallDelta work.

## Stop rule

If B3 does not produce a repeatable material valid-decode improvement, retain the
simpler accepted B2 construction and record B3 NO-GO. Do not compensate by bundling
another optimization.
