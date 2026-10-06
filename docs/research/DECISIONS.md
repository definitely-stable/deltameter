# Decision record 0001 — pre-implementation baseline

Status: **accepted for bootstrap**  
Updated: 2026-10-06 after the latest Q1 research and attached reports.

## Context

DeltaMeter needs a small, composable estimator for symmetric-difference size. The project is maintained by one developer and uses ordinary public GitHub-hosted CI.

The main architectural risk is not lack of infrastructure. It is accidentally turning an asymptotic or empirical observation into a public mathematical guarantee.

## Decisions

### D1 — Primary mathematical model

For true sets:

```text
d = |A △ B| = ||A XOR B||_0
```

over GF(2).

The integer F2 representation remains a valid reference path, not the sole framing.

### D2 — First implementation backend

Implement **EnergyDeltaMeter first**.

Reason:

- simple finite-sample proof path;
- exact specialized variance under explicit assumptions;
- incremental energy statistic;
- useful reference for later Parity validation.

### D3 — Q1 strict-Parity gate

For v0, **NO-GO for strict Parity `Coverage::Proven`**.

Parity may expose:

- point estimate;
- asymptotic metadata;
- empirical/calibrated diagnostics if clearly labelled.

Parity may not expose a proven high-confidence capacity API until a concrete finite-sample theorem exists.

### D4 — Published F-PCSA and set-specialized parity are separate

The first Parity implementation must reproduce the **published finite-field construction**:

```text
FIELDMAP[i,j] in F
h(v) chooses row/level with published probability mass
g(v) is uniform over F
row statistic = highest non-zero level
```

Classical PCSA first-1/bitmap models are intuition only, not the proof object.

A proposed specialization with `g(v)=1` / `cell ^= 1` is a separate research algorithm, tentatively named `ParityPcsaSetV1`.

It inherits no published `1.638/sqrt(m)` claim automatically.

### D5 — Exact small-d recovery

Not in v0.

PinSketch/Minisketch is recorded as a strong future candidate, but no FFI/BCH decoder is added before Energy/Parity measurements show a real product need.

### D6 — Threat model

v0 supports the ordinary oblivious-input randomized-algorithm model.

A secret seed is not required for that theorem model.

Chosen-input-after-seed and adaptive-query robustness are separate future work.

### D7 — Randomness

For Energy, keep the proof contract explicit:

```text
bucket family: pairwise-uniform collisions
sign family:   4-wise independent signs
families:      independent/domain-separated
```

Do not weaken the sign requirement merely to simplify implementation.

Parity randomness remains unfrozen beyond reproduction requirements until its strict-tail path is known.

No cryptographic dependency is required by default.

### D8 — Hash implementation

Do not approve a hash family by brand name alone.

“Universal hash”, collision bounds, PRNG quality and exact k-wise independence are different properties.

The implementation must choose the smallest explicit construction that matches the estimator proof.

### D9 — Serialization

Do not freeze a persisted wire format during bootstrap.

First freeze the estimator state and configuration.

When persistence is later added, it must include at least:

- algorithm/version;
- domain;
- dimensions;
- randomness/hash-suite identity;
- seed/config identity;
- canonical endian-independent encoding.

Rust's default `Hash` is never a persisted identity.

### D10 — Rust shape

- one crate;
- concrete types before broad traits;
- `u64` keys first;
- safe scalar implementation first;
- stable Rust;
- no nightly SIMD requirement.

### D11 — Verification

Use:

- executable mathematics;
- deterministic reference vectors;
- property tests;
- exact small-case enumeration where useful;
- theorem-derived configuration tests.

Monte Carlo is diagnostic, not proof of `1e-6` or `1e-9` failure probabilities.

### D12 — CI

Use public GitHub-hosted runners normally.

Two layers:

```text
PR:
    fast deterministic checks

research/manual/main:
    wider finite-d grids
    moderate Monte Carlo diagnostics
    profile regeneration
    benchmark artifacts
```

The public repository removes quota pressure, but it does not make billion-trial rare-event simulation a mathematically useful proof strategy.

### D13 — M3 Parity public contract

M3 may expose the published GF(2)-F-PCSA reproduction through an experimental public wrapper.

Contract:

~~~text
Parity estimate:
    point estimate
    + Coverage::Asymptotic { relative_standard_error }

Parity capacity:
    no Coverage::Proven
    no recommended_capacity
~~~

Built-in Parity profiles are engineering memory/RSE scales, not strict epsilon/delta profiles.

The public u64 seed is a reproducibility input for the deterministic pseudo-oracle. It is not a cryptographic key and is not claimed to instantiate the paper's ideal random oracle.

Energy/Parity memory comparisons must keep their guarantee mismatch explicit.

### D14 — Formal verification and FFI

Not v0 requirements.

No Verus/Alerus/Creusot gate and no Minisketch FFI unless a concrete later need justifies the maintenance surface.

## Immediate implementation sequence

```text
M0 research bootstrap
-> M1 EnergyDeltaMeter
-> M2 published F-PCSA reproduction
-> experimental ParityDeltaMeter
-> performance/API freeze
-> optional continued strict-Parity research
```

Strict Parity theory is no longer a blocker for beginning the useful Rust crate.

The newest “От Эвристики к Доказательству” report does not change this decision. It sharpens the future proof program, especially exact-state analysis and rigorous de-Poissonization, while also confirming that asymptotic/empirical shortcuts are not enough.
