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
| M6-D | [#19](https://github.com/definitely-stable/deltameter/issues/19) | Pinned exact-lane audit and measured comparison; conditional adoption only |

M6-B/C require M6-0 measurement and may proceed before M6-A completion if their independent evidence justifies it. After M6-0, M6-A and the read-only/source-audit portion of M6-D should proceed in parallel. M6-D production adoption still depends on M6-A product evidence, so research can inform the experiment without prematurely committing to an exact backend. One coherent PR per slice. No automatic merge/release is implied.

The executable first-slice plan is [M6 foundation](superpowers/plans/2026-10-06-m6-foundation.md).
