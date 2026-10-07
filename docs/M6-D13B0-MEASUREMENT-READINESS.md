# M6-D13-B0 — measurement readiness contract

Status: **FROZEN BEFORE TIMING IMPLEMENTATION**. Issue #54; parents #19/#14.
Baseline: `7b5bbdbc7b278c58b85d1e3732d68ac70ee371df` (merged D13-A protocol v2).

B0 does not select a system winner. It exists to prove that a later D13-B comparison
can measure direct exact, maintained D11 and Rateless IBLT without controller,
lifecycle or transport artifacts deciding the result.

## Authority inherited from D13-A

D13-A protocol v2 remains authoritative for exact-list framing, D11 k=1→2→4→8,
sketch-candidate reverse-list verification, exact fallback, oracle separation and
`riblt_pull_pow2`.

A validated complete A-list transfer is terminal exact synchronization. It is never
sent back merely to imitate sketch verification. Production/public ExactSmallDelta
remains NO-GO.

## Measurement boundary

Python is orchestration and evidence validation only. Python subprocess startup,
stdin/stdout control, JSON parsing, fixture construction and artifact writing are
not algorithm compute.

Primary phase metric is `native_elapsed_ns`, measured inside Rust or Go around a
single non-overlapping phase. It is an elapsed compute metric, not a claim of
hardware CPU cycles.

Each native worker additionally records a coarse process-CPU cross-check from Linux
`/proc/self/stat` as user+system ticks plus the runner's `CLK_TCK`. The cross-check
may detect gross contamination but is not substituted for short phase timings.

No phase may overlap another phase in accounting. Warm-up and correctness checks
are outside measured samples.

## Rust direct/D11 worker state

The Rust worker owns two endpoint states, A and B.

Exact source/index representation is one sorted unique `Vec<u64>` per endpoint.
It reports len, capacity and logical payload bytes. Successful membership updates
use binary-search insert/delete and are timed as source-index work.

D11 mode additionally owns one capacity-9 `PinSketch64Lab` per endpoint. Initial
sketch build is charged once per build. Every successful source update toggles all
nine maintained odd syndromes exactly once. Duplicate insert and missing delete
must reject without mutating either exact or sketch state.

No historical D1/D4/D6/D8/D11 support module is edited.

## Persistent session chain

A build starts with A == B == S0. Then repeat for a configured session count:

1. deterministically apply exactly U successful membership changes to A;
2. reconcile B to A using the selected arm;
3. apply the recovered/exact delta to B;
4. require B == A;
5. for D11 require A-sketch == B-sketch == fresh rebuild oracle;
6. continue with the same state into the next session.

Frozen U = 0,1,8,64. Frozen sessions/build = 1,10,100.

For U>1, use a balanced remove/add schedule when possible so source cardinality stays
near N. For U=1 alternate add/remove by session. Keys come from disjoint deterministic
SplitMix64 domains so every requested operation is successful without retries.

D11 candidate success uses the accepted k=1→2→4→8 sequence and independent
verification inherited from D13-A. A full exact fallback is terminal. When fallback
or direct exact transfers A, B updates its maintained D11 sketch from the exact
merge-difference rather than rebuilding the sketch.

B0 discovered a deterministic **early-stage over-capacity false candidate** while
the eventual session difference is still within D11's maximum k=8 envelope. For the
frozen N=1024, d=8 witness, keys 1..4 are replaced by
`0x80000000007a3910..0x80000000007a3913`. At k=1, both the maintained sketches
and fresh rebuilds produce the same provisional but incorrect candidate. This is
not persistence corruption and not an in-capacity k=8 decoder failure; it is direct
evidence that the one-extra-syndrome guard is empirical rather than a theorem.

The readiness gate therefore distinguishes causes instead of merely checking
`d <= 8`: an exact fallback before the eventual sufficient stage is allowed only
when independent verification rejects a candidate, fresh rebuild reproduces the
same early-stage result, and all failed work/fallback cost is retained. Any other
fallback for d<=8 remains INVALID. The witness is a permanent hosted regression.

## Native phase inventory

Direct exact:
- initial_source_build
- source_update
- canonical_serialize
- exact_parse_merge_apply

D11:
- initial_source_build
- initial_sketch_build
- source_update_exact
- source_update_sketch
- prefix_materialize
- locator_decode
- candidate_apply_exact
- candidate_apply_sketch
- verification_prepare
- fallback_serialize
- fallback_parse_merge_apply_exact
- fallback_apply_sketch

RIBLT:
- source_update
- session_encoder_import
- session_decoder_import
- coded_symbol_produce
- coded_symbol_consume_decode
- candidate_apply
- verification_prepare
- reset/rebuild

Every reported session must close exactly to its arm's total native elapsed value.
Missing/duplicated phases are INVALID, not zero.

## Rateless comparators

Keep D13-A `riblt_pull_pow2` unchanged as a concrete request/response transport
control.

Add `riblt_stream_lb`, a deliberately optimistic receiver-completion lower-bound
matching the pinned upstream stream-until-stop algorithm:

- one 48-byte START control from B to A;
- then raw fixed-width 24-byte coded cells in sequence, with no per-cell 48-byte
  application header and no feedback request between cells;
- Bob runs TryDecode after every received coded cell;
- stop at the first decoded cell;
- one 48-byte STOP control is charged to application bytes;
- receiver-completion network latency uses one startup RTT plus serialized bytes;
- bytes/CPU that could be in flight after Bob's stop are not guessed.

Because in-flight stop overrun is intentionally omitted, `riblt_stream_lb` is a
lower-bound comparator. It may prevent a false rejection of rateless architecture,
but it can never by itself authorize production GO.

The exact official pin remains
`yangl1996/riblt@4afa6bc06cb2237d9ea273a51d97a7e05b3f573b`.
No Rust port or formula-only 1.35d substitution is permitted.

## Memory accounting

Per endpoint and arm report separately:

- exact source logical bytes and Vec len/capacity where applicable;
- maintained sketch logical bytes;
- candidate/result Vec len/capacity;
- transport/prefix buffers len/capacity;
- native session structures;
- Go slice len/capacity and runtime heap counters for the comparator;
- Linux VmRSS and VmHWM at named checkpoints.

Peak means simultaneously live process evidence, not the sum of unrelated phase
maxima. Harness/oracle memory is excluded from arm memory and reported separately.

## Scaling extension

D13-A correctness grid remains unchanged.

B0 must additionally validate a sparse scaling matrix:
- N = 1,048,576;
- d = 128, 512, 1024;
- balanced shape;
- one deterministic seed family;
- natural RIBLT cap/exhaustion control.

This extension is correctness/readiness evidence only; it is not yet the five-worker
performance matrix.

## B0 acceptance

Output only `MEASUREMENT_READY` or `INVALID`.

ACCEPT requires:
- persistent chain correctness for every U/session-count combination selected by the
  readiness matrix;
- update rejection atomicity;
- exact D11 fresh-rebuild equivalence after every session;
- pull and streaming RIBLT both use the pinned implementation;
- native phase closure;
- memory/capacity/RSS fields present and internally consistent;
- scaling extension complete;
- exact-head GitHub-hosted Rust/research/readiness CI;
- frozen source hashes for the later D13-B measurement implementation.

B0 records no system performance verdict and no API/product decision.
