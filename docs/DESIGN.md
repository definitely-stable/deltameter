# Minimal v0 design

The purpose of v0 is to learn quickly without creating an infrastructure project around the estimator.

## Scope

Initial domain:

- key type: `u64`;
- data model: sets;
- Energy source-set ingestion: `add_unique(key)`, then `difference(other)`;
- Parity set update operation: `toggle(key)`;
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

The reference implementation uses affine/cubic polynomials over GF(2^64) and accepts exactly six independent uniformly random `u64` words per row (two bucket coefficients + four sign coefficients). DeltaMeter does not ship a seeded PRNG in M1 because a small deterministic seed would not automatically satisfy the information-theoretic independence contract.

### PublishedFpcsaF2

Role: research reproduction.

M2 implements the published finite-field GF(2) FIELDMAP state with:

- packed one-bit field cells;
- one-based geometric levels;
- a separate uniform GF(2) coefficient bit;
- explicit finite level boundary J;
- XOR composition;
- highest/rightmost non-zero level extraction.

The implementation uses a deterministic pseudo-oracle for reproducibility. It is not claimed to instantiate the ideal random oracle assumed by the paper.

The rounded Table 2 constants remain diagnostic/asymptotic only and do not create a finite-sample or Proven coverage contract.

### ParityPcsaSetV1

Role: possible later set-specialized experiment.

Candidate update:

```text
cell[index] ^= 1
```

with `g(v)=1`.

This is **not** assumed equivalent to published F-PCSA statistically. It receives no inherited (1.638/sqrt m) claim.

### ParityDeltaMeter

M3 wraps the published reproduction; it does not fork or rewrite the F-PCSA mechanics.

Public experimental shape:

~~~text
ParityProfile::{Compact, Standard, Accurate}
ParityConfig::{for_profile, new}
ParityDeltaMeter::{toggle, xor_assign, xor_merged, estimate}
~~~

Built-in profiles use 64 stored levels and rows 64/256/1024, corresponding to published asymptotic RSE scales about 20.475% / 10.2375% / 5.11875%.

The convenience u64 seed is deterministically expanded into the M2 pseudo-oracle domains. It is for reproducibility, not a proof that the ideal-random-oracle model has been instantiated.

Strict `recommended_capacity(Proven)` is unavailable.

## M4 v0 API freeze

The supported library surface is the crate root. Research modules are implementation details.

Frozen v0 root types:

~~~text
Coverage

EnergyConfig
EnergyDeltaMeter
EnergyError
EnergyEstimate
EnergyProfile
EnergyRowHash
FailureTarget
RelativeError

ParityConfig
ParityDeltaMeter
ParityError
ParityEstimate
ParityProfile
ParityUpdate
~~~

PublishedFpcsaF2 and its configuration/oracle/update types remain the M2 research reproduction used behind ParityDeltaMeter; they are not frozen as external API.

ParityDeltaMeter does not expose raw packed words. EnergyConfig does not expose its internal row slice. State size metadata may be inspected, but no byte-level persistence contract exists.

Public enums and result records that may need compatible extension are #[non_exhaustive]. Configuration and meter structs keep private fields.

Default profile policy:

~~~text
EnergyProfile::DEFAULT
    epsilon = 10%
    failure probability <= 1e-6
    Coverage::Proven, subject to the documented randomness precondition

ParityProfile::DEFAULT
    Standard
    m = 256
    J = 64
    asymptotic RSE ~= 10.2375%
    Coverage::Asymptotic
~~~

The defaults are not comparable guarantee classes and are not presented as equivalent accuracy promises.

No batch update API is frozen in v0. Both backends already support ordinary caller loops, and M4 found no backend-specific batching mechanism that would justify additional API surface. A later SIMD/vectorized or amortized implementation may reopen that decision with measurements.

## Coverage model

The frozen direction is:

```rust
#[non_exhaustive]
pub enum Coverage {
    Proven { failure_probability_upper_bound: f64 },
    Asymptotic { relative_standard_error: f64 },
}
```

Rules:

- `Proven` requires a finite-sample theorem matching the concrete implementation.
- asymptotic constants stay `Asymptotic`;
- empirical calibration, if ever added, requires a separate explicit contract rather than reusing `Proven`.

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

M4 retained only a demonstrated scalar optimization: for the built-in Parity J=64 layout, each row is one u64, so the highest non-zero level is obtained with leading_zeros and estimate accumulation is allocation-free. Candidate Energy merge and stack-scratch query rewrites were reverted because paired hosted-runner measurements did not show a stable benefit.

Performance evidence is diagnostic. It may justify implementation choices, but it does not become a throughput/SLA promise.

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
