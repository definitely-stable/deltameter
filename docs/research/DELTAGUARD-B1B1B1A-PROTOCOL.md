# DeltaGuard B1-B1B1-A — physically fail-closed receiver reset under TCP faults

Parent [#101](https://github.com/definitely-stable/deltameter/issues/101)
and #99/#97/#92. Protocol frozen BEFORE first new B1-B1B1-A CI run.
Preserve merged D60 scoped WARM_N256 NO-GO and STOP_RESOLVED_NEAR_T.
No public API, Snapshot v1, new probability or security claims.

## Executable failure contract

A receiver has independent sorted exact A and B at generation1,
physically synchronized from **two distinct owner OS processes**;
that cold bootstrap is fully accounted for. Both owner processes
are then terminated. At generation2 an independent owner-1 process
physically attempts an incremental delta via localhost TCP, under
one injected fault. The receiver NEVER modifies its exact state
on an invalid/truncated/unauthenticated/replayed operation.
Because neither source nor receiver has a crash-durable event
journal in this research prototype, an invalid or disconnected
session **invalidates the incremental cursor**. The ONLY
permitted recovery is to start *two NEW distinct owner processes*
holding authoritative fixture-generated generation2 lists,
perform two **complete exact FULL frames over TCP**, verify
owner/epoch/key/profile/generation/canonical membership, then
atomically replace BOTH receiver inventories. The sender state
is deterministically reconstructed from named public fixtures
on restart, **NOT** persisted across a crash. Hence this gate
tests fail-closed physical *full-reset recovery policy*, NOT
durable crash-consistent incremental replay, actual autonomous
remote source availability, or hostile peer authentication.

Special ACK-loss case: correct owner-1 delta is physically
received and applied exactly once; deliberately drop the
first ACK, sender physically retransmits the identical frame
after read timeout, receiver detects equal epoch/generation
and canonical *identical* payload, ACKs without re-applying.
Then receiver is resynchronized from TWO physical generation2
full owners. Different payload at same committed generation
is a fatal conflict; never ACK forged or mutated replay.

## Frozen physical matrix

Two lanes (256/65536 initial tokens, d48, source epoch B1_EPOCH),
three deterministic source fixtures (rep 0,1,2), five independent
GitHub-hosted workers, and **12 named physical faults**:
1. disconnect before header
2. truncate header
3. truncate body
4. checksum corruption
5. wrong owner identifier
6. wrong epoch
7. wrong generation (ahead of expected)
8. wrong key identifier
9. non-canonical token ordering
10. canonical insert already present (membership invalid)
11. claimed payload size > B1_MAX_PAYLOAD (reject pre-allocation)
12. ACK lost after valid frame (identical same-generation replay,
    exactly-once receiver mutation)
Plus a distinct real child **SIGKILL mid-payload** case,
giving 13 faults * 2 lanes * 3 repeats * 5 workers =
**390 unique source-attested physical probes**. Every probe
uses an independent local TCP listener, one fault child sender,
plus four complete source-child transfers (2 bootstrap + 2 reset);
no in-process magically copied receiver lists. Each full source
frame has a physical 24B receiver request and 16B ACK; after
the ACK each healthy source child transmits 16B of final
post-transfer `VmHWM` bytes and actual Linux
`/proc/self/stat` user+system CPU **ticks**. CPU HZ conversion
is intentionally omitted; ticks are NOT Instant wall time.
An incomplete/physically killed owner may have no telemetry.

The checksum is BLAKE3-128 unkeyed public fixture integrity,
NOT malicious sender MAC. All failures require error or
explicit NACK before any incremental mutation. Payload length
bounded before allocation, socket read/write timeouts and
finite sender joins are required. Physical byte model counts
sender 1B hello, complete 24B request, 64B header, exact
payload, 16B ACK/NACK, 16B final telemetry when sent, and
actual prefix bytes for crash/short failures.
No real WAN, process-persistent event journal, malicious
sender authentication, or fsync transactional snapshot.

## Exit

- 390/390 exact records, each of 5 workers 78 unique probes and
  complete 13-case inventory per named source/rep;
- independent source fixture exact oracle, strict parity
  unchanged and fully canonical physical generation2 reset;
- zero partial mutations for invalid/truncated/dropped and
  exactly-once valid ACK-loss replay with real duplicate packet;
- real SIGKILL of a blocked source process after partial body;
- physically metered two-owner bootstrap + reset, no free source
  state; final healthy child CPU ticks and post-transmission
  VmHWM; independent fail-closed source/byte aggregator;
- **B1B1B1A_FAILCLOSED_FULL_RESET_ACCEPT**, not durable
  B1B1B1 crash-resilient GA or SYSTEM_PRODUCT_GO.

Next B1B1B1-B (separate frozen phase) must implement true
durable owner/receiver cursor or fail-reconnect with provenance,
lost/reordered ACK across process restarts, OS actual CPU/full
peak RSS product comparison and the wider proposed bandwidth/
delay grid under a proper source owner lifecycle. #86 and #69
remain independent blockers.
