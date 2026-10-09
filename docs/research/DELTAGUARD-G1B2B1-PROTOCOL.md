# DeltaGuard G1-B2-B1 — preregistered two-owner TCP and retained exact controls

Parent [#92](https://github.com/definitely-stable/deltameter/issues/92).
Prerequisite: merged B2-B0 PR #93, D56. Research-only.
**This document freezes B1-A correctness and B1-B timing BEFORE any B1 measured evidence.**
All later changes to gate, scenario selection or byte model require a new separately
versioned protocol, not reinterpretation of observed outcomes.

## Product question and topology

Independent owners A and B hold canonical u64 inventories. A third-party receiver
can neither assume possession of their present full inventories nor trust a scalar
from one isolated owner to describe A triangle B. The same session/profile/secret
is provisioned to both guard owners out of band. The fixed T=64,b=11,m=2047
near-full ideal-oracle certificate is reused unchanged: c=48, delta<=1e-6 for
one **precommitted, nonadaptive** query/input model. BLAKE3 PRF is a separate
unquantified computational assumption. The publicly committed fixture secret
is NOT a production security control.

**Compare identical topology and exact same source histories:**

1. B2A11 sender-first 256B/owner GF(2) parity: two physical senders,
   receiver XOR, cutoff 48, SAFE_BELOW_T or UNKNOWN (terminal for guard contract).
2. DIRECT_FULL: each physical sender transfers its entire sorted canonical
   source u64 array on every query; receiver performs exact merge comparison.
3. RETAINED_EXACT: receiver accepts authenticated-in-production (fixture-bound
   integrity only in this lab) initial full canonical A and B, then two individually
   sequenced append/delete event streams; receiver independently maintains BOTH
   exact lists and compares them. Initial sync, receiver RSS, event/ACK traffic,
   replay, rollback/recovery, and all amortized bytes are charged. A persistent
   receiver is a LEGAL same-topology competitor, unlike trusted one-sender scalar.
4. RESOLVED_B2A: after UNKNOWN, a second request triggers both complete lists
   and a real full exact comparison; count both rounds and complete retransmission.

A trusted 24B one-sender answer is listed as a distinct-topology lower bound,
never treated as a valid result from two independent owners. The earlier J52
cost-only reference remains a historical B0 control, NOT a bogus bound comparator
without its certified Q32 table. No change to public Energy/Parity/Snapshot v1.

## Frozen scenarios (B1-A first, B1-B only after correctness passes)

Initial sets follow B0's exact canonical source construction:
A has N distinct tokens, B has N-floor(d/2) from A and ceil(d/2) fresh
tokens, therefore symmetric difference is exactly d.

- N = {256, 65536}; d = {16,48,57,65}; T=64; s={1,10,100}.
- Workers = 5 separate GitHub-hosted ubuntu-latest workers.
- Fixed public deterministic seed indices {0,1,2} on each worker;
  never choose a favorable seed after seeing outputs.
- Modes = B2A11 GUARD, DIRECT_FULL, RETAINED_EXACT, RESOLVED_B2A;
  the resolved mode uses the SAME B2A bitmap and fallback when UNKNOWN.
- Actual localhost TCP with two independent sender connections/actors;
  10 Mbps application-paced transfer; target application RTT = {0,10,50} ms.
  This is a controlled emulator, **NOT WAN, tc/qdisc shaping, or measured Internet RTT**.
- At s=1 transfer initial complete source lists for retained-exact.
  At s=10 and 100 send every numbered intervening event from BOTH owners
  exactly once, verifying an exact canonical receiver state at every checkpoint.
  For the primary frozen shared-insertion lane one token is appended to
  EACH owner every generation; true d remains constant. A secondary
  asymmetric insert/delete/overlap/cancellation correctness lane is mandatory
  before any product decision, without cherry-picked performance gates.
- Every mode receives identical logical source snapshot and comparison
  epoch. Source ingestions, sketch maintenance, exact list retention and
  receiver memory are charged separately.

B1-A correctness may start with N=256, d={48,57}, session={1,10}, RTT=0,
one seed on a single hosted worker; this is a smoke gate, not product evidence.
B1-B final evidence MUST cover the complete frozen matrix, or say INVALID.
Any timer overhead, sequential sender scheduling and host variability must
be reported. Rotate mode order per seed/worker; do not compare dissimilar
sessions or precomputed-only sketch costs.

## Wire and failure contract

Fixed research frame envelope: unique magic/version, mode, sender ID,
profile/schema ID, secret/config ID, immutable source epoch, monotone
generation, payload length, canonical bytes, and domain-separated
full-message integrity tag. Explicitly validate *actual* bytes read and written
on BOTH physical sockets, including framing, request and feedback/ACK.
Secret material never goes over the wire. The public test fixture is a
reproducibility marker, not authentication of malicious owners.

Reject wrong sender, profile, key identity, epoch, generation skip/replay,
duplicate, payload mismatch, invalid canonical ordering, corruption,
truncated/oversized frame, mixed owners and missing side. Reject before
returning SAFE; receiver must not classify unknown/missing state as zeros.
The receiver must compute B2A XOR from independently held parity bytes,
not from a precombined central sketch. Receiver-exact retains A and B,
not merely their precomputed d.

Guard-only: SAFE when measured S<=48, otherwise UNKNOWN. UNKNOWN is not
evidence that d>64. Resolved: SAFE may end the protocol; UNKNOWN must
trigger a second physical request and obtain/validate both full sources.
Treat an observed false SAFE at d>64 as a possible statistically allowed
event to REPORT, not automatically a malformed harness; exact B2A proof
has already been verified separately. Still fail on corrupt frames or
incorrectly evaluated cutoff. No repeated/adaptive key security proof.

## Actual measurement and honest cost accounting

- Application bytes physically observed in both directions per owner,
  not inferred from bitmap payload length.
- Wall-clock end-to-end p50/p95 for each actual mode and RTT, including
  sender startup, pacing, request/fallback, source serialization, receiver
  deserialization, XOR/exact compare and ACK.
- Separate source-initialization/maintenance CPU, receiver build/update CPU,
  heap/allocations and process peak RSS; do NOT confuse payload bytes with RSS.
- Two comparison horizons:
  (A) **Cold query** with no receiver-side source cache; DIRECT_FULL vs B2A
  guard-only and resolved, at identical source epoch.
  (B) **Warm repeated query** with a legal receiver cache; accumulate first
  full EXACT bootstrap and all subsequent exact events, plus the B2A guard
  full source construction and each subsequent guard round. Compare total
  traffic and amortized per-query bytes, NOT a one-shot cache-delta frame
  against an omitted initial sync.
- For N256 and S100, calculate/report the receiver-cached exact frontier
  explicitly; if it dominates B2A, retain STOP_PRODUCT in that cell.
- Byte-saving percentage, p95 ratio, unknown rate, fallback rate, extra
  round trips and actual goodput must all be supplied per owner/mode.
  Report all workers separately and pooled only as a secondary summary.

## Locked gates

B1_A_FRAME_CORRECTNESS_PASS requires physical 2-sender framing, all
identity/corruption/order/replay/short-read negatives, independent source
truth, wrong key/profile, malformed payload rejection, and match to
the unchanged B2A exact cutoff. A smoke worker does not mean performance GO.

B1_B_MEASURED_PASS for a named d<=48, T64, N and session:
- all five workers pass source-bound records, exact truth and frame tests;
- guard-only physically transmitted application bytes at least 10% below
  DIRECT_FULL *and* p95 latency no worse on every worker in the named lane;
- repeat the same gate against RETAINED_EXACT with the full cache-init
  amortization for named S=10 or S=100. If exact cached wins, record it;
  do NOT rename the primary gate or omit the losing comparator.
- RESOLVED_B2A measured against DIRECT_FULL separately with exact fallback
  charged. If d near threshold requires fallback and loses, mark
  STOP_RESOLVED_NEAR_T.

If a lane cannot meet every-worker gates, conclusion is NO_GO for that lane
even when the analytical B0 byte estimate was attractive. No product claim
from simulation-only numbers. Never reclassify B1-A or B1-B as public
security approval. #86 threat model and #69 API/key/snapshot lifecycle
are independent mandatory blockers. No self-hosted runners.
