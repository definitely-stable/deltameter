# STRICT-COMPACT OPT-C C2 — physical transfer and modeled network evidence

Issue #74; C0/C1 foundation merged in PR #78.
C2 PR #79; protocol frozen before measurement:
[STRICT-COMPACT-OPT-C-C2-MEASUREMENT-PLAN.md](STRICT-COMPACT-OPT-C-C2-MEASUREMENT-PLAN.md).

## Verdict and boundaries

**C2_PHYSICAL_ACCOUNTING_PASS** with independently reproduced real Rust
header, 533-byte level frames and received-state checks.
The **analytical** lossless-link model produces
**MODELED_C3_CANDIDATE** — NOT a final PROGRESSIVE_PASS.
Actual network RTT, buffering, cancellation, reordering, application
security and full snapshot reuse were **not measured**.

Source commit: `3994b6e0e74ef51d4a50245a883c2e25939d9d55`.
GitHub-hosted workflow:
[37812390251](https://github.com/definitely-stable/deltameter/actions/runs/37812390251).

Five independent workers PASS; aggregator PASS; **10,560** source-complete
raw C2 observations and exact source-SHA attestations. Each worker uses
32 fixed unique u64 token-range offsets for each of three predeclared
d/T scenarios (480 distinct J52 sketch observations across workers).
Orders ascending, descending and predeclared T-centered are included;
seven modes per order plus full transfer control.

Source-level correctness checks reconstruct complete bit words, maintain
all partial received-level bytes, verify the Q32 one-sided bound on
the received subset, preserve missing-vs-explicit-zero semantics and
check all wire-format byte counts.

## Exact transport costs

| Research transfer | Application bytes | Interpretation |
|---|---:|---|
| COMPLETE | 26,750 | 96B session, 8B T request, 18B plan, 26,624B bitmap, 4B CRC |
| PUSH_ALL | 27,838 | 52*533B frames + common 122B; no sender stop |
| Single-level sender-bounded prefix | 655 | one 533B frame + common 122B |
| Interactive batch of four, one RTT | 2,271 | four 533B frames + common 122B + 17B request |

**Critical:** a receiver stopping locally cannot save bytes already sent.
BOUNDED_PREFIX requires sender-side preknowledge of T, while interactive
work requires feedback, per-batch request bytes and RTT.

The sampled worker-1 center-order p95 outputs were:

| d / external T | COMPLETE p95 | Center bounded p95 | Center interactive-4 p95 | Useful in sampled states |
|---|---:|---:|---:|---|
| 4,096 / 8,192 | 26,750 | 655 | 2,271 | 32/32 |
| 65,536 / 131,072 | 26,750 | 655 | 2,271 | 32/32 |
| 1,048,576 / 2,097,152 | 26,750 | 655 | 2,271 | 32/32 |

These workload-specific outcomes **do not** say one level always
certifies a bound, nor that arbitrary T/profiles are as favorable.
The T-centered order uses the predeclared application threshold,
not hidden true d. In the aggregator all five workers satisfy the
*frozen modeled* >=25% p95 bytes and no modeled 10Mbit/s,
RTT<=50ms regression gate for a nonempty set of common candidate orders.

The network grid is modeled at 1/10/100 Mbit/s and
0/10/50/150 ms round-trip latency with measured local CPU contributions.
A lossless serial-link model is not real end-to-end transport evidence.

## Durable provenance

GitHub artifact ZIP object SHA-256 digests:

| Artifact | ID | SHA-256 |
|---|---:|---|
| Summary | 11565960858 | 1e741aadeb50ab84c5f29365b1d38892f02af4d8ecc96562b4e9ae2a75ca0b00 |
| Worker 1 | 11565198880 | 0d3e2946b8981e7cd21f967e8cf1d38d0d5742a117de96a5a397cb8e279fc5b0 |
| Worker 2 | 11566235245 | 6042acfccc18620c84ca9ad4a0c95ef944a4aa1c09a7905bae85b025770311fb |
| Worker 3 | 11566290178 | a5b8267cedead47a8881f9cb14a0e0e5c101cea52594bf58acf3f7aaa62bc604 |
| Worker 4 | 11564664655 | 495184fb277f17874be71e5f80e4755e31e487a8b1a5ac232108bfb6cea83e49 |
| Worker 5 | 11565387968 | b3dba1caf6664518c86de7a3ff124b8de807976ad7cae7c0c319f31919376353 |

## Critical product comparator before final C3

Physical savings versus the COMPLETE state **do not alone justify a
progressive library feature**. They have different semantics:

1. **Threshold-only decision:** for a *trusted* sender, the sender can
   compute Q32 U from its locally held complete bitmap and send a
   `u128` bound or a Boolean, in far fewer than 655 bytes. Without
   sender authentication/attestation, unauthenticated 533B parity frames
   also cannot convince a malicious peer. A sender-result control is
   REQUIRED in final product decision.
2. **Reusable partial parity state:** one or more actual bitmap levels
   can be merged by XOR with matching-key/profile level data and later
   extended. This property is genuinely different from transmitting
   one scalar U. It matters only if multi-round verification, partial
   sketch reuse or compatible distributed accumulation is required.
   Benchmark total lifecycle bytes/latency, and count snapshot persistence.
3. **Full-state interchange:** 655B does not reconstruct all 52 levels.
   Operations requiring a complete canonical sketch must account for
   remaining levels; early-stop traffic savings cannot be counted as
   full-state interchange compression.

Hence C3 must include **fair functionality-equivalent competitors**, a
controlled transport path and an explicitly declared trust/reuse contract.
This is an additional critical-product check, not a retrospective change
to the preregistered C2 timing/bytes samples or thresholds.

## Remaining blockers

- No measured controlled end-to-end latency/RTT or backpressure.
- No authenticated sender contract; nonsecret tag/CRC detect only
  accidental mixups and corruption.
- No app-lifecycle workload where partial sketches are reused and
  baseline simple `sender-computed U` is measured.
- Public profile/API/snapshot freeze #69 remains BLOCKED.
- Keep issue #74 open through final C3 decision.

Do NOT rename the MODEL result to PROGRESSIVE_PASS or publish an API yet.
