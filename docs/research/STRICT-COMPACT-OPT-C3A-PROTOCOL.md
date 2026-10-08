# STRICT-COMPACT OPT-C3-A — controlled transport and functional controls

Issue #80 / #74, continuation of C2, branch `research/strict-compact-opt-c3-transport`.
**Freeze before measurements.** No public API, Snapshot-v1 or oracle change.

## Three functionality classes

A. **Trusted threshold-only**: sender can compute full Q32 U. The minimal
control sends `T:u64` 8B request and `U:u128` 16B response, total
**24 application bytes**. Compare FULL (26,750B) and sender
bounded-prefix transfer (122B + 533B per level). The scalar control is
only allowed for a trusted sender; neither CRC nor public BLAKE3 tag
establishes adversarial proof.

B. **Complete state**: full 52-level packed bitmap must reconstruct byte
for byte. A received incomplete prefix **fails** this functionality; if
the receiver later upgrades to full state, count every missing level and
request RTT. No savings claim from evaluating a partial U.

C. **Two independent parties, shared secret oracle/profile, receiver XOR**:
a sender of each separately held sketch cannot compute `U(A XOR B)`
alone. The receiver obtains the *same level IDs* from each party,
XORs 512B blocks, and evaluates combined Q32 U; retained partial
levels can be extended at a later threshold request. Compare repeated
full-state exchange, level-prefix transmission plus subsequent extension,
and a scalar one-party U control labeled **NOT equivalent** for this
class. Count BOTH senders' bytes and the later extension. A changing
generation must invalidate previously received levels.

## C3-A paired hosted laboratory

Source: identical J52 26,624B LEVEL_MAJOR state, frozen Q32 table and
unmodified keyed BLAKE3. Three preregistered d/T scenarios:
`4096/8192`, `65536/131072`, `1048576/2097152`.
Each of 5 independent GitHub-hosted Ubuntu workers constructs 8
predeclared nonadaptive distinct token-range seeds per scenario;
warm up TCP loopback before first observation. Modes: `scalar`,
`complete`, `bounded`, `interactive4` (T-center order).
Controlled application-level settings: 10Mbit/s and RTT 0/10/50ms.
For 1/100Mbit/s and 150ms only *explicitly identified additional
sensitivity lanes* are permitted; don't silently substitute model grids
for measured wall-clock.

One loopback TCP connection per measured attempt, localhost only.
Record `Instant` end-to-end duration, actual successful application
bytes passed to socket writes and received by reads (NOT Ethernet/TCP
wire overhead), number of request messages/round trips, frozen
snapshot copy/selection time and result U as decimal. Include explicit
finite read/write timeouts, bounded framing and fail-closed short/corrupt
frames. Use a reproducible *application-layer* pacing emulator
(`sleep` between chunks or request/response) for link-rate and RTT;
actual scheduler/network timing can differ from targets, therefore label
results **MEASURED_LOOPBACK_WITH_APP_PACING**, NOT measured WAN or kernel
network shaping. Include no-pacing local control and compare measured
wall times against modeled RTT predictions. Never claim a genuine
public-WAN result.

Expected ideal application counts:
- scalar trusted control: 24B;
- complete: 26,750B;
- bounded one-level: 655B if one level meets T;
- interactive batch of four: 122 + 17 + 4*533 = 2,271B
  on first sufficient four-level batch.

## Acceptance separate from C2

C3-A requires 5/5 hosted workers, complete sample identity,
correct on-wire framing, immutable epoch, same-key per-level XOR
oracle vectors, cached partial extension, full-state reconstruction,
sender bytes == receiver bytes and source HEAD attestations.
Failures mean `INVALID`; then DO NOT apply product gate.

Evaluate original #74 p95 >=25% byte reduction vs functional
complete comparator and no p95 wall-time regression at 10Mbps,
RTT<=50ms on >=2 of 3 d/T workloads, on all five workers.
**BUT** even if this passes, disallow a generic public progressive
API if trusted-scalar is functional equivalent in threshold-only
case; do not claim complete interchange savings from a prefix.
An eventual C3 product decision must distinguish functionality and
trust model and count at least a second update/extension round.

Until the two-party retained-prefix/XOR lifecycle measurement is
complete, output `C3_A_TRANSPORT_FOUNDATION_PASS` or `INVALID`,
never `PROGRESSIVE_PASS`.

## Statistical & engineering caveats

Only five small-sample CI hosts, not representative WAN.
No new finite-sample theorem. Adaptive receiver level choice is
permitted only on a single immutable snapshot under the predeclared
per-level union-bound event; adaptive tokens / hostile oracle probes
still unsupported. No secret oracle key transmitted.
