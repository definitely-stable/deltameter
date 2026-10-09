# DeltaGuard G0: real k-level parity state, inherited coverage and power limits

Research issue #84; parent optimization #70; draft PR #87.
Pre-registration: [DELTAGUARD-G0-PROTOCOL.md](DELTAGUARD-G0-PROTOCOL.md).

## Decision

**G0_FOUNDATION_PASS_EMPIRICAL_POWER_CANDIDATE** on exactly 1,200
source-HEAD-bound observations, five independent GitHub-hosted workers;
**NO PRODUCT/NEW-M/NEW-ALPHA GO**.

Measurement source SHA: `c9036ac68ab13656115a57c970049a696009ab7a`.
[Hosted run 37876830842](https://github.com/definitely-stable/deltameter/actions/runs/37876830842):
five workers and fail-closed aggregate SUCCESS. The deterministic
test MASTER_KEY is public and the corpus consists of fixed token ranges;
these results are **not** an empirical demonstration of a
keyed PRF security bound or statistical failure probability <=1e-6.

## Mathematical/engineering boundary

The existing C0 exact ideal-oracle per-level result grants
`Pr_d[U_j<d] <= alpha_j=10^-6/64` for all predeclared
j=1..52 with m=4096, unchanged Q32 table and nonadaptive
input oracle model. Its simultaneous event shows that any subset
min of original level endpoints remains an upper bound except
with total probability at most 52*alpha_j<=10^-6.
For any d>T, `SAFE_BELOW_T := min(U_j) <= T` is therefore a
one-sided error with that same upper probability. The proof is
inherited, not novel and not automatically transferred to
m!=4096 or adaptive token selection.

Unlike projecting an already built full bitmap, `TinyGuard`
physically stores only selected k levels and updates them from
the SAME keyed BLAKE3 token mapping as canonical J52.
For two independently held compatible sets, XOR of k-level states
matches the projection of the full J52 XOR **bit for bit**; Q32
upper endpoints are identical to the selected full-state endpoints.
The k=1,2,4,8 levels use T-only, predeclared CENTER ordering,
nested and not chosen using observed d or input state.
No per-token bitmap reallocation. Full-source XOR/cancellation,
count invariants and no erroneous smaller bound passed each worker.

### Actual logical bitmap storage

| k | One-owner parity bytes | Two owners' parity bytes |
|---:|---:|---:|
| 1 | 512 | 1,024 |
| 2 | 1,024 | 2,048 |
| 4 | 2,048 | 4,096 |
| 8 | 4,096 | 8,192 |
| J52 full (baseline) | 26,624 | 53,248 |

These are **physical allocated bitmap payload**, not total live Rust
memory: Box slice/owner, key, selection IDs, heap metadata and shared
Q32 table remain extra. The frozen full-wire complete baseline is
53,500 application bytes for two senders; DO NOT equate that to
bitmap bytes or assume the guard has an approved wire format.

## Empirical usefulness, NOT theorem

Each cell is 5 workers x 3 independent *deterministic* token range seeds
(15 observations) under one fixed public test key, for T:
64/4096/65536/1048576, d/T=0,0.25,0.75,1,1.125.

Receiver SAFE_BELOW_T counts when actual d/T=0.75:

| Threshold | k=1 | k=2 | k=4 | k=8 |
|---:|---:|---:|---:|---:|
| 64 | 0/15 | 0/15 | 0/15 | 0/15 |
| 4,096 | 10/15 | 10/15 | 10/15 | 10/15 |
| 65,536 | 10/15 | 14/15 | 15/15 | 15/15 |
| 1,048,576 | 11/15 | 14/15 | 14/15 | 14/15 |

At d/T=0.25, T=64 still yields 0/15 for k<=8,
but at T=4096/65536/1048576 all k values produce 15/15.
At d/T=1 and 1.125 all these cells yield 0/15 SAFE.
No false-safe outcome was observed in 1,200 deterministic records,
which **does not certify** a rare-event probability of 10^-6;
the *mathematical* ideal guarantee is inherited from C0.

### Real implications

1. A 512B sketch can sometimes make a useful safe decision near
   0.75T for T>=4096, but may be too conservative. At 65536, k=4
   recovers useful responses on all 15 named input ranges vs k=1 on
   10/15, at the cost of 2KiB per owner.
2. **Small T is an explicit negative result** for this original
   m4096/alpha=delta/64 Q32 calibration and CENTER-order policy;
   the new guard code is correct yet it returns uncertain in the
   observed small-T lanes. Do not add public small-delta marketing.
3. The 1,200 observations are NOT independent random-key draws,
   no power confidence interval is justified, and no width/latency
   Pareto is proven. Reused deterministic token ranges are a
   correctness/gap discovery tool only.
4. Honest trusted one-sender threshold result remains much
   cheaper than sending even a k=1 guard: existing OPT-C C3-A
   sender-scalar application control 24B. Two separate
   owners who cannot compute cross-source XOR independently are
   the meaningful conditional use case.

## Artifact provenance

| Artifact | GitHub ID | SHA256 ZIP digest |
|---|---:|---|
| Summary | 11592777061 | d71fc6171ff13308eb5b753e65a71bd35a2ad8323bc5aa9f7bb1c55946f9fe99 |
| Worker 1 | 11593175180 | 3b63211436f3b511cba05703ffcc37c8a7a0201c4366b18fc3127399cb2aea7d |
| Worker 2 | 11593041125 | 2f8f56cec680b2acd03c708b757eef08cb0214957585b2b29829a51b6ef6e82d |
| Worker 3 | 11592497899 | ea1cb18fedc239a8da7da1ae1429d0b161cc04e8b05a47fd773866b952c2ef22 |
| Worker 4 | 11593165635 | e65c132562c93c86812d8f1b0574d04ee3157ea67d96e263396d89839e9f9e10 |
| Worker 5 | 11592258384 | c51e1348a8c4e5d926761ece9454cf97ae57fb37a0182ad12b61267efb91dcd8 |

## Next gated slice: G1 exact certificate redesign, only

Before any new performance claim or public Rust type:
- Investigate T~32/64/256 separately; exact finite tail/power
  certification for smaller m and/or different alpha allocation
  is mandatory, old m4096 Q32 table cannot be reused.
- Keep the two-owner XOR compatibility; quantify how many levels
  need to be maintained when T may vary over time, and price
  rebuilding them from a retained source set.
- Compare k=1/2/4 with J52 + CACHE under equal end-to-end system
  semantics; measure actual owner/key/RSS, creation/update/XOR,
  and serialization rather than isolated bitmap payload alone.
- If no rigorous power/frontier win survives, record STOP_GUARD_PRODUCT
  and retain this laboratory as a boundary/counterexample.
- Issue #85 hosts the general m/J/alpha fixed-budget optimization,
  #86 handles adaptive token/key threat model. G0 does not solve either.

No public API, Snapshot v1 or statistical guarantee vocabulary changed.
