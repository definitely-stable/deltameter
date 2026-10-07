# M6-D1 — Pure-Rust PinSketch64 lab reference

Status: complete in PR #25; **ACCEPT as private lab reference, production NO-GO**. Issue: #23. Parent: #19 / #14.

## Goal

Build a private, independently implemented PinSketch64-style reference that can answer one narrow question:

Can a full-u64, fixed-capacity exact-small sketch recover the symmetric difference reliably enough, and cheaply enough, to justify further DeltaMeter work?

This is not a production backend and exposes no public crate API.

## Representation

For configured total capacity c:

- nonzero u64 keys are exact GF(2^64) field elements;
- key zero is one separate XOR-composable presence bit;
- store c odd power sums S1, S3, ..., S(2c-1);
- keep stored syndrome capacity distinct from the explicit maximum decoded element count;
- accepted lab experiments reserve one extra 64-bit syndrome as an over-capacity guard;
- merging sketches XORs every syndrome and the zero bit.

Lab serialization is separate from snapshot v1 and has no compatibility promise.

## Decoder

1. Reconstruct even power sums with Frobenius squaring.
2. Run Berlekamp-Massey to obtain the locator recurrence.
3. Reverse it into the monic root polynomial.
4. Factor deterministically over GF(2^64) using absolute-trace splitting.
5. Use the 64 polynomial-basis coefficients as deterministic split candidates; no RNG is required in the lab reference.
6. Recover nonzero roots, add zero from the separate bit, sort, and enforce total candidate count <= c.
7. Recompute the same lab sketch as an internal consistency check.

Important: step 7 is not independent final verification. Over-capacity aliases can satisfy the same syndrome equations. Integration tests compare against the exact symmetric-difference oracle.

## Correctness gates

- existing GF(2^64) reference vectors;
- field inverse identities;
- polynomial division/GCD sanity;
- trace split recovers deterministic known root sets;
- exact decode for capacities 1, 2, 4 and 8;
- d=0 and d=c;
- zero-only and zero plus nonzero cases;
- symmetric-difference merge;
- full-width/high-bit/u64::MAX keys;
- deterministic randomized grid of in-capacity sets;
- d>c inventory records failure versus false-success candidate without treating either as exact success;
- malformed lab serialization fails closed.

## Performance/evidence gates

Hosted release harness reports separately:

- build/update time;
- merge time;
- decode time;
- serialized lab sketch bytes;
- direct exact-list bytes;
- maintained-state versus cold-build interpretation.

No absolute performance threshold.

## Boundaries

- stable Rust only;
- no unsafe;
- no FFI;
- no dependency;
- no public root export;
- no snapshot-v1 changes;
- no Coverage::Exact;
- capacities above the audited lab ceiling require a new decision;
- production final verification is out of scope.


## Completion record

Final measured implementation head: `7b610a1e99e49a08e8aaf923000b329ccfc6d3db`.

Hosted evidence: `m6d1-lab #15 / 37523031317`.

Canonical evidence: [M6-D1 PinSketch64 evidence](../../M6-D1-PINSKETCH64-EVIDENCE.md).

Verdict:

- private pure-Rust reference: ACCEPT;
- unguarded `stored_capacity == max_elements`: REJECT;
- guarded `stored_capacity = max_elements + 1`: retain for future lab work;
- production/public exact backend: NO-GO;
- next candidate: guarded incremental syndrome-prefix experiment.
