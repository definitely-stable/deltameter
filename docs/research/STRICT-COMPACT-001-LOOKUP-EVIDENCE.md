# STRICT-COMPACT-001 — 32 KiB lookup prototype evidence

Issue: #59. PR: #62.

Status: **LOOKUP_PROTOTYPE_PASS**. This is implementation/performance evidence only;
it is not yet the independent numerical certificate required for a public Proven
backend.

## Frozen profile

- rows m = 4096
- stored levels J = 64
- primary packed state = 32,768 bytes
- delta = 1e-6
- equal Bonferroni alpha = delta/64
- normalized threshold table = 2,048 u64 slots
- finite thresholds = 1,853
- infinity sentinels = 195
- static table bytes = 16,384

The Q32 table generator rounds normalized thresholds upward and rechecks the
emitted integer with 80-digit Decimal evaluation.

Canonical table SHA-256 from hosted generation:

`634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`

Generation is deterministic across two independent invocations in the same hosted
job.

## Actual packed-state Rust path

The private Rust lab reuses `src/fpcsa.rs` directly. No synthetic count vector is
used for the primary performance path:

1. build the actual 4096x64 packed parity state;
2. extract all 64 odd-row counts from 4096 row-major u64 words;
3. perform Q32 table lookup, power-of-two rescaling and min reduction using integer
   arithmetic only.

Hosted lookup-lab #6 / run `37719566521`:

~~~text
STRICT_COMPACT_LOOKUP_READY
  rows=4096
  levels=64
  state_bytes=32768
  table_bytes=16384
  finite_entries=1853

d=4096
  bound=5184
  ratio=1.265625
  counts_ns=1677
  lookup_ns=132
  total_ns=1526

d=65536
  bound=85370
  ratio=1.302643
  counts_ns=3930
  lookup_ns=122
  total_ns=3973

d=262144
  bound=324882
  ratio=1.239326
  counts_ns=4749
  lookup_ns=115
  total_ns=4808

STRICT_COMPACT_LOOKUP_PASS
~~~

Timing is descriptive single-run hosted evidence, not a hard performance gate.
Nevertheless it establishes that runtime inference itself is tiny relative to the
existing update path: the lookup/min step is ~0.1 microsecond and total extraction
plus inference is only a few microseconds on these fixtures.

## Interpretation

The implementation shape is viable:

- no exp/log/KL in the hot path;
- no heap allocation is required for the 64 level counters;
- one 16 KiB immutable table serves every level because the threshold depends on
  normalized d/2^j;
- Q32 rescaling uses u128 intermediates and upward rounding;
- the primary state remains the existing 32 KiB GF(2) packed state.

The remaining blocker is **not runtime cost**.

It is proof closure:

1. independently certify every committed Q32 threshold;
2. prove the conservative integer rescaling contract end to end;
3. close a declared useful d-range rather than sampled anchors;
4. define saturation/unavailable semantics;
5. separate ideal-oracle probability theorem from deterministic pseudo-oracle
   engineering assumptions.

Until those items close, `ParityDeltaMeter::estimate()`,
`Coverage::Asymptotic`, snapshot v1 and public profiles remain unchanged.

## Next slice

Proceed to a dedicated certification/range-closure slice.

The correct next decision is no longer whether lookup inference is practical; it is
whether the 32 KiB candidate can be turned into an auditable finite-sample contract
over a useful declared range.
