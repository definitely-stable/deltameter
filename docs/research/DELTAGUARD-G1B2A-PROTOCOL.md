# DeltaGuard G1-B2-A — frozen near-full single-level parity protocol

Parent #89 comment [6074465241](https://github.com/definitely-stable/deltameter/issues/89#issuecomment-6074465241).
Follow-up to accepted D54 and PR #90. **Preregistered BEFORE reading B2 numerical cutoff results or sampling B2 two-party cases.**
Research-only, no public API, no existing Snapshot v1, no changed G0/G1B oracle/table, no release. GitHub-hosted CI only.

## Question and decision order

Before spending engineering capacity on correlated multi-level joint DP, test a
near-always-active **one-level** GF(2) set-symmetric-difference guard.
A token hashes to one of D=2^b equiprobable slots: m=D-1 real rows
0..m-1, and single final dummy slot m which changes no bit. XOR of two
independently maintained bitmaps equals XOR parity for A triangle B.
Rows are stored as ceil(m/64)*8 bytes per owner, including one unused
padding bit in the final u64. Test b=6,7,8,9,10,11,12,13
(m=63,127,255,511,1023,2047,4095,8191), all versioned profile
identity includes b, T, key-derived oracle version, schema ID.
Selection of T and b must be ex-ante, not based on the bitmap.
Do not reuse G1B unit-coefficient or J52 state; derive a distinct
BLAKE3 context. Lower b bits of the first eight hash bytes give exact
uniform slot in ideal-oracle model, no modulo bias and no division.
The public deterministic fixture key is NOT a secret or a PRF proof.

**Why retain a dummy slot:** unconditional D=m toggling has a parity
periodicity and fails the simple all-d stochastic-order theorem. Our
D=m+1 makes the exact kernel stochastically monotone at equality.
Even for very large d, the distribution of S remains ordered.
Not a newly discovered theorem, and not a public random-oracle bound
against adaptive key-informed token selection or chosen transcript.

## Exact finite-sample proof contract (independent of CPU benchmarks)

T={32,64,128}, delta=1/1,000,000 per ONE predeclared (T,b,key).
Each one-token update from S=s has integer numerator weights under D:

- go to s+1: (m-s)
- go to s-1: s
- stay at s: D-m=1

The exact probability-numerator recurrence is

`n_(d+1)[s]=(D-m)n_d[s]+(m-s+1)n_d[s-1]+(s+1)n_d[s+1]`
with n_0[0]=1, outside-range n zero, denominator D^d.
At each step verify `sum_s n_d[s]=D^d`; no floating point, Poisson,
Chernoff or Monte Carlo may affect acceptance.

Adjacent stochastic kernels are ordered iff
`P_s(s+1)+P_(s+1)(s)=(m+1)/D<=1`, equality for D=m+1;
starting at S_0=0 yields `S_d <=st S_(d+1)` for all d.
Thus for all integers d>T including d<=2^64, the largest
`Pr_d(S<=c)` equals `Pr_(T+1)(S<=c)`.
Choose maximal c or -1 satisfying exactly
`1000000*sum_{s<=c} n_(T+1)[s] <= D^(T+1)`.
For d<=c, S<=d<=c pointwise, so useful SAFE is **deterministic**,
not an empirical confidence statement.

Run an INDEPENDENT small full 2^m bitmask Markov chain oracle for
b=1,2,3, m=1,3,7 and d=0..12 (39 exact match cases),
including total probability normalization.
Cross-check exact cutoff maximality, all proposed 24 profiles,
tiny CDF time-order and mutation/malformed parameter fail closed.
Generate fixed-certificate TSV + JSON with source SHA256 and explicit
version. Include a no-lazy D=m negative witness (e.g. m=1,d=1->2
changes P(S=0) upward) to prevent accidental proof reuse.
The full-u64 input domain is nonadaptive, unique distinct u64 tokens:
repeated same token cancels through XOR; no guaranteed adversarial
key-sensitive assignments.

## Frozen utility/integration matrix

Before observation: 5 independent hosted workers x 3 thresholds
T={32,64,128} x 9 ratios {0,1/4,1/2,3/4,9/10,99/100,
1,101/100,11/10} x 3 deterministic input seeds x 8 bitmap sizes
= **3240 physical two-owner rows**.
For <=1 ratios use d=floor(T*num/den); for >1 use
d=ceil(T*num/den), ensuring d>T at 101/100 even when T<100.
Run all eight b candidates, no outcome-based subset filtering.
Every row logs T, b, m, d, seed, ratio, odd-count, cert-cutoff,
SAFE and false-SAFE marker, bytes per owner and metadata footprint.
True bitmap physically maintained independently by each owner
(no extracting a candidate from full J52); XOR parity checked
against independent direct per-token bitmap/reference, wrong
key/T/b and old B1 profile incompatibility fail closed.
Use the b-bit mask row mapping and handle the dummy row correctly.
Use immutable disjoint owner sets, plus deterministic duplicate/
cancellation and overlapping-token vectors.
All evidence tied to exact Git SHA, cert SHA256, Rust version,
complete and unique source keys, including mutated TSV rejection.
False-safe sample=0 is not the 10^-6 theorem.

## Precommitted product decision gates

- `B2A_EXACT_CERT_PASS`: independent exact small-bitmask oracle (39),
  exact full cutoff for 24 (T,b), symbolically proven monotonicity
  and all invalid-input rejection; if absent FAIL_CLOSED.
- `B2A_PHYSICAL_3240_PASS`: 3240 unique, source-bound,
  two-owner XOR rows with correct proof-bound cutoffs and
  compatibility/errors on all five hosted workers.
- `B2A_UTILITY_CANDIDATE` at T64 and true d48, with <=256B owner
  bitmap and >=95% *ideal* exact power, plus strict delta <=1e-6.
  Note if c>=48, power=1 for every input, much stronger than >=95%.
- `B2A_PRODUCT_GO` is **not** authorized by the math/physical
  stage. Later explicit stage must compare end-to-end 2-owner retained
  update cost, key/Box metadata/RSS, actual wire+RTT with repeated
  sessions/fallback against G1B1/J52/direct exact and trusted
  one-sender scalar. Standalone 32/256/512B bitmap isn't enough.
- If near-T d≈.99T remains weak, say STOP_NEAR_T; do not
  misrepresent a wide safe margin as reliable near-T decision.
- Do not build a correlated joint-level decoder unless this
  simpler proven frontier fails meaningful cost/utility criteria.
  Repeated/adaptive query/key threat remains open in #86.

## Prior art / scope

Classic hypercube/lazy Ehrenfest occupancy and XOR parity sketching,
including Mitzenmacher, Pagh, Pham, **Odd Sketches** (WWW 2014)
https://doi.org/10.1145/2566486.2568017 .
No mathematical novelty claim for bitmap, XOR or recurrence.
Any product value must be demonstrated by this narrow finite-sample
one-sided *threshold* profile and complete system comparison.
