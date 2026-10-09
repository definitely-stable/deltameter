# DeltaGuard G1-B2-B1-B0 — actual retained exact and fallback TCP evidence

Parent issue [#92](https://github.com/definitely-stable/deltameter/issues/92),
research [PR #96](https://github.com/definitely-stable/deltameter/pull/96).
Frozen [B1-B0 protocol](DELTAGUARD-G1B2B1B0-PROTOCOL.md)
preregistered before the first B1-B0 CI run.
Ancestor merged B1-A proof of physical framing D57, fixed exact
B2A cutoff T64/b11/c48 and ideal nonadaptive false-SAFE <=1e-6,
NOT a new probability claim or original theorem.

## Evidence and version-bound checkpoint

**B1B0_RETAINED_PHYSICAL_EXACT_FALLBACK_PASS**, research/correctness
only; no p95 network or public product GO.

Initial fully successful five-worker source-attested run:
[37902586379](https://github.com/definitely-stable/deltameter/actions/runs/37902586379),
code commit `3aabf44ccf68687c0251d1e344813f39a4ec96d7`.
Independent exact B2A profile certificate recreated and pinned:
`cutoffs.tsv` SHA-256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.
All 5 workers (72 rows each) and fail-closed aggregator passed:
**360/360** unique fixed-source entries, source SHA/shape/byte equality,
keyed per-token independent XOR reference, complete exact two-list truth.

Follow-up review corrected a real atomicity issue:
`b1b_apply` now prevalidates **all** canonical batch events before
mutating the exact receiver list, preventing partial commits if a
late malformed operation occurs. Added adversarial late-batch regression
and regenerated full five-worker results at code source
`f885785749aa607aac5173ffb8755153bc26a8d0`,
[run 37902855683](https://github.com/definitely-stable/deltameter/actions/runs/37902855683):
**certificate, 5 workers, aggregator all SUCCESS**.
Rust/research CI are separate runs; cite exact branch HEAD CI after
documentation/review additions rather than retroactively mapping runs.

## Physically exercised source and receiver lifecycle

- Two source-owning threads hold independent canonical sorted `u64`
  vectors and 256B B2A bitmaps, each separately updated for
  **99** shared-token insertions after initial generation.
- A separately allocated *third-party receiver* physically obtains both
  initial exact source arrays over loopback TCP, then applies each
  independently sequenced and checked exact 9B event in a 64B frame.
  Exact receiver vectors are compared against source truth at EVERY
  one of the 100 generations (not just milestones).
- A second sparse receiver buffers those same canonical updates and
  physically receives one batched frame per source per checkpoint
  (at generations 10 and 100), testing canonical order and exact
  final-list equality.
- Two physical guard frames are actually transferred on EVERY
  generation (dense), with checkpoint transcript accounting for
  S=1/10/100 (sparse). Actual receiver XOR matches independent
  BLAKE3 per-token rehashing of exact A triangle B.
- Direct exact complete source lists transmitted separately on
  each checkpoint; when guard returns UNKNOWN, a new TCP exchange
  physically sends two 24B receiver requests (and 1B peer hello each)
  plus both full canonical exact responses. Both fallback owners,
  generation and full exact truth verified.
- All frame identity, missing/wrong owner, shape, key ID/epoch/generation,
  noncanonical and tampered message validations use B1-A; this new slice
  explicitly rejects duplicate insert, missing remove and a malformed
  late batch with **zero partial state mutation**.

These are actual bytes observed at the `TcpStream` application boundary,
not simulated application bytes; network is loopback and senders are
*threads in one process*, NOT independently deployed remote hosts.
The checksum uses a public test fixture and is NOT a malicious-peer MAC.
No ACK/retry/restart after crash or authenticated epoch protocol yet.

## Main result: query cadence reverses the preferred algorithm

All numbers below are cumulative *actual physical application bytes*
for N=256, T64, d48 and S=100. The same fixed dataset is used for
all compared modes.

| Contract through generation S100 | Actual bytes |
|---|---:|
| B2A guard queried EVERY generation (dense) | 64,000 |
| Receiver-retained exact with one owner event per generation | 18,678 |
| Receiver-retained exact with checkpoint batching (S1/10/100) | 6,262 |
| B2A guard queried only at S1/10/100 (sparse) | 1,920 |

Thus on N256, **dense guard transfers MORE bytes than a retained
streaming exact receiver** when exact's initial sync is paid and
an event is delivered every generation. Conversely, when only three
sparse checkpoints are requested, B2A guard wins against BOTH fixed
exact-update variants. A third party cannot magically obtain a
one-source 24B scalar; both modes have real owners A and B.

At N65536,d48,S100:

| Contract | Actual bytes |
|---|---:|
| Guard dense | 64,000 |
| Retained streaming exact | 1,063,158 |
| Retained sparse batched exact | 1,050,742 |
| Guard sparse | 1,920 |

Here the exact receiver's **1,048,704B cold bootstrap** dominates
over the 100-generation horizon, so the guard transfers fewer bytes
even under dense query cadence. The result is conditional on how many
sources, how often queried, whether complete exact receiver state
already exists, and initial-sync amortization. A receiver that already
holds both lists needs a separately measured *warm-start-only*
comparison; do not charge that topology cold bootstrap by fiat.

## UNKNOWN and exact-resolution stop

For N256,d57,S100, the 15 public-fixture samples were UNKNOWN:
guard response is 640B on this checkpoint, but physical fallback
adds TWO 24B physical requests, 2B sender hello, and complete
canonical source transfer 5,816B, for **6,506B total resolved**.
That is MORE than DIRECT_FULL 5,816B before any RTT penalty.
For N256,d48,S100 the fixed c48 cutoff makes SAFE deterministic
pointwise, so 640B per single cold guard query ends without fallback.

The keyed public deterministic fixture is not an independent
statistical test of a one-in-a-million ideal-oracle theorem.
A rare permitted false-SAFE at d>T must be *counted*, not
automatically classified as implementation corruption.
The current nonadaptive one-profile confidence statement is NOT
proven for arbitrary adaptively chosen token sets/queries.

## Known exclusions / next evidence-gate

B1-B0 does NOT measure accepted p95 end-to-end latency:
the per-job nanosecond diagnostics are unpaired and not a
calibrated network-performance experiment. It does not supply
bandwidth pacing at 1/10/100Mbps, controlled RTT at 0/10/50ms,
interprocess isolation, ACK/retry or authenticated peer/epoch/key
rotation, complete receiver peak RSS accounting, cross-worker
throughput comparability or crash-safe exact log replay.

Next B1-B1 must predeclare a smaller representative subset with
five independent hosted-worker paired p95 measurements, actual
pacing and feedback/ACK, measured RSS/heap, *both* warm and cold
receivers, near-T resolved comparison, plus disconnection/rollback
tests. Preserve every prior B1-B0 scenario and STOP result,
rather than selecting a favorable query cadence after evidence.
Security [#86](https://github.com/definitely-stable/deltameter/issues/86)
and public product boundary
[#69](https://github.com/definitely-stable/deltameter/issues/69)
remain blockers. No new public API, coverage claim or Snapshot v1.
