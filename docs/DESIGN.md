# Minimal v0 design

The purpose of v0 is to learn quickly without creating an infrastructure project around the estimator.

## Scope

Initial domain:

- key type: `u64`;
- data model: sets;
- set update operation: `toggle(key)`;
- comparison operation: XOR/difference of compatible sketches;
- target output: point estimate plus clearly labelled coverage information.

No generic key codecs, distributed protocol, persistent wire format, keyed security mode, exact recovery subsystem, multi-crate workspace, plugin system, or unsafe SIMD is required in the first implementation slice.

## Planned backends

### EnergyDeltaMeter

Role: proof-friendly reference and strict-capacity backend.

Planned state shape:

```text
R tables
each table: B signed counters
cached table energy T
```

A unit update changes one counter per table. The table energy can be updated incrementally:

```text
T' = T + 2*c*delta + delta^2.
```

For set/signed unit changes, `delta^2 = 1`.

The first Rust version should use the simple theorem-derived `(B,R)` profiles from the research harness. Better tails can replace them later without changing the basic estimator.

### ParityDeltaMeter

Role: compact and fast experimental set-only backend.

The first goal is reproduction, not invention:

1. reproduce the published finite-field behavior;
2. reproduce the expected middle-range error curve;
3. verify finite-(d) moment formulas;
4. then investigate strict finite-sample tails.

No strict `recommended_capacity(delta)` is exposed from Parity until that last step is justified.

## API direction

Do not freeze a wide trait hierarchy yet. Start with concrete types.

Illustrative shape:

```rust
pub struct EnergyDeltaMeter { /* ... */ }
pub struct ParityDeltaMeter { /* ... */ }

pub enum Coverage {
    Proven { failure_probability: f64 },
    Asymptotic,
    Empirical,
}

pub struct DeltaEstimate {
    pub point: f64,
    pub upper: Option<u64>,
    pub coverage: Coverage,
}
```

The exact Rust names can change before the first API freeze.

## Compatibility

In-memory merge must reject incompatible configurations.

At minimum this means equal algorithm, dimensions and seed/configuration material.

We intentionally postpone a serialized wire format until the state layout and practical hash mapping are measured. Prematurely freezing a binary header would make a research prototype harder to correct.

## Randomness

For v0:

- deterministic test seeds;
- a small explicit hash/randomness implementation chosen to match the proof assumptions of each backend;
- no secret keys;
- no cryptographic dependency by default.

The concrete family is an implementation decision for the Energy slice and a separate research decision for Parity tails.

## Performance policy

Order of work:

```text
correct scalar
-> profile
-> reduce allocations / improve layout
-> compiler auto-vectorization
-> stable std::arch only if measurement shows a real bottleneck
```

Nightly `std::simd` is not a baseline dependency.

## Verification policy

Pull requests should prove implementation consistency, not rediscover probability (10^{-9}) through brute-force Monte Carlo.

Fast checks:

- formatting/lints once Rust exists;
- unit/property tests;
- algebra/merge identities;
- deterministic reference vectors;
- theorem-profile generator checks.

Research workflow:

- regenerate profile JSON;
- finite-(d) moment grids;
- moderate Monte Carlo only as a diagnostic;
- performance reports as artifacts, never absolute-nanosecond merge gates.

## Explicit non-goals for v0

- adaptive adversary robustness;
- cryptographic key management;
- Minisketch/IBLT integration;
- FFI;
- persistent protocol compatibility;
- AVX-512-specific code;
- formal verification frameworks;
- multiple packages for components that still fit in one crate.
