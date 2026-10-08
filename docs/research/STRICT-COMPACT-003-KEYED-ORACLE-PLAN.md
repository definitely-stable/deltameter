# STRICT-COMPACT-003 — keyed PRF oracle feasibility plan

Issue: #65. Parent: #59.

Status: research-only product gate. No public API or snapshot change.

## Frozen candidate

Use pinned Rust `blake3 = 1.8.7` as a dev-only dependency.

At construction time derive a protocol-specific 256-bit subkey using the fixed
context string

`deltameter 2026-10-08 strict-compact keyed oracle v1`.

Per u64 token perform exactly one keyed BLAKE3 invocation over the canonical
little-endian token bytes. Consume disjoint 64-bit output words for row, level and
GF(2) coefficient.

For the frozen STRICT-COMPACT profile m=4096:
- row uses 12 output bits, hence exactly uniform in the ideal random-function model;
- level uses trailing zeros of a separate 64-bit word;
- coefficient uses one bit from a third word;
- level-word zero is the 2^-64 truncation event already represented by J=64.

No rejection loop and no per-toggle heap allocation is present in DeltaMeter's
candidate code path.

## Security/coverage statement under test

This is not information-theoretic Coverage::Proven.

The intended research statement is:

> For a token set chosen independently of a fresh uniform secret oracle key, and
> assuming keyed BLAKE3 behaves as a PRF/random-function replacement for this use,
> the ideal-sampling STRICT-COMPACT theorem transfers up to the computational
> distinguishing term.

Adaptive adversarial token choice after observing sketch responses under the same
key is explicitly outside this contract.

## Correctness gates

The research lab must verify:
- same token/key maps deterministically;
- toggling the same token twice cancels;
- XOR merge matches combined toggles;
- state remains exactly 4096 u64 words = 32 KiB;
- broad row/level/coefficient distribution sanity passes (diagnostic only).

No empirical distribution check is theorem evidence.

## Hosted timing protocol

Five independent GitHub-hosted workers.

Each worker:
- release build;
- deterministic 250k-token stream;
- warm-up outside samples;
- 3 measured rounds;
- alternating parity/keyed arm order;
- median native Rust ns/token.

Compare:
1. current PublishedFpcsaF2 / mix64 pseudo-oracle;
2. keyed BLAKE3 32 KiB candidate;
3. Energy default as descriptive context.

Frozen PASS gates on **every worker**:
- keyed median <=500 ns/token;
- keyed/parity median ratio <=10x.

Energy ratio is reported but not gated.

Any correctness failure or missed performance gate => candidate does not proceed.

## Operational costs retained

A future production design would require:
- externally provisioned 32-byte secret;
- same secret/config on all merge participants;
- key rotation implies rebuild;
- snapshot v1 must not silently embed the secret;
- key identity/versioning requires separate format/API design.

This slice does not solve key management.

## Evidence amendment before canonical run

The first hosted attempt, workflow run `37721691447`, is **INVALID**.
Terminal output was piped through `tee`, which both left the artifact timing files
empty and masked a possible nonzero `cargo run` status because the shell pipeline
did not use pipefail. It therefore proves neither performance nor correctness and
contributes no verdict evidence.

The first durable follow-up exposed a separate fixture bug before timing: a fixed
token was assumed to mutate state, but a valid F-PCSA sample may have coefficient
zero (or be truncated) and is then correctly a no-op. The semantic regression now
searches deterministic fixtures from the keyed oracle itself: one guaranteed stored
sample must toggle and cancel, and one guaranteed no-op sample must leave state
unchanged.

Neither invalid attempt contributes numeric timing evidence.

Before the canonical run:
- the Rust lab now writes its own durable result record;
- CI requires the result file to be non-empty;
- a separate Python aggregator requires exactly five worker files and reapplies the
  frozen <=500 ns and <=10x gates;
- the aggregate summary becomes the durable evidence surface.

The measurement algorithm, token stream, warmup, round count and performance gates
are unchanged.

## Outcomes

GO_COMPUTATIONAL_STRICT_PRIVATE_API:
five-worker performance and correctness gates pass. A later issue may design a
private computational-coverage API.

GO_RESEARCH_ONLY_ORACLE_GAP:
candidate is mathematically interesting but crypto/update/key-management cost is not
worth productization.

STOP_COMPACT_PRODUCT:
practical oracle realization destroys the product advantage.

No public Coverage::Proven is authorized by any outcome here.
