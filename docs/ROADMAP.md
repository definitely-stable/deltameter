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
