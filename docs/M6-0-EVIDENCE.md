# M6-0 evidence

Status: **ACCEPT** the single-buffer snapshot encoder.  
PR: #20. Issue: #15. Evidence date: 2026-10-06.

## Reproducible run

Snapshot-performance run #11 (`37509992741`):

~~~text
base  1ae3be55df218421437c56271a7d9a9f660b0ff7
head  68147d703d15d407ac3b29244699d40ac18c59d3
CPU   AMD EPYC 7763 64-Core Processor
Rust  1.99.0
order AB / BA / AB / BA
~~~

Artifact: `snapshot-perf-37509992741-1`. It contains raw rounds, refs, CPU/compiler metadata, compatibility logs, source hashes and `summary.csv`.

Base and head both pass the injected snapshot-v1 compatibility suite: 13 passed, 0 failed.

## Timing verdict

Median paired deltas across the three content fixtures:

| Path | Delta |
| --- | ---: |
| Energy default encode | about -0.27% |
| Energy small encode | about -0.76% |
| Parity Standard encode | about -2.47% |
| Parity padded encode | about -21.7% |
| all decode controls | effectively unchanged |

Default Energy encode is therefore **neutral**, not a speedup claim. The small Parity case has a large relative gain but tiny absolute cost.

## Resource result

Old encoding built a backend payload Vec and copied it into a second envelope Vec. M6-0 writes directly into one pre-sized envelope and validates the actual payload length before CRC32C.

Logical temporary payload/copy removed:

- Energy default: 377,952 bytes;
- Energy small: 37,312 bytes;
- Parity Standard: 2,064 bytes;
- Parity padded: 48 bytes.

These are logical payload bytes, not allocator-capacity or process-RSS claims.

## Compatibility result

M6-0 preserves snapshot-v1 bytes and meaning. It adds independent signed-Energy and multiword-Parity vectors, continuation after decode, all Energy profile shapes, and exact writer length/overflow tests.

Unchanged: public API, decoder semantics, CRC32C, Energy theorem assumptions, Parity semantics and coverage classes.

## Decision

Retain the candidate because it removes a redundant buffer/copy by construction, preserves canonical bytes, strengthens compatibility coverage and shows no material regression.

After M6-0, run M6-A (#16) as an estimator-assisted decision-value experiment and start the read-only/source-audit portion of M6-D (#19) in parallel. Production ExactSmallDelta/hybrid adoption remains gated on M6-A evidence.
