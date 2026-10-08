# STRICT-COMPACT OPT-C — progressive level-transfer protocol

Issue #74; parent #70; OPT-A merged (#76); OPT-B merged (#77).
Status: **PRE-REGISTERED C0/C1 FOUNDATION; NO TRANSFER VERDICT YET**.

Mathematical/stopping conditions are stated and justified FIRST in
[STRICT-COMPACT-OPT-C-PROOF.md](STRICT-COMPACT-OPT-C-PROOF.md).
No new finite-sample theorem, random-oracle replacement or public format.

## Frozen source/profile

m=4096, J=52, LEVEL_MAJOR, physical parity bitmap 26,624 B.
Each level has exactly 512 B of parity data. Shared Q32 table remains
16,384 B with SHA-256
`634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`;
ideal failure <=1e-6 and the previously declared keyed-BLAKE3 v1
computational/nonadaptive input assumptions are unchanged.
The derived OPT-B cache is NOT part of transmitted canonical state.

## C1 — wire foundation (research representation v0, not a snapshot)

Sender MUST freeze an immutable generation prior to transmission.
Research session header binds:
- protocol version, m, J, oracle/table profile;
- nonsecret config/key binding ID (32 B; does not reveal the key);
- immutable generation ID (16 B) for the entire transfer.

A level frame has an explicit 1-based index in 1..=J,
an immutable generation ID, **exactly 512 B** payload,
and an error-detection checksum. Experimental reference frame:
`epoch[16] | level[1] | parity[512] | crc32[4]` = **533 B**.
CRC32 is for accidental corruption only, not adversarial authentication.
The session header (including config/key identity) is additional
application bytes and must be charged exactly in the cost model.
No secret key is serialized.

Receiver state has an explicit presence bitmap:
- no levels => bound = `2^64` as u128;
- absent != received-zero-level;
- a duplicate identical frame is idempotent or explicitly rejected,
  but a duplicate conflict always fails closed;
- unknown index, wrong epoch/config/version, truncated payload, bad checksum,
  mixed snapshots and reordered conflicting chunks reject;
- in-order and permuted delivery must yield the same bound for the same
  received subset of chunks;
- bound never increases as new valid levels are received;
- completed transfer must reconstruct the exact canonical bitmap and
  OPT-A Q32 estimate.

C1 evidence: deterministic Rust fixtures, property/corruption tests and
standard GitHub-hosted CI. No product PASS is inferred from C1 alone.

## C2 — frozen candidates and orders

Compare four physically accounted alternatives:
1. COMPLETE: one framed session + contiguous 26,624 B full state + checksum.
2. ORDERED_STREAM: one framed session followed by 533-B level frames in a
   fixed, deterministic order; receiver can stop on an observable U<=T.
3. INTERACTIVE: batched request/response growth (batch size 1,2,4,8),
   all additional RTTs charged, full fallback = request remaining levels.
4. ZERO_LEVEL_ELISION: explicit bitmap of omitted **proven all-zero** levels
   produced from a frozen sender snapshot; missing levels must never be
   interpreted as zero without the explicit canonical elision metadata.

Evaluate low-to-high, high-to-low, and profile/T-informed deterministic order,
with explicit level IDs (the last depends on externally predeclared application
threshold T, NEVER on unknown d or an adaptive token oracle attack).

## C3 — measurement/accounting gates frozen before timing

Receiver-only stopping criterion:
`U(L) <= T` for externally configured T. A deadline or budget can return
ExceededOrUncertain. `U/d<=1.5` is only an offline label for synthetic
known-d evaluation, never an execution stop predicate.

A fixed representative synthetic corpus grid:
- d=4,096, T=8,192;
- d=65,536, T=131,072;
- d=1,048,576, T=2,097,152.

Thresholds are application policy inputs selected **before observation**;
they are not calculated by the receiver from unknown true d.

For each order/protocol/profile report initial bytes, p50/p95 bytes to a
useful bound, max bytes, framing/metadata bytes, number of requests, RTTs,
and exact upper-bound results. Report probabilities of no useful bound
before the full profile.

Network grid: 1 / 10 / 100 Mbit/s, RTT 0 / 10 / 50 / 150 ms.
The network model must account for in-flight bytes and sender-side
backpressure/cancellation, not assume unsent bytes disappear after a stop.
Compare wall-clock completion to COMPLETE, with explicit per-request RTT
for interactive lanes and exact framing bytes for every candidate.

C3 product-gate recommendation frozen here:
- PROGRESSIVE_PASS iff a valid ordered or interactive candidate achieves
  >=25% p95 application-byte savings vs COMPLETE in at least **two of the
  three named workloads**, while not regressing end-to-end completion time
  vs COMPLETE in those workloads at 10 Mbit/s and RTT <=50ms (including 0/10/50);
  no hidden sender-copy or session-setup cost.
- STREAM_ONLY iff progressive bytes are materially smaller in at least two
  scenarios (>=20% p95), but interaction/RTT fails the time gate; keep one-shot
  ordered stream only if its actual framing/time wins.
- DROP_PROGRESSIVE otherwise.

No post-hoc promotion from tuning workload grid to observed results.
An INVALID correctness, measurement completeness or provenance result
blocks ALL product verdicts even when savings appear attractive.
Keep raw counts/bytes/time provenance separately from the mathematical proof.

## Deferred public decisions

No Snapshot v1 edit; no new public API, persistent format or `Coverage::Proven`
alias. #69 remains blocked until the C3 gate closes. D/E/F research lanes
remain deferred.
