# M5 — Canonical sketch interchange v1

Status: implementation in PR #13.  
Issue: #12.  
Format name: `deltameter.snapshot.v1`.

## Why interchange is a product requirement

DeltaMeter estimates set symmetric-difference size with mergeable sketches.

If both source sets are already co-resident in one process, an exact comparison is often available. The sketch becomes materially useful when the sets are separated by process, host, storage boundary, or time. A stable byte representation is therefore the minimum transport boundary for the intended use case.

M5 defines that boundary without adding a serialization framework.

## Goals

The v1 format is:

- canonical;
- self-contained;
- little-endian;
- versioned;
- backend-tagged;
- exact-length checked;
- corruption-detecting with CRC32C;
- safe to decode without trusting cached/derived state;
- dependency-free.

The format is not:

- encrypted;
- authenticated;
- compressed;
- a filesystem abstraction;
- a network protocol;
- zero-copy/mmap oriented;
- a promise to preserve Rust in-memory layout.

## Common envelope

All integer fields are little-endian.

~~~text
offset  size  field
0       8     magic = ASCII "DELTAMTR"
8       2     format_version = 1 (u16)
10      1     backend
11      1     flags = 0
12      8     payload_len (u64)
20      N     backend payload
20+N    4     CRC32C(header || payload)
~~~

Backend tags:

~~~text
1 = Energy
2 = Parity
~~~

The total encoded length is exactly `20 + payload_len + 4`. Trailing bytes are invalid.

CRC32C uses the Castagnoli polynomial. It detects accidental corruption only. It is not a MAC and does not authenticate the sender.

Unknown versions, non-zero flags, wrong backend tags, invalid lengths and checksum failures are rejected.

## Energy payload

~~~text
offset  size                 field
0       8                    buckets (u64)
8       4                    table_count (u32)
12      1                    profile_kind
13      1                    relative_error_tag
14      1                    failure_target_tag
15      1                    reserved = 0
16      table_count * 48     EnergyRowHash records
...     buckets*tables*8     signed counters (i64)
~~~

Each row record is exactly six u64 words:

~~~text
bucket_coefficients[0]
bucket_coefficients[1]
sign_coefficients[0]
sign_coefficients[1]
sign_coefficients[2]
sign_coefficients[3]
~~~

Profile encoding:

~~~text
profile_kind = 0:
    custom configuration
    relative_error_tag = 0
    failure_target_tag = 0

profile_kind = 1:
    theorem-profile configuration

relative_error_tag:
    1 = 5%
    2 = 10%
    3 = 20%

failure_target_tag:
    1 = 1e-3
    2 = 1e-6
    3 = 1e-9
~~~

For a theorem profile, decoded dimensions must exactly match the profile.

Counters are encoded in the implementation's logical table-major order: all B counters for row 0, then all B counters for row 1, and so on.

Cached row energies and pending-update scratch state are not serialized. They are recomputed from decoded counters with checked arithmetic.

Preserving a `Proven` profile marker preserves the original configuration contract; decoding does not prove that the original row coefficients were sampled according to the documented independent-uniform precondition.

## Parity payload

~~~text
offset  size               field
0       4                  rows (u32)
4       1                  stored_levels J (u8)
5       3                  reserved = 0
8       8                  deterministic seed (u64)
16      ceil(rows*J/64)*8  packed GF(2) words
~~~

The bit offset for one logical cell is:

~~~text
bit = row * J + (level - 1)
~~~

Levels are one-based.

The final packed word may contain unused high bits. Every unused padding bit must be zero. A decoder rejects otherwise so one logical Parity state has exactly one canonical v1 representation.

The seed plus shape reconstructs the exact deterministic pseudo-oracle configuration used by `ParityConfig`.

The seed remains a reproducibility input, not a cryptographic key and not a proof that the paper's ideal-random-oracle assumption is instantiated.

## Public API

M5 adds:

~~~rust
EnergyDeltaMeter::encode_snapshot(&self) -> Result<Vec<u8>, SnapshotError>
EnergyDeltaMeter::decode_snapshot(bytes: &[u8]) -> Result<Self, SnapshotError>

ParityDeltaMeter::encode_snapshot(&self) -> Result<Vec<u8>, SnapshotError>
ParityDeltaMeter::decode_snapshot(bytes: &[u8]) -> Result<Self, SnapshotError>
~~~

`SnapshotError` is exported from the crate root.

No generic serde trait, file helper or network helper is added.

## Canonicality rules

A v1 decoder fails closed when any of the following occurs:

- input is shorter than the envelope;
- magic differs;
- version is not 1;
- backend does not match the requested meter type;
- flags are non-zero;
- declared payload length does not exactly match input length;
- CRC32C differs;
- a reserved byte is non-zero;
- profile tags are invalid;
- Energy profile dimensions disagree with the encoded profile;
- Energy state dimensions overflow;
- Energy counters cannot produce a valid checked derived-energy cache;
- Parity shape is invalid;
- Parity packed length differs from the exact shape-derived length;
- Parity unused padding bits are non-zero;
- trailing bytes exist.

## Compatibility policy

The v1 byte layout is a persisted compatibility contract after M5 merges.

Changes that alter any of the following require a new format version or backend tag:

- hash/oracle interpretation;
- coefficient ordering;
- state bit ordering;
- counter ordering or width;
- profile-tag meaning;
- backend payload layout;
- checksum coverage/algorithm.

Adding a new public Rust API does not by itself require a new snapshot version if v1 bytes and semantics remain unchanged.

## Security and resource model

The decoder treats bytes as untrusted with respect to structural correctness and corruption.

CRC32C does not provide authenticity. If snapshots cross an adversarial boundary, authentication or a secure outer transport is required.

M5 does not add cryptographic signing, MACs, encryption or secret seeds.

The implementation validates shape and exact remaining payload size before allocating primary state derived from that shape.

## Verification

M5 commits fixed byte vectors for:

- an empty custom Energy sketch;
- an empty Parity sketch;
- a Parity snapshot with a deliberately non-canonical padding bit.

Tests cover:

- byte-for-byte deterministic encoding;
- custom Energy round-trip;
- theorem-profile Energy round-trip;
- exact config preservation;
- post-roundtrip algebraic cancellation;
- Parity custom-shape round-trip;
- wrong-backend failure;
- corruption failure;
- unknown version/flags failure;
- trailing-byte failure;
- canonical padding rejection;
- CRC32C Castagnoli reference vector `123456789 -> 0xE3069283`.

## Non-goals

- serde;
- file APIs;
- network framing;
- compression;
- streaming decode;
- mmap/zero-copy;
- cryptographic authenticity;
- strict finite-sample Parity;
- changing Energy or Parity update/merge/estimate semantics.
