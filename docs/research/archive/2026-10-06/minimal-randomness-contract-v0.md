# Snapshot — “Минимальный контракт случайности v0”

Date imported: 2026-10-06.

## Main thesis

The report tries to define a small, reproducible, non-cryptographic randomness contract for the oblivious-input model.

Strong points:

- no secret seed requirement for v0;
- persisted identity must not depend on Rust's default `Hash`;
- canonical byte encoding matters if sketches are serialized;
- chosen-input-after-seed and adaptive-query models must not be silently claimed.

## Independence discussion

The report recommends centering the contract on 2-wise independence and treating 4-wise independence as potentially optional.

Canonical correction:

For the current Energy variance proof the repository retains:

```text
bucket family: pairwise-uniform collisions
sign family:   4-wise independent signs
families:      independent/domain-separated
```

We will not weaken that proof contract merely to simplify the hash implementation.

## Hash implementation discussion

The report mentions UMASH, ChainHash and polynomial families.

Canonical decision:

- do not approve a production hash family from this report alone;
- a collision guarantee or “universal hash” label is not automatically the exact k-wise-independence property used by the proof;
- select the smallest explicit construction only when implementing Energy.

## Persistence

The report proposes an early full persisted header.

Canonical decision:

- keep only in-memory configuration identity in v0 bootstrap;
- define a wire format after estimator layout/API stabilization;
- when persistence is added, algorithm/version, dimensions, randomness suite, seed/config identity and canonical endian encoding become mandatory.
