# Roadmap

The roadmap is intentionally linear. One developer should be able to understand the whole repository without an orchestration layer.

## M0 — Research bootstrap

Status: complete; merged in PR #1.

Deliverables:

- structured research corpus;
- source audits and decision record;
- dependency-free research scripts;
- one GitHub Actions research workflow;
- reproducible Energy profile derivation;
- reproducible finite-(d) published-F-PCSA moment checks;
- explicit Q1 v0 decision.

Acceptance:

- documentation distinguishes proven, asymptotic, empirical and open claims;
- contradictory research inputs are archived and reconciled;
- Q1 does not falsely claim a strict Parity theorem;
- scripts run with stock Python 3;
- workflow produces reproducible artifacts.

## M1 — Energy reference implementation

Status: complete; merged in PR #3.

Build one Rust crate, not a workspace.

Deliverables:

- `EnergyDeltaMeter`;
- explicit pairwise bucket + 4-wise sign randomness;
- theorem-profile construction from 6R caller-supplied independent uniform u64 words, with no RNG dependency;
- incremental energy maintenance;
- compatible source-sketch difference/subtract;
- theorem-derived profiles;
- deterministic and property tests.

Acceptance:

- implementation matches the documented estimator;
- profile generator and Rust configuration agree;
- algebraic merge/difference properties pass;
- no unsafe code required;
- no crypto dependency.

## M2 — Published F-PCSA reproduction

Status: complete; merged in PR #5.

Do not start with the custom g(v)=1 variant and do not substitute a classical-PCSA first-1-position model.

Deliverables:

- faithful FIELDMAP[i,j] finite-field state;
- published h row/level distribution;
- published uniform field coefficient g;
- rightmost-nonzero row statistic;
- explicit finite level policy J;
- packed state;
- XOR composition;
- finite-d moment regression where analytically justified;
- asymptotic/middle-range reproduction.

Acceptance:

- observed behavior is consistent with the published construction;
- the implementation documents the paper's middle-range limitation;
- published and custom set-specialized variants are not conflated;
- universe size N, row count m, Hamming weight d and level bound J are not overloaded;
- no strict capacity claim.

## M3 — Experimental ParityDeltaMeter

Status: complete in PR #7.

Wrap the reproduced algorithm in the small public API.

Deliverables:

- explicit `Coverage::Asymptotic { relative_standard_error }` status;
- `ParityConfig` + `ParityDeltaMeter` wrapper over M2;
- Compact/Standard/Accurate experimental profiles;
- exact configuration compatibility for XOR merge;
- visible finite-J truncation outcomes;
- stable-Rust state/update/merge/query benchmark harness;
- documented memory/guarantee comparison with Energy.

Post-M3 research status:

- exact tiny full-state enumeration/DP: implemented for proof/regression-sized cases;
- W_i non-sufficiency: established by a finite counterexample;
- finite-J truncation budget: implemented, with signal and state-distortion events separated;
- Poissonized cell/row law: derived and implemented as a research artifact;
- fixed-d de-Poissonization/inversion: no practical certified bridge;
- full published-statistic monotonicity: unresolved;
- strict published-W_i path: **STOP / NO-GO**.

A narrow `ParityLevelCounts` research track may continue post-v0, but it is not a release blocker and has no public `Coverage::Proven` contract.

## M4 — Performance and API freeze

Status: complete; merged in PR #11.

Deliverables:

- paired base/head stable-Rust benchmark for update/merge/query on the same GitHub-hosted runner;
- explicit v0 defaults:
  - Energy = 10% relative error / failure probability <= 1e-6;
  - experimental Parity = Standard (m=256, J=64);
- no batch API in v0: current scalar backends expose no measured batch-specific amortization opportunity;
- demonstrated Parity query optimization for J=64:
  - no highest-level Vec allocation in estimate;
  - one u64 highest-set-bit operation per row instead of scanning 64 levels;
- rejected Energy merge and query micro-optimizations when paired measurements did not show a stable benefit;
- root-only supported public API; published F-PCSA reproduction remains internal research machinery;
- raw Parity packed words are not public, so an in-memory layout is not accidentally frozen as a wire format;
- extensible public enums/result records marked non-exhaustive before the freeze;
- integration compile contract for the supported root API;
- rustdoc warnings denied in CI;
- persisted serialization deliberately remains unfrozen.

Acceptance:

- cargo fmt --all -- --check;
- cargo clippy --all-targets -- -D warnings;
- cargo test --all-targets;
- RUSTDOCFLAGS="-D warnings" cargo doc --no-deps;
- release-mode examples compile;
- GitHub-hosted paired performance workflow succeeds and uploads evidence;
- Energy remains the only Coverage::Proven backend;
- Parity remains Coverage::Asymptotic;
- no unsafe, SIMD, nightly, new runtime dependency or self-hosted runner requirement.

See docs/M4-PERFORMANCE-API-FREEZE.md for the freeze record and performance interpretation.

Only after M4 is merged should a separate requirement decide whether persisted serialization is actually needed.

## M5 — Canonical sketch interchange v1

Status: complete; merged in PR #13.

Rationale:

- when both full sets are already co-resident, exact comparison is often available;
- the mergeable-sketch use case becomes materially useful across process, host, storage or time boundaries;
- therefore a minimal self-contained interchange representation is a product requirement after the M4 API freeze.

Deliverables:

- versioned canonical envelope shared by Energy and Parity;
- little-endian fixed-width integers;
- exact backend and length validation;
- CRC32C accidental-corruption detection;
- self-contained Energy hash-row configuration plus primary counters;
- Energy derived energies recomputed on decode rather than persisted;
- self-contained Parity shape/seed plus packed GF(2) state;
- zero-padding canonicality for the final Parity word;
- public encode_snapshot/decode_snapshot methods and SnapshotError;
- fixed byte vectors and fail-closed malformed-input tests;
- no serde, file/network helper, compression, crypto, unsafe or new runtime dependency.

The exact format and compatibility rules are in docs/M5-INTERCHANGE-V1.md.

## M6 — Evidence-gated workflow and compatible optimization

Status: **complete**. Final system verdict: **STOP_SYSTEM_PRODUCT** in M6-D13-B / PR #57. Parent issue #14 is closed.

[Critical audit and design](M6-RECONCILIATION-AND-OPTIMIZATION.md) records the research constraints, corrections to the initial proposal and acceptance gates.

- M6-0 / #15: snapshot compatibility fixtures, serialization evidence, single-buffer candidate.
- M6-A / #16: complete in PR #21; estimator-assisted admission is NO-GO as a generic/public workflow on the measured matrix (+29.838% application bytes vs direct exact).
- M6-B / #17: complete. B1 table-driven scalar CRC32C ACCEPT in PR #40; B2 exact structural preflight ACCEPT in PR #49; B3 direct one-pass Energy decoded-state construction NO-GO in PR #51 because every Energy decode lane regressed by 2.45-3.24% while unchanged guardrails stayed near neutral. B3 code is not retained. The optional residual CRC selector is skipped for this milestone; future codec work requires a new measured trigger.
- M6-C / #18: complete in PR #22; bit-equivalent Energy sign hashing ACCEPT, with about 68–71% lower update latency and negligible amortized setup after the optimized mask builder.
- M6-D / #19: complete. D1-D11 build and optimize the private guarded PinSketch reference; D12 stops algebraic micro-optimization. D13-A/B0 freeze a fair system protocol and measurement substrate. D13-B / #56 then runs five independent hosted workers, 3,900 raw observations and 360 named network/amortization cells. **STOP_SYSTEM_PRODUCT**: D11 qualifies in 0/360 cells and even the optimistic RIBLT stream-lower-bound qualifies in 0/360 cells versus direct exact under the frozen >=10% every-worker gate. The best D11 cell is still ~0.04% worse on its worst worker; the best optimistic stream cell is ~0.44% worse. Production/public ExactSmallDelta remains NO-GO; retain D11 as private research/reference only. No further M6 reconciliation-backend work is authorized without a materially new protocol contract.

Snapshot plus an unchanged full-list exchange is an overhead control, not a demonstrated benefit. M6 may conclude that a proposed workflow is not worthwhile. No public networking API or strict Parity work is implied.

[First-slice implementation plan](superpowers/plans/2026-10-06-m6-foundation.md). M6-B: [research audit](M6-B-CODEC-DECODER-RESEARCH-AUDIT.md), [corrected post-research plan](superpowers/plans/2026-10-07-m6b-post-research.md), [B2 evidence](M6-B2-STRUCTURAL-VALIDATION-EVIDENCE.md), and [B3 NO-GO evidence](M6-B3-DIRECT-DECODE-EVIDENCE.md). M6-0 measured verdict: [evidence](M6-0-EVIDENCE.md). M6-A measured verdict: [NO-GO evidence](M6-A-EVIDENCE.md). M6-C measured verdict: [ACCEPT evidence](M6-C-EVIDENCE.md). M6-D Phase-1 decision: [exact-lane audit](M6-D-EXACT-LANE-AUDIT.md). D1 measured verdict: [PinSketch64 lab evidence](M6-D1-PINSKETCH64-EVIDENCE.md). D2 measured verdict: [incremental prefix evidence](M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md). D3 measured verdict: [incremental BM NO-GO evidence](M6-D3-INCREMENTAL-BM-EVIDENCE.md). D4 measured verdict: [trace-square ACCEPT evidence](M6-D4-TRACE-SQUARE-EVIDENCE.md). D5 profiling verdict: [root-profile evidence](M6-D5-ROOT-PROFILE-EVIDENCE.md). D6 measured verdict: [quadratic ACCEPT evidence](M6-D6-QUADRATIC-EVIDENCE.md). D7 profiling verdict: [post-quadratic residual evidence](M6-D7-RESIDUAL-ROOT-PROFILE-EVIDENCE.md). D8 measured verdict: [GF(2^64) scalar-square ACCEPT evidence](M6-D8-GF64-SQUARE-EVIDENCE.md). D9 profiling verdict: [post-D8 residual evidence](M6-D9-POST-D8-PROFILE-EVIDENCE.md). D10 measurement verdict: [trace-internal reduction selector evidence](M6-D10-TRACE-INTERNAL-EVIDENCE.md). D11 measured verdict: [fixed-constant reduction ACCEPT evidence](M6-D11-FIXED-REDUCTION-EVIDENCE.md). D12 profiling verdict: [post-D11 stop-gate evidence](M6-D12-POST-D11-PROFILE-EVIDENCE.md). D13-A protocol: [system comparison foundation](M6-D13A-SYSTEM-PROTOCOL.md). D13-B0 readiness: [measurement-readiness evidence](M6-D13B0-MEASUREMENT-READINESS-EVIDENCE.md). D13-B final system verdict: [STOP_SYSTEM_PRODUCT evidence](M6-D13B-SYSTEM-EVIDENCE.md).

## STRICT-COMPACT optimization before public freeze

Issue #70 defines the optimization program that follows the accepted 32 KiB
STRICT-COMPACT private candidate.

Public guarantee/key/snapshot semantics may continue under #69, but the first
public memory profile and interchange shape must not be frozen until the cheap
product-shape work closes:

1. range-aware stored-level frontier;
2. maintained odd-level counts;
3. progressive/prefix level transfer.

Only after those slices should the project decide whether to open the generalized
Certified Parity Ladder, DeltaGuard threshold primitive, or a broader oracle
portfolio.

Mathlab-facing theorem candidates — finite independence, adaptive tokens,
heterogeneous ladder optimization, state lower bounds and nested-prefix optimality
— are tracked as research transfers rather than production assumptions.

Canonical program:
[STRICT-COMPACT-OPT-001](research/STRICT-COMPACT-OPT-001-PROGRAM.md).

Canonical execution plan with child issues #72/#73/#74:
[STRICT-COMPACT-OPT-NEXT-WORK](research/STRICT-COMPACT-OPT-NEXT-WORK.md).

OPT-A / #72: **RANGE_FRONTIER_PASS** in PR #76. The exact frontier proves J=56
already preserves the full accepted u64 q95 range while reducing instance state
from 32 KiB to 28 KiB (-12.5%). Level-major is retained for subsequent research:
keyed update cost is effectively unchanged, estimate extraction is substantially
cheaper, and each level is a contiguous 512-byte chunk. J=24/32/40/48 remain valid
bounded-range frontier points. Canonical evidence:
[STRICT-COMPACT-OPT-A](research/STRICT-COMPACT-OPT-A-EVIDENCE.md).

OPT-A follow-up on PR #76: **J52_REFINEMENT_PASS** independently reproduces
full-u64 continuous certification at **J=52 / 26 KiB / LEVEL_MAJOR**. The
five-worker hosted refinement passed the frozen viability thresholds; J=51's
current certificate prefix ends at 11,316,578,792,889,469,415 (not a
mathematical impossibility for other constructions). Thus **J=52 supersedes
J=56 as the preferred full-u64 *research* baseline**, saving 18.75% versus
J=64. Original OPT-A Stage-1/2 evidence remains a valid historical frozen
matrix. Canonical addendum:
[STRICT-COMPACT-OPT-A-REFINEMENT](research/STRICT-COMPACT-OPT-A-REFINEMENT-EVIDENCE.md).

OPT-B / #73: five-worker research evidence gives **KEEP_CACHE** for the J52
LEVEL_MAJOR maintained per-level odd-count cache: +104 B derived non-serialized
payload, worst hosted update regression +2.670%, query speedup >=13.563x
on every worker and >=15.22% total mixed-100 gain on every worker versus
an allocation-free scan comparator. Evidence:
[STRICT-COMPACT-OPT-B](research/STRICT-COMPACT-OPT-B-EVIDENCE.md).
This is a private research/product decision, NOT a public API or snapshot change.

OPT-C / #74: C0/C1 frame and simultaneous-coverage foundation passed in
merged PR #78. C2 physical transport research collected **10,560** five-worker
observations on fixed d/T scenarios: unaided PUSH_ALL increases full transfer
26,750B to 27,838B; the T-informed one-level bounded prefix costs 655B
when the requested upper-bound criterion is satisfied. This is explicitly
**MODELED_C3_CANDIDATE**, not actual-network PROGRESSIVE_PASS and not
equivalent to transmitting a full reusable sketch. A trusted sender can
also return U alone much more cheaply. Canonical C2 evidence:
[STRICT-COMPACT-OPT-C-C2](research/STRICT-COMPACT-OPT-C-C2-EVIDENCE.md).

OPT-C C3-A / #80: five-worker localhost TCP foundation with real
request/response framing and deliberately application-emulated 10Mbps
RTT=0/10/50ms produced **C3_A_TRANSPORT_FOUNDATION_PASS** (1,440
socket trials plus 360 local two-party parity-XOR extension oracles).
A T-bounded one-level prefix gives 655B under the frozen scenarios,
but a trusted sender's scalar result costs just 24B.
C3-B must test two actual independently held sketch senders over TCP,
retained partial-level reuse, full fallback and changing epochs.
[STRICT-COMPACT-C3A](research/STRICT-COMPACT-OPT-C3A-EVIDENCE.md).

OPT-C C3-B / #80: genuine two-sender retained TCP states plus
full-state fallback passed functional/canonical correctness on five
hosted workers, but **C3B_TWO_PARTY_NO_GO** for the strict
no-p95-RTT-regression gate (0/3 named scenarios qualify at
application RTT up to 50ms). Retained first-level transfer is
1,344B versus dual full 53,500B but incurs another RTT.
Full fallback costs 55,778B (4.26% MORE than dual full).
No gate reclassification; a preplanned sender-first prefix is an
independent follow-up C3-C hypothesis.
[STRICT-COMPACT-C3B](research/STRICT-COMPACT-OPT-C3B-EVIDENCE.md).

OPT-C C3-C / #80: the separately preregistered sender-first
one-level probe gives **C3C_CONTROLLED_TWO_PARTY_PASS** on all five
hosted workers, all three named d/T scenarios at app-paced 10Mbps
RTT 0/10/50ms, without altering the C3-B request-first NO-GO.
A first useful XOR bound from TWO independent senders costs 1,310B
vs dual-full 53,500B; full fallback costs 55,744B (4.19% more).
This is a **narrow compatible two-party threshold result**, not
authenticated WAN transport or general interchange compression.
Trusted sender-only scalar-U workflows remain cheaper (24B).
[STRICT-COMPACT-C3C](research/STRICT-COMPACT-OPT-C3C-EVIDENCE.md).

Final product-shape decision: **NO GENERAL PUBLIC PROGRESSIVE API
IN V1**. The separated #69 key/snapshot/profile design may proceed
using J52 LEVEL_MAJOR and the OPT-B derived nonserialized counters.
Do not freeze a public wire/key lifecycle until #69's own acceptance;
keep sender-first as an optional future application-specific lane.

## Post-OPT-C research — DeltaGuard G0 / #84

G0 completed as a research-only independently maintained k-level
parity guard using the exact same m4096 / J52 keyed BLAKE3 mapping
and original alpha=delta/64 Q32 per-level bound. TinyGuard physically
stores k=1/2/4/8 selected levels (512/1024/2048/4096B bitmap per
source, not total allocation), XORs compatible states and returns
SAFE_BELOW_T only when the inherited one-sided upper bound is <=T.
Hosted 5-worker/1200 fixed-input observations prove *implementation
equivalence*, not a new probabilistic or adaptive-input theorem.
Observed T=64 usefulness was 0/15 even at d=0.25T; for T>=65536 and
d=0.75T k=2/4 yielded 14–15/15 deterministic safe decisions.

Verdict **G0_FOUNDATION_PASS_EMPIRICAL_POWER_CANDIDATE**,
**NO PUBLIC GUARD GO**. See
[protocol](research/DELTAGUARD-G0-PROTOCOL.md),
[evidence](research/DELTAGUARD-G0-EVIDENCE.md), D52.
Next #85: new *exact* finite-sample calibration for changed m/J/alpha,
especially tiny thresholds; #86: repeated/adaptive input/key model.
Do not reuse old Q32 for m!=4096 or promote deterministic power curves
to failure probability, and do not change public snapshot v1.

## DeltaGuard G1-A / #85 — exact alpha frontier

G1-A research-only calibration passes independent exact certificates for
`k=1/2/4` and `alpha=10^-6/k` using unchanged m=4096/J52.
Five GitHub-hosted workers and fail-closed 900-row paired old/new
source observations succeeded. At T4096,d=.75T,
k1 SAFE changes 10/15→14/15; k2/4 11/15→14/15.
At T32/64,d=.25T, no new useful replies; exact certified
minimum possible bounds are 59/61/64 for k1/2/4,
making T32 unanswerable by this method (not universally).
No change to production or snapshot; do not treat
fixed public-test-key results as 1e-6 Monte Carlo proof.
[Evidence](research/DELTAGUARD-G1A-EVIDENCE.md).
Next G1-B: exact fixed-d Ehrenfest occupancy CDF and
small-T stochastic-order proof, then independently certified
decision cutoffs, not more binary64 fitting or untargeted
m reduction. Key/adaptive threat issue #86 remains open.

## DeltaGuard B1B1B1-B1B1-B0 — persisted-source hot query contracts and cold accounting

Open #109, research PR #111, decision D66; frozen-before-samples
`docs/research/DELTAGUARD-B1B1B1-B1B1B0-PROTOCOL.md`.
Five GH-hosted workers, 60 independent persisted source
sessions, **1,200/1,200 physically paired**
guard/full/direct exact vs RAM-resident maintained
exact questions, N256/N65536, d48/d57 and
20 hot queries per fixture. Two long-lived source
owner processes reopen independently persisted
canonical snapshots+WAL, build fixed B2A keyed
bitmap once and respond on SAME physical socket
lifecycle. Exact receiver pays physical cold
source transfer, crash-atomic B1B0 checkpoint,
real physically delivered two-owner gen2 update
and actual receiver fsync BEFORE Q20.

N256,d48 **14,720B guard Q20** versus
**4,322B exact cold bootstrap +356B
physical gen2 update +0B exact hot queries**.
N65536,d48 guard **14,720B** vs exact
**1,048,802B cold +356B update +0B hot**,
but guard-only supplies S/nearfull
statistical observation, NOT precise
set reconciliation; UNKNOWN/resolution
must pay FULL, not magic offline
receiver data. All d57 fixtures have
S>48 and require fallback; derived
resolved TCP is always worse than
direct full, reinforcing near-T
RESOLVED STOP. Odd d57 source2
contains N+1, correctly byte accounted.
Warm n20 p95 was checked per worker,
but different user-facing outputs and
receiver-state contracts prohibit any
equal-capability performance GO.

**RESEARCH ACCEPT for strict measured
contracts; NO SYSTEM_PRODUCT_GO**. Source
Fs/FACK/Disk and cold receiver writes
recorded, but real fsync wall,
auth/malicious/adaptive #86, power cut,
remote WAN, Q1/Q10/Q100 and
1/10/100Mbps with delays 0/10/50ms
remain. Next #109 must freeze same-
capability fully RESOLVED fallback,
fresh/hot resets, exact decoder
vs durable maintained exact, CPU/RSS,
source/sink I/O and n20 per-worker p95;
public API #69 still blocked.
Evidence:
[DELTAGUARD-B1B1B1-B1B1B0-EVIDENCE.md](research/DELTAGUARD-B1B1B1-B1B1B0-EVIDENCE.md).

## DeltaGuard B1-B1B1-B1B1-A — crash-cut committed WAL repair and read-path audit

Issue #109, research PR #110, D65; premeasurement
[DELTAGUARD-B1B1B1-B1B1A-PROTOCOL.md](research/DELTAGUARD-B1B1B1-B1B1A-PROTOCOL.md).
The prior B1B1B1-B1B0 receiver could accept a
valid committed checkpoint and ignore a torn
uncommitted WAL tail, but then append a new
generation BEHIND that tail, damaging future
replay. Fixed under exclusive receiver writer
by verifying committed state and atomically
normalizing the WAL to the exact durable
post-checkpoint prefix BEFORE any new append
or ACK. A healthy normal WAL adds no repair
fsync; a corrupted COMMITTED prefix is fatal.

Five GitHub-hosted CI workers: **150/150
real receiver SIGKILL** across partial WAL,
fsynced-uncommitted WAL, committed marker
before ACK, conflicting same-gen event after
commit, checkpoint after fsync+rename before
WAL reset. **90/90 canonical repairs**;
new receiver OS processes physically replay
two separately persisted source records and
verify exact source state. 600/600 paired
nonmutating read audits: BF loader rereads
**4,608B N256** or **1,049,088B N65536**
per access after checkpoint; hosted n20
empirical p95 **0.023–0.035ms** / **0.310–
0.951ms**, respectively, on warmed Linux
page cache. These are NOT device IO metrics,
WAN or matched service competitor comparisons.

**Research ACCEPT only, NO SYSTEM_PRODUCT_GO.**
The fixed O(256B) WAL append/write cost from
D64 remains useful but rebuilding exact
vectors from full checkpoint ON EVERY query
still imposes avoidable O(N) read, allocation
and CPU overhead. Next #109 B1B1B1-B1B1-B:
hot materialized receiver + same fsync atomic
WAL, paired SAME lifecycle against equally
durable maintained exact vs guard-only
SAFE/UNKNOWN and fully resolved fallback;
measure real CPU/RSS, fsync, physical IO,
cold Q1/Q10/Q100 and n20 p95, with
#86 security / #69 public API still blocked.
Evidence:
[DELTAGUARD-B1B1B1-B1B1A-EVIDENCE.md](research/DELTAGUARD-B1B1B1-B1B1A-EVIDENCE.md).

## DeltaGuard B1-B1B1-B1B0 — bounded atomic receiver WAL/checkpoint

Issue #107, research PR #108, D64; frozen-before-run protocol
`docs/research/DELTAGUARD-B1B1B1-B1B0-PROTOCOL.md`.
Five hosted workers, **3,000/3,000** physically
source-bound two-owner ABA generations (100 per
fixture, N256/N65536, d48, 50 INSERT and
50 DELETE per owner). Thirty real receiver
process SIGKILL after ACK1 but before ACK2;
a fresh receiver child physically reloads
the fsync'd two-owner checkpoint+WAL and
acknowledges identical repeated events
WITHOUT reapplication. Independent exact
source oracle and failure/rollback/torn
journal tests are all PASS.

The receiver now commits one fixed **256B
two-owner WAL transaction** and a **48B
atomic commit marker** BEFORE either source
ACK, with full fsync checkpoint every
50 generations, bounded active WAL max
**12,544B**. Replaces D63's O(N) full
two-source rewrite per event. N65536
receiver logical write bytes per 100-generation
run: **104,902,400B old → 2,128,480B**
(~49.3x reduction), but these are *file
payload bytes*, NOT physical disk blocks.
Normal generation invokes 3 sync calls;
checkpoint generations 7. Cold initial
receiver snapshot and source WAL/ACK sync
costs remain separately payable.

**Research correctness ACCEPT, NO SYSTEM_PRODUCT_GO.**
This receiver improvement can be shared by
equally durable exact. Full CPU/RSS, disk
reads, fsync elapsed latency and a same-
lifecycle true exact-vs-guard n20 tail
comparison are STILL MISSING, as are
many-key/checkpoint-boundary crash tests,
real WAN, #86 security and #69 release
boundary. D60 warm N256 guard NO-GO and
nearT resolved STOP unchanged. Continue
issue #107 with separately frozen matched
durable exact/guard physical system cost
and crash boundary protocol.
Evidence:
[DELTAGUARD-B1B1B1-B1B0-EVIDENCE.md](research/DELTAGUARD-B1B1B1-B1B0-EVIDENCE.md).

## DeltaGuard B1-B1B1-B1-A — hash-chained ABA receipts and real partial-ACK receiver crash

Parent #105, PR #106, decision D63: frozen protocol
`docs/research/DELTAGUARD-B1B1B1-B1A-PROTOCOL.md`.
Five GitHub-hosted workers, **3,000/3,000** physical
two-owner TCP event rounds across 100 successive
INSERT/DELETE (50+50, actual ABA) generations per
fixture and 30 receiver subprocess SIGKILL after
the FIRST owner ACK but before the SECOND ACK.
A newly spawned receiver reopens its fsync'd two-owner
source inventories, chain digests and exact 88B
last accepted event receipts; the replay is safe
because it compares exact **accepted event identity**,
not merely token membership.

Normal physical two-owner chain event + ACK is
**356B**; interrupted first attempt **340B** then
retransmit **356B** over actual localhost sockets.
Source fsync WAL, commit and post-ACK markers survive
source process recreation. Integrity tests reject
changed/reordered sequences, invalid deletes, checksum
or source/receiver disk corruption.

**Important negative product result:** atomic
receiver storage is currently implemented by
rewriting both complete exact source lists every
generation, costing **1,573,536,000 logical receiver
write bytes** over 15 N65536 ×100-generation
runs. This is NOT measured disk-device blocks,
but proves **STOP_FULL_RECEIVER_RECEIPT_PER_EVENT
AS PRODUCT DESIGN**. The next bounded B1B1B1-B1B
step must build crash-atomic incremental receiver
WAL/checkpoint, count fsync/disk bytes and compare
with an equally durable exact baseline. No public
Snapshot v1/API changes, no WAN p95 or security GO.
D60 warm N256 general-guard NO-GO, d57 resolved
STOP and #86/#69 blockers remain unchanged.
Evidence:
[DELTAGUARD-B1B1B1-B1A-EVIDENCE.md](research/DELTAGUARD-B1B1B1-B1A-EVIDENCE.md).

## DeltaGuard B1-B1B1-B0 — fsync'd two-owner WAL/ACK and physical process restart

Parent #103, PR #104, D62; frozen premeasurement protocol
at `docs/research/DELTAGUARD-B1B1B1B0-PROTOCOL.md`.
Five hosted CI workers, **210/210 source-bound physical
SIGKILL/restart probes** with two independently
persisted owner snapshots/WAL/commit/ACK cursors
and a separately restarted receiver reading
atomically persisted two-owner exact inventories.
Uncommitted source WAL tails ignored; accepted
generation2 events reopen from disk and replay
exact once, including case where receiver synced
state before the physical ACK and owner still
has stale ACK watermark. Real `File::sync_all`,
atomic rename and directory fsync. All owner
restart paths read disk, not fixture regeneration.

One-event recovered two-owner TCP cost: **308B
committed generation2** independent of N,
versus **4,386B N256** / **1,048,866B N65536**
for uncommitted generation1 full source sync.
These link bytes EXCLUDE the cost of initial
source ingestion, WAL/commit/ACK fsync, metadata,
CPU and durable exact comparator. This is
**DURABLE_PREFIX_REPLAY_RESEARCH_ACCEPT**,
not power-cut endurance or a public product
GO. D60 warm-N256 guard NO-GO and near-T
resolved STOP persist. Next bounded B1B1B1-B1:
multi-generation/removal+rollback, crash
between two owner ACKs, actual source CPU/RSS
and disk write amplification, fault-frequency
cold N65536 niche vs equally durable retained
exact, and #86/#69 security/API gates.
Evidence:
[DELTAGUARD-B1B1B1B0-EVIDENCE.md](research/DELTAGUARD-B1B1B1B0-EVIDENCE.md).

## DeltaGuard B1-B1B1-A — fail-closed real TCP faults and physical full reset

Issue #101, PR #102, D61: **390/390** exact
source-bound failure probes across 5 hosted runners.
13 physical fault classes for both N256 and N65536,
including real SIGKILL midbody, ACK lost with
same-generation identical retransmission ACKed
without double-application, checksum/key/owner/
generation rejection, truncation, oversize,
noncanonical and exact-membership conflicts.
Fresh two-owner OS sender processes physically
transmit generation2 complete source inventories
after each failed session, verified against
an independent exact oracle before both receiver
lists are atomically replaced. No silently free
receiver/source state.

Measured post-fault FULL recovery costs:
**4,354B (N256,d48)** and **1,048,834B
(N65536,d48)** per fault at generation2,
including 2 owner frames, requests, ACK
and full post-transfer child RSS + CPU ticks
telemetry; failure traffic extra. This
demonstrates how a missing durable replay
journal can erase savings of cheap exact
deltas or sparse guard checks.

**RESEARCH ONLY / NO SYSTEM_PRODUCT_GO**.
Resumed source states are regenerated from
public deterministic fixtures, **not**
crash-durable source event files/ACK records.
Next B1-B1B1-B: actual fsync journals,
restart/reconnect/ACK loss across crashes,
source provenance and complete CPU/RSS,
fault-frequency and cold N65536 viable-niche
comparisons. D60 warm N256 exact dominance,
near-T resolved STOP, #86 security and #69
public release gates unchanged.
Evidence:
[DELTAGUARD-B1B1B1A-EVIDENCE.md](research/DELTAGUARD-B1B1B1A-EVIDENCE.md).

## DeltaGuard B1-B1B0 — persistent same-lifecycle hot/hot receiver comparison

Parent #99/#97/#92, PR #100, decision D60:
five GitHub-hosted workers, **1000/1000 source-bound physical**
samples, two distinct long-lived OS sender processes through 20
consecutive generations, source receiver exact sync, per-owner real
request/frame/ACK and strict corruption→NACK→retry (15 probes).
Initial physical exact bootstrap, shared update control, and final
post-transmission VmHWM are separately charged.
N256,d48, 20 queries: B2A guard **14,720B**, warm maintained
exact **4,840B** (+ paid earlier bootstrap 4,322B);
direct full 89,760B. Empirical n20 worker p95:
guard 0.42–0.51ms, exact 0.20–0.28ms.
**SCOPED NO-GO for guard as general warm N256 replacement**.
For N65536 cold exact bootstrap 1,048,802B, so a distinct
large-N sparse cold receiver niche remains possible.
Near-T fully resolved B2A is still STOP (N256,d57,
104,640B vs direct full 89,920B).

Evidence: [B1-B1B0](research/DELTAGUARD-B1B1B0-EVIDENCE.md).
**NO SYSTEM_PRODUCT_GO**. Next B1-B1B1 must physically test
dropped/truncated/reordered replies, durable restart/reconnect
and ACK persistence, actual CPU accounting and expanded
predeclared latency/pacing workloads; all on GitHub-hosted CI.
Threat #86 and public lifecycle #69 are independent blockers.
Do not return to sketch algebraic microoptimization.

## DeltaGuard B1-B1-A — true two OS sender processes and paced localhost cost

Issue #97, PR #98, D59: five GitHub-hosted workers, exactly
300 preregistered source-bound physical two-process TCP records,
and strict evidence aggregation. Owners run in **distinct OS
processes** communicating over separate localhost sockets;
real control bytes, per-owner app pacing, artificial response
delay, physical UNKNOWN→full-list fallback, physical cold
source bootstrap and receiver-warm exact, child process RSS
high-water marks and source-build/update wall timing are recorded.
At N256,d48,S10: guard 738B vs warm exact query 338B
(+ previously paid physical source sync 4322B).
At N256,d48,S100: guard 738B vs warm exact 1958B.
At N256,d57,S10: fully resolved guard costs 5162B vs
direct full 4474B — retain **STOP_RESOLVED_NEAR_T**.

**B1-B1-A is DIAGNOSTIC ONLY, not product p95 GO**.
The initial guard timer includes process startup; warm exact
timer excludes its separately reported completed bootstrap,
so direct p95 comparisons between them are NOT lifecycle-fair.
Each worker has only three repeats (nearest-rank p95 = max),
insufficient for a reliable tail-latency verdict. Next #97
B1-B1B: persistent process sources and hot/hot paired
latency, ACK/retry, crash/restart, sufficient p95 repetitions
and explicit cold/warm bootstrap and RSS accounting.
Security #86 and public product #69 are independent gates.

Evidence:
[DELTAGUARD-B1B1A-EVIDENCE.md](research/DELTAGUARD-B1B1A-EVIDENCE.md).

## DeltaGuard G1-B2-B1-B0 — physically retained exact and real fallback

Parent #92, research PR #96, D58:
**B1B0_RETAINED_PHYSICAL_EXACT_FALLBACK_PASS** on five
GitHub-hosted workers (360/360 source-bound physical TCP records),
independent receiver-retained exact lists with per-generation and
batched update streams, true two-owner B2A physical XOR and full
resolved request/response. The fixed T64/b11/c48 ideal certificate
and frozen Snapshot v1 are unchanged.

Crucial N256,d48,100-generation *cumulative real app bytes*:
guard dense 64,000; streaming retained exact 18,678;
batched exact with sparse queries 6,262; sparse guard 1,920.
N65536,d48: guard dense 64,000 vs streaming exact 1,063,158.
Thus query cadence, N, cold vs warm bootstrap and admissible
batching determine which system is economical; do not infer a
universal product win. Unknown d57 resolved full-list fallback
remains STOP (N256,S100: 6,506B vs direct 5,816B).
Evidence [B1-B0](research/DELTAGUARD-G1B2B1B0-EVIDENCE.md).
Atomic malformed-batch rejection is tested.

**NO SYSTEM_PRODUCT_GO**: next B1-B1 is real paired five-worker
p95/bandwidth/RTT with request/ACK/retry, separate process owners,
full heap/RSS, warm receiver state, crash/epoch recovery.
Security #86 and public API/key lifecycle #69 still block exposure.

## DeltaGuard G1-B2-B1-A — actual two-owner TCP frame correctness

Parent #92, research PR #95. Frozen
[protocol](research/DELTAGUARD-G1B2B1-PROTOCOL.md) makes a receiver-retained
exact inventory with initial sync + sequenced incremental events an explicit
**same-topology comparator**, not a magically trusted 24B single-source scalar.
B1-A validates two actual loopback TCP sender connections, schema/profile/
sender/key ID/epoch/sequence/length/integrity checks, independently reconstructed
B2A XOR against direct per-token oracle, physical byte equality and malformed
frame rejection on five GitHub-hosted workers. Verified source-head run
[37899112626](https://github.com/definitely-stable/deltameter/actions/runs/37899112626):
5/5 worker success, 75 two-sender physical transactions including negative
cases; N256/d48 transfers 640B guard vs 4224B direct full. The research
tag/key is a PUBLIC test fixture, not malicious-sender authentication.

**B1-A ONLY:** no p95/RTT/throughput, full fallback, separate processes,
receiver-cached exact update lifecycle or security proof yet; no public/product
GO. Evidence:
[DELTAGUARD-G1B2B1A-EVIDENCE.md](research/DELTAGUARD-G1B2B1A-EVIDENCE.md).
Next B1-B must implement and physically compare retained exact vs B2A
at cold/warm horizons, full two-owner fallback, p95, RSS and hostile/mismatched
epoch failure; issue #86 security and #69 public lifecycle still open.

## DeltaGuard G1-B2-B0 — retained physical sources and model cost gate

Issue #92, PR #93: **SCOPED PHYSICAL/MODELED FOUNDATION PASS**.
On five GitHub-hosted workers with exactly 3,240 source-attested
physical records, two owners retain actual u64 lists and independently
maintain B2A b11/b12, B1 unit and J52. B2A certificate remains
unchanged; physical XOR verified against the canonical exact
symmetric-difference source. Native build/update/query CPU
measurements are recorded, but application transfer bytes and
network latency remain a fixed **analytical model**, not TCP.

At T64, d48, b11 physical payload 256B/owner gives deterministic
SAFE and 640B modeled two-owner response versus direct exact
4,368B for N256 or 1,048,848B for N65536, session10.
For near-threshold d57 UNKNOWN, fully resolved exact fallback
costs MORE than direct exact: **STOP_RESOLVED_NEAR_T**.
J52 remains physical size/XOR cost-only control in B0;
no uncertified Q32 threshold is assumed.

Evidence: [DELTAGUARD-G1B2B0-EVIDENCE.md](research/DELTAGUARD-G1B2B0-EVIDENCE.md).
Next stage B2-B1: genuine two-owner TCP, p95 latency, framed
identity/fail-closed tests, and explicit exact-fallback accounting.
No public release/API/Snapshot v1; #86 and #69 remain blockers.

## DeltaGuard G1-B2-A near-full single-level strict guard

#89 B2-A mathematical/integration stage **ACCEPTED RESEARCH ONLY**.
Preregistered new m=2^b-1, D=2^b single-dummy-slot
XOR bitmap, b=6..13, T32/64/128. With D=m+1 the
lazy Ehrenfest kernel is stochastically monotone, enabling
an exact all-d>T integer cutoff at d=T+1,
independent of the old G1B1 or J52 probability tables.

24 full exact certificate profiles, 39 independent tiny
whole-bitmap-state oracles and 3,240 physical two-owner
source-attested GitHub-hosted rows passed. T64,d48,b11:
256B/owner bitmap, cutoff S<=48, hence *guaranteed*
SAFE for every input with d<=48, under a fixed, compatible
profile. The false-SAFE tail for all d>T is bounded
by 10^-6 only in the **nonadaptive ideal-oracle** model;
keyed BLAKE3 has an unquantified computational replacement
assumption. Close-to-T utility remains poor.

Canonical evidence:
[DELTAGUARD-G1B2A-EVIDENCE.md](research/DELTAGUARD-G1B2A-EVIDENCE.md).
**NO PUBLIC API/SNAPSHOT GO**. Next G1-B2-B costed retained
state/system comparison vs direct exact, G1B1, J52, trusted
24B scalar where admissible, before considering joint-level
multivariate DP or productization. #86 key/adaptive and
#69 public contract remain open.

## DeltaGuard G1-B1 exact fixed-d GF(2) thresholds

#89 G1-B1 research-only mathematical foundation **ACCEPT**:
24 exact integer-cutoff profiles (m4096, T32/64/256, j1..4,
original g random vs new domain-separated g(v)=1), 162
independent full bitmask-state oracle comparisons, and 1,800
five-hosted-worker two-owner physical parity observations.
Single-level exact fixed-d lazy Ehrenfest monotonic law avoids the
conservative de-Poissonization/KL inversion for *precommitted T,j*.
Unit g=1 j1 at T32,d8 has exact ideal-oracle SAFE power
0.364215 and at T64,d16 0.997956; near d=.75T
unit power just 0.000145/0.001189 respectively.
Experimental named 15-seed cells gave 7/15 and 15/15.
Original and unit keys/profiles are incompatible by construction;
no old Q32/canonical wire transfers apply to unit.

Evidence:
[DELTAGUARD-G1B](research/DELTAGUARD-G1B-EVIDENCE.md).
NO PRODUCT API GO. Next #89 G1-B2 honest joint-level/delta
budget proof OR STOP_NEAR_T; #86 repeated/adaptive key threat;
then fully-accounted two-owner maintained-state performance.
Do not call this a novel theorem, a PRF security proof, or
a near-T 95%-power strict guarantee.

## Post-v0 candidates

Only after evidence:

- `ParityLevelCounts` strict finite-sample research, only if certified fixed-d tails become practical;
- `ParityPcsaSetV1` with `g(v)=1`;
- ExactSmallDelta: retain the private pure-Rust PinSketch64 reference for research/regression use only. M6-D13-B is STOP_SYSTEM_PRODUCT for the current exact-verification system contract; any production revisit requires a materially new protocol and fresh evidence;
- SIMD/unsafe specialization;
- formal verification of a narrow proof/code boundary.

## Working rules

- Prefer one PR per coherent milestone slice.
- Avoid framework-first changes.
- Preserve historical research, but keep one canonical decision layer.
- Public GitHub-hosted runners may be used normally.
- CI simplicity matters more than artificial runner-minute minimization.
- No benchmark number becomes a promise until reproduced on named hardware.


## M6-D13-A comparator/system foundation

Issue #52 freezes the next system comparison before measurements: direct exact,
maintained guarded D11 with exact fallback, and a pinned external Rateless IBLT
`pull_pow2` comparator. See [protocol](M6-D13A-SYSTEM-PROTOCOL.md) and
[plan](superpowers/plans/2026-10-07-m6d13a-system-foundation.md).
Protocol v2 makes a validated complete target-list transfer terminal for direct
and fallback; only provisional sketch candidates pay independent reverse-list
verification. The pull comparator alone cannot select/reject the rateless
architecture. D13-B timing is blocked on native CPU/peak-memory instrumentation,
persistent multi-session D11 state, a faithful RIBLT streaming lane and a larger
scaling extension. D13-A correctness/accounting acceptance is distinct from
performance or product acceptance. D12 STOP_ALGEBRAIC_MICRO_OPT and public
ExactSmallDelta NO-GO remain in force.
