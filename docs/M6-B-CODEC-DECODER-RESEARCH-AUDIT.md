# M6-B — codec/decoder research audit

Status: **research synthesis complete; implementation sequence corrected**.  
Issue: #17. Parent: #14. Date: 2026-10-07.  
Repository baseline reviewed: `a0c860bd8071927e0f586cb00371ebb25af9ed0f`.

## Purpose

This note reconciles two external deep-research reports with the actual DeltaMeter
snapshot-v1 implementation and the already measured M6-B1 candidate. The reports
contain useful directions, but several claims are either contradictory, based on
unmeasured assumptions, or inconsistent with the current decoder.

The rule for M6-B remains: **one candidate at a time, exact compatibility first,
paired base/head evidence second**.

## Ground truth from the repository

Snapshot v1 envelope is 24 bytes plus payload plus 4-byte CRC32C. The checksum is
Castagnoli CRC32C and is accidental-corruption detection only.

For Energy, with `B = buckets` and `R = tables`:

~~~text
payload = 16 + 48R + 8BR
total   = 44 + 48R + 8BR
~~~

Default Energy is `B=2048, R=23`, therefore `377,980` bytes.

The smaller benchmark profile is **not** `B=256, R=18`. The current benchmark uses
`RelativeError::TwentyPercent` and `FailureTarget::OneInThousand`, i.e.
`B=512, R=9`, which also gives `37,340` bytes.

For Parity:

~~~text
N = rows * stored_levels
W = ceil(N / 64)
payload = 16 + 8W
total   = 44 + 8W
~~~

The current padded benchmark is `rows=17, levels=13`, giving four packed words and
a 76-byte snapshot. It is not the `16 x 8` shape quoted in one report.

## M6-B1 evidence and what it does not prove

PR #40 replaces the bit-at-a-time CRC32C hot loop with a compile-time 256-entry
scalar table. Exact-head hosted evidence at `b0e850dc876b7b3373448405b8b6b7ba3d0de71f`
showed roughly 45.5-49.5% lower end-to-end snapshot encode/decode latency across
the measured Energy and Parity cases.

This is strong empirical evidence that the old bit-at-a-time CRC was expensive.
It does **not** identify the residual share of CRC after B1.

One external report derives a post-B1 CRC share of about 10.7% by assuming a
10x standalone CRC speedup. That is a conditional Amdahl calculation, not an exact
measurement. The assumed cycles/byte values also conflict across the two reports.
DeltaMeter therefore does not adopt the 10.7% number as evidence.

Before any further CRC optimization, a checksum-only measurement must establish the
residual share on the same hosted environment.

## Critical correction: B2 is not a fix for the claimed short-header OOM bug

Both reports describe an existing decoder that trusts Energy dimensions and creates
a huge primary state before validating available bytes. That description is false
for the current code.

Current behavior is:

1. `decode_envelope` checks exact total length and CRC before returning payload.
2. Energy computes the complete row block size and checks that those bytes exist
   before `Vec::with_capacity(table_count)`.
3. After rows/config are validated, Energy computes exact counter bytes and requires
   `cursor.remaining() == counter_bytes` before `Self::new(config)` allocates the
   primary counters/energies/pending state.
4. Parity derives the packed word count and requires exact state bytes before
   allocating the packed-state vector.

Therefore M6-B2 must **not** be documented as closing a current arbitrary
primary-allocation vulnerability or as a newly discovered critical DoS fix.

B2 is still useful. It can compute the complete expected backend payload shape from
metadata before row allocation, reject impossible valid-CRC payloads earlier, make
resource reasoning simpler and strengthen malformed-input coverage.

That is **resource hardening and invariant simplification**, not evidence of a
present exploit.

## B2 exact-shape candidate

For Energy, after the 16-byte backend metadata has been read:

~~~text
row_bytes     = checked_mul(R, 48)
counter_count = checked_mul(B, R)
counter_bytes = checked_mul(counter_count, 8)
expected      = checked_add(16, row_bytes, counter_bytes)
~~~

Require `payload.len() == expected` before allocating the row vector.

The validation must not invent new `B_max` or `R_max` limits. A transport-level
wire-size cap is a separate product policy. Core snapshot v1 should continue to
derive safety from checked arithmetic, exact byte availability and existing
configuration validity.

For Parity, retain the current config-derived packed-size validation. If an explicit
formula is used, avoid the overflow-prone `(N + 63) / 64`; prefer the existing
checked implementation or:

~~~text
W = if N == 0 { 0 } else { 1 + (N - 1) / 64 }
~~~

All structural-negative tests must recompute a valid CRC32C. Otherwise the test only
exercises `ChecksumMismatch`.

B2 acceptance is primarily semantic/resource hardening. A valid-input speedup is
not required; a material valid-input regression is not acceptable.

## B3 should be widened to direct decoded-state construction

The strongest useful idea in the reports is broader than the original wording
"reuse decoded energy cache allocation".

Current Energy decode:

~~~text
Self::new(config):
    allocate + initialize counters
    allocate + initialize energies
    allocate pending
    derive sign masks

parse counters and overwrite the initialized counter array

recompute_energies:
    scan every counter again
    allocate a replacement energies array
~~~

The better isolated B3 candidate is:

~~~text
allocate counters with exact capacity
allocate energies with exact capacity

for each table:
    energy = 0
    for each bucket:
        counter = decode i64
        counters.push(counter)
        x = i128::from(counter)
        energy = checked_add(energy, x * x)
    energies.push(energy)

construct the final meter from validated decoded parts
~~~

For every `i64`, the individual square fits in `i128`:

~~~text
max |c| = 2^63
c^2 <= 2^126 < 2^127
~~~

Only the row sum needs checked addition. A separate `checked_mul` is unnecessary
for an `i64 -> i128` square.

The one-pass candidate preserves the exact summation order used by
`recompute_energies`, so overflow occurs at the same row/bucket partial sum.

### Memory-work model

For default Energy:

~~~text
BR = 2048 * 23 = 47,104 counters
counter bytes = 376,832
~~~

At the source-code logical-touch level, current decode performs on the destination
counter state:

~~~text
zero initialization: 376,832 B written
decode overwrite:    376,832 B written
energy recompute:    376,832 B read
~~~

The direct candidate needs one destination write and no post-decode counter scan.
It therefore removes **753,664 bytes (736 KiB) of logical heap touches** for the
default shape.

This is not a claim of 736 KiB of DRAM traffic. Cache residency, allocator behavior,
page zeroing, write allocation and compiler code generation make physical traffic a
measurement question. The 33-40% "bus traffic reduction" and DRAM-latency estimates
in the reports are therefore treated as heuristics, not exact results.

The reports' steady-state memory formula is also not authoritative:
`PendingUpdate` contains `usize + i64 + i128`; its layout is target dependent and
cannot be modeled as 8 bytes per table. Use `size_of` in measurement if exact
allocation accounting is needed.

## CRC follow-ups

### Slicing-by-4/8

Slicing-by-N is a real, well-established software technique. The Linux kernel CRC
tutorial explains why multiple 256-entry tables expose independent lookups and more
instruction-level parallelism. Intel historically reported substantial standalone
CRC gains for slicing-by-8.

That does not establish an end-to-end DeltaMeter gain after B1. Larger tables also
consume more L1D capacity:

~~~text
slicing-by-4:  4 KiB
slicing-by-8:  8 KiB
slicing-by-16: 16 KiB
~~~

Verdict: **DEFER**. Do not add B1.1 until residual checksum attribution shows enough
end-to-end headroom.

### Encode CRC fusion

The two reports disagree directly:

- one labels incremental CRC during serialization an obvious GO because it removes a
  later read pass;
- the other predicts neutral/negative performance because a single contiguous CRC
  scan is a tight loop while per-field CRC updates extend dependency chains.

Neither claim is measured on DeltaMeter.

Verdict: **DEFER / measurement-selected only**. It is not the next production
candidate. If future profiling still finds checksum/memory reread material, test
encode fusion in its own base/head slice.

### Decode CRC fusion

Current `decode_envelope` completes checksum verification before backend parsing.
That boundary is simple and fail-closed.

Mixing backend construction with checksum calculation is unnecessary for the current
buffered format and risks changing failure/resource ordering.

Verdict: **NO-GO for current M6-B**.

### CRC combine

CRC concatenation is linear over GF(2), and combine operators can derive the checksum
of `A || B` from checksums of the parts plus the length of `B`. This is useful
for parallel or precomputed segments.

DeltaMeter currently constructs one small-to-medium monolithic snapshot. Combine
adds machinery without avoiding the need to checksum freshly serialized payload
bytes.

Verdict: **informative only; no current candidate**.

### Hardware CRC / carryless-multiply folding

Potentially much faster, but platform specialization is outside the accepted M6-B
foundation and conflicts with the present no-unsafe/no-FFI/no-new-runtime-dependency
boundary.

Verdict: **future/out of scope**.

## Row/sign-mask fusion

The row block is only `48R` bytes (1,104 bytes for default `R=23`). M6-C already
made sign-mask construction cheap enough that there is no evidence it is a material
decode bottleneck.

Verdict: **DEFER** until profiling says otherwise. Do not bundle it into B3.

## Corrected implementation sequence

1. **Finish B1**: synchronize PR #40 with current main, keep the table CRC change
   isolated, rerun exact-head compatibility and paired snapshot evidence, then
   review/merge.
2. **B2**: exact backend payload-shape validation before row/state allocation.
   Valid-CRC malformed corpus; audit error precedence. No security-exploit claim.
3. **B3**: one isolated direct Energy decoded-state construction experiment:
   capacity-only counters, one-pass counter parse + checked energy accumulation,
   final construction from validated parts. Keep sign-mask fusion out.
4. **Residual selector**: only if further codec work is desired, measure checksum-only
   and phase-level residual cost. Slicing-by-8 and encode fusion remain candidates,
   not assumptions.
5. Stop CRC micro-optimization if the residual selector cannot justify a material
   end-to-end opportunity.

Do **not** combine B2 and B3. Doing so would destroy attribution and contradict issue
#17's one-candidate-at-a-time requirement.

## Benchmark policy

Keep the existing dependency-free hosted harness and paired base/head workflow.
Do not introduce Criterion/benchstat merely because an external report recommends
them.

For B2:

- correctness and valid-CRC malformed-input behavior are the primary gate;
- measure valid decode to detect regression;
- measure invalid rejection separately;
- no positive performance threshold is required.

For B3:

- measure Energy default and small independently;
- preserve Parity as an unchanged guardrail;
- record exact allocations/logical bytes if instrumentation is added;
- accept only a repeatable material valid-decode improvement with byte-identical
  encode/re-encode and unchanged accepted/error semantics.

No absolute nanosecond SLA belongs in CI.

## External research classification

| Claim | Audit classification |
| --- | --- |
| B1 removed a dominant old CRC cost | ACCEPTED empirical repository evidence |
| post-B1 CRC is exactly ~10.7% | REJECT as exact; conditional Amdahl heuristic |
| slicing-by-8 is faster standalone | established technique; DeltaMeter gain unmeasured |
| encode fusion is definitely GO | INSUFFICIENT EVIDENCE |
| decode fusion should replace checksum-first boundary | NO-GO |
| current decoder has arbitrary short-header primary OOM | REJECT; contradicted by code |
| B2 exact shape before allocation is useful | GO as hardening/simplification |
| B3 direct one-pass Energy decode is promising | GO-TO-EXPERIMENT |
| B3 reduces physical memory traffic by exactly 33-40% | REJECT as exact; heuristic only |
| row/sign-mask fusion is material | DEFER; no evidence |
| transport byte cap belongs in snapshot core | REJECT; outer admission remains separate |

## Primary references

- RFC 7143, iSCSI consolidated protocol, CRC32C generator and error-detection role:
  https://www.rfc-editor.org/rfc/rfc7143.html
- Linux kernel CRC tutorial, Sarwate and slicing-by-N rationale:
  https://kernel.org/doc/html/v5.14/staging/crc32.html
- Koopman and Chakravarty, *Cyclic Redundancy Code (CRC) Polynomial Selection for
  Embedded Networks*, DSN 2004:
  https://users.ece.cmu.edu/~koopman/roses/dsn04/koopman04_crc_poly_embedded.pdf
- zlib manual, incremental CRC and combine API as an engineering reference:
  https://www.zlib.net/manual.html

The references support CRC semantics and algorithm families. They do not replace
DeltaMeter-specific hosted measurement.
