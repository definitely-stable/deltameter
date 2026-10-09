# DeltaGuard G1-B2-A: near-full single-level strict guard evidence

Issue [#89](https://github.com/definitely-stable/deltameter/issues/89),
research [PR #91](https://github.com/definitely-stable/deltameter/pull/91),
threat/model #86, product gate #69.
Precommitted [protocol](DELTAGUARD-G1B2A-PROTOCOL.md) was merged to
the branch at `bb11353ddc261dc65614c53412c1615dade22c05`
**before any candidate B2A CI measurements**.

## Verdict: RESEARCH B2A_EXACT_CERT_PASS and B2A_PHYSICAL_3240_PASS_UTILITY_CANDIDATE

Source/test HEAD `68b2e2c3d894f3716c79f41919d840449b3a774b`,
five GitHub-hosted proof/physical
[CI run 37889355518](https://github.com/definitely-stable/deltameter/actions/runs/37889355518)
exact certificate PASS, workers 1–5 PASS, fail-closed 3,240-row aggregate PASS.
Corresponding suite uses only available GitHub-hosted runners;
standard Rust/research CI are separate baseline gates.

Exact cutoff table SHA256:
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.
The complete machine-readable certificates are in CI artifact.
All inference validity is explicitly **ideal-independent-oracle,
single precommitted T/b, immutable nonadaptive set difference**.
Keyed BLAKE3 behavior is a separate unquantified cryptographic
assumption, not a new statistical theorem.

## Mathematical model (known finite Markov chain, not new)

For b in 6..13, m=2^b-1 parity cells, D=2^b equally probable
hash slots. Real rows 0..m-1 toggle the corresponding cell;
one dummy slot m leaves the bitmap unchanged.
Every distinct token selects an independent slot under the
ideal-oracle assumption; symmetric-difference elements contribute
the XOR of two separately maintained source bitmaps.

For S(d)=number of odd cells, conditional integer transition
numerators: rise `m-s`, fall `s`, stay `D-m=1`, denominator
D=m+1. Exact unnormalized probabilities:

`n_(d+1)[s]=n_d[s]*(D-m)+n_d[s-1]*(m-s+1)+n_d[s+1]*(s+1)`.

Each whole-state integer weight sum must equal `D^d`;
accepting cutoff c demands
`1,000,000 * sum_{s<=c} n_(T+1)[s] <= D^(T+1)`.
The adjacent stochastic kernel satisfies
`P_s(s+1)+P_(s+1)(s)=(m+1)/D=1`.
It follows from this stochastic monotonicity and zero initial
state that for **every integer d>T**,
`Pr_d(S<=c) <= Pr_(T+1)(S<=c) <= 10^-6`.
No approximation/Poisson/Chernoff/float was used for acceptance.

The apparently tempting no-dummy D=m profile has period two
(e.g. m=1: S(1)=1, S(2)=0) and is NOT covered by
the above all-d monotonicity proof.

The deterministic inequality `S<=d` holds for every realization:
if d<=c and c is certifiably safe at T, the guard ALWAYS returns
SAFE. The result is not a guarantee of useful responses for all d<=T.
UNKNOWN does NOT imply d>T.

The certificate has **24 named (T,b)** profiles and **39**
independent exhaustive `2^m` bitmask-state oracle comparisons
(b=1,2,3, d=0..12). Mutated distributions, unsupported b,
invalid ratios/denominators, non-lazy transition weights are
rejected; aggregator tests corrupted cutoff, row count,
key metadata, ratio, odd count and SAFE indicator fail-closed.

## Exact certified cutoffs (safe if observed odd count S<=c)

| b | bytes per owner | m rows | T32 c | T64 c | T128 c |
|---:|---:|---:|---:|---:|---:|
| 6 | 8 | 63 | 5 | 9 | 12 |
| 7 | 16 | 127 | 10 | 19 | 29 |
| 8 | 32 | 255 | 14 | 29 | 51 |
| 9 | 64 | 511 | 18 | 37 | 71 |
| 10 | 128 | 1023 | 20 | 44 | 87 |
| 11 | 256 | 2047 | 22 | **48** | 98 |
| 12 | 512 | 4095 | **24** | **52** | 106 |
| 13 | 1024 | 8191 | 26 | 54 | 112 |

**Frozen utility gate met**: at T=64, d=48, b=11 (256B
bitmap per owner), S<=d<=48 for every pair, so the safe
reply has exact pointwise power 100%, stronger than
preregistered >=95%. Under ideal oracle, when d>64,
one-shot *false*-SAFE still <=10^-6. This is not a proof
about an adversary selecting tokens from the secret key.

For T=32, d=24, b=12 (512B), the same pointwise argument
holds. At T64, d=57 near the boundary, 512B candidate
does not have deterministic guaranteed SAFE; separate
exact ideal-oracle power is low.

## Actual Rust two-party parity evidence

Each owner independently allocates the selected
`Box<[u64]>` bitmap and maintains it using a NEW
`derive_key("deltameter 2026-10-09 deltaguard b2a nearfull GF2 v1", MASTER_KEY)`
research oracle. Individual token hash uses exactly the
lowest b bits of BLAKE3 keyed digest as the uniform
slot under the ideal model, no modulo bias or retries.
There are m usable bits and one permanently zero padding bit.
After source XOR the result equals an independently
constructed one-shot per-token parity reference, word by word.

b/T/key mismatches reject XOR; type/layout incompatible
with old G0/G1B original and unit context, and their
lookup tables do NOT apply. Overlapping-element cancellation,
double toggles, padding bit invariance and all eight
bit-widths are tested.

Frozen 5 workers × 3 T × 9 ratios × 3 seeds × 8 b =
**3,240** source SHA and exact TSV digest-bound,
uniquely enumerated rows; includes above-threshold d
using ceil(101T/100) and ceil(110T/100) even for T32,
not accidental rounding back to d=T. Each worker produces
648 rows, all 5 workers PASS, aggregate PASS.

Selected deterministic public-key fixture cells (15 samples
per T/d/b combination):

| T | d | b | bitmap/owner | certified c | physical SAFE |
|---:|---:|---:|---:|---:|---:|
| 32 | 8 | 8 | 32B | 14 | 15/15 |
| 32 | 24 | 11 | 256B | 22 | 3/15 |
| 32 | 24 | 12 | 512B | 24 | 15/15 |
| 32 | 28 | 12 | 512B | 24 | 0/15 |
| 64 | 48 | 10 | 128B | 44 | 3/15 |
| 64 | 48 | 11 | 256B | 48 | **15/15** |
| 64 | 48 | 12 | 512B | 52 | 15/15 |
| 64 | 57 | 11 | 256B | 48 | 0/15 |
| 64 | 57 | 12 | 512B | 52 | 0/15 |

A 15/15 sample is an *engineering check*, never a
statistical certification of 10^-6 risk; the independently
proved exact chain gives the scoped ideal guarantee.

Measured by Rust `std::mem::size_of::<NearFullGuard>()`
on hosted workers: **64B** in-struct metadata including
32B derived key, PLUS independently allocated bitmap
8..1024B per owner and allocation bookkeeping.
At b11: 256B payload + 64B Rust object = 320B
before allocator overhead. Both-owner payload = 512B.
No RSS high-water mark or end-to-end network latency
performance has yet been measured by this protocol.

## Source and artifact provenance

- Hosted exact/proof/worker/aggregate workflow:
  https://github.com/definitely-stable/deltameter/actions/runs/37889355518
- Source HEAD `68b2e2c3d894f3716c79f41919d840449b3a774b`.
- Certificate artifact ID `11598300197`, ZIP SHA256
  `a4d8d0d017a199feabe3440db5d566b8ed921d3925260148cdae960867fa49a3`.
- Summary artifact ID `11597931172`, ZIP SHA256
  `1eb4514471ac2c154fc6b04619ad32c9ee474ced17505af878bde277bf9fb878`.
- Workers 1/2/3/4/5 ID:
  `11598085816`, `11597861298`, `11598055898`,
  `11598325155`, `11598165545`.
- All test keys in GitHub fixtures are PUBLIC and are
  not a basis for computational PRF security claims.

## Decision and remaining limits

**ACCEPT B2A exact mathematical and maintained-state two-owner
prototype.** It beats the 512B B1 level at T64,d48 in
a pointwise sense with half payload, but no direct
Rust update/heap/retained-state network-cost gate has yet run.
No release or public API GO.

Keep B2A unit profile/key separate from G1B/strict J52,
preserve existing Snapshot v1 and all existing strict proof
coverage vocabulary. Do not claim novelty for the classical
lazy Ehrenfest walk/odd sketch XOR.

**Next B2A-B/system frontier gate**: preregister realistic
number-of-updates/repeated sessions, allocation/heap/RSS,
process CPU, network bytes/RTT and both-owner retained-lifecycle
against direct exact, G1B1 and J52, including
trusted scalar 24B alternative where applicable.
For near-T workflows where SAFE power collapses,
record STOP_NEAR_T and avoid endlessly expanding bitmap
to conceal a different product requirement.
Threat issue #86 (nonadaptive secret key/adaptive observer)
and #69 (public profile/snapshot semantics) remain open.
