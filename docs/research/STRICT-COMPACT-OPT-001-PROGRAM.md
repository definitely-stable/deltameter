# STRICT-COMPACT product/state optimization program

Issue: #70. Parent research program: #59. Public-boundary gate: #69.

Status: **research/product optimization before public profile freeze**.

## Baseline entering this program

STRICT-COMPACT has already crossed the basic feasibility boundary.

Accepted research candidate:

~~~text
token domain                  all u64
rows m                        4096
stored levels J               64
primary packed state          32,768 B
static certified Q32 table    16,384 B
ideal statistical failure     <= 1e-6
useful width range            every integer d in [4096, 2^64-1]
q95 width                     U/d <= 1.5
domain-cardinality ceiling    2^64, represented as u128
oracle                         keyed BLAKE3 v1 research contract
oracle update cost             ~96.5 ns/token median on canonical 5-worker run
private API status             GO_PRIVATE_API_FREEZE
~~~

The current state is approximately 11.5x smaller than the default strict Energy
primary state.

This document does **not** reopen the accepted finite-sample theorem or range
certificate. It asks whether the current J=64/m=4096 design is the best product
shape.

## Design principle

Do not freeze a public profile merely because the first certified point is good.

The optimization sequence must distinguish:

1. cheaper engineering improvements that preserve the current theorem;
2. representation/product improvements that preserve the same sufficient
   statistics;
3. generalized statistical constructions requiring a new proof;
4. oracle/security research that can change the coverage model.

A change from class 1 or 2 should not wait on class 3 or 4.

---

## A. Range-aware stored-level frontier

### Motivation

J=64 exists because the current profile targets nearly the entire u64 difference
domain.

For many object-store, cache, index or backup inventories, an application can state
a useful maximum divergence far below 2^64.

At m=4096, each stored level costs exactly:

~~~text
4096 bits = 512 B
~~~

Therefore:

| J | primary state |
|---:|---:|
| 24 | 12 KiB |
| 32 | 16 KiB |
| 40 | 20 KiB |
| 48 | 24 KiB |
| 56 | 28 KiB |
| 64 | 32 KiB |

### First experiment

Keep every other accepted component unchanged:

- m=4096;
- delta=1e-6;
- current level law;
- current certified threshold construction;
- current keyed-oracle model;
- conservative existing per-level alpha allocation initially.

For every J in:

~~~text
24, 32, 40, 48, 56, 64
~~~

certify the largest continuous interval

~~~text
[4096, d_max]
~~~

for which q95 U/d <=1.5.

### Required output

One Pareto table:

~~~text
state bytes | J | d_max | proof intervals | q95 gate | saturation behavior
~~~

Do not select a new default until this table exists.

### Useful early property

Using fewer levels with thresholds built for alpha=delta/64 is automatically
conservative with respect to familywise error because the used alpha mass only
decreases.

This permits a very cheap first pass before re-optimizing alpha allocation.

---

## B. Maintained odd-level counts

### Motivation

Current private estimate extraction scans the packed state to reconstruct all
S_j counts.

But every toggle already knows:

- row;
- level;
- old bit;
- new bit.

Therefore maintain:

~~~text
odd_counts[J]: u16
~~~

alongside the packed words.

For J=64:

~~~text
64 * 2 B = 128 B
~~~

additional state.

### Update rule

~~~text
0 -> 1 : odd_counts[level] += 1
1 -> 0 : odd_counts[level] -= 1
~~~

No probabilistic semantics change.

### Research questions

- exact scalar update overhead;
- repeated-estimate latency;
- cost of rebuilding counts after XOR merge;
- whether counts should remain purely derived/cache state;
- whether counts belong outside canonical snapshots.

### Default design bias

Keep counts **derived and non-serialized** unless measurement proves persistence is
valuable.

This preserves the canonical mathematical state as the parity bitmap.

---

## C. Progressive/prefix transfer

### Observation

At m=4096 one complete level is a 512-byte bitmap.

The sufficient statistics are naturally layered by scale.

A research representation can therefore transmit:

~~~text
initial prefix -> add more levels -> add more levels -> ...
~~~

without retransmitting old data.

### Coverage argument to preserve

If each level j has a predeclared failure budget alpha_j and

~~~text
sum_j alpha_j <= delta,
~~~

then with probability at least 1-delta all per-level upper confidence statements
hold simultaneously.

Consequently a receiver may inspect any received set L of levels and use:

~~~text
U_L = min_{j in L} U_j
~~~

without consuming a new error budget merely because it chose to request another
level after seeing an earlier bound.

A formal note must state the exact conditions before calling this an "anytime"
contract.

### Product questions

- bytes needed to reach useful width at each d regime;
- prefix order: low-to-high, high-to-low or profile-specific;
- trailing-zero level elision;
- independent level chunks;
- incremental corruption/authentication framing;
- exact RTT cost if levels are requested interactively;
- one-shot ordered stream vs request-response.

No public snapshot format is created in this program.

---

## D. Generalized Certified Parity Ladder

The current construction inherits several design choices from F-PCSA that may not be
optimal for a set-specific one-sided upper-bound primitive.

### General object

For level j choose:

- selection probability pi_j;
- number of rows m_j;
- error budget alpha_j.

One token chooses a level, chooses a row and toggles parity.

If the random GF(2) coefficient is removed and every selected cell is toggled, the
level process remains an odd-occupancy birth-death chain.

For fixed level:

~~~text
P(s -> s+1) = pi_j * (m_j-s)/m_j
P(s -> s-1) = pi_j * s/m_j
P(s -> s)   = 1-pi_j
~~~

Under Poissonization:

~~~text
S_j ~ Binomial(
    m_j,
    (1-exp(-2*pi_j*d/m_j))/2
)
~~~

The current proof machinery may therefore generalize, but that generalization must
be proved separately.

### Optimization problem

Seek:

~~~text
minimize sum_j m_j
~~~

subject to:

~~~text
P_d(U < d) <= delta
P_d(U > C*d) <= beta
for every d in declared range
~~~

plus:

- XOR mergeability;
- bounded per-token work;
- deterministic certified lookup construction.

### Variables worth optimizing

- geometric ladder ratio;
- non-geometric pi_j;
- heterogeneous m_j;
- nonuniform alpha_j;
- number of levels;
- supported d range.

### Novelty discipline

Do not call the basic odd-occupancy chain new.

Possible contribution is the optimized certified multiscale construction and its
state/width frontier.

---

## E. DeltaGuard threshold primitive

Many product workflows do not need an estimate.

They need a policy decision such as:

~~~text
is drift definitely small enough?
~~~

Define a one-sided guard around a caller-declared threshold T.

Conceptual result:

~~~text
WithinThreshold
ExceededOrUncertain
~~~

Required safety property:

~~~text
P(declare WithinThreshold when d > T) <= delta
~~~

under the declared oracle model.

Because T is fixed, a guard may require only the few scales informative around T.

### Target

Investigate whether a useful guard can fit in:

~~~text
hundreds of bytes to a few KiB
~~~

rather than 12-32 KiB.

### Candidate uses

- replica drift SLO;
- cache divergence alarm;
- object-store inventory health;
- backup/catalog consistency gate;
- trigger/no-trigger decision for an expensive exact audit.

This is potentially a distinct small crate surface even if the general upper-bound
estimator remains specialized.

---

## F. Oracle portfolio

### Control

Keep keyed BLAKE3 as the reference oracle.

A replacement must win on a named product axis, not merely produce a smaller
nanosecond number.

### Candidates

Research only candidates with a defensible keyed PRF/MAC contract:

- keyed BLAKE3;
- KMAC128;
- HMAC-SHA256;
- AES-CMAC or a carefully defined AES-based PRF;
- SipHash-128 only after implementation/security provenance review.

### Do not promote as theorem-carrying PRF replacements

Do not substitute:

- XXH3;
- wyhash;
- aHash;
- rapidhash;
- another unkeyed/non-cryptographic mixer

merely because it benchmarks faster.

### Comparison dimensions

- ns/token for 8-byte token input;
- output bits / extraction convenience;
- dependency and build footprint;
- standardization/compliance value;
- Rust portability;
- transitive unsafe/native code implications;
- key derivation/domain separation;
- security/PRF statement quality.

### Current expectation

BLAKE3 remains the likely general default.

KMAC/HMAC/AES candidates are more likely to win a compliance/deployment lane than a
raw-speed lane.

---

## G. Mathlab research transfer

The following should not be improvised inside production Rust.

### G1 — finite-independence odd-occupancy theorem

Question:

What degree of independence is actually sufficient for the fixed-d lower-tail and
multilevel familywise guarantee used by STRICT-COMPACT?

Goal:

replace the computational PRF transfer with an information-theoretic random-family
contract if practical.

Candidates include:

- k-wise independent hashing;
- tabulation families;
- permutation/tabulation constructions;
- other explicitly analyzable small-seed families.

### G2 — adaptive-token theorem

Current transfer assumes token selection is independent of the fresh secret key and
is non-adaptive with respect to sketch observations.

Research whether the theorem can survive adaptive token choice through:

- deferred decisions;
- martingale/stopping-time arguments;
- conditional random-function exposure;
- e-process/confidence-sequence machinery.

A successful result would materially simplify public UX.

### G3 — heterogeneous ladder optimum

Study the certified optimization over:

~~~text
(pi_j, m_j, alpha_j)
~~~

with finite-state/state-width constraints.

This is adjacent to Mathlab's locality/resource-allocation program but is
probabilistic, not exact ASET.

### G4 — state lower bound

Derive a lower bound for any mergeable odd-occupancy/multiscale construction that
provides:

~~~text
failure <= delta
q_beta width <= C
d in [d_min,d_max]
~~~

If the construction is close to a lower bound, DeltaMeter gains a much stronger
scientific/product claim.

### G5 — nested/prefix optimum

Study whether a common prefix family can be simultaneously near-optimal over several
d ranges while preserving bounded update locality.

This is the closest direct bridge to Mathlab LENT-001 Priority 3.

### Repository separation

These questions deserve a separate Mathlab research family rather than being folded
into LENT-001 exact ASET.

Suggested working label:

~~~text
OCC-001 — Certified Odd-Occupancy Ladders
~~~

No novelty claim follows from the label.

---

## H. Product/API implications

### Do not freeze public profile yet

Issue #69 may continue resolving:

- computational coverage vocabulary;
- BLAKE3 runtime dependency;
- key lifecycle;
- snapshot/key binding.

But the first public profile/state layout should wait for A-C.

Reason:

- A may reduce primary state;
- B may alter derived in-memory cache layout;
- C may change the natural interchange shape.

### Keep existing boundaries

Until a separate public GO:

- Energy remains the existing strict/theorem-facing backend;
- ParityDeltaMeter remains asymptotic;
- snapshot v1 is unchanged;
- STRICT-COMPACT remains private/research;
- D11/reconciliation remains stopped.

---

## Execution order

### Phase OPT-A

1. range-aware J certificate;
2. Pareto frontier;
3. decide whether J=64 remains the general profile.

### Phase OPT-B

1. maintained level counts;
2. paired update/query benchmark;
3. keep only if measured value exists.

### Phase OPT-C

1. progressive level representation;
2. theorem note for adaptive prefix extension;
3. byte/RTT product model.

### Phase OPT-D

Generalized parity ladder only after A-C establish remaining headroom.

### Phase OPT-E

DeltaGuard threshold prototype.

### Phase OPT-F

Oracle portfolio after product shape stabilizes.

Mathlab G1-G5 can proceed independently.

---

## Decision markers

### PRODUCT_OPT_PASS

At least one optimization materially improves state, wire, query cost or product UX
without weakening the guarantee contract.

### KEEP_32K_BASELINE

A-C show that the existing J=64/m=4096 candidate is already the best simple v0
profile.

### OPEN_GENERALIZED_LADDER

D has credible certified headroom and deserves a new proof program.

### OPEN_DELTAGUARD

Threshold mode has a substantially stronger state/UX frontier.

---

## Forbidden shortcuts

- no public profile selected from Monte Carlo alone;
- no new hash accepted only because it is faster;
- no weakening delta without an explicit product decision;
- no hidden automatic key generation;
- no snapshot-v1 mutation;
- no resurrection of D11 as part of this program;
- no new theorem claim before the corresponding Mathlab/prior-art gate.
