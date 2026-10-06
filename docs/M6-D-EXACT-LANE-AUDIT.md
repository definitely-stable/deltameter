# M6-D — ExactSmallDelta / hybrid source and failure-semantics audit

Status: **LAB-GO for a pure-Rust PinSketch64 reference only; NO-GO for a production exact backend in this slice.**  
Issue: #19. Parent: #14.  
Audit date: 2026-10-07.

## Executive decision

DeltaMeter should not adopt a production ExactSmallDelta backend yet.

The strongest small-d candidate is still PinSketch/Minisketch because it has the cleanest recovery contract:

- for a sketch whose actual element count is not above the configured capacity, the set is recoverable exactly;
- sketches combine linearly by XOR into a symmetric-difference sketch;
- storage is b*c bits for b-bit field elements and capacity c.

However, the production decision is blocked by four concrete issues:

1. Minisketch's C/C++ implementation would introduce FFI/unsafe/toolchain scope that the current crate deliberately does not have.
2. Minisketch cannot encode field element zero, and fields smaller than 64 bits truncate u64 keys.
3. If the true difference exceeds the decoder's admitted bound, Minisketch may still return a plausible but wrong decode rather than failure.
4. Symmetric-difference recovery alone does not prove final reconciliation and does not directly identify direction without local membership information.

The recommended next step is therefore a **private pure-Rust PinSketch64 reference lane**, not a public backend.

The reference lane must use the full 64-bit field for nonzero identifiers plus one explicit out-of-band zero-membership bit. This preserves the complete DeltaMeter u64 domain without hashing or truncation.

No public API, Coverage variant, snapshot revision or production dependency is authorized by this audit.

## Classification vocabulary

This audit keeps the following categories separate:

- **EXACT WITHIN CAPACITY** — successful mathematical recovery is deterministic when the stated capacity/domain preconditions hold.
- **BOUNDED FALSE SUCCESS** — over-capacity or malformed state can sometimes decode to a wrong candidate, with a separately parameterized probability bound.
- **HIGH PROBABILITY LISTING** — recovery succeeds with high probability under a load/randomness model.
- **CERTAINTY CLAIM** — the cited construction claims deterministic/guaranteed listing under its own stated communication conditions.
- **EMPIRICAL** — implementation/benchmark evidence only.
- **OPEN / NOT YET AUDITED** — not sufficient for a DeltaMeter contract.

## Pinned implementation/source inputs

### PinSketch / Minisketch

Primary implementation pin:

- bitcoin-core/minisketch
- commit: `a12f5de1c96e21d76f64bc647ae4f65e4fc1b636`
- commit date: 2026-09-16
- license: MIT
- implementation: C/C++ with C API

Primary implementation docs:

- https://github.com/bitcoin-core/minisketch
- https://github.com/bitcoin-core/minisketch/blob/master/include/minisketch.h
- https://github.com/bitcoin-core/minisketch/blob/master/doc/math.md
- https://github.com/bitcoin-core/minisketch/blob/master/doc/protocoltips.md

Historical PinSketch implementation/reference:

- https://www.cs.bu.edu/~reyzin/code/fuzzy.html

### Simple Set Sketching

Primary paper:

- Jakob Bæk Tejs Houen, Rasmus Pagh, Stefan Walzer
- "Simple Set Sketching"
- arXiv:2211.03683
- https://arxiv.org/abs/2211.03683

No implementation is adopted or treated as canonical by this audit.

### Classical IBLT

Primary paper:

- Michael T. Goodrich, Michael Mitzenmacher
- "Invertible Bloom Lookup Tables"
- arXiv:1101.2245
- https://arxiv.org/abs/1101.2245

### Rateless IBLT

Primary paper:

- Lei Yang, Yossi Gilad, Mohammad Alizadeh
- "Practical Rateless Set Reconciliation"
- ACM SIGCOMM 2024
- https://arxiv.org/abs/2402.02668

Official Go implementation pin:

- https://github.com/yangl1996/riblt
- commit: `4afa6bc06cb2237d9ea273a51d97a7e05b3f573b`

Rust comparison implementation pin:

- https://github.com/samWighton/rateless_iblt
- commit: `e71c26b48bbc8eac917e24b63d0c11f5cd076681`

The Rust implementation is comparison input, not an audited dependency.

### CertainSync

Primary paper:

- "CertainSync: Rateless Set Reconciliation with Certainty"
- arXiv:2504.08314
- https://arxiv.org/abs/2504.08314

Reference repository pin:

- https://github.com/toto9820/Rateless-Set-Reconciliation-with-Listing-Guarantees
- commit: `eb00ed0f2220cb7f6ceb762b3d4eb126662bf002`

### Self-Sizing IBLT

Primary paper:

- "IBLTs Measure Before They Decode: Self-Sizing Set Reconciliation from Pre-Peeling Counts"
- arXiv:2608.26537
- https://arxiv.org/abs/2608.26537

This is a very recent 2026 result. Treat it as research input, not a mature dependency.

### XYZ-Sketch

Primary paper:

- "Toward Optimal Time-Space Tradeoffs for Set Reconciliation"
- arXiv:2609.14442
- https://arxiv.org/abs/2609.14442

Reference repository pin:

- https://github.com/djwj233/XYZ-Sketch
- commit: `99bbe7ae42df1f4bfff92f48bcd78ac1e4e6abaf`

This is a September 2026 result and remains too fresh for production adoption without independent validation.

## Candidate 1 — PinSketch / Minisketch

### Recovery contract

**EXACT WITHIN CAPACITY.**

Minisketch documents that for b-bit elements and capacity c, when the number of elements is at most c, the entire set can be recovered.

Two compatible sketches XOR to a sketch of the symmetric difference.

This is the strongest small-d contract in the current candidate set.

### Serialized size

A b-bit sketch of capacity c uses:

~~~text
b * c bits
~~~

before any protocol framing or extra false-success protection.

For a direct 64-bit DeltaMeter identifier domain:

~~~text
64 * c bits = 8 * c bytes
~~~

plus an explicit zero-membership bit/field in the proposed DeltaMeter mapping.

### Over-capacity behavior

**BOUNDED FALSE SUCCESS, not fail-certain.**

The Minisketch API documentation explicitly states that when the sketch contains more elements than its intended decodable bound, or when it contains random bytes, decoding can still succeed and return nonsense.

`minisketch_compute_capacity(bits, max_elements, fpbits)` increases the stored capacity so that sketches with more than `max_elements` have a bounded probability of incorrectly decoding instead of failing.

This means:

~~~text
decode returned elements
!=
proof that true d <= admitted d
!=
proof of final reconciliation
~~~

DeltaMeter must keep decoder false-success probability separate from any Energy estimation failure probability.

### u64 mapping

The C API has two behavior rules that matter directly:

1. an element whose low b bits are zero is a no-op;
2. if b < 64, high bits are dropped.

Therefore:

- b < 64 is not collision-free for DeltaMeter's public u64 domain;
- b = 64 preserves every nonzero u64 exactly;
- u64 zero still needs a separate representation.

Recommended exact-domain mapping:

~~~text
backend field value = key, for key != 0
zero_present        = set contains key 0
~~~

The zero flag composes by XOR across two source sketches.

This produces a bijective full-u64 domain representation without hashing.

Do not map zero to a nonzero field value by permutation unless the entire 2^64 domain remains provably bijective; there are only 2^64-1 nonzero field elements.

### Direction recovery

PinSketch/Minisketch recovers the symmetric difference, not an inherent +/- side for each element.

Direction can be recovered if the caller retains exact membership access to one source set:

~~~text
if recovered key is in local set:
    remote is missing key
else:
    local is missing key
~~~

This requires the source set or an exact membership index to remain available.

A probabilistic membership structure is not enough for an "exact direction" contract.

### Incremental capacity

Standard Minisketch is a fixed-capacity object.

The syndrome sequence is structurally nested, but the current public API does not provide a rateless protocol contract comparable to Rateless IBLT.

If a deployment did not maintain enough syndromes, increasing capacity later requires source data to rebuild/extend the sketch.

Do not assume a failed fixed-capacity decode can be extended without retained source state.

### Decode cost and adversarial inputs

Minisketch uses Berlekamp-Massey plus polynomial root finding.

Its implementation randomizes root-finding choices to make intentionally worst-case decode behavior harder to force.

Protocol guidance recommends:

- limit capacity / max decode count;
- arrange flow so a party decodes requested sketches rather than arbitrary unsolicited work.

A production exact lane therefore needs explicit CPU admission limits independently of byte limits.

### FFI and license

License is compatible: MIT.

The implementation boundary is not currently compatible with DeltaMeter's accepted engineering constraints:

- C/C++;
- FFI required from Rust;
- unsafe boundary required;
- external build/toolchain complexity.

Direct libminisketch adoption is therefore **NO-GO under current crate constraints**.

### PinSketch verdict

**LAB-GO for an independent pure-Rust reference.**

It has the best semantics for a small fixed-capacity exact lane, but production adoption needs an implementation that preserves the crate's Rust/unsafe/dependency policy and independent verification model.

## Candidate 2 — Simple Set Sketching

### Recovery contract

**HIGH PROBABILITY LISTING.**

The paper stores XOR aggregates in three independently mapped tables.

For load factor below approximately 0.81, it proves full recovery in linear time with high probability.

It avoids an explicit per-cell checksum by quotienting: some key bits are implicit in the selected cell and serve as an implicit consistency check.

The analysis specifically has to account for decoding errors caused by collisions, so this is not deterministic within-capacity recovery in the PinSketch sense.

### Strengths

- simple XOR-oriented data path;
- linear-time recovery;
- fewer explicit checksum fields than conventional IBLT;
- potentially attractive pure-Rust implementation surface.

### Risks for DeltaMeter

- probability model and table mapping become part of the correctness contract;
- no existing mature implementation is selected here as a canonical dependency;
- a successful peel is not automatically an unconditional exact result;
- full-u64 domain mapping and independent final verification still need explicit contracts;
- the construction is optimized around probabilistic recovery, while M6-D's main value proposition is a small-d exact lane.

### SSS verdict

**DEFER.**

It remains a credible pure-Rust candidate if PinSketch64 decoding complexity proves unacceptable, but it does not currently beat PinSketch on failure-semantics clarity for the exact-small target.

## Candidate 3 — Classical IBLT

### Recovery contract

**HIGH PROBABILITY LISTING.**

Classical IBLT can list contents with high probability when the number of entries is below its designed threshold.

Set reconciliation subtracts two tables and peels the difference.

Typical designs use:

- signed counts;
- key XOR/sums;
- value sums where applicable;
- a hash/check field to decide whether a cell is pure.

### Direction recovery

Unlike plain PinSketch, signed counts can naturally indicate which side a peeled item came from.

That is a meaningful advantage for reconciliation workflows.

### Failure semantics

Failure can occur from an unpeelable residual core.

With checksum/hash-based purity tests, a false pure-cell classification is also a separate probabilistic event.

Final success therefore still needs an independently defined verifier if DeltaMeter wants a no-false-success contract.

### Capacity

Conventional IBLT needs sizing against d.

Oversizing wastes bytes/memory; undersizing causes peel failure and usually requires another round/rebuild.

### IBLT verdict

**DEFER as the fixed-size exact-small lane.**

It has attractive implementation simplicity and direction information, but its recovery contract is probabilistic where PinSketch offers deterministic correctness inside the admitted capacity.

It remains important as the base of the rateless candidates.

## Candidate 4 — Rateless IBLT

### Recovery contract

**HIGH PROBABILITY / rateless.**

The SIGCOMM 2024 construction generates an incremental coded-symbol stream rather than requiring a fixed capacity in advance.

The official implementation describes the required coded-symbol count as linear in d with an asymptotic coefficient approaching about 1.35.

This directly attacks the "unknown d" problem that fixed Minisketch does not solve.

### Engineering strengths

- incremental transmission;
- no need to commit to one capacity before communication;
- low update/communication asymptotics;
- official Go implementation exists;
- independent Rust implementations exist.

### Engineering risks

- duplicate elements violate the set/peeling assumptions in the examined Rust implementation;
- source set iteration/state is still required by current implementations;
- element hashing introduces a collision budget if original elements are not already exact fixed-size identifiers;
- randomized peeling/listing semantics are not the same as PinSketch's within-capacity exactness;
- more protocol machinery is required than a local exact-small primitive.

### Rateless IBLT verdict

**DEFER for DeltaMeter production, retain as the strongest unknown-d reconciliation comparator.**

M6-A already found no generic public reconciliation workflow value for the current estimator-first protocol. Rateless IBLT would be a different product direction, not a small extension of Energy.

## Candidate 5 — Self-Sizing IBLT

### Research result

The August 2026 paper observes that pre-peeling IBLT cell counts can estimate d.

It derives exact mean/variance expressions and uses a chi-square confidence construction to size a second round after a first-round failure.

This is especially relevant to the question "do we need a separate DeltaMeter estimate before IBLT reconciliation?"

### DeltaMeter implication

A future IBLT-based product may be able to obtain its own sizing signal from the IBLT state it already transmits.

That weakens the architectural case for an Energy+IBLT hybrid whose only purpose is capacity estimation.

The paper is too recent to replace the current product architecture without independent reproduction.

### Self-Sizing verdict

**WATCH / REPRODUCE BEFORE ADOPTION.**

It materially changes the hybrid design space but does not justify production code in M6-D.

## Candidate 6 — CertainSync

### Research result

The 2025 framework claims rateless set reconciliation with certainty, avoiding prior d parametrization for the constructions it analyzes.

The reference implementation is a research Python repository.

### DeltaMeter implication

CertainSync is important because it shows that "unknown d implies probabilistic false-success" is not universally unavoidable.

However:

- the construction/protocol is substantially broader than ExactSmallDelta;
- implementation maturity is research-grade;
- universe-reduction variants reintroduce their own tradeoffs;
- adopting it would effectively create a reconciliation subsystem rather than a compact exact lane.

### CertainSync verdict

**WATCH.**

Do not use a 2025 theoretical certainty claim as justification for an immediate production rewrite.

## Candidate 7 — XYZ-Sketch

### Research result

The September 2026 paper claims, for sufficiently large d:

- communication near (1+epsilon)*d elements;
- O(1) insertion time;
- O(d log V) decoding.

It targets a near-optimal time-space point and supplies a C++ reference repository.

### DeltaMeter implication

This is the most relevant new frontier result for a future general reconciliation product.

It is not the obvious implementation for ExactSmallDelta:

- the headline regime is sufficiently large d;
- the paper/repository are extremely recent;
- independent reproduction and failure-semantics review are still required;
- the current reference implementation expands the language/toolchain surface.

### XYZ verdict

**TRACK SEPARATELY; no production dependency yet.**

If DeltaMeter later evolves from estimator library to full reconciliation engine, XYZ-Sketch deserves a dedicated benchmark/reproduction issue rather than being folded into a small-d backend.

## Comparative decision matrix

| Candidate | In-bound recovery | Over-bound behavior | Unknown d | Direction | Full-u64 exact mapping | Current implementation fit |
| --- | --- | --- | --- | --- | --- | --- |
| PinSketch/Minisketch | deterministic exact | can false-success; bound separately | poor/fixed | requires local membership | yes with b=64 + zero bit | algorithm GO, C++ FFI NO-GO |
| Simple Set Sketching | high probability | probabilistic peel/errors | fixed table/load | requires additional side logic | requires explicit mapping | pure-Rust plausible, semantics weaker |
| classical IBLT | high probability | peel failure / hash-purity risk | fixed | natural signed side | needs exact key/hash contract | easy lab candidate |
| Rateless IBLT | high probability rateless | probabilistic decoding model | strong | supported by coded difference | depends on symbol/hash contract | separate protocol direction |
| Self-Sizing IBLT | estimator + IBLT | paper-specific probability model | two-round adaptive | IBLT-dependent | IBLT-dependent | too new |
| CertainSync | certainty claim under construction | construction-specific | strong/rateless | construction-specific | needs full audit | research maturity |
| XYZ-Sketch | near-optimal/high-performance claim | needs dedicated audit | fixed-support construction | needs dedicated audit | needs dedicated audit | too new for production |

## Why PinSketch64 wins the lab slot

The target is not "best set reconciliation algorithm in 2026."

The target is a minimal exact-small primitive that can be compared against:

- direct exact transfer;
- Energy alone;
- a future Energy+exact hybrid.

PinSketch64 best isolates that question.

Its useful property is deterministic recovery inside the declared difference bound.

The implementation experiment can therefore distinguish:

- capacity admission;
- decoder failure;
- over-capacity false-success;
- final verifier behavior;

without conflating those with probabilistic peel thresholds.

## Proposed DeltaMeter PinSketch64 reference contract

Private/lab only:

~~~text
domain              exact u64 set
field               GF(2^64), existing independently selected irreducible representation
nonzero mapping     key -> same 64-bit field element
zero mapping        one out-of-band XOR-composable presence bit
capacity            explicit small integer c
serialized payload  odd syndromes + zero bit + lab metadata
result              candidate symmetric-difference u64 values
direction           resolved only against exact local membership
verification        independent exact oracle in lab
~~~

Important: rechecking the same syndrome is not an independent verifier. An over-capacity alias can satisfy the same syndrome equations.

A later production verifier must be independent, for example a separately designed set digest/protocol contract. This audit does not select that verifier.

## Failure-budget decomposition for a future hybrid

If a future session uses Energy plus an exact sketch, keep these independent terms explicit:

~~~text
P(session failure)
<=
P(Energy bound failure)
+ P(exact decoder false success)
+ P(identifier mapping collision/loss)
+ P(final verifier failure)
+ P(transport/session failure event)
~~~

This inequality does not require event independence.

Do not hide exact-decoder failure inside Energy's Coverage::Proven number.

For the proposed PinSketch64 lab:

- mapping collision/loss is zero for nonzero u64 plus explicit zero bit;
- within admitted capacity decoder correctness is exact;
- over-capacity false-success must be measured/bounded separately;
- lab final verification is exact-oracle comparison.

## Product conclusion after M6-A

M6-A already rejected a generic public snapshot-admission workflow.

Therefore an exact lane must not be justified with "it makes M6-A work" after the fact.

A future exact primitive needs one of two independent justifications:

1. it is useful as a standalone library primitive for callers that already know/limit d; or
2. a named maintained-state workload demonstrates that hybrid recovery beats direct transfer end-to-end.

Until then, the exact lane remains private research.

## Phase 1 verdict

**NO-GO:**

- direct C/C++ Minisketch FFI dependency;
- public ExactSmallDelta API now;
- adding Coverage::Exact;
- changing snapshot v1;
- treating SSS/IBLT/XYZ successful decoding as unconditional exactness;
- using hashed u64 down-mapping without a separately budgeted collision contract.

**LAB-GO:**

Build a private pure-Rust PinSketch64 reference with:

- small explicit capacities only;
- exact nonzero u64 mapping;
- out-of-band zero bit;
- deterministic reference vectors;
- exact-oracle verification;
- over-capacity adversarial tests;
- no public API and no snapshot-v1 integration.

The next implementation slice must first prove the field/syndrome/root-recovery reference on tiny capacities, then benchmark it against direct exact transfer. Production adoption remains separately gated.
