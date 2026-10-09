# DeltaGuard G1-B2-B0 — retained-source system foundation evidence

Issue [#92](https://github.com/definitely-stable/deltameter/issues/92),
research [PR #93](https://github.com/definitely-stable/deltameter/pull/93),
preregistered [protocol](DELTAGUARD-G1B2B0-PROTOCOL.md),
ancestor D55 G1-B2-A, no public API/Snapshot v1 edits.

**Verdict: B2B0_PHYSICAL_SOURCE_PASS_GUARD_SCOPED_MODEL_CANDIDATE,
NOT a measured-transport or product GO.**

## Checkpoint evidence

First full valid physical+aggregation GitHub-hosted run:
[37896624499](https://github.com/definitely-stable/deltameter/actions/runs/37896624499);
source commit `a7fa51c3f2e9da2da9f7b32ccb019e40d26cd839`.
B2A exact certificate regenerated independently with unchanged
24-profile/39-tiny-oracle logic and pinned B2A cutoff TSV SHA256
`c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935`.
Five 648-row physical worker batches PASSED, then fail-closed
aggregation PASSED exactly **3,240** distinct source/condition/profile
rows: N={256,4096,65536}, d={0,16,48,57,64,65},
sessions={1,10,100}, 3 repetitions, 4 profiles, 5 hosts.

Note: after this first proof run, a review correction removed
an incorrect assumption that a rare observed statistically permitted
false-SAFE at d>T MUST fail test infrastructure; it now counts the
empirical occurrence independently, without upgrading the secret-PRF
assumption. This does not alter input grid, hashing, cutoff tables or
the probabilistic theorem. Cite final post-correction hosted run
separately upon green status.

## Source lifecycle and correctness

Each worker independently maintains two actual canonical sorted
u64 lists A,B with exact symmetric difference d verified by
a separate two-pointer direct exact implementation at **every**
session milestone. Full initial source ingestion creates physical
separately versioned XOR sketches:

- B2A m2047/b11, 256B parity payload **per owner**, cutoff c48 at T64;
- B2A m4095/b12, 512B per owner, cutoff c52;
- G1-B1 unit j1, 512B per owner, cutoff c13 (separate oracle domain);
- J52 LEVEL_MAJOR, 26,624B per owner, full physical source
  state under the unmodified reference domain.

For sessions 2..100 actual shared new tokens are inserted into
BOTH lists; each owner incrementally updates its persisted parity
state without rebuilding. The symmetric difference remains exactly d.
At each checkpoint, the physical owner XOR is compared word-for-word
against an independent fresh reference calculated from the canonical
two-list symmetric difference. All 3,240 rows record actual Rust
monotonic wall clock native init/update/XOR/query timings.

**J52 here is ONLY a complete physical state, update and bytes
control.** B0 does not load the original J52 Q32 lookup table
and therefore does NOT fabricate a J52 SAFE decision. J52's
synthetic fallback model is not its true certified decision
performance and must not be sold as such. Likewise G1B and
B2A have distinct keyed-hash versions and cannot XOR across profiles.

## Application wire model (not physical TCP)

Two physically independent senders; 64B research-only *assumed*
control header from each sender, without implemented authentication.
The fixture and retained state are real, while these frame bytes
are exact **equations under a frozen hypothetical research framing**.
Not measured sockets, not an authenticated/wire-ready format,
not WAN RTT.

- Direct: 128+8*(|A|+|B|) bytes, actual canonical u64 lists.
- B2A b11: 2*(64+256) = **640** bytes for guard-only.
- B2A b12 / G1B unit: 2*(64+512) = **1,152** bytes.
- J52 full physical exchange: 2*(64+26624)=**53,376** bytes.
- Resolved contract: on UNKNOWN add **24B** request plus the
  *entire* direct full-list response. The extra query RTT is
  counted. A 24B trusted scalar is only a correct comparator
  when a trusted calculator already has both complete lists.

Exact modeled records, session 10, T=64:

| N | d | Profile | SAFE / 15 fixture rows | Guard-only B | Direct-exact B | Resolved median B |
|---:|---:|---|---:|---:|---:|---:|
| 256 | 48 | B2A b11 | **15/15** | 640 | 4,368 | 640 |
| 256 | 48 | B2A b12 | 15/15 | 1,152 | 4,368 | 1,152 |
| 256 | 48 | G1B unit j1 | 0/15 | 1,152 | 4,368 | 5,544 |
| 256 | 48 | J52 physical only | N/A | 53,376 | 4,368 | N/A |
| 256 | 57 | B2A b11 | 0/15 | 640 | 4,376 | **5,040** |
| 256 | 57 | B2A b12 | 0/15 | 1,152 | 4,376 | **5,552** |
| 65,536 | 48 | B2A b11 | **15/15** | 640 | 1,048,848 | 640 |
| 65,536 | 48 | B2A b12 | 15/15 | 1,152 | 1,048,848 | 1,152 |
| 65,536 | 48 | G1B unit j1 | 0/15 | 1,152 | 1,048,848 | 1,050,024 |
| 65,536 | 57 | B2A b11 | 0/15 | 640 | 1,048,856 | **1,049,520** |

**Why data transfer collapses only for the narrow contract:** with
predeclared T64 and b11 c48, every true d<=48 necessarily
satisfies S<=d<=48, regardless of the particular hash mapping,
so SAFE terminates without fallback. This is deterministic
utility, **not** a statement of transport speed or a proof of
computationally keyed BLAKE3 security.

Near threshold d57 was UNKNOWN in those fixture cells. With
a *mandatory resolved decision* the protocol sends 640B probe
+24B request + full list, strictly more than direct exact
and at least one extra RTT. No result asserts that UNKNOWN
proves d>T; error target <=10^-6 covers false SAFE only
under a single precommitted nonadaptive ideal-oracle model.

For B2A11, N256,d48, 10 sessions, hosted median actual native
initial ingestion of two source states ~45,762 ns; XOR/query
p95 ~141 ns. With N65536 the initial-ingestion median is
~10,772,339 ns, query p95 ~264 ns. J52 complete baseline
at N65536 has build median ~11,463,617 ns and XOR/query
p95 ~4,278 ns. These are five hosted pooled fixture
statistics, not repeatable device-wide performance contracts.
The raw full records include incremental CPU and direct exact
two-pointer cost; the byte model intentionally excludes
unmeasured serialization/TCP scheduling and authentication.

## Evidence provenance from first source-attested full run

- Workflow: https://github.com/definitely-stable/deltameter/actions/runs/37896624499
- Source commit: `a7fa51c3f2e9da2da9f7b32ccb019e40d26cd839`.
- Summary artifact ID `11600368648`, SHA256
  `6e2fd3c9dc074b12efa1d246ad64851ca3ab5eab9c2a49625bbca87e96729ca2`.
- Certificate artifact ID `11600950629`, SHA256
  `650da19865d9f6d892934c6ff576a049d069ace43877ad81ab90d517599f37b5`.
- Workers 1/2/3/4/5:
  `11600657555`, `11601010284`, `11600501864`,
  `11599904791`, `11600572782`.
- All source pairs are deterministic PUBLIC fixtures; 0 observed
  false-SAFE in any finite test set does NOT prove delta=10^-6.
  The preexisting exact ideal oracle certificate is the basis
  for its narrow statistical statement.

## Decision / remaining work

**B2-B0 physical maintained-state and byte-accounting foundation:
ACCEPT as scoped research evidence.**

- B2A11 T64 d<=48, two independent owners:
  a sound 640B modeled one-round guard with a large byte
  reduction vs full direct canonical lists at N256/4096/65536.
- G1B j1 at T64,d48 is much weaker in observed utility and
  must fall back if an exact yes/no decision is required.
- Near-T and above-T d=57/65 and UNKNOWN: fully resolved
  B2A exchange loses to direct exact; **STOP_RESOLVED_NEAR_T**.
- J52 is a physical size/update control only in this B0
  experiment; its full certified upper-bound semantics are not
  compared until the valid Q32 lookup/calibration is loaded.
- The 64B header/session/key binding and wire timing are
  modeled. NO TCP, no measured physical RTT, no practical WAN
  security, no lifecycle/epoch/replay proof.
- A future **B2-B1** must preregister and run real
  two-independent-sender loopback TCP measurements,
  source-auth/provenance checks and five-worker p95 comparisons
  before any SYSTEM_PRODUCT_GO. Threat #86 and product #69
  still prohibit public exposure.

No frozen public API or Snapshot v1 was modified.
