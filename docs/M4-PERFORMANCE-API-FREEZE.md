# M4 — Performance and API freeze

Status: complete; merged in PR #11 as `4941b17a5fae5d74d629f97df3c03fff3c25c0b6`.  
Issue: #10.  
Evidence date: 2026-10-06.

## Purpose

M4 converts the M1–M3 implementation into a deliberately small v0 library contract without turning research internals, raw memory layout or unstable benchmark observations into compatibility promises.

The freeze is constrained by two existing guarantee classes:

- Energy is the strict finite-sample backend and may return Coverage::Proven;
- Parity is experimental/asymptotic and may return Coverage::Asymptotic only.

M4 does not change either mathematical contract.

## Frozen defaults

### Energy

EnergyProfile::DEFAULT is:

~~~text
RelativeError::TenPercent
FailureTarget::OneInMillion

B = 2048 buckets
R = 23 tables
primary counter state = 376,832 bytes
~~~

The strict guarantee still depends on the documented theorem-facing randomness precondition: exactly 6R independent uniformly distributed u64 words are supplied to EnergyConfig::for_profile_assuming_uniform_words.

The default does not introduce a seeded PRNG.

### Parity

ParityProfile::DEFAULT is ParityProfile::Standard:

~~~text
m = 256 rows
J = 64 stored levels
packed cell state = 2,048 bytes
asymptotic RSE = 1.638 / sqrt(256) = 10.2375%
~~~

This remains an asymptotic model quantity, not a finite-sample epsilon/delta guarantee.

## Public API boundary

The supported external surface is the crate root:

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

The M2 PublishedFpcsaF2 reproduction, its oracle/configuration types and raw packed words remain internal research/implementation details.

The freeze intentionally removes two representation leaks:

- no public ParityDeltaMeter::packed_words();
- no public EnergyConfig::rows().

State-size metadata remains public because it is useful for capacity comparison without defining byte compatibility.

Extensible public enums/result records are #[non_exhaustive] before v0 freeze. Private fields remain private.

## Batch API decision

NO batch API in v0.

Current update operations are scalar and already compose efficiently with caller loops:

~~~text
EnergyDeltaMeter::add_unique(key)
ParityDeltaMeter::toggle(key)
~~~

M4 found no distinct batching primitive that amortizes hashing, changes state layout, or enables vector lanes. Adding *_many now would freeze redundant API rather than performance capability.

Reopen batching only when an implementation has a measurable reason, for example SIMD/vectorized hashing, parallel lanes or another amortized state transition.

## Performance methodology

examples/m4_bench.rs is dependency-free and stable-Rust.

For pull requests, .github/workflows/perf.yml:

1. checks out the PR base;
2. injects the same M4 benchmark source;
3. runs a warm-up and repeated samples;
4. checks out the PR head on the same runner;
5. repeats the same workload;
6. uploads base/head text artifacts.

This removes cross-runner hardware mismatch from the primary comparison. It does not eliminate frequency scaling, virtualization noise, ordering effects or the limitations of a small microbenchmark.

No benchmark number below is a throughput promise.

## Final paired evidence

Representative final code run: GitHub Actions performance run #14, code head 65fa5f04d204015fa050de455f554bacf1894fc2.

Environment:

~~~text
GitHub-hosted ubuntu runner
AMD EPYC 9V74
rustc 1.99.0
keys = 2,000
samples = 5
merge repeats = 64
query repeats = 1,024
~~~

| Operation | Backend | Base ns/op | Head ns/op | Interpretation |
| --- | --- | ---: | ---: | --- |
| update | Energy | 7031.847 | 7023.192 | unchanged; no update optimization claimed |
| update | Parity | 21.099 | 15.100 | incidental run delta; no update-path claim |
| merge | Energy | 65362.500 | 65017.734 | unchanged; merge rewrite was rejected/reverted |
| merge | Parity | 225.359 | 209.078 | incidental run delta; no merge-path claim |
| query | Energy | 130.474 | 132.304 | unchanged; stack-scratch candidate was rejected/reverted |
| query | Parity | 12630.841 | 183.068 | accepted: about 69x faster, about 98.55% lower time |

Only the Parity query delta is attributed to a deliberate retained optimization.

## Retained Parity query optimization

Before M4, the published-statistic query path:

- allocated Vec<Option<u8>> for all row highest levels;
- for each built-in J=64 row, searched levels 64..1 one bit at a time.

For J=64, one row occupies exactly one u64. M4 therefore:

- scans rows without building an intermediate Vec;
- loads the row word directly;
- uses leading_zeros to obtain the highest set level;
- accumulates empty-row count and level sum in one pass.

The generic non-J=64 research path keeps the scalar fallback.

No unsafe code, explicit SIMD, nightly feature or new dependency is introduced.

## Rejected performance candidates

M4 also tested two Energy changes:

1. a one-pass/non-mutating difference rewrite plus transactional in-place subtraction;
2. stack scratch space for the small built-in table-count median query.

The merge rewrite did not improve the paired benchmark. The stack-scratch query result changed sign across repeated paired hosted-runner measurements. Both were reverted.

This is intentional: M4 optimizes measured bottlenecks, not code that merely looks faster.

## Verification and freeze guards

PR CI now requires:

~~~bash
python3 research/energy_profiles.py --check-csv research/energy_profiles.csv
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo check --release --examples
~~~

tests/public_api.rs compiles and exercises the supported root surface.

The M4 performance workflow is diagnostic and artifact-producing, not a threshold gate.

## Persisted serialization decision

NOT FROZEN.

M4 deliberately does not expose raw Parity words as a supported persistence mechanism and does not define an Energy/Parity byte format.

A future persisted format requires a separate requirement and must version algorithm, dimensions, randomness/config identity and canonical encoding. It is not automatically part of v0.

## Acceptance verdict

M4 acceptance was completed and merged with:

- Rust CI green;
- paired performance workflow green;
- Energy guarantee unchanged;
- Parity guarantee unchanged;
- default profiles explicit;
- batch API decision recorded;
- unsupported research/layout surface removed before freeze;
- no persisted wire format frozen.
