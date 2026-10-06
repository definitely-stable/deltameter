# M3 — Experimental ParityDeltaMeter

Status: implementation active in issue #6 / branch `m3/experimental-parity`.

## Objective

M3 turns the source-faithful M2 reproduction into a deliberately small user-facing experimental backend.

It does **not** change the Q1 decision. Parity remains asymptotic/experimental and has no finite-sample one-sided capacity guarantee.

## Public shape

The intended API is:

~~~text
ParityProfile
    Compact
    Standard
    Accurate

ParityConfig::for_profile(profile, seed)
ParityConfig::new(rows, stored_levels, seed)

ParityDeltaMeter::new(config)
ParityDeltaMeter::toggle(key)
ParityDeltaMeter::xor_assign(other)
ParityDeltaMeter::xor_merged(other)
ParityDeltaMeter::estimate()
~~~

The wrapper delegates state mechanics to `PublishedFpcsaF2`; it does not introduce a second F-PCSA implementation.

## Coverage contract

M3 extends the shared coverage enum with:

~~~text
Coverage::Asymptotic {
    relative_standard_error
}
~~~

Energy continues to return:

~~~text
Coverage::Proven {
    failure_probability_upper_bound
}
~~~

Parity never returns `Coverage::Proven` in M3 and exposes no `recommended_capacity` API.

The asymptotic RSE metadata is the rounded published GF(2) Table-2 constant:

~~~text
RSE ≈ 1.638 / sqrt(m)
~~~

The implementation remains subject to the boundaries documented in M2: deterministic pseudo-oracle rather than an ideal random oracle, finite J, no random-offset implementation, and no finite-sample theorem.

## Experimental profiles

All built-in profiles use `J = 64`.

| Profile | Rows m | Packed state | Published asymptotic RSE |
| --- | ---: | ---: | ---: |
| Compact | 64 | 512 B | 20.475% |
| Standard | 256 | 2,048 B | 10.2375% |
| Accurate | 1,024 | 8,192 B | 5.11875% |

The names describe an engineering memory/accuracy scale only. They are not strict relative-error profiles.

## Energy comparison

For comparison, the theorem-backed Energy profiles at `delta = 1e-6` use:

| Approximate scale | Energy strict profile | Energy counter state | Parity experimental profile | Parity packed state |
| --- | --- | ---: | --- | ---: |
| ~20% | epsilon=20%, delta=1e-6 | 94,208 B | Compact, RSE≈20.475% | 512 B |
| ~10% | epsilon=10%, delta=1e-6 | 376,832 B | Standard, RSE≈10.2375% | 2,048 B |
| ~5% | epsilon=5%, delta=1e-6 | 1,507,328 B | Accurate, RSE≈5.11875% | 8,192 B |

The state ratio in this selected comparison is 184x.

That number must not be read as “Parity is 184x better”. The guarantees are different:

- Energy: finite-sample relative-error event amplified to a theorem-backed failure probability;
- Parity: asymptotic RSE metadata for the published middle-range estimator.

The comparison exists to motivate measurement, not to collapse the two contracts.

## Seed and compatibility contract

M3 accepts one public `u64` seed for reproducibility.

The seed is deterministically domain-separated into row, level, and coefficient pseudo-oracle seeds.

This is not a cryptographic construction and is not claimed to instantiate the paper's ideal random oracle.

Two Parity sketches may XOR-merge only when their complete configuration matches:

- rows;
- stored levels J;
- seed/pseudo-oracle mapping.

Mismatch fails closed.

## Update semantics

`toggle(key)` preserves the GF(2) set semantics and returns one of:

~~~text
Stored { row, level }
ZeroCoefficient { row, level }
Truncated { row }
~~~

The result keeps the finite-state boundary visible to callers.

Repeated toggles of the same key cancel.

## Estimate availability

The published Table-2 reference estimator is a middle-range object.

M3 therefore fails closed when the M2 backend reports empty rows instead of inventing a small-d fallback.

That gap is intentional. Exact-small-d recovery remains a later research/product decision.

## Benchmark harness

A dependency-free stable-Rust harness is provided:

~~~bash
cargo run --release --example m3_compare -- 100000
~~~

It reports:

- exact state bytes;
- update elapsed time / ns per update;
- merge time;
- repeated query time.

The harness uses deterministic benchmark inputs. Its Energy coefficient words are reproducible pseudo-random test material and are not used to claim theorem randomness.

No absolute timing result is committed as a product promise. Performance numbers become evidence only when reproduced on named hardware/toolchain.

## Acceptance

M3 is ready when:

- Parity is exposed through the small wrapper API;
- coverage is explicitly Asymptotic;
- Energy remains Proven;
- merge compatibility is fail-closed;
- truncation stays visible;
- profile memory/RSE values are tested;
- benchmark harness compiles in CI;
- docs preserve the guarantee distinction;
- all Rust and research checks are green.

## Handoff to M4

M4 may measure and optimize both backends, select practical defaults, and freeze the public API.

M4 must not silently promote the Parity asymptotic metadata into a strict capacity contract.
