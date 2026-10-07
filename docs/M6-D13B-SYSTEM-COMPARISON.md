# M6-D13-B — five-worker system comparison

Status: **FROZEN BEFORE PERFORMANCE EVIDENCE**. Issue #56; parents #19/#14.
Baseline: `8b5421709965e88dfbb74cd61eb742321d738efa` (merged D13-B0
MEASUREMENT_READY). Machine contract: `research/m6d13b/contract.json`.

This slice is the first and only D13 system-performance comparison. It does not
change the accepted D11 decoder, the pinned RIBLT implementation, the public API,
snapshot-v1 or Coverage.

## Question

For one-way exact synchronization of retained u64 sets, is there any named workload
where maintained D11 or a rateless direction beats the simplest direct exact
transfer by a material margin after charging maintenance, native compute, bytes and
RTTs?

The four reported arms are:

1. `direct`: canonical complete A-list transfer;
2. `d11`: maintained accepted D11, k=1→2→4→8, independent verification and exact
   fallback;
3. `riblt_pull`: pinned upstream RIBLT under the concrete pull/power-of-two
   transport control;
4. `riblt_stream_lb`: the B0 optimistic receiver-completion streaming lower bound.

The stream lane is intentionally favorable to rateless reconciliation: in-flight
stop overrun is omitted. It can preserve a rateless research direction but can
never authorize production GO.

## Measurement source

Reuse the exact B0 native Rust worker and pinned Go adapter. No historical D11
module is edited. Python constructs deterministic operations, drives workers,
validates records, performs wire accounting and evaluates the frozen model only.
Python/subprocess elapsed time is never algorithm compute.

Before the first timing run, Git blob hashes of every contract, controller,
validator, worker and D11 support source used by the comparison are committed and
verified fail-closed in CI.

## Hosted replication

Run five independent GitHub-hosted timing jobs. Every worker executes three
deterministic process/replicate families. Arm order rotates by worker and process so
one arm does not always run first.

Within a core arm/cell, initialize once, execute two unreported warm-up sessions,
then retain the same state for ten measured sessions. Native timers remain those
inside the B0 workers.

Raw observations are retained. Aggregation never averages unrelated RTT/bandwidth
cells.

## Frozen workloads

Core maintained matrix:

~~~text
N = 8,192; 65,536
updates/session = 1; 8; 64
warm-up sessions = 2
measured persistent sessions = 10
~~~

The measured per-session and initial-build components are used to evaluate
sessions/build = 1,10,100. B0 already established 100-session state correctness;
D13-B does not need to repeat 100 timed sessions to model build amortization.

Required guards, not hidden favorable exclusions:

- N=65,536 with exact d=8 and d=9 one-shot boundary rows;
- N=1,048,576 with d=128,512,1024 one-shot scaling rows;
- the B0 deterministic d=8/k=1 false-candidate witness remains a regression;
- every fallback and false candidate is retained and charged.

Core inserted-key families use deterministic full-width SplitMix64 domains,
disjoint from the initial 1..N source range. Operations are membership-validated.
For even U, remove U/2 and insert U/2 so cardinality stays near N. For U=1,
alternate insertion/removal.

## Native compute and amortization

For each raw session:

~~~text
session_native =
    sum(successful native update work)
  + native reconciliation/sync work
~~~

Initial build is recorded separately.

For modeled sessions-per-build S:

~~~text
amortized_compute(S) = session_native + initial_build / S
~~~

D11 includes sketch-build and sketch-update work. RIBLT includes its per-session
encoder/decoder source import because the pinned implementation is rebuilt for each
session. Direct exact includes its canonical serialization/apply work.

The primary comparison is elapsed native compute, not process CPU ticks. CPU ticks
remain a coarse contamination cross-check.

## Wire accounting

Header is the frozen 48-byte D13-A frame.

Direct exact:

~~~text
bytes  = 2*48 + 8 + 8*|A|
rounds = 1
~~~

D11 charges every attempted request/response stage and only the new syndrome bytes
(17,8,16,32). A successful provisional candidate additionally pays the full
reverse-list verification exchange. A false candidate pays that verification plus
the terminal exact fallback. A reject-through-k=8 pays all four stage attempts plus
the terminal exact fallback.

For a false candidate the B0 worker does not expose the temporary candidate length.
D13-B therefore uses a predeclared conservative verification upper bound
`|A| + final_k` keys. This can only make D11 less favorable and changes at most
64 payload bytes at k=8.

RIBLT pull charges 24 bytes per coded cell plus one request/data header pair per
batch. Successful candidates pay independent full-list verification. Exhaustion
pays terminal exact fallback.

RIBLT stream-lower-bound charges one START header, 24 bytes per real coded cell and
one STOP header, then the same candidate verification. No in-flight stop overrun is
invented.

Core RIBLT fallback with d<=64 is retained. A fallback before the 1024-cell cap is
treated as a verified false candidate and pays verification plus exact fallback.
A cap-bound fallback is reported as exhaustion and cannot create a qualifying
rateless cell.

## Network model

For each raw row and each frozen network cell:

~~~text
T = amortized_compute
  + rounds * RTT
  + 8 * application_bytes / (bandwidth_Mbps * 1e6)
~~~

RTT = 0,1,10,50,100 ms.
Bandwidth = 1,10,100,1000 Mbps decimal.

No sleep, socket timing, TLS/IP assumptions or subprocess wall time enters T.

## Decision gates

For each named tuple
`(N, updates/session, sessions/build, RTT, bandwidth)`, compute the median modeled
T separately on each of the five workers.

D11 qualifies a cell only if:

~~~text
D11 <= 0.90 * direct
~~~

on **every** worker.

`riblt_pull` is descriptive transport evidence only.

`riblt_stream_lb` establishes `RATLESS_POTENTIAL` only if its optimistic lower
bound is at least 10% below direct on every worker. It never authorizes production
GO. If even the optimistic lower bound cannot beat direct, that is valid evidence
against rateless value for that named cell.

Overall:

- D11 qualifying cells only → `D11_CONDITIONAL_PRIVATE_ONLY`;
- stream-lower-bound qualifying cells only → `RATELESS_RESEARCH_ONLY`;
- both → `MIXED_CONDITIONAL_PRIVATE_ONLY`;
- neither anywhere → `STOP_SYSTEM_PRODUCT`.

Any missing worker/process/cell, correctness failure, frozen-source drift,
unexplained D11 fallback, malformed accounting, oracle-driven stage/cell choice or
non-finite modeled cost → `INVALID`.

Memory/RSS/HWM, fallback counts and false candidates are reported as guardrails.
No favorable cell can erase a correctness failure.

Regardless of outcome, production/public ExactSmallDelta remains **NO-GO**. A
future compact digest-verification tier or public reconciliation API would require
a separate contract and decision.
