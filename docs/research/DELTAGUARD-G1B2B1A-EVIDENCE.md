# DeltaGuard B2-B1-A — two-owner TCP frame correctness evidence

Parent [#92](https://github.com/definitely-stable/deltameter/issues/92);
draft [PR #95](https://github.com/definitely-stable/deltameter/pull/95);
frozen [B1 protocol](DELTAGUARD-G1B2B1-PROTOCOL.md) before experiments.
Research-only; original strict one-sided B2A cutoff and snapshot v1 untouched.

## Scoped verdict: B1_A_FRAME_CORRECTNESS_PASS (not measured product GO)

Last source-code HEAD verified:
`e63608dc3cc870bb45b835012ff1cc1e28103609`.
[Five-hosted-worker TCP run #37899112626](https://github.com/definitely-stable/deltameter/actions/runs/37899112626):
**all 5 workers SUCCESS**.
[Research #37899112517](https://github.com/definitely-stable/deltameter/actions/runs/37899112517)
SUCCESS. Dedicated flow includes format/clippy, compile, true loopback sockets,
independent hash-bit reference over exact symmetric difference, actual
sender/receiver frame byte equality, and fail-closed malformed frames.

Precisely 3 source differences d={48,57,65}, N=256, T=64 per worker;
15 physical two-sender TCP pair transactions per worker = 75 across five
GitHub-hosted jobs: valid GUARD, valid DIRECT_FULL, tampered GUARD,
real short-payload transfer, and duplicate-owner transfer per d.
The fixture source IDs differ by worker. Negative offline vectors
also cover wrong key/profile/owner/epoch/generation, invalid bitmap padding,
noncanonical exact sorted lists, invalid delta shape/op, truncation, and
modified header/body. Unkeyed BLAKE3-128 research checksum is **not
authentication**. The receiver rejects before evaluating S on malformed
or incompatible input. Sender actors use separate TCP connections but
remain threads in the same research process, not independent hosts.

Observed physical application bytes (including 64B frame per owner):

| Source difference | Guard actual two-owner bytes | Direct full bytes | SAFE outcomes / five workers |
|---:|---:|---:|---:|
| d=48 | 640 | 4,224 | 5/5 |
| d=57 | 640 | 4,232 | 0/5 |
| d=65 | 640 | 4,232 | 0/5 |

Each receiver XOR is independently validated against direct BLAKE3
per-token rehashing of the exact two-list symmetric difference.
For d48, S<=d<=48 gives SAFE pointwise. At d57/65 the five specific
deterministic fixture outcomes are UNKNOWN; this is not a quantitative
power curve or adversarial guarantee. No exact fallback, TCP ACK/retry,
network pacing, p95 latency, RSS or cached receiver-exact lifecycle
is measured in B1-A.

## Why B1-B remains mandatory

A 640B physical TCP **size** in a cold single-query lab does not show that
repeated DeltaGuard beats a receiver that retains exact A/B and applies
properly framed incremental event records after once-only bootstrap.
The next slice must implement physical retained exact (not model-only),
all retry/generation checks, full fallback, measured per-worker p95 and
cumulative S=1/10/100 query horizon on the frozen N/d/RTT matrix.
Preserve the public-deterministic-fixture vs secret-key security distinction;
#86 and #69 remain required before any public API.

The accepted B1-A protocol correctness does not supersede D56's B2-B0
analytic-wire limitation and does not reopen public Snapshot v1.
