# STRICT-COMPACT-OPT-B — maintained odd-count cache experiment

Issue #73; parent #70; hard dependency: merged #76 OPT-A.
Status: **FROZEN PROTOCOL BEFORE B PERFORMANCE MEASUREMENT**.
Research-only; no public surface or snapshot changes.

## Hypothesis / competing implementations

Use m=4096 and the latest certified J=52 / LEVEL_MAJOR / 26 KiB physical
bitmap with the unchanged keyed BLAKE3 oracle and delta/64 Q32 inference
table. Include J=24 bounded-range scaling guard and J=64 control.

Two matched implementations:

- SCAN: update only canonical packed bitmap; estimate by an allocation-free
  `count_ones` scan of all 64 words per level and the unchanged integer Q32
  lookup. **No per-query heap Vec** in the timing control.
- CACHE: identical canonical bitmap updates plus `odd_counts[J]: u16`,
  maintained on each *actual stored bit* toggle. Query uses that cache and
  the exact same Q32 lookup.

For an accepted stored toggle at (level,row):
`old=(word & mask)!=0`, `word ^= mask`, and `count[level]+=1` if old=0
else `count[level]-=1`. Coefficient=0, level>J and level-word=0 are idle
and MUST leave bitmaps and counts unmodified.

Exact derived cache only: no serialization or external exposure.
Logical cache payload = 2*J bytes (J52=104, J24=48, J64=128).
Measure/report object/allocator overhead separately if pertinent.
Counts are bounded [0,4096], so no u16 overflow.

XOR merge: XOR packed words first; SCAN has no cache to rebuild;
CACHE rebuilds all J counts from the resulting canonical bitmap.
Arithmetic addition/XOR of two caches is invalid; explicitly test it is
never used. Reject profile/key mismatch fail-closed.

## Correctness before performance

1. Keyed oracle reference vectors and full packed state must equal the
   independent OPT-A reference after each deterministic seeded stream.
2. Cached counts must equal allocation-free full scan, including repeated
   toggles, empty/dense levels, inactive events and cancellation.
3. Cache-based Q32 upper bound must equal scan path and reference upper bound.
4. XOR merge + rebuild must exactly equal the direct combined token stream;
   repeated XOR with the same state cancels.
5. No allocation or growth in the steady-state update loop.
6. Host CI must run rustfmt, clippy, tests and the lab correctness lane.

Any failure means INVALID, no benchmark decision.

## Measurement protocol (predeclared)

Five independent GitHub-hosted Ubuntu workers.
Each runs two warmups and eight measured rounds.
Alternate SCAN/CACHE order between worker/round; use identical deterministic
token corpora and identical initial parity states. Report raw per-round
measurements, head SHA, Rust version, Q32 SHA, memory bytes.

Primary per-profile lanes (both SCAN and CACHE):
- update-only: ns/token (include one keyed-BLAKE3 computation per token);
- query-only: ns/estimate (no per-query allocation in either variant);
- XOR merge: ns/merge with cache rebuild charged entirely to CACHE;
- mixed operation: ns/operation for query every k updates, k=1,10,100,1000.

Each paired measurement starts from equivalent pre-populated states.
Freeze query density and number of updates; no per-candidate tuning.
Benchmarks use `black_box` to prevent removal and protect from
identical-result constant folding. Rotate workload order.

## Gate (J=52 primary, all five workers)

KEEP_CACHE iff:
1. UPDATE: CACHE/SCAN ns/token <=1.10;
2. QUERY: SCAN/CACHE ns/estimate >=5.0;
3. MIXED: CACHE improves total *real* time >=10% on at least one of
   the predeclared k=1,10,100 mixed lanes on **every** worker.

A gain at k=1000 alone does not satisfy the practical-benefit gate.
No post-hoc threshold edits after observations.
Report all J=24/J=64 guards and true break-even estimate frequency.
Uncertainty from hosted noise must not be promoted to portable API promise.
If any gate fails: DROP_CACHE (scan design stays valid).

## Evidence and transport

Worker logs must include exactly 2 variants * 3 profiles * 7 lanes * 8 rounds = 336 samples per worker;
the single truth is **2 variants * 3 J * 7 lanes * 8 rounds = 336 raw samples
per worker = 1680 overall**. Aggregator must strictly validate each
(worker,J,mode,lane,round), source HEAD equality, finite/positive values,
cache bytes and complete PASS markers. Null/missing/duplicate/extra
observations invalidate the decision.

Artifact digests and five-worker summary are recorded only after the hosted
run. Original OPT-A data remain immutable.

## Boundaries

No public API, new table, profile change, PRF change, snapshot-v1 mutation,
D11/reconciliation or generalized ladder. OPT-C (#74) depends on OPT-A layout,
not on cache result. #69 remains blocked until OPT-B + OPT-C close.
