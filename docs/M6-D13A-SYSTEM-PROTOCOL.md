# M6-D13-A — Comparator/system protocol foundation

Status: **FROZEN FOR FOUNDATION v2; no performance measurements or conclusions.**
Issue #52; parents #19/#14. Baseline: `e63a70ff10374a622a77f0a897441024613b11f6`.
Machine contract: `research/m6d13a/protocol.json`. The protocol-only commit is
recorded in git before implementation. Amendments require a new version before
collecting affected measurements; never select a favorable run retroactively.

## Audit and choice

D2 proves reusable syndrome bytes (17/25/41/73), not complete system cost.
D11 is the accepted decoder; D12 says STOP_ALGEBRAIC_MICRO_OPT. B1/B2 ACCEPT and
B3 NO-GO do not change this comparison. Frozen historical modules stay unchanged.
See D2 evidence/comparator audit and D11/D12 evidence in this directory.

Compare direct exact transfer, maintained D11 guarded PinSketch, and an explicitly
scoped Rateless IBLT **pull/power-of-two external Go comparator**, pinned to
`yangl1996/riblt@4afa6bc06cb2237d9ea273a51d97a7e05b3f573b`.
Its example's u64 SipHash(123,456), dependency go.mod/go.sum and mapping are used
unchanged. This is a trusted deterministic fixture, not an adversarial guarantee.
No Rust port, theoretical 1.35d byte substitution, or library dependency is used.
The Go/Rust language/runtime difference prevents attributing CPU differences to
algorithms alone; it is explicitly part of a future system comparison. The pull
lane is not the natural upstream streaming transport and by itself may not select or
reject the rateless architecture. A faithful streaming comparator is a mandatory
D13-B0 readiness item before any architecture-level verdict.

Alternatives rejected for this slice: a new unvalidated Rust RIBLT port (too much
semantic drift); a formula-only RIBLT mock (cannot establish physical bytes,
failure behavior or retained state). An external pinned executable is the smallest
honest comparator. D13-A is an in-memory transport simulator, not WAN evidence.

## Task, identity, ownership

One-way synchronization: immutable target A at sender, mutable replica B at
receiver; successful terminal state must equal A exactly. Not bidirectional union.
Keys are unique full-width u64 identities, including zero, not hashes of larger IDs.
Direction is obtained from exact B membership for PinSketch and Local/Remote for
RIBLT. Source vectors/indexes remain available and charged in all arms. There is
no Energy estimator and no oracle access for selecting k, batches or fallback.
Dataset/session and both generation IDs are fixed for each session. No updates
inside a session. The harness knows d only for post-session validation.

## Wire and RTT model

Reliable, ordered, already-established transport. Physical **application** bytes
in both directions are counted; TCP/TLS/IP packets, handshake, congestion,
retransmission and scheduling are excluded. No compression or free auth claim.
Header = 48 bytes, little-endian `<4sBBBBQQQIIQ>`:
magic `D13A`, version=1, kind, lane, reserved=0, session u64, A-generation u64,
B-generation u64, global sequence u32, payload length u32, parameter u64.
Kinds: REQUEST=1, DATA=2, VERIFY=3, ACK=4. Lanes: direct=0, D11=1, RIBLT=2.
Responses echo the requested parameter; sequence starts at zero across both
peers. Wrong identity, version, lane, ordering, reserved bytes, kind, parameter,
length or noncanonical payload aborts the session, never retries as success.
A complete bounded frame is validated before decoding/allocating derived state.
Max source keys=1,048,576; max frame payload=8*max_keys+8 bytes.

Each B->A request plus A->B data response costs exactly one model RTT (two
messages). No extra ACK per data response and no overlapped requests. Target-list
parameter=0. List payload=u64 count followed by strictly increasing LE u64 keys.
Direct sends the full target list in one exchange. Receipt of that validated
canonical target list is terminal success at B under this reliable/ordered
transport model; it is not sent back to A as a redundant second O(N) verification
list. No identity compression.

D11 requests k=1,2,4,8. Data consists of the **new** odd syndrome words only;
first response also carries one byte zero presence (0 or 1). Capacities=2,3,5,9;
increment payload=17,8,16,32. Receiver retains prefix and merges with its local
prefix; every attempt uses accepted D11 and a fresh locator. Stop on candidate,
not oracle d. After all rejects request the complete A list: four failed RTTs
and all 73 syndrome bytes remain charged. Thus d>8 is not secretly known to the
session. The complete exact fallback is terminal once validated; it is not
reverse-list verified again. False successful over-capacity sketch candidates are
caught at verification before an exact fallback.

RIBLT pull data cells are exactly 24 bytes: Symbol u64, Hash u64, Count i64, LE.
Request cumulative cells=1,2,4,8,...,1024; only new cells are sent. TryDecode after
each received cell but report success only at batch boundary, paying for the
**whole requested batch**. Prior encoder/decoder state persists across batches.
A zero-cell `Decoded()` is never success. At 1024 cells without candidate,
request full A list. These finite caps are resource policy, not knowledge of d.

This lane intentionally measures `riblt_pull_pow2`, not natural rateless streaming.
The upstream implementation describes Alice streaming coded symbols while Bob
repeatedly decodes and signals stop. Power-of-two pull can add batch overshoot and
avoidable feedback RTTs. Therefore D13-B0 must add a separately frozen streaming
lane before any conclusion about the rateless architecture itself. Pull results may
only be reported as pull-transport results.

## Oracle versus independent final verification

The untimed harness oracle computes A symmetric_difference B and exact equality.
It cannot drive decoder success, stage growth or ordinary fallback.

A **sketch-derived** candidate is provisional. B sends its reconstructed canonical
list to A (VERIFY), A compares against A and returns ACK parameter 1/0. This costs
another RTT, two headers and 8+8|B'| bytes. Guard/all-syndrome cancellation is
**not** this boundary. A failed verification is counted as a false candidate and
the sketch arm then requests exact A once, retaining all earlier cost.

A complete direct/fallback A list is different: it is the exact representation
being synchronized. Under the frozen reliable/ordered transport and canonical-list
decoder, receiving and validating that list is terminal at B. Re-sending the same
O(N) list to A would measure a common audit mode rather than the minimum exact
synchronization protocol and would systematically overcharge direct/fallback.

No successful completion follows malformed messages, stale generations, worker
failure or timeout. Report sketch transfer, sketch verification and exact fallback
separately. A future compact digest-verification tier would need its own contract,
algorithm, collision budget, bytes and CPU before measurement; it is absent here
and cannot inherit an `Exact` claim.

## Maintained state and update cost

Separate cold source/index build, sketch build, logical insert/delete, session
preparation, retries/decode, direction/application and verification. Set updates
are membership-validated: duplicate insertion/missing deletion do not toggle a
sketch. Fixed generation after updates, before session. Future workloads use
0,1,8,64 successful membership changes between sessions, and 1,10,100 sessions
per initial build. Count both endpoints and any source rescan.

D11 maintains nine words + zero flag at each endpoint; all nine odd syndromes are
updated per change. Prefix requests do not rebuild the full sketch. Foundation
checks update replay against a fresh rebuild. Source/index update cost is not free.

Pinned RIBLT `Encoder.AddSymbol`/`Decoder.AddSymbol` forbid mutation after stream
consumption begins. Reset/new generation must reimport the retained source and
rebuild scheduler state. Charge this per session, including hashes, queues and
mapping initialization; do not label it O(1) maintained update. An optimized
checkpoint/clone policy would require a separate protocol amendment.

## Memory accounting

Per endpoint report source/index capacity, maintained primary state, session
encoder/decoder capacity, transport buffers, candidate/application buffers,
verification buffers, and peak simultaneously-live scratch. Report payload,
allocated capacity and process RSS separately. Count overlapping lifetimes, not
sum of unrelated phase maxima. Harness oracle/fixture memory is separate.
D11 primary syndrome payload=72 bytes per peer plus metadata/object layout;
D11 reduction-table payload can reach 16 KiB; this is not total peak memory.
RIBLT retains hashed symbols, per-symbol mapping and heap entries, received
cells, decodable queue and recovered local/remote windows, with Go capacity/GC.
No unsafe allocator instrumentation. D13-A records known logical payload and
marks allocated peak/CPU **unmeasured**, never zero. D13-B timing is blocked until
tracked capacity accounting plus external per-process peak evidence is complete.

## Frozen workloads and future measurement gates

Foundation correctness grid: N=64,8192,65536; five D4/D5/D6/D7a/D7b-derived
seed families; d=0,1,2,3,4,5,8,9,16,64; balanced/add-only/remove-only shapes. N refers to initial
B size; record actual |A|/|B|. Full-width bijective splitmix64 key generation,
with deterministic zero/high-bit/u64::MAX edge fixtures separately. d<=2N;
validate exact cardinality, never silently drop collisions. Boundary 8/9 is
mandatory in every N/seed/shape. Empty, duplicate, malformed, forced exhaustion,
false-candidate, replay and capacity-limit cases are correctness controls.
No probability distribution is inferred from this grid or averaged silently.
This foundation grid only supports conclusions inside N<=65,536 and d<=64.
Before an architecture-level D13-B verdict, a separately frozen scaling extension
must include N=1,048,576 with d=128/512/1024 and a natural RIBLT cap/exhaustion
case; it need not repeat the full Cartesian seed/shape grid.

Frozen network grid: RTT=0,1,10,50,100 ms; bandwidth=1,10,100,1000 Mbps (decimal).
T = total non-overlapping CPU + rounds*RTT + 8*application_bytes/(Mbps*1e6).
Report cold and maintained separately. Maintained amortized total includes
initial_build/session_count + updates_per_session*update_cost + session CPU;
RIBLT session rebuild and all failed work remain inside session CPU. No sleep
or local subprocess elapsed time is used as simulated network evidence.

D13-A ACCEPT only when complete matrix, exact final equality, byte/message/RTT
closure, update equivalence, pin/source provenance and negative tests pass on
GitHub-hosted CI for the PR HEAD. Output `FOUNDATION_PASS`, performance decision
`NOT_MEASURED`; no decoder optimization, public API or production GO follows.
Missing/duplicate rows, wrong pins, changed frozen source, malformed evidence or
oracle-driven decisions are hard validity failures, not favorable exclusions.

D13-B timing may begin only after A review/merge and a separate D13-B0
measurement-readiness gate completes all of the following:

- native/non-overlapping CPU instrumentation for direct exact, D11 and RIBLT
  phases; Python orchestration/subprocess control is never treated as algorithm CPU;
- tracked container/capacity accounting plus per-process peak evidence;
- genuine persistent multi-session D11 state with the frozen 0/1/8/64 successful
  membership changes and 1/10/100 sessions-per-build grid;
- a separately frozen RIBLT streaming comparator faithful to the upstream
  stream-until-stop model, while retaining `riblt_pull_pow2` as a transport control;
- the N=1,048,576 and d=128/512/1024 scaling extension plus natural exhaustion.

Only then freeze measurement-source hashes before timing: five independent hosted
workers, three processes each, balanced arm permutations per process, all raw
observations retained. No automatic CI timing threshold. A named grid cell is a
private system candidate only with >=10% median total-cost reduction versus the
relevant comparators on every worker; report 8/9 separately, all fallbacks, memory
and update amortization. Pull-only RIBLT results cannot reject the rateless
architecture. No aggregate across unrelated network cells. Incomplete
memory/verification or correctness failure => INVALID, not GO. No qualifying cell
=> STOP_SYSTEM_PRODUCT. Mixed cells => CONDITIONAL_PRIVATE_ONLY; a separate product
decision is always required. An additional digest tier cannot inherit these
outcomes. Production/public ExactSmallDelta remains NO-GO.
