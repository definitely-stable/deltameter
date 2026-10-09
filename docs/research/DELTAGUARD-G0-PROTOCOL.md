# DeltaGuard G0 — fixed-m proof inheritance and selection-projection pre-registration

Parent: #84; related #85 exact m/alpha opt and #86 adaptive threat model.
**Frozen before execution.** Research-only, no public API, no snapshot changes.

## Correct statement (theorem inherited, not new)

Fix one immutable pair A,B ⊂ u64, d=|A△B|, secret keyed-BLAKE3
oracle independent of the preselected nonadaptive input tokens, the
existing exact-certified m=4096 table with preallocated
alpha_j=10^-6/64 for j=1..52, and stable oracle/profile.
Each U_j is the same Q32 per-level upper endpoint as the accepted
OPT-A/C proof. Select exactly k∈{1,2,4,8} levels via the C2
T-centered **T-only** deterministic order; their IDs are fixed before
seeing A/B or the oracle outcomes. Maintain ONLY those k 512-byte
parity levels per source. The receiver computes XOR parity levelwise.

The function returns SAFE_BELOW_T precisely when min_{j selected} U_j<=T,
else EXCEEDED_OR_UNCERTAIN (which does NOT establish that d>T).

On the simultaneous event E={all 52 U_j>=d} with
Pr(E)>=1-52*10^-6/64>=1-10^-6, every possible k/T selected
subfamily has U_subset>=d. For fixed nonadaptive inputs and T (or even
T selected by viewing endpoints for the same frozen snapshot),
Pr_d(SAFE_BELOW_T and d>T)<=10^-6. **This does not make
cross-snapshot, newly chosen adaptive tokens/key-reuse experiments valid**.
Finite failure guarantee is ideal-oracle only; keyed BLAKE3 is a
separate computational assumption with NO concrete distinguishing
advantage bound claimed.

The result holds regardless of correlations between levels because
the existing familywise event is simultaneous. It is an elementary
corollary of the existing proof, not a new tail bound or original theorem.
It provides SAFETY only; fixed selected levels may always saturate
(2^64) and hence have poor answer utility/power.

## G0 frozen executable experiment

Exactly five GitHub-hosted workers (worker 1..5).
Thresholds T={64,4096,65536,1048576}, nonadaptive D/T numerator
{0,2,6,8,9}/8, seeds {0,1,2}, active levels k={1,2,4,8}.
Total 5*4*5*3*4=**1200** source-bound observation rows.
Distinct deterministic u64 token sets split between two independent
senders (half/half), same keyed oracle config. Two source-only compact
sketches are built **from token toggles** (not a projected complete state)
and XOR-ed after independent maintenance. Compare their selected
per-level words/counts exactly to the canonical J52 full sketch XOR,
and subset U>=complete U for every record. Check toggle cancellation,
unchanged backing allocations during update, invalid profile/key/order
failure boundaries as applicable.

Physical *payload* per compact owner: m*k/8 = 512,1024,2048,4096B.
Total **two-owner** bitmap payload doubles this; key, Vec header,
heap allocation overhead, common Q32 table and wire framing are
separately declared, NOT concealed or mislabeled as full footprint.

Power/utility: per (T,d/T,k) percentage of deterministic seeds returning
SAFE_BELOW_T. Count any observed false-safe row; do not claim that
zero finite empirical failures proves delta=1e-6. A single failed
full/subset equivalence or corrupted table digest marks INVALID.
No p95 latency claims in G0: this is a feasibility/power footprint
study, not a five-worker CPU regression benchmark.

## Stop/go criteria (frozen before samples)

G0_FOUNDATION_PASS iff all exact identity/count/order/cancellation
tests and 1200 observations pass source SHA and table digest checks.
If a fixed-T/k profile demonstrates useful returns for d<=0.75T
on the named sample set, classify it as an *empirical candidate*
only, never a public guard approval. If useful-power is negligible,
the result is G0_FOUNDATION_PASS_WITH_LOW_POWER and narrow guard
work should stop unless #85's new m/alpha exact certification helps.
GO_PRODUCT is forbidden at G0: a later G1 must independently
certify changed m/alpha and real T-selection utility before any
smaller profile or external API freeze.

No new PRF, no reuse of m4096 lookup for m!=4096, no Minisketch,
no changes to the frozen C3 sender-first product decision.
