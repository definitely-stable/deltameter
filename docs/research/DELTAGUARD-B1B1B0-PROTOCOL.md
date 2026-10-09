# DeltaGuard B1-B1B0 — persistent-sender hot/hot evidence protocol

Parent [#99](https://github.com/definitely-stable/deltameter/issues/99), nested under
#97/#92. This protocol was committed **before any B1-B1B0 measurements**.
D58/D59, original B2A certificate, B1-A wire format and Snapshot v1 stay frozen.
This is **transport/correctness and fair hot/hot latency foundation**, not the
complete recoverable distributed service or a product GO.

## First explicitly bounded phase

Two OS sender processes per scenario persist throughout 20 distinct *consecutive*
source generations 2..21. The receiver is a separate parent OS process. Each
owner builds and retains BOTH its canonical exact u64 list and independently
maintained B2A11 parity bitmap. The receiver physically transfers each owner's
initial exact list at generation1 before starting any hot/hot timer; this
**bootstrap is charged separately** and never represented as a free scalar.
Source generations are advanced through physically transmitted control commands,
charged separately as common control bytes and acknowledged by both owners.
Thus all query modes start with *both owner processes already running, ready,
same generation*, and compare identical source sets from an independent oracle.

Three frozen lanes:
- A: N256, d48, 10Mbps per sender, injected response delay 0ms.
- B: N256, d57, 10Mbps per sender, injected response delay 10ms.
- C: N65536, d48, 100Mbps per sender, injected response delay 10ms.

Exactly 20 distinct generations per lane; five independent GitHub-hosted runners.
Modes at each generation A/C: `guard`, `retained_delta`, `direct_full`;
at B additionally `resolved_guard`. For each generation, rotate mode order
deterministically from worker+lane+generation. **200 query records per worker,
1000 total** (20 * (3+4+3) * 5); one n20 sample distribution per mode/lane/
worker. Do not reselect seeds, filter out expensive modes or pool machines to
hide a bad p95. Nearest-rank p95 at n20 is the 19th sorted sample, still
limited evidence; both socket requests, app pacing, two response reads,
canonical decode/oracle comparison, and 16B physical ACK per owner fall
inside the measured hot/hot query latency window. Each query mode has the
SAME process lifetime and caller timing boundary.

Requester-to-sender control (new research-only, not production protocol):
- 32B fixed request: 8B magic, 1B opcode, 7B canonical zeros,
  8B frozen epoch, 8B expected generation.
- Response: existing B1-A 64B fully validated frame followed by
  physical 16B ACK/NACK containing 8B magic and 8B generation.
- `advance` physically instructs each already connected owner to
  insert precisely the same fixture-shared new token for next generation;
  it carries a physical 16B ACK, is NOT timed as query and is
  charged to the common source update channel. Generations may not
  skip, rewind or repeat.
- `retained_delta` sends physically one canonical 9B insertion event
  per owner since last receiver cursor, maintains the independent exact
  receiver state, and commits the sender's cursor ONLY after ACK.
- `guard` physically sends two 256B bitmap payloads; independent
  symmetric-difference rehash verifies XOR and exact cutoff c48.
- `direct_full` sends both real current u64 sorted lists.
- `resolved_guard` charges initial guard query and a distinct
  physically transmitted 2-owner extra full-list request/response
  if UNKNOWN. SAFE is one-sided, UNKNOWN is not falsely described
  as d>T.
- If receiver NACKs a corrupt checksum at generation22, sender MUST
  retain the uncommitted delta cursor; physically retry exact same
  generation and verify exact receiver state was not partially mutated.
  All 5 hosted workers run this fail-closed test. Missing/duplicate/
  reordered generation, owner/key/profile/epoch, truncated/corrupt
  frames are rejected by B1-A decoder and strict generation control,
  but durable crash/reconnect replay is **NOT** implemented here.
- `quit` sends command and final child VmHWM; process status must
  exit successfully, no orphan senders. Child VmHWM is reported
  **after all query serialization** (correcting D59 pre-response
  sampling gap). Receiver VmHWM recorded after session. Timings
  of source construction/updates are elapsed time, not OS CPU use.

Real app-wire accounting:
initial owner hello 1B each; per query pair:
GUARD 2*(32+64+256+16)=736B;
RETAINED_DELTA 2*(32+64+9+16)=242B;
DIRECT_FULL 2*(32+64+8*owner_list_size+16);
RESOLVED = 736B + DIRECT_FULL if UNKNOWN.
ADVANCE: 2*(32+16)=96B per generation, common to all modes;
initial two FULL transfers and QUIT/ACK/final VmHWM are charged
once per session, not duplicated per hypothetical mode.
No host-derived untransmitted source-side scalar used as baseline.

The pacing is application-side **per sender** at the configured Mbps.
Injected delay per request is a server sleep, not real WAN RTT;
processes share one hosted VM, not independent machines or a
shared bottleneck. Public test key and unkeyed checksum are
NOT malicious peer authentication. No ACK persistency/fdatasync,
process-crash recovery, restart nonce/key rotation, actual CPU clock
or WAN benchmarking is implied.

## Gates

- Exactly 1000 source-SHA-bound query records, complete 5 hosted workers,
  no duplicate/missing lane/gen/mode, 5/5 real corruption→NACK→valid
  replay tests and zero receiver mutation on rejected bad frames.
- Source u64 lists and bitmap maintained across 21 generations.
  Independent per-token keyed rehash matches physical A XOR B,
  exact receiver maintained state equals independently derived
  current two source lists after every generation and replay.
- Every byte including physical request, ACK, bootstrap, advance,
  fallback, quit and VmHWM telemetry counted and independently
  checked by a fail-closed Python aggregator.
- Report per-worker n20 empirical p50 and p95 without inventing
  confidence bounds; measure full lifetime child peak RSS after query
  completion. No PRODUCT_GO solely from these bounded lanes.
- Next B1-B1B1 extends dropped/short TCP, reconnect/crash, duplicate
  ACK, adaptive cases, CPU timing and complete 1/10/100Mbps×0/10/50ms
  matrix before the parent #99/#97 system/product gate can close.
