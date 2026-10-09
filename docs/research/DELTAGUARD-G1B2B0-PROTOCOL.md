# DeltaGuard G1-B2-B0 — frozen retained-state system foundation protocol

Issue [#92](https://github.com/definitely-stable/deltameter/issues/92).
Prerequisite merged G1-B2-A PR #91 / D55. Research-only.
Frozen **before B0 GitHub CI evidence**. B0 is NOT the final B2-B transport
product decision and cannot certify WAN/TCP. No public API or snapshot edits.

## Explicit comparison, fairness and scope

Two independently maintained owner sets A,B. Receiver is a THIRD party
which cannot inspect either source list and has a separately provisioned
*identical* oracle context/key; no malicious peers or adaptive input.
Compare four physically updated, two-owner candidates:

1. B2A near-full b11 m2047, physical bitmap 256B per owner, T64 cutoff c48.
2. B2A near-full b12 m4095, physical bitmap 512B per owner, T64 c52.
3. Old G1-B1 unit j1, physical 512B/owner, T64 c13; its DISTINCT key/context
   and cutoff are preserved. No new certificate or cherry-pick.
4. Old J52 LEVEL_MAJOR packed *complete* sketch, 26624B/owner, old
   keyed oracle and XOR algebra. For B0 J52 is a **cost/projection baseline**;
   do NOT claim its Q32-derived SAFE result without independently loading
   its exact certified lookup table.
Also compare direct exact using both *canonical sorted u64 source lists*,
and 24B one-sender scalar result ONLY as a topology-dependent lower bound.
Never call a scalar from one isolated owner a valid estimate of A△B.

Same test source pairs for all candidates: A=N tokens, B contains
N-floor(d/2) shared tokens and ceil(d/2) disjoint new tokens.
Thus |A△B|=d and |B|=N+(d mod 2). Unique keys are disjoint across
worker/scenario/repetition. Canonical u64 lists are actually maintained,
not just a predicted byte count. To represent retained reuse, each
completed next generation inserts the same new token into BOTH owners;
d remains fixed. Initial full hash ingestion is timed and separately
charged, incremental updates measured on actual retained state.
No rebuild per session, no inferred savings from direct exact storage
elimination. Distinct epoch/generation fields are logical model metadata,
not an already authenticated published wire protocol.

## Frozen physical matrix and statistics

N={256,4096,65536}, d={0,16,48,57,64,65}, session milestone
S={1,10,100}, three independent fixture repetitions per worker,
four physically maintained profiles, five GitHub-hosted workers.
5*3*6*3*3*4=**3,240 output records**, each with physical parity
correspondence, native build/update/query CPU nanoseconds, source
lengths, one-shot/retained state bytes, model application bytes,
exactness/fallback, metadata, and p50/p95 later from separately
reported raw wall clocks. All source/CI inputs fixed before timing.

One new retained token per BOTH owners per generation. For each
N/d/repeat baseline, build all 4 physical states from scratch once,
then advance to three session milestones in order with actual
incremental maintenance. Duplicated token XOR identity tested,
T/b/secret-key incompatible merges fail closed. The source truth
is independently calculated from two canonical sorted vectors
by two-pointer set difference and compared at EVERY sampled milestone,
including odd d and d>T. The J52 physical projection is verified
by XOR over actual 26624-byte states, but no uncertified J52
threshold decision is invented.

All updates use the existing distinct cryptographic oracle contexts
from merged B2A, merged B1 and J52. Public deterministic key fixture
does NOT prove keyed BLAKE3 as a secure PRF. The profile/secret/key
is not serialized or sent over the wire in the model.

## Frozen modeled wire formats — NOT sockets or signed frames

For B0 only, each peer sends `HEADER=64` model bytes
(protocol/version/profile/key ID/epoch/length reserved capacity; no
physical frame encoding/authentication is claimed).
- one-round sender-first B2A11: 2*(64+256)=640 application bytes.
- B2A12, B1-unit: 2*(64+512)=1152 bytes.
- J52 full: 2*(64+26624)=53376 bytes.
- direct exact: 2*64 + 8*(|A|+|B|) application bytes. The actual
  canonical arrays exist; hashes/lengths used to verify provenance.
- trusted source scalar: 24B in a topology where one party really has
  both lists; NOT usable by the two separated owners for A△B.

Guard-only contract: receiver returns SAFE_BELOW_T or UNKNOWN after
one sender-first round; UNKNOWN is terminal and must NOT be rebranded
"above". A bounded false-SAFE probability is ONLY inherited
from each specific existing exact certificate, fixed predeclared T/b.
Resolved contract: UNKNOWN triggers 24B next-request + full exact
source transfer, one additional feedback RTT. Count all bytes, CPU,
and fallback. SAFE may terminate; no \u201cEXACT_NOT_BELOW\u201d on UNKNOWN
without fallback.

Compare analytic ideal transfer lower bound
`one_rtt_ms + application_bytes*8/(Mbps*1000)` with RTT={0,10,50}ms,
bandwidth={1,10,100}Mbps. For fallback add exactly an additional RTT.
**This is a model, not a physical TCP/WAN experiment**;
sender concurrent handling is NOT accounted for (a serial link is
assumed for payload bytes) and will need separate B2-B1 validation.

## Gating and uncertainty

- B0_PHYSICAL_SOURCE_PASS: 3240 unique, source-attested records,
  canonical exact truth, no skipped/missing scenario, physical XOR,
  generation/session update, model equations independently checked,
  corrupt record/dup/profile rejected. Five independent hosted workers.
- B0_GUARD_SCOPED_MODEL_CANDIDATE: T64,d48, b11, session10;
  100% SAFE by exact c48, 640 modeled bytes vs direct exact
  >=10% less for N>=256. Must pass all workers. This is **not**
  a full product GO; no fitted alpha or observed zero false SAFE proof.
- RESOLVED_STOP_UNKNOWN: when UNKNOWN forced exact fallback, show
  cost >= direct exact. Report d>=57 near-T usefulness failures.
- Native CPU measurements must be reported, even if a byte win exists;
  never infer measured CPU cost from hashmap or byte formula.
  G0/J52 complete projection remains available, but no Q32 decision
  invention in this B0 stage.
- NO full SYSTEM_GO until separate B2-B1 **genuine loopback TCP**
  on a frozen subset, p95 latency gate, 5 workers, source/auth contract,
  and #86/#69 life cycle decisions. B0 results do not alter D55,
  Snapshot v1, published Energy coverage, or product release.

Raw evidence must record the exact checked-out HEAD SHA, Rust compiler
version, canonical cutoff TSV SHA256 and fixed run parameters.
Five-worker aggregate fails closed for any absent, duplicated, malformed
source/certificate, session/cost or truth cell. If observed B2A power
near T differs from single oracle expectations, label fixture-derived,
not proof-grade confidence.
