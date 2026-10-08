# OPT-C C3-C — sender-first level probe after C3-B NO-GO

Issue #80/#74. Independent preregistered post-C3-B transport hypothesis;
do NOT reinterpret or edit C3-B result. Status: FROZEN BEFORE MEASUREMENT.

## Hypothesis

C3-B required a second RTT for the first parity request after session
negotiation and therefore failed the strict p95 end-to-end gate at RTT<=50ms.

Instead, each compatible independent sender receives the externally
known `T:u64` and **eagerly sends one predeclared T-centered bitmap
level** in the same first response as the session header, before
receiving a separate level request. The two senders do not exchange
sketch contents; neither knows the combined XOR state.
The receiver XORs the two level payloads, evaluates U, and can stop
immediately if `U<=T`. If not, it requests more identical levels
from each sender in a second RTT with exact fallback accounting.

No data-dependent tuning of the first level. Its index is
`clamp(round(log2(T/m))+1,1,J)` as specified in C2 and uses ONLY
preconfigured external T; this may be poor on other distributions.
The statistical per-level alpha remains fixed and simultaneous.

## Fair comparison

- DUAL_FULL: two independent complete canonical sketch transfers:
  53,500B application bytes, 1 RTT for each concurrently served peer.
- DUAL_PUSH1: two independent header+plan+first-level frames on first
  response. First stage = `2*(122+533)=1,310B` and one RTT.
  Keep both actual received 512B parity levels and validate XOR upper
  bound. The receiver is free to stop on U<=T, or explicitly extend.
- Forced retained extension checkpoints (not automatically needed
  for original T): 4 levels = `2*(122+4*533+17)=4,542B`
  and full 52 = `2*(122+52*533+34)=55,744B`.
  A full fallback is more expensive than direct DUAL_FULL;
  do not reclassify it as compression.

Both senders must be separately hosted in TCP loopback sockets;
receiver must count actual successful read/write application bytes
and verify sender counters, level-tag/CRC, distinct immutable
epochs with the same key/table/config, full restoration after
52 levels, and wrong-epoch/duplicate rejection.

## Hosted evaluation

Five independent GitHub-hosted workers; frozen 3 workloads d/T:
4096/8192,65536/131072,1048576/2097152. Eight nonadaptive seeds
each; application pacing at 10Mbps and RTT 0/10/50ms, same control
as C3-B, modes DUAL_FULL and DUAL_PUSH1; at stage1/4/52 record
cumulative bytes/time/U and success. Pair order rotates by seed.

Exactly 5*3*8*3*4=1,440 control/stage records; source HEAD,
all modes/stages, terminal full XOR and frame counts validated.
The frozen C3 product gate: p95 bytes reduction >=25% and no p95
wall-clock regression against functionally matched DUAL_FULL at
10Mbps RTT 0/10/50, in >=2/3 scenarios on **all 5 workers**.
No changes to thresholds if it fails. Missing/wrong records INVALID.

A pass is at most `C3C_SENDER_FIRST_SCOPED_PASS` for a **threshold
assessment combining two independently held compatible states**,
under trusted/externally authenticated protocol configuration.
It is NOT a general-purpose full interchange compression result,
a WAN measurement, or an authenticated malicious-peer guarantee.

This sender-first policy is an application-layer transport
experiment. No public API, Snapshot v1 change, new finite-sample
guarantee or oracle migration. Key distribution and lifecycle
remain part of separate public gate #69.
