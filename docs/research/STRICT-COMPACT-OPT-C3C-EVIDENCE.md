# OPT-C C3-C sender-first two-party TCP: final research evidence

Issue #80 / #74. Independent hypothesis after frozen C3-B
request-first NO-GO; not a revision of D50 or the original thresholds.
Protocol: [STRICT-COMPACT-OPT-C3C-PROTOCOL.md](STRICT-COMPACT-OPT-C3C-PROTOCOL.md).

## Gate decision

**C3C_CONTROLLED_TWO_PARTY_PASS** under the ORIGINAL >=25% p95 byte
savings and no p95 time regression conditions at controlled 10Mbps
application pacing and RTT=0/10/50ms, on all five GitHub-hosted workers
in **3/3 predeclared d/T scenarios**.

Measured source:
`9db84aecd04a93ad72cd603453bb6970d1fb42fa`.
CI [37817271128](https://github.com/definitely-stable/deltameter/actions/runs/37817271128):
5/5 workers, strict fail-closed aggregate, **1,440 actual two-source
TCP checkpoint observations**. Source SHA and complete per-worker
d/T/seed/RTT/mode/stage identity verified.

One session per independent sender, shared keyed oracle/profile but
distinct immutable epochs, T-known deterministic T-centered first
level. The sender transmits this first canonical 512B parity level
together with its initial session header (NO separate first-level
request). Receiver XORs both and computes unchanged Q32 upper bound.
Later new queries extend both stored partial states to 4 and 52 levels
and validate every bit of full reconstruction. No oracle/table,
statistical alpha or confidence theorem changed.

## Measured physical application bytes

| Two independent senders | Bytes | Functionality |
|---|---:|---|
| DUAL_FULL | 53,500 | entire two sketches immediately |
| DUAL_PUSH1, first level per sender | 1,310 | partial combined upper U, not full state |
| DUAL_PUSH1 extended to 4 levels | 4,542 | retained compatible partial levels |
| DUAL_PUSH1 extended to all 52 | 55,744 | entire two sketches, **4.19% overhead** |

Sender and receiver successful application byte totals match exactly.
Each frame has its C1 session binding and CRC, and failed/mixed epochs
are rejected. CRC/unkeyed BLAKE3 binding are **not authentication**.

## Representative hosted observations

Worker 1, **application-emulated** RTT target 10ms, 10Mbps:

| d / external T | Dual complete p95 | PUSH1 p95 | 4 levels p95 | all levels p95 |
|---|---:|---:|---:|---:|
| 4,096 / 8,192 | 32.626ms | 11.294ms | 23.454ms | 62.648ms |
| 65,536 / 131,072 | 32.647ms | 11.915ms | 23.715ms | 58.824ms |
| 1,048,576 / 2,097,152 | 32.402ms | 11.701ms | 23.492ms | 58.143ms |

All 8/8 workload-1 seeded samples reached the predeclared U<=T
threshold at one level in the shown worker; complete sampling and
five-worker aggregated gate PASS all three workloads.

**Contrast with C3-B D50:** the request-first retained level incurred
an extra network feedback RTT and failed 0/3 frozen workload gates.
The *distinct* sender-first design passes 3/3 without changing alpha,
T/d grid, p95 thresholds, J52 or confidence assumptions.
The C3-B NO-GO evidence remains canonical and unchanged.

## Evidence provenance (GitHub artifact ZIP digests)

| Artifact | ID | SHA-256 |
|---|---:|---|
| summary | 11567483079 | 65168ee63826208507838a0c993d00d32198710bbaf7843d182ab3272ccfdb9c |
| worker1 | 11568345112 | a5ba7ca1b0e0464713b5b0293726f4425dd803015b5bafc7e7f8e82c921a932e |
| worker2 | 11567542286 | 0f769ae4e0e7358a43818b122c35fb26b76cde6d4ee43852817b5f8e13dd27d1 |
| worker3 | 11568405103 | 731957f078757da1019ba5e449fb1a0431c8631f704f95f0ec06c8372853dcfb |
| worker4 | 11567512658 | 46cffb9c0ff0d0dd1e79c12ac45717e3e31052384b8956408453401ccaa432ca |
| worker5 | 11566859843 | 0a0be6a2ff26d6c01926bf3d2079ab828dff722e4555afad20fdae467a3f5880 |

## Critical interpretation and public product decision

This is **real localhost TCP IO** paced by application-level sleeps at
10Mbps and nominal RTT (0/10/50ms), NOT kernel tc shaping, public
WAN, hostile-peer authenticated networking or cross-hardware
performance guarantees.

The product utility is sharply limited:
- Threshold-only trusted peer: sender-computed U/Boolean (24B)
  dominates progressive parity framing; **NO generic progressive**
  feature is warranted for this use.
- Two mutually independent, compatible sketch owners where the
  receiver must XOR source levels: PUSH1 can beat full state **when
  one early predeclared T-level suffices** and its state is immutable.
  A generic one-party scalar cannot compute this combined U.
- True full interchange: sending all 52 framed levels costs MORE
  than full-state v1, so no compression/free interchange claim.
- Recipient trust/ownership/key provisioning: a secret keyed-BLAKE3
  oracle is shared outside the research frames. The draft does NOT
  specify secure key distribution, authenticated peers, epoch
  persistence/rollback, retries, replay protection or hostile
  adaptive-token security; therefore no public protocol is approved.

**Final C3 research recommendation**: accept the scoped sender-first
two-party threshold result as a future *optional* application-specific
research lane, but **DO NOT expose a general progressive interchange
API/profile in public v1**. Issue #69 can proceed with J52+cached
counts and a separate deliberate public key/snapshot lifecycle
contract, without waiting for further speculative transport tweaks.
