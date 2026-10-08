# OPT-C C3-A — controlled TCP and two-party parity evidence

Issue #80 / #74, draft PR #81. Frozen protocol:
[STRICT-COMPACT-OPT-C3A-PROTOCOL.md](STRICT-COMPACT-OPT-C3A-PROTOCOL.md).

## Result

**C3_A_TRANSPORT_FOUNDATION_PASS**, not final product
`PROGRESSIVE_PASS`.

Source commit measured:
`0968adec45d14261f950b3f8d6a9690c27407424`.
GitHub-hosted five-worker run:
[37815243109](https://github.com/definitely-stable/deltameter/actions/runs/37815243109).
Five workers and fail-closed aggregator all PASS: **1,440 actual
localhost TCP request/response observations** plus **360 deterministic
two-party per-level XOR/reuse-oracle observations**.

C1/C2 reference logic is extracted to one private shared include
`research/strict_compact_opt_c1_core.rs`; the earlier C1/C2 example
and newly added C3 example use identical oracle, Q32 inference and frames.
This is research-only, not a public library change.

## What was measured

Per worker: 3 predeclared d/T workloads × 8 independent fixed
nonadaptive seeded sketches × 3 application-layer RTT settings
(0,10,50ms) × 4 protocols (scalar/full/bounded/interactive4).
All requests use real `TcpListener`, `TcpStream`, framing,
`read_exact`/`write_all`, timeouts, counters and `Instant` timestamps.
The application emulates a 10Mbps sending link by sleeping before writes
and an RTT by delaying responses. **This is not network shaping and
does not establish actual WAN or kernel buffer behavior.** Client
request bytes equal server received bytes; server reply bytes equal
client received bytes.

### Worker 1, RTT target 10ms, p95

| Scenario d / T | Protocol | Application bytes | p95 wall time |
|---|---|---:|---:|
| 4,096 / 8,192 | scalar U from trusted sender | 24 | 10.609ms |
| 4,096 / 8,192 | full canonical bitmap | 26,750 | 32.386ms |
| 4,096 / 8,192 | bounded sender-level prefix | 655 | 11.305ms |
| 4,096 / 8,192 | interactive 4-level batch | 2,271 | 23.181ms |
| 65,536 / 131,072 | scalar | 24 | 10.576ms |
| 65,536 / 131,072 | full | 26,750 | 32.405ms |
| 65,536 / 131,072 | bounded | 655 | 11.298ms |
| 65,536 / 131,072 | interactive4 | 2,271 | 23.396ms |
| 1,048,576 / 2,097,152 | scalar | 24 | 10.616ms |
| 1,048,576 / 2,097,152 | full | 26,750 | 32.277ms |
| 1,048,576 / 2,097,152 | bounded | 655 | 11.340ms |
| 1,048,576 / 2,097,152 | interactive4 | 2,271 | 23.131ms |

All named worker-1 cases yield U<=T in 8/8 seeded observations.
The aggregator confirms the **predeclared restricted controlled-lab
criterion** on all five workers for `bounded` across all three
scenarios, while `interactive4` qualifies in 0/3 scenarios.
The positive restricted gate MUST NOT be restated as a public
PROGRESSIVE_PASS or universal latency claim.

### Functional comparison

1. **Threshold-only with trusted sender**: returning the sender's
   precomputed `u128 U` after 8B `T` input is only **24B total**;
   it is strictly smaller than every 655B parity-prefix transaction
   in these scenarios. The scalar carries no independently verifiable
   witness against a malicious sender. Neither does the unkeyed
   session digest/CRC of the research frame.
2. **Two separately held compatible states**: a receiver can XOR
   corresponding 512B level payloads and compute Q32 on the merged
   subset, then extend the same retained state with missing levels.
   On this seeded matrix, one level from EACH sender gives a useful
   partial bound; two senders together cost **1,310B** in the
   current logical per-sender framing model. Four levels: 4,508B;
   all 52: 55,676B. All levels reproduce exact full canonical
   XOR state and inference. These are a local functional
   oracle and **logical byte model, NOT two actual TCP sessions**.
3. **Complete interchange**: full-state reconstruction still costs
   full state; partial upper confidence result cannot masquerade as
   full snapshot transfer.

## Evidence provenance

GitHub artifact ZIP SHA256:

| Kind | Artifact ID | Digest |
|---|---:|---|
| summary | 11566972287 | 803d2b420d9beb145cb7b7d066be1d09647de9832b3a18ea5a64e83468409c0b |
| worker1 | 11565969252 | d38cbc31dd1ef2530f6b72af3ea26795837f1cffb3d4a517cd5279716a960a62 |
| worker2 | 11567305750 | cdd1848b5fc517ad1eee7c595969adca72901b3931fc0d208e5cf0348d5fa1d3 |
| worker3 | 11566857466 | 2f5ba4b38ba6f431a0da606f375bfff4c87cb9dbbbbe83f1cfad7ad890fc7140 |
| worker4 | 11567315789 | c189aad40fc89409c35314cd0c4a64395bbbe0f52f69009272b701e5b1801cbf |
| worker5 | 11567011654 | 1b0848dd271989657807e352ff5c21290832a244500b7b4a3aa43b2ebf75d851 |

## Next product gate

C3-B must execute **actual two-party incremental TCP lifecycle**,
with two changing or unchanged frozen generations, charged RTTs,
sender copy, per-round requests, bytes, retention/storage, and
full-state fallback. Compare both senders independently transferring
full state and an authenticated/trusted sender-result control
where it is functionally meaningful. Include recovery and failure
cases. Only the final C3 controlled network measurement can decide
PROGRESSIVE_PASS/STREAM_ONLY/DROP_PROGRESSIVE for a precisely
defined application class.

Keep #80/#74 open, #69 public profile/snapshot frozen pending final
decision. No mathematical coverage promotion or new public wire format.
