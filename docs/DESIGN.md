# Minimal v0 design

The purpose of v0 is to learn quickly without creating an infrastructure project around the estimator.

## Scope

Initial domain:

- key type: `u64`;
- data model: sets;
- set update operation: `toggle(key)`;
- target output: point estimate plus explicitly labelled coverage information.

No generic key codecs, distributed protocol, persistent wire format, keyed-security mode, exact-recovery subsystem, multi-crate workspace, plugin system or unsafe SIMD is required in the first implementation slice.

## Backend order

### EnergyDeltaMeter

Role: first implementation and strict finite-sample backend.

State shape:

```text
R tables
each table: B signed counters
cached table energy T
```

A unit update changes one counter per table.

```text
T' = T + 2*c*delta + delta^2
```

For unit signed changes, `delta^2 = 1`.

Initial randomness contract:

```text
bucket hash: pairwise-uniform collisions
sign hash:   4-wise independent signs
independent families
```

### PublishedFpcsaF2

Role: research reproduction.

This must match the published finite-field GF(2) construction, including its coefficient randomness and level/register mapping.

Its published asymptotic claims belong only to this reproduced construction.

### ParityPcsaSetV1

Role: possible later set-specialized experiment.

Candidate update:

```text
cell[index] ^= 1
```

with `g(v)=1`.

This is **not** assumed equivalent to published F-PCSA statistically. It receives no inherited (1.638/sqrt m) claim.

### ParityDeltaMeter

The public experimental backend should initially wrap the published reproduction, not the unproved set-specialized variant.

Strict `recommended_capacity(Proven)` is unavailable.

## Coverage model

Illustrative direction:

```rust
pub enum Coverage {
    Proven { failure_probability: f64 },
    Asymptotic,
    Empirical,
}
```

Rules:

- `Proven` requires a finite-sample theorem matching the concrete implementation.
- asymptotic constants stay `Asymptotic`;
- Monte Carlo calibration stays `Empirical`.

## Compatibility

In-memory merge/difference rejects incompatible configurations.

At minimum:

- algorithm;
- dimensions;
- seed/randomness configuration.

No persisted wire format is frozen yet.

## Threat model

v0: oblivious input.

No secret seed requirement. No chosen-input or adaptive-query guarantee.

## Performance policy

```text
correct scalar
-> profile
-> reduce allocations/layout cost
-> compiler auto-vectorization
-> stable std::arch only if measurement justifies it
```

Nightly `std::simd` is not a baseline dependency.

## Verification policy

PR:

- deterministic unit/property/algebra tests;
- reference vectors;
- theorem/profile checks.

Research workflow:

- wider exact grids;
- moderate Monte Carlo diagnostics;
- profile regeneration;
- benchmark artifacts.

Rare-event Monte Carlo is never a substitute for the theorem behind `Coverage::Proven`.

## Explicit non-goals for v0

- adaptive adversary robustness;
- cryptographic key management;
- Minisketch/IBLT integration;
- FFI;
- persistent protocol compatibility;
- AVX-512-specific code;
- formal verification framework;
- multiple packages for components that still fit in one crate.
