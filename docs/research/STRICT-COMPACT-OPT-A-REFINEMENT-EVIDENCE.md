# STRICT-COMPACT OPT-A — post-Stage-2 J=51/52 refinement evidence

Parent #70; issue #72; PR #76. Protocol:
[STRICT-COMPACT-OPT-A-REFINEMENT-PROTOCOL.md](STRICT-COMPACT-OPT-A-REFINEMENT-PROTOCOL.md).

Decision: **J52_REFINEMENT_PASS** on the preregistered source
`a0ba405aaa045db4490cde28b248a0739eb09f9c`.
All six PR workflows completed with success on this source, including
standard Rust, research, full-range certificate, original OPT-A Stage 1,
original OPT-A Stage 2, and the new J52-refinement workflow.

This is a **separate addendum**. Original Stage-1 and Stage-2 observations,
protocols and historically valid J=56 shortlist remain preserved as originally
recorded; their verdict was for the tested J matrix and is not a mathematical
minimality theorem.

## Mathematical finding

For m=4096 and the unchanged per-level delta/64 certified Q32 table,
the original J=64 exact interval proof chose no certifying level above 52.
Its witnesses therefore remain admissible for J=52. The independent refinement
script reproduces continuous interval proof rather than merely relying on the
inheritance argument; it checks the existing exact-rational proof boundary.

The new hosted certificate reports:

| J | stored bitmap bytes | continuous certified d_max | full u64 width range |
|---:|---:|---:|---|
| 51 | 26,112 (25.5 KiB) | 11,316,578,792,889,469,415 | NO |
| 52 | 26,624 (26 KiB) | 18,446,744,073,709,551,615 | YES |
| 56 | 28,672 (28 KiB) | 2^64-1 | prior PASS |
| 64 | 32,768 (32 KiB) | 2^64-1 | accepted control |

**Warning:** the J=51 failure is *the current certificate construction
ceasing to certify* beyond the shown boundary. It is not a lower-bound
theorem and does not prove a different construction with J=51 impossible.

The exact table is unchanged:
`sha256:634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`.
The ideal one-sided statistical budget remains <=1e-6, and the same
keyed-BLAKE3 computational/nonadaptive token assumptions apply.
No public `Coverage::Proven` claim follows from these assumptions.

## Hosted correctness and performance

Refinement workflow run: [37801885016](https://github.com/definitely-stable/deltameter/actions/runs/37801885016).

- five independent GitHub-hosted workers, all PASS;
- J=52, 56 and 64 with rotated measurement order;
- 2 warmups, 8 raw measured rounds per worker/profile = 120 samples;
- exact state-size and equivalence/cancellation/XOR tests on J=51 and J=52;
- all J=52 worker medians within **predeclared** +15% update,
  +25% estimate and +25% XOR ns/KiB viability ceilings against J=64;
- raw per-worker source HEAD SHA validated against aggregating checkout.

J=52 per-worker median timing (performance observations, **not** API promises):

| worker | update ns/token | estimate ns | XOR ns/KiB | viability |
|---:|---:|---:|---:|---|
| 1 | 96.843 | 2336.172 | 37.143 | PASS |
| 2 | 87.219 | 2007.343 | 21.937 | PASS |
| 3 | 96.727 | 2342.281 | 28.508 | PASS |
| 4 | 96.666 | 2332.093 | 30.380 | PASS |
| 5 | 94.270 | 1646.633 | 18.158 | PASS |

Artifact provenance (GitHub artifact object digests, not a hash of uncompressed JSON):

| artifact | ID | sha256 |
|---|---:|---|
| exact certificate | 11560469190 | 61b93b18bacfed7c19e42b3d99c6f8155ce619a83e5ccbaa1f5949a0f10145ec |
| summary | 11560643296 | 554182b566cc7055dd689b79edbb8cff02b9a686302d3b8beda8de1f6511ae5c |
| worker 1 | 11560309797 | a63078097e8cc77bc9363c4254340fb474533294aefda9a4085419e31e451131 |
| worker 2 | 11560697987 | 9759a07f257f1bd9f3c7e14666191b8099ce79c906d1a91505b4bcd1a86cfa1b |
| worker 3 | 11560329764 | bff51516e76670562b6e9a42c3bb78257580751af83d2bc0ad0e689eddd8f2f1 |
| worker 4 | 11561691667 | 47dd72922850addb8a3b509b32adf71ae0b509cc36d380647d01fd7d342977c5 |
| worker 5 | 11561547638 | 82fa07e65a58ed8df94c0f1bf603857824ee51dcf736d1f748da9a7234404b5c |

## Engineering interpretation

Relative to the J=64 original physical bitmap, J=52 saves 6 KiB
(**18.75%**) of **per-instance parity bitmap state**, without changing the
shared 16 KiB Q32 lookup table. Relative to J=56 it saves 2 KiB (7.14%).

The exact bitmap size is not a Rust-object total allocation/footprint claim:
oracle key, Rust owner object and allocator overhead must be accounted for
separately in future public design.

The preferred **full-u64 research profile** for #73 OPT-B and #74 OPT-C is now:

```text
m=4096, J=52, LEVEL_MAJOR, 26 KiB packed bitmap
per-level alpha=delta/64, delta=1e-6
keyed BLAKE3 v1 + existing Q32 lookup
```

J=64 is retained as the original full-range control; J=56 is the previous
validated candidate. Bounded-range Pareto profiles J=24/32/40/48 remain
valid and not overridden by the refinement.

## Decision boundaries

- OPT-A: **RANGE_FRONTIER_PASS; J52_REFINEMENT_PASS**.
- #73: evaluate exact maintained odd-level counts vs packed scan; cache derived
  and non-serialized.
- #74: separately evaluate progressive/ordered 512-byte level transfer.
- #69: **NO PUBLIC PROFILE/SNAPSHOT FREEZE** until #73/#74 decisions close.
- No generalized ladder, alpha reallocation, new oracle or public API work
  is authorized by this refinement.
