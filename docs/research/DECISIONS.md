# Decision record 0001 — pre-implementation baseline

Status: **accepted for bootstrap**, revisable before public API freeze.

Date: 2026-10-06.

## Context

DeltaMeter needs a small, composable estimator for symmetric-difference size. The project is maintained by one developer and uses normal public GitHub-hosted CI.

The main failure mode to avoid is premature infrastructure: multiple backends, security modes, protocol layers, formal-verification plumbing and performance kernels before the estimator itself is validated.

## Decisions

### D1 — Primary mathematical model

For true sets, the canonical model is

[
d=|A\triangle B|=\|A\oplus B\|_0
]

over GF(2).

The equivalent integer F2 formulation remains a reference, not the sole framing.

### D2 — First implementation backend

Implement **EnergyDeltaMeter first**.

Reason:

- simple finite-sample proof path;
- exact specialized variance;
- easy incremental query statistic;
- useful oracle for later Parity validation.

### D3 — Second backend

Implement **ParityDeltaMeter second** as the compact GF(2) candidate.

Strict high-confidence capacity remains disabled until a finite-sample theorem or wrapper justifies it.

### D4 — Small-d exact recovery

Not in v0.

Reconsider after Energy and Parity measurements.

### D5 — Threat model

v0 supports the ordinary oblivious-input randomized-algorithm model.

No default cryptographic hash/key-management subsystem.

Chosen-input and adaptive-query robustness are separate future work.

### D6 — Randomness

Use the smallest explicit family that matches each proof.

Do not substitute “cryptographic” for “mathematically sufficient” without a concrete reason.

### D7 — Serialization

Do not freeze a persisted wire format during the research bootstrap.

First freeze the actual state/configuration that survives implementation and measurement.

### D8 — Rust shape

- one crate;
- concrete types before broad traits;
- `u64` keys first;
- safe scalar implementation first;
- stable Rust;
- no nightly SIMD requirement.

### D9 — Performance work

Profile before adding SIMD, unsafe or batch complexity.

### D10 — Verification

Use executable mathematics, reference vectors, property tests and theorem-derived configuration tests.

Do not use Monte Carlo as the proof of (10^{-6}) or (10^{-9}) failure probabilities.

### D11 — CI

Use ordinary GitHub-hosted public runners.

Optimize for simplicity and reproducibility, not artificial runner scarcity.

## Consequences

The immediate implementation sequence is:

```text
research bootstrap
-> EnergyDeltaMeter
-> ParityDeltaMeter reproduction
-> finite-sample Parity GO/NO-GO
-> performance/API freeze
```

The following are intentionally absent:

- Minisketch FFI;
- IBLT/BCH recovery;
- secret-key modes;
- adaptive-adversary framework;
- multi-crate workspace;
- formal verifier in the merge gate;
- persisted protocol;
- hand-written SIMD.
