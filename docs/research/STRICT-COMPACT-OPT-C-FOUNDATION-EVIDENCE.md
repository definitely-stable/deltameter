# STRICT-COMPACT OPT-C — C0/C1 proof and frame-foundation evidence

Issue #74; parent #70. PR #78.
Status: **C0_PROOF_REVIEWED / C1_FRAME_FOUNDATION_PASS**;
**C2/C3 performance and transfer-product decision NOT YET MEASURED.**

Sources:
- [C0 simultaneous-coverage proof](STRICT-COMPACT-OPT-C-PROOF.md)
- [C1–C3 preregistered protocol](STRICT-COMPACT-OPT-C-PROTOCOL.md)

Canonical hosted source commit:
`dfe12448a76499be78a734036ac0e566d6b00e41`.

GitHub Actions workflow
[37809620059](https://github.com/definitely-stable/deltameter/actions/runs/37809620059):
**SUCCESS**, including Rust format, Clippy, test suite, reference Q32 checksum
and dedicated research-only frame lab.
Other workflows on the same PR source: `rust` SUCCESS and `research` SUCCESS.

Frame lab PASS marker:

```text
STRICT_COMPACT_OPT_C_FOUNDATION_PASS state_bytes=26624 level_bytes=512 frame_bytes=533 levels=52
```

GitHub artifact: ID `11565265759`,
ZIP digest `sha256:04f42f21aae77c950a56deb4547ffaffa32b6f22c0e8517804790132a39f0e30`.

## C0 coverage note reviewed

For one immutable snapshot and fixed nonadaptive u64 token difference, the
preallocated per-level confidence intervals with `sum alpha_j<=delta`
give simultaneous coverage by the union bound. Arbitrary receiver-side
subset/stopping selection among these *same* statements cannot spend another
statistical budget. More levels can only tighten the min upper bound.
This **does not** prove security under adaptive chosen-token attacks against
the keyed BLAKE3 oracle, nor allow changing state epochs.

## C1 tested frame behavior

The research lab physically packs J=52 * m=4096 bits into 26,624 B, freezes
an immutable copy, extracts contiguous 512-B level payloads, and tests:
- empty receiver => u128 domain ceiling `2^64`;
- per-level presence separate from a received all-zero payload;
- deterministic forward and reverse delivery orders reconstruct identical
  canonical words and Q32 upper-bound inference;
- receiver bound is non-increasing as valid received levels are appended;
- identical duplicates idempotent; conflicting same-ID payload fails closed;
- fail closed on a shortened frame, corrupt payload, invalid level index,
  wrong session and mismatched externally provided config/table identifiers;
- a session tag includes immutable epoch, profile, nonsecret config/key binding,
  and a fingerprint of **actual certified Q32 table bytes**;
- live-state updates after snapshot creation cannot mutate frozen frame source.

Frame payload `session_tag[16] + index[1] + bitmap[512] + CRC32[4]` totals
**533 B**. The session tag is an unkeyed/nonsecret digest truncation and
CRC32 is for accidental corruption only. Neither proves sender authenticity.
The session header is *modelled as in-memory metadata in C1*; complete wire
header encoding and receiver authentication are intentionally not frozen here.

## Non-results and next stage

- No p50/p95 byte saving, RTT improvement or progressive product advantage
  has been measured yet.
- C2 must implement ordered/interactive/zero-elision representation candidates
  with an actual accountable session header and sender snapshot-copy cost.
- C3 must measure the predeclared d/T/Rtt/bandwidth grid and produce a single
  PROGRESSIVE_PASS / STREAM_ONLY / DROP_PROGRESSIVE verdict.
- No public API, Snapshot v1 mutation, public profile or confidence-coverage
  promotion. #69 stays blocked until C3 finishes; issue #74 stays open.
