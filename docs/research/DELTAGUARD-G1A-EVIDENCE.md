# DeltaGuard G1-A — exact alpha reallocation certificate and paired utility

Related issues #85 (exact m/J/alpha frontier), #84 (DeltaGuard) and #86
(key/adaptive tokens). Frozen pre-measurement protocol:
[DELTAGUARD-G1A-PROTOCOL.md](DELTAGUARD-G1A-PROTOCOL.md).

## Research verdict

**G1A_CERT_PASS / G1A_ALGEBRA_PASS_EMPIRICAL_UTILITY_CANDIDATE**
on measurement source `9e173913e6f3e4e86f2ae7799dcdb718681e0a9f`,
[GitHub run 37877960458](https://github.com/definitely-stable/deltameter/actions/runs/37877960458),
five independent hosted workers, independently verified 900 source-attested
paired records (1800 logical old/new outcomes) and fail-closed summary PASS.

**NO PUBLIC API, NO G1B/M!=4096/ADAPTIVE-TOKENS GO.**
The table guarantee is in the SAME nonadaptive ideal-oracle model as G0,
not a theorem asserting concrete keyed-BLAKE3 adversarial security.
Exactly one fixed T/k profile must be committed before observing input.

## What was actually changed and proved

- Unchanged m=4096, J52 global oracle/token-to-(row,level,coefficient),
  512B/level packed parity, two independently maintained XOR-compatible
  source states, same G0 T-only level selection.
- Original alpha=delta/64, lookup SHA256
  `634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a`
  unchanged and independently re-certified on exact source.
- New table per k, `alpha_j=delta/k`, with `delta=1/1,000,000`.
  All 2048 observation-count entries, finite exact-rational p(q)
  lower enclosures, exact integer likelihood-ratio crossings and
  upper-limiting sentinel checks passed the **independent**
  `strict_compact_lookup_certify.py` target-parameterized
  verifier. Its proof uses original de-Poissonization factor 2
  and KL-Chernoff, neither fitted Monte Carlo nor an asymptotic tail.
- G1-A generator validates a known false/narrow Q32 entry, a misplaced
  sentinel and spurious finite value are rejected; original generator
  remains backward compatible. New relaxation is checked to be
  `new_q32[count] <= old_q32[count]` for all 2048 entries.
- The simple simultaneous/union budget is
  `sum_{selected j} delta/k=delta` for ONE ex-ante
  `(T,k,chosen-levels,table)` profile. It is NOT a guarantee for
  a later adversarially chosen T or k across many profiles using the
  same sketch/key. Different k means different lookup-table identity.

### Fresh table certificates

| k | alpha per selected level | finite counts | sentinel counts | SHA256 (ASCII normalized table) |
|---:|---|---:|---:|---|
| 1 | 1/1,000,000 | 1876 | 172 | `ae3a8988f20d56de96d38e028790c4713417ae8185d34138f4cb13bb727005ba` |
| 2 | 1/2,000,000 | 1872 | 176 | `9f4f86a6b435ea464be03eaddf2b0a4620f0f85a57a541ed3e7b018a70c001ea` |
| 4 | 1/4,000,000 | 1868 | 180 | `126786c91c96fd05ed2432dcc406d84ba7345c21471d546aed2b64518caffff7` |
| G0 | 1/64,000,000 | 1853 | 195 | `634ea5dd196e0e03fc74422a85d926351c7c81b42b0d9ba724fccfd45fb3cc7a` |

This is newly **certified mathematical calibration**, not newly discovered
mathematical theory. A fixed preselected k/T subset has independent of d
one-sided ideal-oracle coverage at least 1-delta under the accepted
per-level theorem. Observed power does not establish a lower-power bound
in a randomized-key distribution.

## Deterministic informativeness floor (exact)

The Q32 lookup is monotone in the observed odd-cell count.
Consequently min_possible U_j is U_j(0).
For this preregistered T∈{32,64,256,4096}, the original CENTER
selector clamps to level 1 and picks levels j=1..k.
The exact minimum among all possible k-level observation states
is, as reported in the complete certificate artifact:

| k | min possible U_selected | impossible T |
|---:|---:|---|
| 1 | 59 | 32 |
| 2 | 61 | 32 |
| 4 | 64 | 32 |

**For T=32, no possible bitmap state can make this certified
decision algorithm return SAFE.** This is an exact finite table
property, not a proof that no other statistical method can do so.
For T=64 the floor only says a SAFE decision is algebraically
possible; it does NOT say it is likely at d=16.

## Paired G0-vs-G1A useful output

Named matrix: five workers, three deterministic (public-key)
token-range seeds per worker, T={32,64,256,4096},
d/T={0,0.25,0.75,1,1.125}, k={1,2,4}.
Each cell has 15 observations. All old/new comparisons are for the
**identical two-owner XOR physical parity bits**.
No source state may be synthesized by cutting down an already
stored full J52 in this laboratory.

| T | true d/T | k | old SAFE /15 | new SAFE /15 |
|---:|---:|---:|---:|---:|
| 32 | 0.25 or 0.75 | 1,2,4 | 0 | 0 |
| 64 | 0.25 or 0.75 | 1,2,4 | 0 | 0 |
| 256 | 0.25 | 1,2,4 | 15 | 15 |
| 256 | 0.75 | 1,2,4 | 0 | 0 |
| 4096 | 0.25 | 1,2,4 | 15 | 15 |
| 4096 | 0.75 | 1 | 10 | **14** |
| 4096 | 0.75 | 2 or 4 | 11 | **14** |

At d/T=1 or 1.125 no false-safe was observed in the named
fixed-key/seed sample. This empirical fact is not a
one-in-a-million guarantee; the preceding exact one-sided
theorem, with its stated assumptions, supplies the certificate.

The separate source artifact contains all 900 paired raw records
plus SHA256 digests and machine-readable JSON. No CPU/RSS/
end-to-end network performance gate was executed in G1-A.

## Provenance

- CI run:
  [37877960458](https://github.com/definitely-stable/deltameter/actions/runs/37877960458)
- Exact certificate artifact ID 11592669764, ZIP SHA256:
  `fc2a637f76cd33ed978e895be8dacdf3ffb547cbc8ca5bc8bdff4f8c6fe56b38`.
- Complete 900-record summary artifact ID 11593088530, ZIP SHA256:
  `d90a5e417128fa7926ef06e9e724b9a509995ee9ab2a52bab0194dc680b136f9`.
- Worker artifacts 1/2/3/4/5 IDs:
  11592649883 / 11592888891 / 11592977341 /
  11592659931 / 11593655393.
- Worker ZIP SHA256, respectively:
  `04ab443d8a810da7b7e3b4b230041d18f9c37ea53c1085ac5950420a4c82eec9`,
  `b397f470266c8b73d9a9991fd28cbe3c049dc4ddbedca5284331d2e0e81f9046`,
  `662d4ff87b1c9d8cddb1106a103cf46176f176bd3bb8a9fd59b078069af0601d`,
  `23624a118f639de19e2bd958be3c0c2345430a8b1a4f3eadaa8703fbdb1438ed`,
  `53ec5dd57e2206493b7880928bc6d3155f8b4e71eea138ad07faf075c376cf4d`.

## Decision, failure boundaries and next research

1. G1-A per-k alpha allocation **ACCEPTED as a certified
   optional research calibration**, with named narrow empirical
   improvement T4096 and no measured T32/T64 usefulness.
2. No automatic promotion to generic J52 public estimate:
   the guarantee was budgeted to one prior-selected T/k
   profile, not simultaneous adaptive arbitrary k/T requests.
3. **G1-B recommended next:** replace conservative factor-two
   de-Poissonization+KL tail by exact fixed-d
   lazy Ehrenfest/odd-occupancy Markov law for SMALL T.
   Each token independently toggles one of the m cells at
   selected level j with probability `1/(m*2^(j+1))`.
   At odd count s the transition probabilities are
   `P(s→s+1)=(m-s)/(m*2^(j+1))`,
   `P(s→s-1)=s/(m*2^(j+1))`,
   and `P(s→s)=1-1/2^(j+1)`.
   First prove stochastic monotonicity of its finite kernel:
   `P(s→s+1)+P(s+1→s)=(m+1)/(m*2^(j+1))<=1`.
   Then a T-fixed downward event `S_j<=c` has worst
   probability for d>T at d=T+1; exact rational DP can certify
   the cutoff there without a full u64-wide Q32 inversion
   (for sufficiently small frozen T). This is an existing
   classic Ehrenfest chain, **not a novel theorem claim**.
   A different layout/level/hash/alpha must be separately certified.
4. **STOP** claiming small-T DeltaGuard product value
   if G1-B cannot beat the exact fixed-d worst-case one-sided
   cutoff at T32/64 with meaningful power; then #85 may
   investigate a new hashing/estimator family or record
   STOP_SMALL_T rather than indefinitely tweak alpha.
5. Continue #86 adaptive token, repeated key and
   profile/table identity threat-boundary audit before any public release.
