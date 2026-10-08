# STRICT-COMPACT-OPT-A — range-aware state/layout evidence

Issue #72. Parent #70. Public-boundary gate #69.

Verdict: **RANGE_FRONTIER_PASS**.

This slice preserves the accepted STRICT-COMPACT statistical/oracle contract and
asks only how many stored levels are required and how those bits should be laid out
physically.

## Baseline

Entering OPT-A:

~~~text
rows                       4096
stored levels              64
instance parity state      32,768 B
shared Q32 table           16,384 B
ideal delta                <= 1e-6
q95 width                  U/d <= 1.5
declared useful d          4096 .. 2^64-1
oracle                     keyed BLAKE3 v1 research contract
~~~

The already certified Q32 table is reused unchanged:

`sha256:634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`.

For J<64 this is conservative because each active level still uses alpha=delta/64,
so total used alpha mass only decreases.

## Stage 1 — exact range frontier

Canonical Stage-1 source head:

`0ab6824b6e5cd7e7967eb4bb66d2a09495b6121f`

Workflow:

- strict-compact opt-a stage1 #5 / `37755547131` — PASS;
- artifact `11539829494`;
- artifact digest
  `sha256:da79cbb6fd628bdd8bf03b358b4f47fd4b754a716fdd9fde4d421e8c69331814`.

The independent legacy J=64 range workflow also passes on this source state.

Exact continuous q95<=1.5 frontier:

| J | instance state | certified continuous d_max | first uncertified d | intervals | max level used |
|---:|---:|---:|---:|---:|---:|
| 24 | 12 KiB | 84,315,082,377 | 84,315,082,378 | 445 | 24 |
| 32 | 16 KiB | 21,584,661,088,732 | 21,584,661,088,733 | 587 | 32 |
| 40 | 20 KiB | 5,525,673,238,715,561 | 5,525,673,238,715,562 | 726 | 40 |
| 48 | 24 KiB | 1,414,572,349,111,183,676 | 1,414,572,349,111,183,677 | 867 | 48 |
| 56 | 28 KiB | 18,446,744,073,709,551,615 | none | 876 | 52 |
| 64 | 32 KiB | 18,446,744,073,709,551,615 | none | 876 | 52 |

The important result is structural:

**J=56 already preserves the complete accepted u64-domain width contract.**

The J=64 certificate never needs a certifying level above 52. Therefore levels
57..64 are unnecessary for the current full-domain q95 contract.

No alpha optimization or new theorem is used to obtain this result.

## Physical packing and layout

OPT-A does not infer memory from J. The Rust lab physically allocates exactly

`ceil(4096*J/64)` u64 words.

Observed instance state therefore matches the table above.

Two layouts represent the same logical parity matrix:

~~~text
ROW_MAJOR   bit = row*J + level
LEVEL_MAJOR bit = level*4096 + row
~~~

Correctness passes for every J:

- row-major and level-major logical cells agree;
- repeated toggle cancels;
- level counts agree;
- same-layout XOR merge equals a combined input stream;
- integer upper-bound inference agrees;
- keyed-oracle reference vectors remain unchanged;
- state buffer does not reallocate during update.

### Stage-1 layout screen

Keyed BLAKE3 dominates update address arithmetic, so update medians are essentially
the same across layouts (~96.4-96.9 ns/token in the screen).

Level-major makes each level one contiguous 512-byte chunk and substantially
reduces estimate extraction:

| J | row-major estimate | level-major estimate |
|---:|---:|---:|
| 24 | 16.72 us | 1.08 us |
| 32 | 16.76 us | 1.43 us |
| 40 | 16.83 us | 1.79 us |
| 48 | 16.84 us | 2.14 us |
| 56 | 16.85 us | 2.50 us |
| 64 | 16.91 us | 2.89 us |

XOR ns/KiB remains broadly comparable.

The frozen layout rule therefore selects **LEVEL_MAJOR for every retained profile**.

This also creates the correct physical primitive for OPT-C: one level is exactly
64 contiguous u64 words / 512 bytes.

## Stage-2 protocol amendment

The original deterministic shortlist omitted a "smallest full-domain J" tier.
Stage 1 revealed J=56 as strictly smaller than J=64 at the same certified d_max.

Before any Stage-2 timing, protocol v2 added the smallest J reaching
`2^64-1`.

Frozen Stage-2 matrix:

~~~text
J=24  LEVEL_MAJOR  12 KiB
J=40  LEVEL_MAJOR  20 KiB
J=56  LEVEL_MAJOR  28 KiB
J=64  LEVEL_MAJOR  32 KiB control
~~~

No performance gate changed.

## Invalid Stage-2 attempt

Workflow run `37756677369` is **INVALID for the final verdict**.

The five raw workers completed, but a hand-authored bridge JSON had encoded d-domain
values through an IEEE-754 JavaScript Number path. Values above 2^53 were rounded;
`2^64-1` became `18446744073709552000`, so the aggregator could not recognize
J=56 as full-domain.

This is an evidence serialization defect, not a sketch/performance failure.

The correction stores every d-domain value as a decimal string and parses it with
Python arbitrary-precision integers. Benchmark code and gates were unchanged.

## Canonical Stage 2

Canonical Stage-2 measurement source head:

`cfa7e48a4784146573073c6992ed9536f24b1d24`

Workflow:

- strict-compact opt-a stage2 #4 / `37756954797` — PASS;
- five independent GitHub-hosted workers — 5/5 PASS;
- four profiles per worker;
- eight measured raw rounds per profile;
- 160 complete raw timing observations;
- candidate order rotated deterministically across workers/rounds.

Summary artifact:

~~~text
id      11540208493
sha256  ce8bd1a324d3bbbc0241a31b9493509e005166bbcb1d09dca4dabd12a9cc03df
~~~

Worker artifacts:

~~~text
worker 1  11540591637  sha256:bb541e0a2eba5de567163a37715753a2e377d870aafe541ea5a3dc3a7cc1deac
worker 2  11540526850  sha256:4e17e0fe3dcc768de759572c23de163002e6fe6b076bae28ca8de68b1fbf66bd
worker 3  11539979700  sha256:6f5e82e15fb7434cb144b6ed17a23d4c5a76f0769813352f4d24823d1069b9de
worker 4  11540591579  sha256:660ef9ad259c6a7a43480cd58dbcaa90ead00290ce77a504eaa9971745c89ece
worker 5  11540307489  sha256:2ad35fbedf271e70115752e848bc0b8600b7039c7091fafd1f13a4dd4f44662d
~~~

### Five-worker result

Worker-median metrics:

| J | state | d_max | update ns/token | estimate ns | XOR ns/KiB | viability |
|---:|---:|---:|---:|---:|---:|---|
| 24 | 12 KiB | 84,315,082,377 | 87.175 | 1,174 | 22.051 | PASS |
| 40 | 20 KiB | 5,525,673,238,715,561 | 87.168 | 1,941 | 22.129 | PASS |
| 56 | 28 KiB | 2^64-1 | 87.128 | 2,759 | 26.339 | PASS |
| 64 | 32 KiB | 2^64-1 | 87.106 | 3,189 | 28.286 | control |

Predeclared viability limits were checked on **every worker**.

For J=56 versus J=64 the worst observed worker regressions are:

~~~text
update      +0.105%
estimate    +3.670%
XOR/KiB     +2.648%
~~~

all far inside the allowed +15% / +25% / +25% gates.

Across worker medians, J=56 is actually faster on estimate and XOR while update is
effectively unchanged.

## Decision

**RANGE_FRONTIER_PASS**.

Research/profile decisions:

1. **J=56 / 28 KiB / LEVEL_MAJOR becomes the preferred full-u64 research
   baseline for subsequent OPT-B/OPT-C work.**
2. J=64 remains a regression/control profile, not the preferred full-range
   research profile.
3. J=24 is a useful 12 KiB bounded-range profile through 84,315,082,377.
4. J=40 is a useful 20 KiB bounded-range profile through
   5,525,673,238,715,561.
5. J=32 and J=48 remain valid exact frontier points even though they were not
   needed in Stage 2.
6. The first public default is **not** selected here; #69 remains blocked on
   OPT-B/OPT-C product-shape evidence.

The full-range per-instance state improves from 32 KiB to **28 KiB (-12.5%)**
without new estimator mathematics, a new table, a weaker delta, or a narrower
declared range.

The shared 16 KiB Q32 table remains separate static data and is not multiplied per
sketch instance.

## Post-Stage-2 refinement / current research baseline

The above canonical frozen Stage-1/Stage-2 result remains historically valid.
A separately preregistered exact/hosted J51/J52 experiment subsequently
confirmed **J52_REFINEMENT_PASS**: J=52 / 26 KiB / LEVEL_MAJOR preserves the
accepted full-u64 continuous width range and passes all five hosted workers.
This supersedes J=56 as the **preferred current full-range research baseline**,
not as a public default. J51's failure to reach the full range with the existing
certificate algorithm is not a mathematical lower bound.

Full evidence, raw artifact IDs, source SHA and boundaries:
[OPT-A refinement evidence](STRICT-COMPACT-OPT-A-REFINEMENT-EVIDENCE.md).

## Next work

- #73 OPT-B should evaluate maintained odd counts primarily on J=56, with J=24 as
  a small-profile scaling guard.
- #74 OPT-C should use LEVEL_MAJOR and evaluate 512-byte level chunks across the
  retained J frontier.
- #69 may continue coverage/key-lifecycle design but must not freeze public
  snapshot/profile layout until OPT-C closes.

No generalized ladder, oracle replacement or public API work is justified before
those cheap product-shape slices finish.
