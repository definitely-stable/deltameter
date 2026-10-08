# OPT-C C3-B — preregistered two-party incremental lifecycle

Issue #80 / #74; stacked on successful C3-A source, no public changes.

## Fair functional comparison

Two *independently held* token sets each maintain keyed J52 parity states
with **identical** frozen profile/key, distinct epoch and sender identity.
Neither sender knows the other party's state, so neither can
compute the receiver's symmetric-difference bound unilaterally.
A trusted scalar bound from one sender is NOT a functional competitor.

Using real TCP loopback with separate listener/stream per sender,
compare:
- DUAL_FULL: two independently serialized complete 26,624B
  canonical bitmaps, CRC, session headers, exact XOR at receiver.
  Expected total application traffic = 2*26,750 = 53,500B.
- DUAL_RETAINED: two live sessions with explicit request masks.
  Receiver gets same first T-centered level from both senders,
  XORs levels and evaluates U. Retains both immutable states for a
  second, tighter or differently parametrized query, extends to 4
  levels; final third request retrieves all remaining levels and
  validates exact canonical full XOR state. No re-sending old levels.
  Expected application byte checkpoints:
  n=1: 2*(122 + 17 + 533) = 1,344B
  n=4: 2*(122 + 2*17 + 4*533) = 4,576B
  n=52: 2*(122 + 3*17 + 52*533) = 55,778B.
  The n=4 formula includes two request control frames; no
  compression or zero-elision assumption.

If client abandons sessions after initial U<=T, count all completed
writes and in-flight bytes. For forced extension to 4/52 levels the
cost of extra states is charged, never mislabeled as one-shot savings.

## Evidence

Five GitHub-hosted independent workers, each with the previously
frozen 3 d/T profiles and 8 fixed nonadaptive token seeds, 10Mbps
application pacing and RTT 0/10/50ms. All stages in one session pair.
5*3*8*3*4 = **1440 logged checkpoints** (full, retained1,
retained4, retained52). Strict provenance for every worker and
exact source HEAD. Compare per-worker/scenario/RTT matched
p50/p95 bytes/time, including fallback to full. Pair order rotated
by seed to avoid a fixed warming artifact.

Each sender transmits its own 96B session, 18B plan, level frames,
and receives the actual 8B T request plus 17B request mask per
incremental round. Both sender write/receiver read byte counters
must match. The receiver validates CRC, level index, generation
binding, no duplicates, same-key/profile and u128 Q32 bounds.
Only compatible level IDs may be XORed. Confirm final reconstructed
XOR equals direct full sketch XOR. Generation change and corrupted
frames must fail closed.

Measure and report clone/freeze, local TCP wall-clock from first
connect through both joined workers, serialization/parse, total
application bytes. App-pacing sleep is an **emulated** lossless
10Mbps and RTT, NOT WAN evidence.

## Decision limits

- At first threshold only, require >=25% p95 bytes and no p95
  end-to-end regression at 10Mbps RTT<=50ms, in >=2/3 scenarios
  for all 5 workers; report, do not cherry pick.
- For second-stage reuse, compare incremental cumulative bytes
  and wall time vs making two separate full transfers; document
  the *different output* (partial upper bound rather than full state).
- For complete interchange, compare retained52 cumulative
  against DUAL_FULL; **expect prefix framing overhead**. This must
  not be marketed as full-state compression.
- A final PROGRESSIVE_PASS is only meaningful for a clearly scoped
  **compatible two-party partial-upper-bound application**, with
  shared-key operational/trust assumptions. No universal backend
  success, no authenticated hostile-peer guarantee, no public API
  until a separate lifecycle/key-management approval under #69.

No snapshot-v1/algorithm/public API changes. If a planned lane
cannot be executed accurately in GitHub-hosted CI, return INVALID
rather than substituting a network model.
