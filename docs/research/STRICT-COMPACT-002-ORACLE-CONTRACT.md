# STRICT-COMPACT-002 — oracle contract analysis

Issue: #63. Parent: #59.

Status: **ORACLE_GAP_CONFIRMED for the current deterministic implementation.**

## The mathematical probability space

The STRICT-COMPACT theorem is derived for the same idealized sampling model used by
the F-PCSA construction:

- each active difference element selects a row uniformly;
- selects a geometric level with the prescribed law;
- selects the GF(2) coefficient independently;
- the resulting element samples are independent in the probability model used by
  the finite-sample tail argument.

The fixed-d theorem is over this sampling randomness.

## What the Rust code actually does

`PublishedFpcsaOracle` is deliberately documented as a deterministic
pseudo-random engineering reproduction.

For a u64 key it derives row, level and coefficient from `mix64` with fixed
u64 seeds and domain constants. `mix64` is a deterministic integer mixer, not a
cryptographic PRF and not a proven k-wise independent hash family matching the
concentration proof.

Therefore:

- for a fixed oracle and fixed key set there is no remaining sampling randomness;
- an adversarial key chooser is not covered by the ideal-oracle probability theorem;
- changing only the inference formula cannot turn the existing deterministic oracle
  into an unconditional `Coverage::Proven` implementation.

## Decision for STRICT-COMPACT-002

Do not modify the oracle in the range-certificate slice.

If the continuous range certificate passes, record:

`GO_RESEARCH_ONLY_ORACLE_GAP`.

That means:

- the state/inference construction has a finite-sample theorem under the ideal
  sampling model;
- the exact Q32 implementation is conservatively certified;
- the width range is certified;
- but the current deterministic pseudo-oracle remains an engineering reproduction,
  not the probability source required for a public strict coverage claim.

## Production routes considered

### A. Keyed computational PRF contract

Use a fresh unpredictable per-meter key with a real keyed cryptographic PRF/hash
and derive row/level/coefficient with domain separation.

The resulting claim would be computational:

> assuming the keyed function is indistinguishable from a random function to the
> relevant adversary, the ideal-oracle finite-sample theorem applies up to the PRF
> security assumption.

This is materially different from an information-theoretic `Coverage::Proven`
claim and must be named accordingly.

It also introduces key generation/ownership/snapshot compatibility questions and
likely a new dependency or primitive. It requires a separate issue.

### B. Proven finite-independence hash family

Replace the oracle with a concrete universal/k-wise independent family and prove
that the amount of independence is sufficient for the exact tail construction.

This would give a stronger non-cryptographic statement, but the current
de-Poissonization/Chernoff proof uses a sampling model stronger than we have shown
for any small-k family. Do not assume pairwise or 4-wise independence is enough.

This is a mathematics/research task, not an implementation detail.

### C. Research-only ideal-oracle contract

Keep the existing deterministic oracle and expose no new strict public coverage
surface. Use the result as proof that the *sketch family/state representation* has
a compact strict construction under the ideal model.

This is the selected outcome for #63 unless a separate oracle project closes A or B.

## Public wording rule

Until an oracle follow-up is complete, none of the following may be introduced:

- `Coverage::Proven` for the current Parity implementation;
- documentation claiming unconditional `1-delta` coverage for arbitrary u64 keys;
- snapshot/profile names containing "strict" without an explicit ideal-oracle or
  computational assumption;
- claims that deterministic hosted fixtures empirically validate the theorem for
  adversarial inputs.

The correct current statement is narrower:

> The 32 KiB GF(2)-F-PCSA state plus certified lookup admits a finite-sample
> one-sided upper-bound theorem in the ideal independent-sampling model; the current
> deterministic pseudo-oracle does not yet realize that model as a public guarantee.
