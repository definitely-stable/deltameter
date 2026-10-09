# DeltaGuard B1-B1-A — frozen independent-process TCP cost foundation

Parent [#97](https://github.com/definitely-stable/deltameter/issues/97),
[#92](https://github.com/definitely-stable/deltameter/issues/92).
Predecessor merged B1-B0 PR #96, D58. **Frozen BEFORE any B1-B1-A
timing or five-worker observations.** This is a restricted, explicitly
labelled sub-slice. The complete #97 p95 product gate is NOT met
merely by completing this stage. No public API/Snapshot v1 changes.

## Scope and exact topology

A receiver is one OS process. Owner A and owner B are each a distinct
spawned OS sender process communicating over independent localhost TCP
connections. Source sets, guarded B2A11 parity and exact receiver
inventories are separately computed and held. Sender executable is the
same reproducible Rust research example run with a different role;
it is not a real independent trust domain or WAN. Senders use public
fixed-fixture secrets, protocol key IDs and BLAKE3-128 corruption
checksums, **NOT malicious-peer authentication**.

Every physical sender opens one TCP stream per trial, sends its
separately identified *25-byte hello* (owner + peak RSS bytes +
source initial-build time ns + incremental-update time ns),
waits for a real *24-byte receiver request*, then transmits one
complete framed payload. Frame and request lengths, hello and exact
number of actual bytes read/written are counted for BOTH owners.
For two-way resolved UNKNOWN, receiver transmits ANOTHER real
24-byte request to BOTH connected owners and they respond with
complete canonical sorted full lists, including 64-byte frames.

Actual per-sender application pacing is implemented by splitting
frames into <=1024B chunks and sleeping until the
`(bytes_written * 8)/bandwidth` elapsed-time deadline. This
provides controlled **per-sender application pacing**, not a
shared link, kernel shaping, physical network throughput, or WAN.
Before first response the child sleeps user-specified `rtt_ms`:
application-injected *request-response delay*; it is **not**
a 2-way network RTT measurement. UNKNOWN second-response delay
is applied again. All modes are subject to exactly the same control
and pacing parameters; process startup+initial source/guard build
are included in cold measured wall time. Compute/maintenance and
child VmHWM are separately recorded, not silently omitted.

Mode definitions:
- `guard`: two physical 256B B2A parity frames; receiver XORs,
  verifies against independently rehashed exact source delta,
  reports SAFE iff S<=48, otherwise UNKNOWN.
- `full`: two physical complete canonical sorted inventories
  in B1-A 64B frames; receiver calculates exact A triangle B.
- `cold_exact`: two separate complete initial-owner inventory
  transfers (generation1) followed by physically batched, canonical
  per-owner ordered events through requested generation S;
  receiver constructs/updates two genuinely separate exact vectors.
  Both transfer rounds' traffic and process costs count.
- `warm_exact`: receiver first physically obtains the SAME two
  full generation1 source inventories as above, checks both against
  fixture oracle, then excludes this real completed bootstrap from
  QUERY traffic/timing (but reports its bytes and RSS separately).
  Next receives one physical batched event pair through S. It does
  **not** acquire a free scalar answer. Receiver state must exactly
  match the independent final sources; compare SOURCE/receiver exact.
- `resolved`: same guard, then UNKNOWN triggers two true second
  physical receiver requests and complete two-owner exact responses.
  For SAFE it terminates after guard (one-sided contract), otherwise
  the resolution is exact. No UNKNOWN rebranded as above threshold.

Source lifecycle in every sender process: initial A/B of N tokens,
then real shared insert updates through S (not rehashing an
artificial full source sketch at each generation). The CPU
initialization/update costs of B2A states are physical but timing
still includes process startup, unlike durable production workers.

## Frozen B1-B1-A CI-sized matrix

Only these named diagnostic lanes are executed; B1-B1 complete stage
must later extend to full #97 grid before PRODUCT_GO.

1. N256, d48, S10, bandwidth 10Mbps, delay 0ms.
2. N256, d48, S100, bandwidth 10Mbps, delay 10ms.
3. N256, d57, S10, bandwidth 10Mbps, delay 50ms
   (guard UNKNOWN, resolved physical fallback).
4. N65536, d48, S10, bandwidth 100Mbps, delay 10ms.

Five separately hosted GitHub ubuntu workers; three fixed fixture
replicates per lane and `guard/full/cold_exact/warm_exact/resolved`
five modes = **5*3*4*5 = 300 paired source-attested records**.
Deterministic mode-order rotation keyed by worker+scenario+rep to
reduce one-sided process/warmup bias. Do not select alternate
seeds/lanes after inspecting physical timing. Include both owners'
25B hello, per-owner 24B requests, 64B framed payloads and
second-round fallback bytes. No physically transmitted update
ACK/retransmission in this sub-slice; explicitly record its absence.

The sample count for per-worker per-lane/mode p95 is just THREE;
report it as empirical maximum (nearest-rank p95), not a statistically
well-powered performance guarantee. Store raw wall-clock ns, process
init/update ns, child RSS high water, controller VmHWM and actual
bytes. No all-workers p95 product acceptance from three samples.
P95 of pooled workers is descriptive only; never hide worst worker.

## B1-B1-A gate and stop

- FAIL any missing/duplicate malformed source SHA, unexpected
  fixture epoch/profile/owner/generation, exact truth mismatch,
  wrong cutoff, wrong fallback decision, unaccounted wire bytes,
  or failed per-process result; all workers must pass.
- Source independent XOR reference and exact receiver agreement
  must hold every named checkpoint. Valid nonadaptive false SAFE
  at d>T is counted, not misdiagnosed as algorithm corruption.
- Pass requires the COMPLETE fixed 300-record matrix, exact
  certificate SHA `c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`,
  and fail-closed independent Python aggregation. No fake
  significant p95 inference.
- Resolved near-threshold d57 overhead relative to direct full
  must be reported, not optimized away. Guard-only and exact
  resolved remain distinct products.
- True warm exact *bootstrap* transfer and cost must be recorded,
  but not unfairly charged to a receiver whose state is already
  synchronized and live. All consumer decisions must separately
  consider cold vs warm.
- **B1-B1-A can never approve SYSTEM_PRODUCT_GO**: the complete
  #97 B1-B1B needs robust request/ACK/retry, source crash/restart/
  deletion/rollback, true sustained multi-session daemon/source
  state, stronger independent-process ownership, more repeats
  and hostile key/adaptive threat gate #86; API #69 separately.
