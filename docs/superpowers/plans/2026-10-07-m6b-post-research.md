# M6-B post-research implementation plan

Issue: #17. Parent: #14.  
Research audit: [M6-B codec/decoder research audit](../../M6-B-CODEC-DECODER-RESEARCH-AUDIT.md).

## Goal

Finish the accepted-direction CRC work, harden snapshot-v1 structural validation,
then test one direct Energy decode candidate that removes redundant initialization
and rescanning. Preserve canonical bytes, public semantics and the existing
checksum-before-backend-decode boundary.

## Constraints

- stable safe Rust;
- no unsafe, FFI, nightly or new runtime dependency;
- one candidate per measured verdict;
- GitHub-hosted runners only;
- no snapshot-v1 reinterpretation;
- no new core transport/admission byte limit;
- no ExactSmallDelta work in this track.

## Slice 1 — finish M6-B1

PR #40 is evidence-complete at its measured head but must be synchronized with
current main before merge because independent M6-D work advanced the base.

Retain only:

- compile-time 256-entry CRC32C table;
- exact Castagnoli/init/final semantics;
- independent bitwise oracle;
- byte-identical snapshot-v1 vectors;
- hosted paired base/head evidence.

After synchronization, rerun exact-head Rust/research/snapshot-performance gates.
Do not add B2/B3 or new CRC algorithms to PR #40.

## Slice 2 — M6-B2 exact structural validation

### Candidate

Energy should compute the complete expected payload shape after fixed metadata and
before allocating the row vector:

~~~text
row_bytes     = checked_mul(table_count, 48)
counter_count = checked_mul(buckets, table_count)
counter_bytes = checked_mul(counter_count, 8)
expected      = checked_add(16, row_bytes, counter_bytes)
~~~

Require exact payload length before row allocation.

Parity already derives config and exact packed-state length before allocating its
word vector. Audit whether only tests/documentation are needed there; do not invent
work to make the slice look symmetric.

### Required behavior

Preserve the existing envelope order:

~~~text
minimum length
magic/version/backend/domain/flags/reserved
declared total length
CRC32C
backend payload validation
~~~

For Energy Proven snapshots, preserve the current `ProvenanceRequired` ordering
unless an intentional change is documented.

No arbitrary `B_max`/`R_max` is added to snapshot v1. Outer wire admission stays
outside the codec.

### Test matrix

Every backend-structure mutation must have CRC32C recomputed so the test reaches
payload validation.

Energy minimum cases:

- zero/non-power-of-two buckets;
- zero/even table count;
- arithmetic-overflow boundaries where representable;
- row block short/long;
- counter block short/long;
- invalid custom profile tags;
- invalid Proven profile tags;
- Proven shape mismatch;
- valid shape with energy-overflow counters.

Parity minimum cases:

- invalid rows/levels/config;
- packed-state short/long;
- non-zero metadata reserved bytes;
- non-canonical final-word padding.

Record exact expected `SnapshotError` where precedence is intentionally frozen.

### Acceptance

B2 is accepted for clearer fail-closed resource semantics and malformed-input
coverage, not for a claimed current short-header OOM fix.

Valid decode must not materially regress. Invalid rejection latency is diagnostic,
not an SLA.

## Slice 3 — M6-B3 direct Energy decoded-state construction

### Candidate

Replace the current decode path:

~~~text
Self::new(config)
overwrite zeroed counters
recompute_energies over counters
replace energies
~~~

with a private decoded-state construction path:

~~~text
counters = Vec::with_capacity(B * R)
energies = Vec::with_capacity(R)

for each row:
    energy = 0
    for each encoded counter:
        c = read_i64()
        counters.push(c)
        x = i128::from(c)
        energy = checked_add(energy, x * x)
    energies.push(energy)

construct EnergyDeltaMeter from validated config + counters + energies
allocate pending exactly once
derive sign masks by the existing accepted path
~~~

Do not bundle sign-mask fusion.

### Correctness invariants

- exact counter order preserved;
- `i64 -> i128` square is always representable;
- checked row-sum overflow preserved at the same partial sum;
- accepted snapshot bytes/re-encode bytes unchanged;
- continued updates after decode unchanged;
- custom/Proven semantics unchanged;
- failure is atomic: no partially constructed public meter escapes.

### Measurement

Use the existing `snapshot-performance` protocol with identical base/head harness
and balanced ordering.

Primary cases:

~~~text
Energy default: B=2048, R=23, 377980 B
Energy small:   B=512,  R=9,   37340 B
~~~

Parity cases remain unchanged guardrails.

If resource instrumentation is added, separate:

- allocation count;
- logical bytes initialized/written/read;
- target-specific `size_of::<PendingUpdate>()`;
- end-to-end latency.

Do not convert logical byte reductions into a DRAM-throughput claim.

### Acceptance

ACCEPT only if correctness is exact and valid Energy decode shows repeatable,
material hosted improvement without a material guardrail regression.

If the result is noise-scale, keep the simpler current construction and record
NO-GO.

## Slice 4 — residual codec selector, only if needed

No further CRC implementation is pre-authorized.

If post-B3 work is still justified, first measure:

- checksum-only B1 CRC32C at 76, 2092, 37340, 377980, 1 MiB and 4 MiB;
- end-to-end encode/decode residual attribution.

Then apply Amdahl reasoning to the **measured** residual fraction.

Only if the maximum plausible end-to-end gain is material may one isolated follow-up
be selected:

- safe scalar slicing-by-4/8; or
- encode-side incremental CRC fusion.

Do not test both in the same production candidate.

Decode-side CRC fusion, CRC-combine machinery for monolithic snapshots,
slicing-by-16, hardware CRC, PCLMUL/PMULL, unsafe and new dependencies remain out of
scope.

## Explicit corrections to external research

The implementation plan intentionally does not adopt these report claims:

- no claim that current Energy decode allocates arbitrary primary state from a short
  malicious payload;
- no fixed "~10.7% residual CRC" number without isolated measurement;
- no exact 33-40% physical memory-traffic claim;
- no `PendingUpdate = 8 bytes` accounting;
- no `B=256,R=18` Energy-small fixture;
- no `rows=16,levels=8` 76-byte Parity fixture;
- no Criterion/benchstat dependency requirement;
- no combining B2 and B3 into one PR.

## Completion

M6-B is complete when:

1. B1 has a synchronized merge verdict;
2. B2 has a separate correctness/resource verdict;
3. B3 has a separate measured performance verdict;
4. any further CRC work is either selected by residual measurement or explicitly
   stopped.

Every retained production change must have exact-head CI and a durable ACCEPT/NO-GO
record.
