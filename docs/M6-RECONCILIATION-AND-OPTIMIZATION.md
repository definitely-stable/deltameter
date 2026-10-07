# M6 — Reconciliation value and compatible optimization

Status: M6-0 complete in PR #20; M6-A evidence complete in PR #21 with a generic-workflow NO-GO. Parent: [#14](https://github.com/definitely-stable/deltameter/issues/14).
Audit baseline: `1ae3be55df218421437c56271a7d9a9f660b0ff7` (2026-10-06).

## Decision and purpose

Choose a bounded distributed **estimator-assisted value experiment** before a production ExactSmallDelta/hybrid backend. This is a sequencing decision, not evidence that a distributed product wins. The deliverable must determine when a difference-size estimate changes a useful decision enough to pay for maintaining and exchanging it. A NO-GO result is acceptable.

Energy remains the only theorem-backed backend, conditional on its documented assumptions. Parity remains experimental/asymptotic. Snapshot v1 is the current compatibility baseline. The user authorized format changes on 2026-10-06 when justified; M6-0 still preserves v1 because its optimization needs no format change. The first slice is M6-0: measurement and compatibility prerequisites, not a completed network workflow.

## Critical review of the preceding proposal

| Earlier point | Audit verdict and required correction |
| --- | --- |
| Workflow is the best next step | Conditional recommendation. It reuses M5 but has no demonstrated economic benefit yet. Restrict it to an experiment with a decision gate. |
| Snapshot followed by sorted full identifiers | Useful overhead/control arm only. It necessarily adds snapshot bytes if the identical full transfer still occurs. Do not market it as bandwidth-saving reconciliation. |
| Energy/Parity support reconciliation | They estimate cardinality; neither recovers missing elements or their direction. A separate exact transfer/recovery path is required. |
| ExactSmallDelta is expensive new scope | Supported by the existing D5 decision. Also audit wrong successful decodes above capacity, verification, identifiers, zero handling and license/FFI costs. |
| Default Energy snapshot is 377,980 bytes | Exact format calculation: 28 envelope + 16 metadata + 23*48 coefficients + 2048*23*8 counters. Primary counters alone are 376,832 bytes. |
| Parity Standard snapshot is 2,092 bytes | Exact calculation: 28+16+256*8. It does not buy the same guarantee as Energy. |
| About 47,000 identifiers is a break-even | Withdraw as an economic threshold. It is only a byte-equivalence comparison to uncompressed u64s; both directions, framing, compression, reuse and RTT matter. |
| Energy 92 to 25 GF64 multiplies | Correct operation-count derivation for R=23 and B>1; throughput remains unmeasured. Setup, cache and decode costs must also be counted. |
| Single-buffer serialization | Directly removes the separate payload allocation/copy; retain only with compatibility evidence and explicit resource/latency results. Do not claim fewer total allocations across the whole application. |
| Table CRC32C | Candidate only; small payloads and table cache costs matter. Evaluate separately from the buffer change to preserve attribution. |
| Earlier decode validation | Desirable hardening. Existing code already bounds row allocation by available coefficient bytes and validates primary-state size before allocation; no arbitrary short-header allocation exploit was established. |
| Reuse decoded energy allocation | Plausible small improvement; measure separately. Maintain checked arithmetic and rejection of invalid counters. |
| More benchmarks | Required: snapshot encode/decode and construction, multiple shapes/contents, raw rounds, AB/BA ordering, pinned SHAs and environment. No timing threshold CI gate. |
| Revisit merge/query | Rejected absent new workload evidence: M4 already tested and reverted those candidates. |
| Batch/SIMD/unsafe | Deferred. Scalar algebraic optimization is a separate candidate; no reason yet to relax the unsafe prohibition or freeze batching. |
| Documentation drift | Fix current authority documents; leave dated M4 freeze/history intact and link forward to M5/M6. |

## Research experience carried forward

The authority is [DECISIONS](research/DECISIONS.md), [STRICT-PARITY-POST-M3](research/STRICT-PARITY-POST-M3.md), [FOUNDATION](research/FOUNDATION.md), and the executable research scripts. Archived model reports are input, not interchangeable proofs.

- **THEOREM, conditional:** Energy uses pairwise bucket collision control, 4-wise signs and independent rows/families. The profile generator plus exact binomial amplification supplies the accepted profile dimensions. Do not replace them using attractive but incompatible sizes from a report.
- **EXACT research results:** tiny full-state DP, a W_i non-sufficiency witness, and finite-J truncation accounting remain regression evidence. Tiny-grid success is not a full-domain finite-sample theorem.
- **EXACT under another model:** unconditional Poissonized cell independence does not establish the required fixed-d tail. A monotone mean does not justify monotone-tail inversion or binary search.
- **Stop gate:** strict published-W_i Parity stays NO-GO. The ideal-model all-zero coefficient event has probability 2^-d; the theorem-implied zero-observation floors 9/19/29 at target deltas are not a usable certified Parity upper bound. Cancellation adds further zero states.
- **Engineering evidence:** M4 retained J=64 leading-zeros query optimization, rejected Energy merge/query changes, and preserved scalar stable Rust. Reproduce benefits instead of accumulating speculative rewrites.
- **Deferred candidates:** Minisketch/PinSketch, Simple Set Sketching and IBLT require independent failure-semantics/source audits. In-capacity recovery, probabilistic recovery and unconditional exactness are different contracts.
- **Rejected transfer:** HLL/Jaccard subtraction is poorly conditioned for nearly equal large sets; Gaussian/chi-square and fitted rare-event Monte Carlo remain oracles/diagnostics rather than substitute coverage proofs.
- **Source hygiene:** the unverified 2026 IBLT estimator claim recorded in REFERENCES is not an accepted dependency or theorem. New candidates must pin primary source/version, model, theorem and matching implementation before adoption.

Upstream navigation for M6-D: [Minisketch contract](https://github.com/bitcoin-core/minisketch/blob/master/include/minisketch.h), [PinSketch mathematics](https://github.com/bitcoin-core/minisketch/blob/master/doc/math.md), [Simple Set Sketching](https://arxiv.org/abs/2211.03683). These moving links are discovery pointers, not a completed pinned audit.

## Format evolution policy

The freeze is not an absolute product constraint. A later slice may propose changing the format when measured requirements justify it (for example configuration references, new backend semantics or a materially better representation). Record the requirement, alternatives, byte/CPU effect and migration cost first. Prefer a new version/backend tag when existing bytes would acquire a different meaning. A coordinated pre-release v1 correction is possible only after explicitly checking consumers/fixtures and recording the compatibility decision; the crate being unpublished is not proof that no snapshots exist. Keep reader compatibility or provide a converter, retain old golden vectors, and test unknown-version rejection and migration. Do not revise v1 merely to accommodate an internal cache.

## Missing boundaries now made explicit

1. **Identity:** v1's u64 domain tag is not dataset/schema identity. If original IDs are hashed into u64, collisions need a separate contract/budget. Do not silently collapse content identity into the estimator's key domain.
2. **Set semantics:** duplicate Energy ingestion changes F2; repeating a Parity toggle cancels. Neither implements idempotent replay. The source must define uniqueness and membership changes. Energy currently has no public delete/merge-add API; initial workflow uses immutable snapshots.
3. **Epoch consistency:** define dataset, A/B generations, config and session identity outside snapshot v1. Different generation IDs are legitimate when explicitly requested; an unexpected/stale generation is rejected.
4. **Provenance:** CRC32C authenticates nothing. Authentication alone does not prove uniform sampling or honest set construction. Accept uniform-row provenance only from a trusted configured source, independently of the received marker.
5. **Oblivious inputs:** fix datasets independently of the random configuration. Reusing a public config for attacker-chosen future data or outcome-dependent adaptive inputs has no inherited guarantee.
6. **Session probability:** bound a predeclared finite collection of valid per-query events by their sum; independence between events is not required by the union bound. Adaptive selection may invalidate the component bounds. Repeating the same sketch is not a fresh independent trial.
7. **Capacity units:** upper_capacity bounds difference cardinality under the profile assumptions, not decoder cells or bytes. Translate using a separately justified decoder load rule, checked u128-to-usize conversion and memory admission limits. Fixed profiles do not represent every arbitrary session delta.
8. **Final verification:** zero energy, zero XOR state or successful decode is not unconditional set equality. The reference workflow uses exact canonical-set comparison; a later digest adds its own assumptions/error budget.
9. **Transport resources:** validate a bounded outer length before buffering. Handle EOF, partial reads, replay, cancellation/timeouts and child cleanup. A structurally valid huge snapshot still consumes CPU and memory; CRC is a full scan.
10. **Cost accounting:** cold construction, maintained-state amortization, cloning, both peers, full coefficients, derived caches/scratch, framing, verification, retries and rejected useful work all count. Compression/config negotiation belong to a future outer protocol; v1 remains self-contained.
11. **Failure atomicity:** preserve checked counters/energies and transactional update behavior. No optimization may mutate the visible sketch on failure.
12. **Performance claims:** scalar popcount lowering depends on the CPU/compiler. A reduced operation count or allocation site is not an observed speedup. Record named CPU/rustc, inputs, raw rounds and limitations.

## M6-A estimator-assisted experiment design and acceptance

Use two subprocess peers first, with a harness that owns exact oracle data outside the measured decision path. No public networking API, service, TLS stack or new runtime dependency. Define bounded framing and request identity in the example support layer.

Three arms use the same datasets and exact completion oracle:

1. Direct exact sorted-set transfer/reconciliation.
2. Snapshot v1 followed by the identical transfer (overhead control).
3. A **predeclared** admission or strategy decision that can avoid expensive work; include false rejection/opportunity cost. If no concrete useful decision can be specified, stop after the control experiment with NO-GO rather than invent a decoder.

Cover |A|/|B| asymmetry, d=0,1,2,8,32,128, large d, disjoint sets, full-width keys, repeated comparisons, and cold/maintained snapshots. Count physical application bytes in both directions and model RTT separately; local subprocess timing is not WAN evidence. Production identity hashing/compression is not simulated as free.

Acceptance: both successful paths reach the exact target set; malformed/replayed/stale messages never report success; timeouts kill/reap children; benchmark artifacts identify workload/SHAs; interpretation separates functional correctness, empirical cost and theorem assumptions. GO requires a named workload and useful cost reduction, otherwise retain estimator-only scope or prioritize the independent exact-lane audit.

## M6-A measured outcome

M6-A completed the bounded two-process experiment and recorded a **NO-GO** for a generic/public snapshot-admission workflow.

Final hosted evidence run: `m6a-value #12 / 37514920381`.

For the fixed 8192-element workload matrix:

- direct exact: 722,356 application bytes;
- snapshot control: 1,134,900 bytes;
- snapshot admission: 937,896 bytes;
- snapshot admission therefore costs +29.838% versus direct exact;
- equal-size byte-only break-even requires >57.111% rejects, while the matrix rejected 3/11 comparisons.

Cold on-demand sketch construction is also not competitive with the in-memory exact control. The only retained conditional case is an already-maintained sketch under a named reject-heavy or expensive-downstream workload.

No public network/admission API is justified. See [M6-A evidence](M6-A-EVIDENCE.md).

## M6-C accepted algebraic optimization

Let L(y) extract bit zero in the existing polynomial basis. Multiplication by fixed c is GF(2)-linear, so define bit i of M(c) as L(c * 2^i). Then L(c*y)=parity(M(c)&y).

Distributivity gives L(c0+c1*x+c2*x²+c3*x³) as XOR of L(c0) and three masked parities. The coefficient order, field reduction 0x1B and sign convention do not change. Compute x² and x³ once per key; bucket hashing remains unchanged. For B>1 this replaces 4R field multiplications by R+2 plus masked parity work. B=1 needs a separate cost count.

Three u64 masks require 24R raw bytes (552 at R=23), excluding layout overhead. They are derived private state, rebuilt on construction/decode and never serialized. The initially measured basis-by-basis builder was rejected because of setup overhead. The accepted builder derives all mask bits by walking c*x^i with a cheap multiply-by-x recurrence, verified against full gf64_mul for every basis bit.

Acceptance requires basis/linearity reasoning, Horner differential vectors including zero/high bits/all-ones, all profiles, signed differences, post-decode continuation, overflow atomicity and byte-identical v1 snapshots. A future adjoint-square transformation might avoid per-key powers but is not part of this candidate.

M6-C measured result: **ACCEPT**. Final paired hosted evidence reports about 68–71% lower update latency, while optimized construction/decode overhead amortizes after roughly 1–1.4 updates. Difference/query remain near baseline and snapshot-v1 bytes are unchanged. See [M6-C evidence](M6-C-EVIDENCE.md).

## Implementation sequence

| Slice | Issue | Deliverable / gate |
| --- | --- | --- |
| M6-0 | [#15](https://github.com/definitely-stable/deltameter/issues/15) | Current docs, nonempty v1 fixtures, hosted serialization evidence, isolated single-buffer candidate |
| M6-A | [#16](https://github.com/definitely-stable/deltameter/issues/16) | Two-process estimator-assisted decision experiment; GO/NO-GO before public workflow expansion |
| M6-B | [#17](https://github.com/definitely-stable/deltameter/issues/17) | CRC/decoder improvements evaluated individually |
| M6-C | [#18](https://github.com/definitely-stable/deltameter/issues/18) | Bit-equivalent sign optimization with setup and steady-state evidence |
| M6-D | [#19](https://github.com/definitely-stable/deltameter/issues/19) | D1/D2 LAB-GO; D3 BM NO-GO; D4 trace-square ACCEPT; D5 profile; D6 quadratic ACCEPT; D7 rejects another degree-specific solver and selects the general trace path; production/public ExactSmallDelta remains NO-GO |

M6-B/C require M6-0 measurement and may proceed before M6-A completion if their independent evidence justifies it. After M6-0, M6-A and the read-only/source-audit portion of M6-D should proceed in parallel. M6-D production adoption still depends on M6-A product evidence, so research can inform the experiment without prematurely committing to an exact backend. One coherent PR per slice. No automatic merge/release is implied.

The executable first-slice plan is [M6 foundation](superpowers/plans/2026-10-06-m6-foundation.md).


## M6-D Phase-1 exact-lane decision

The primary-source and failure-semantics audit is now recorded in [M6-D exact-lane audit](M6-D-EXACT-LANE-AUDIT.md).

Verdict:

- direct C/C++ Minisketch FFI: NO-GO under the current safe-Rust/no-FFI crate boundary;
- public ExactSmallDelta API: NO-GO in Phase 1;
- pure-Rust PinSketch64 reference: LAB-GO only;
- Simple Set Sketching / classical IBLT: retained comparators, not selected;
- Rateless IBLT: strongest unknown-d reconciliation comparator, but a different product direction;
- Self-Sizing IBLT, CertainSync and XYZ-Sketch: current frontier inputs requiring independent reproduction before adoption.

The lab mapping is exact over the complete u64 domain: nonzero keys map identically into GF(2^64), while key zero is carried by one separate XOR-composable presence bit.

Successful decode is not final verification. The lab uses an exact oracle; any production verifier must be independent from the same syndrome equations.


## M6-D2 guarded incremental prefix result

D2 replaces Energy-first sizing in the audited small-d lab with a nested guarded PinSketch64 prefix.

Frozen stages:

~~~text
k=1 -> k=2 -> k=4 -> k=8
stored syndromes = k + 1 guard
~~~

The incremental arm sends each odd syndrome at most once. Cumulative bytes therefore equal the ideal fixed-known-k payload: 17, 25, 41 or 73 bytes for the audited stages. Naive full-prefix resend costs 42, 83 or 156 bytes once retries occur.

Hosted evidence reports exact-oracle completion for all d<=8 frozen workloads, guarded rejection for d=9/10/16 and zero false-success in the matrix. This remains empirical guard evidence, not a theorem or independent final verification.

The retry tax is CPU/RTT rather than communication. The reference decoder reaches about 89.6 ms cumulative decode CPU at d=8 because failed earlier stages are decoded again.

Verdict: private LAB-GO only. The next justified optimization is decoder-state reuse while freezing D2 bytes/RTTs. See [M6-D2 evidence](M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md) and [Rateless IBLT comparison boundary](M6-D2-RIBLET-COMPARATOR.md).


## M6-D3 incremental BM result

D3 keeps the D2 1/2/4/8 guarded-prefix protocol byte-for-byte unchanged and reuses Berlekamp–Massey state across retries.

Correctness is exact against the frozen reference: the incremental syndrome sequence and locator polynomial equal fresh recomputation at every audited stage, and D3 candidates equal D1 candidates and the exact symmetric-difference oracle.

The optimization verdict is **NO-GO**. Hosted decomposition shows approximately -0.3% to +0.6% end-to-end change for d=2..8. At d=8, incremental BM is only about 74 us while root factorization plus candidate verification is about 106.84 ms.

The previously proposed even-syndrome cache is also skipped: measured reconstruction is about 1.3–1.4 us, far below the dominant root-factor cost.

Next permitted candidate: preserve all D2/D3 semantics and optimize only the deterministic trace/root-factor path, beginning with characteristic-2 polynomial squaring and monic reduction. See [M6-D3 evidence](M6-D3-INCREMENTAL-BM-EVIDENCE.md).


## M6-D4 trace-square result

**ACCEPT D4 Candidate A for private research; production/public ExactSmallDelta remains NO-GO.**

Canonical hosted implementation `cf6434d4007825b8fc05ebc9b064ce4860d4442b`, run
37578505762, preserves frozen D2 bytes/RTTs, mapping and guard verification.
At d=8 the complete decoder median changes from 48.328 to 6.575 ms; median paired
reduction is 86.384% across 12 pairs in three processes. d=2..5 improve 88.2–93.6%;
d=0/zero-only d=1 are noise-scale controls. All required CI and correctness gates
pass. This is hosted empirical evidence, not an SLA or an over-capacity theorem.

Root+verify remains ~98.7% of the d=8 phase sum. Next, after D4 merge, profile factor
work by degree and verification separately, then evaluate only deterministic
safe-Rust quadratic specialization if justified. No field-multiply, trace-reuse,
CLMUL/SIMD or protocol changes are bundled. Stop if the profile does not justify it.
See [D4 evidence](M6-D4-TRACE-SQUARE-EVIDENCE.md) for provenance and raw observations.


## M6-D5 root-factor profile result

D5 is a profiling-only slice over the accepted D4 decoder. It makes no root
algorithm change.

The instrumented path records non-overlapping local factor work by polynomial
degree and times candidate rebuild/all-syndrome verification separately. Frozen D1,
D4 generic and D4 specialized decoders are used as untimed correctness controls;
the accepted D4 specialized path remains the uninstrumented performance control.

Canonical hosted result at d=8:

~~~text
D4 control             ~7.137 ms
factor wall            ~7.125 ms
degree-two self        ~1.964 ms  (~27.5% of control)
degree >=3 self        ~5.130 ms
verification           ~0.0039 ms (~0.055% of control)
~~~

Degree-two self work is also ~62.8% of d=2, ~51.7% of d=3 and ~40.9% of d=4
control latency on the frozen corpus. d=5 is dominated by degree-four work.

Verdict: **GO-to-experiment for exactly one deterministic safe-Rust quadratic
specialization**. Verification is not selected. Degree >=3 splitting, D4 trace
squaring, field representation, protocol bytes/RTTs and guard semantics remain
frozen.

This is not pre-acceptance of the solver. The next slice must demonstrate actual
paired end-to-end benefit and may still conclude NO-GO.

See [M6-D5 evidence](M6-D5-ROOT-PROFILE-EVIDENCE.md).


## M6-D6 deterministic quadratic result

D6 changes only degree-two root resolution. A monic quadratic is reduced to the
Artin-Schreier equation y^2+y=c and solved deterministically by a safe-Rust GF(2)
linear solve in the existing polynomial basis.

All higher-degree factorization remains the accepted D4 path.

Correctness remains exact against the frozen controls and exact oracle; d=9/10/16
remain reject and the D2 protocol/guard semantics do not change.

The performance result is real but factor-tree dependent. Across three deterministic
8192-key corpora:

~~~text
d=2 aggregate paired reduction  ~39.6%
d=4 aggregate paired reduction  ~37.5%
d=8 aggregate paired reduction  ~15.3%

d=8 per-corpus medians:
  ~3.7%
  ~27.1%
  ~15.3%
~~~

The matching D5 corpus realizes almost the complete ~27.5% opportunity previously
profiled, while the other corpora contain less degree-two work. Therefore the solver
is accepted for private research, but no universal percentage is claimed.

The standalone Artin-Schreier solve is ~10.2 us and is not selected for immediate
precomputation/cache optimization.

Next: profile the post-D6 residual factor path across multiple corpora before any
degree-3/4/high-degree specialization. See [M6-D6 evidence](M6-D6-QUADRATIC-EVIDENCE.md).


## M6-D7 residual root profile result

D7 is a profile-only slice over the accepted D6 decoder.

Five deterministic 8192-key corpora show that the dominant residual polynomial
degree is not stable enough to justify another degree-specific solver.

For d=8:

~~~text
degree 3 share: ~5.7-71.5%
degree 4 share: ~13.6-82.3%
~~~

Some corpora are degree-3 dominated, others degree-4 dominated, and one has a
near split.

The common phase is much more stable: degree>=3 trace self work consumes roughly
63-83% of accepted D6 latency across all five d=8 corpora. GCD is secondary and
corpus-dependent; division, accepted quadratic work and candidate verification are
not the next bottlenecks.

Verdict:

- immediate cubic solver: NO-GO;
- immediate quartic solver: NO-GO;
- next candidate: one bit-equivalent safe-Rust GF(2^64) scalar-squaring
  specialization inside the shared trace square/mod path;
- keep generic multiplication/GCD/field representation/protocol unchanged in that
  experiment.

See [M6-D7 evidence](M6-D7-RESIDUAL-ROOT-PROFILE-EVIDENCE.md).


## M6-D8 scalar GF(2^64) square result

D8 changes only coefficient squaring inside the degree>=3 trace square/mod path.
The accepted D6 quadratic solver, generic multiplication, GCD/division, field
representation, D2 bytes/RTTs and verification semantics remain frozen.

The performance threshold was frozen before the first D8 run. On the same five
deterministic corpora selected by D7, d=8 paired end-to-end reductions are:

~~~text
D4    13.043%
D5    14.213%
D6    16.984%
D7a   13.144%
D7b   14.940%

aggregate median 14.263%
~~~

All five corpora clear the predeclared 5% primary threshold and every per-process
median is positive. d=3/4/5 guardrails also pass; false-success total is zero in
the deterministic matrix.

Verdict: **ACCEPT D8 scalar square for private research.** Production/public
ExactSmallDelta remains NO-GO.

Next: re-profile the post-D8 residual decoder before selecting any further algebraic
micro-optimization. If no common residual phase remains materially large, stop and
move to maintained-state system comparison.

See [M6-D8 evidence](M6-D8-GF64-SQUARE-EVIDENCE.md).


## M6-D9 post-D8 residual profile result

D9 is profiling-only over the accepted D8 decoder.

The profiler preserves D8 semantics and adds only D7-style non-overlapping local
phase timing plus count-only diagnostics inside trace square/mod. Median d=8
profile-wall overhead is 0.5-1.0% across the five corpora, below the predeclared
5% validity ceiling.

For d=8, share of accepted D8 control latency:

~~~text
trace         median 62.2%, range 57.5-80.6%
GCD           median 35.5%, range  8.1-39.3%
division      median  0.8%, range  0.6- 2.8%
quadratic     median  0.8%, range  0.6- 3.0%
verification  median  0.09%, range 0.08-0.36%
~~~

The predeclared common-material gate requires >=25% in every corpus. Only trace
passes.

Count-only diagnostics show roughly 2.5-3.1 generic polynomial-reduction
multiplications per coefficient-square operation in the d=8 trace paths. Counts do
not establish timing dominance and do not authorize a multiplication optimization.

Verdict: **GO only to a narrower trace-internal measurement slice.** Do not select
another algebraic candidate yet.

See [M6-D9 evidence](M6-D9-POST-D8-PROFILE-EVIDENCE.md).


## M6-D10 trace-internal replay result

D10 measures the trace internals selected by D9 without changing accepted D8.

Offline replay over exact collected operands closes to 97.8-99.4% of accepted
square/mod timing across the five d=8 corpora.

Square/mod itself accounts for 99.75-99.93% of full trace replay. Within square/mod:

~~~text
modulus preparation   1.9-2.8%
square-build          5.0-6.7%
net polynomial reduction 88.3-92.4%
trace accumulation    0.5-0.7% of full trace
~~~

The predeclared selector required net reduction >=50% of square/mod in every
corpus; the measured minimum is 88.299%.

Verdict: **GO to one isolated polynomial-reduction experiment only.** No general
GF(2^64) rewrite, GCD/division change, SIMD/CLMUL/unsafe or protocol change is
authorized.

See [M6-D10 evidence](M6-D10-TRACE-INTERNAL-EVIDENCE.md).


## M6-B codec/decoder research correction

The 2026-10-07 external-research review is recorded in
[M6-B codec/decoder research audit](M6-B-CODEC-DECODER-RESEARCH-AUDIT.md).

The audit keeps the original one-candidate-at-a-time rule and corrects several
external claims against the current implementation:

- B1's table-driven CRC32C has strong hosted evidence, but its residual CRC share is
  not known; an Amdahl result based on an assumed standalone CRC speedup is not
  repository evidence.
- Current Energy decode already checks the complete row block before row allocation
  and exact counter bytes before primary meter allocation. B2 therefore must not be
  framed as closing an existing arbitrary short-header primary-allocation
  vulnerability.
- B2 remains useful as exact backend-shape validation before row allocation and as
  valid-CRC malformed-input hardening. It stays separate from B3.
- B3 is widened from narrow cache-allocation reuse to a direct Energy decoded-state
  candidate: capacity-only counters plus one-pass counter parsing and checked row
  energy accumulation. This removes redundant zero initialization and the later
  full counter rescan at the logical source-code level, but no physical DRAM/cache
  percentage is claimed before measurement.
- further slicing-by-N CRC, encode-side CRC fusion and row/sign-mask fusion are
  deferred until residual attribution shows a material opportunity;
- decode-side CRC fusion remains NO-GO for the current buffered v1 path because the
  checksum-before-backend-decode boundary is intentionally preserved;
- transport byte admission remains outside snapshot v1.

The corrected execution sequence is:

~~~text
finish/synchronize B1
-> B2 exact structural validation
-> B3 direct Energy decoded-state experiment
-> residual codec selector only if further optimization is justified
~~~

B2 and B3 must not be bundled into one performance verdict. See the
[post-research plan](superpowers/plans/2026-10-07-m6b-post-research.md).


## M6-D11 fixed-constant reduction result

D11 changes only multiplication inside degree>=3 trace polynomial reduction.

For every monic factor frame, an ephemeral fixed-constant `ReductionPlan` turns
each modulus coefficient into 16 positional nibble tables. The real timed candidate
includes plan construction; the plan is dropped before child recursion.

Across the five deterministic d=8 corpora, paired end-to-end reductions versus
accepted D8 are:

~~~text
D4    51.614%
D5    45.916%
D6    66.087%
D7a   51.228%
D7b   47.342%

aggregate median 51.166%
~~~

The predeclared gate required >=10% in every corpus and >=15% aggregate. d=3/4/5
guardrails also pass and false-success total is zero.

Verdict: **ACCEPT D11 for private research.** The ~2 KiB table payload per fixed
coefficient (up to ~16 KiB for an audited degree-8 frame) is a private research
trade-off, not a production layout decision.

Next: re-profile accepted D11 before selecting any further algebraic work.

See [M6-D11 evidence](M6-D11-FIXED-REDUCTION-EVIDENCE.md).


## M6-B1/B2 measured result

B1 is complete in PR #40. The synchronized scalar table-driven CRC32C candidate
preserves snapshot-v1 bytes and delivered a large repeatable hosted improvement
across all audited encode/decode fixtures. No slicing-by-N, fusion, hardware CRC or
dependency was required.

B2 then tightens the decoder boundary selected by the research audit:

- Energy custom/proven metadata is validated before row allocation;
- the complete row block must fit before counter-size arithmetic;
- the complete checked Energy payload shape must match before rows are allocated;
- public Proven decode keeps `ProvenanceRequired` precedence;
- malformed fixtures recompute a valid CRC32C before asserting backend failures;
- Parity production code is unchanged because its state length was already checked
  before packed-state allocation.

Hosted B2 evidence shows valid Energy decode remains noise-scale neutral
(-0.097% to +0.190% across the audited Energy lanes). The dedicated 229,364-byte
valid-CRC structural rejection lane improves by 3.544%.

Verdict: **ACCEPT B2 for resource hardening and invariant simplification, not as a
security-vulnerability fix or a valid-input speedup.**

Next M6-B candidate is one isolated B3 direct Energy decoded-state construction
experiment. See [B2 evidence](M6-B2-STRUCTURAL-VALIDATION-EVIDENCE.md).


## M6-D12 post-D11 whole-decode residual result

D12 re-profiles the complete accepted D11 staged decode after the fixed-constant
trace-reduction optimization.

Single-run exact-head summaries exposed a boundary-sensitive D6-derived GCD share
around the frozen 25% common-material threshold. D12 therefore froze a replication
closure before collecting more data: five independent GitHub-hosted workers, three
profiler processes per worker and four paired samples per process.

The original 25% threshold is unchanged. A phase must reach it for every corpus on
every worker to authorize more algebraic work.

All five workers pass correctness and the +5% profiler-overhead ceiling. Across the
25 worker/corpus medians:

~~~text
prefix        0.017- 0.092%
locator       3.422-17.951%
validation    0.003- 0.017%
plan build    0.363- 2.716%
trace        18.034-36.786%
GCD          23.574-74.713%
division      1.120- 8.434%
quadratic     1.367- 9.453%
verification  0.161- 1.113%
~~~

The D6-derived GCD worker medians are 23.657%, 24.750%, 24.575%, 23.574% and
23.583%. None clears the original threshold.

Verdict: **STOP algebraic decoder micro-optimization.** The next M6-D action is a
maintained-state system comparison of accepted private D11 PinSketch, direct exact
reconciliation and a rateless-style comparator. Production/public ExactSmallDelta
remains NO-GO.

See [M6-D12 evidence](M6-D12-POST-D11-PROFILE-EVIDENCE.md).
