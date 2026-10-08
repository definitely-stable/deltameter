# STRICT-COMPACT-OPT-A — Stage 1 evidence

Issue #72. Parent #70.

Status: **STAGE1_PASS; Stage-2 protocol amended before Stage-2 timing**.

Canonical Stage-1 source head:

`0ab6824b6e5cd7e7967eb4bb66d2a09495b6121f`

Hosted workflow:

- strict-compact opt-a stage1 #5 / `37755547131` — PASS;
- artifact `11539829494`;
- artifact SHA-256
  `da79cbb6fd628bdd8bf03b358b4f47fd4b754a716fdd9fde4d421e8c69331814`.

The independent legacy J=64 range workflow and standard Rust CI also pass on the
same source head.

## Exact state/range frontier

The existing independently certified delta/64 Q32 table was reused unchanged.

| J | state | exact continuous q95<=1.5 d_max | first fail | intervals | max certifying level |
|---:|---:|---:|---:|---:|---:|
| 24 | 12 KiB | 84,315,082,377 | 84,315,082,378 | 445 | 24 |
| 32 | 16 KiB | 21,584,661,088,732 | 21,584,661,088,733 | 587 | 32 |
| 40 | 20 KiB | 5,525,673,238,715,561 | 5,525,673,238,715,562 | 726 | 40 |
| 48 | 24 KiB | 1,414,572,349,111,183,676 | 1,414,572,349,111,183,677 | 867 | 48 |
| 56 | 28 KiB | 2^64-1 | none | 876 | 52 |
| 64 | 32 KiB | 2^64-1 | none | 876 | 52 |

This immediately shows that levels 57..64 are unnecessary for the already accepted
full-u64 q95 width contract: J=56 reproduces the full declared range and the exact
certificate never needs a level above 52.

This is a result about the existing conservative alpha=delta/64 construction; no
alpha reallocation is used.

## Physical packing

Both row-major and level-major labs allocate exactly m*J bits. The observed state
sizes therefore match the mathematical profile rather than retaining the old
one-u64-per-row J=64 shape.

Deterministic row-major/level-major logical-state checks pass for every J.

## Stage-1 layout result

Update cost is effectively unchanged across layouts (~96.4-96.9 ns/token on this
screen), because keyed BLAKE3 dominates the bit-address arithmetic.

Level-major estimate extraction is dramatically cheaper:

| J | row-major estimate | level-major estimate |
|---:|---:|---:|
| 24 | 16.72 us | 1.08 us |
| 32 | 16.76 us | 1.43 us |
| 40 | 16.83 us | 1.79 us |
| 48 | 16.84 us | 2.14 us |
| 56 | 16.85 us | 2.50 us |
| 64 | 16.91 us | 2.89 us |

XOR ns/KiB is broadly comparable. Therefore the frozen layout rule selects
**LEVEL_MAJOR for every J**.

This result also materially strengthens OPT-C: for level-major, one level is exactly
one contiguous 512-byte chunk.

## Protocol v2 amendment before Stage-2 timing

The original deterministic shortlist tiers were:

- J=64 control;
- smallest nontrivial J;
- smallest J reaching 2^32-1;
- smallest J reaching 2^48-1.

That produced `64,24,40`.

Stage 1 exposed a missing product tier: the smallest J preserving the complete u64
declared range. J=56 is strictly smaller than the J=64 control while preserving the
same certified d_max.

Before any Stage-2 measurement, protocol v2 adds:

- smallest J reaching 2^64-1.

The frozen Stage-2 shortlist is therefore:

~~~text
J=24  LEVEL_MAJOR  12 KiB
J=40  LEVEL_MAJOR  20 KiB
J=56  LEVEL_MAJOR  28 KiB
J=64  LEVEL_MAJOR  32 KiB control
~~~

No other Stage-1 result is used to alter performance gates.

## Interpretation

Stage 1 already establishes a major product finding:

**32 KiB is not the minimum simple full-domain profile. 28 KiB/J=56 preserves the
entire accepted u64 range without new mathematics.**

Stage 2 exists only to confirm that the smaller physically packed profiles do not
create a hosted performance regression hidden by the single-run screen.
