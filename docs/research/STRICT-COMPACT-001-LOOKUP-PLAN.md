# STRICT-COMPACT-001 — conservative lookup prototype

Issue: #59

Status: research implementation slice after FOUNDATION_GO. No public API change.

## Goal

Turn the theorem-derived 32 KiB candidate into a realistic inference hot path
without evaluating exp/log/KL during every estimate.

Profile frozen for this slice:

- rows m = 4096;
- stored levels J = 64;
- packed primary state = 32,768 bytes;
- delta = 1e-6;
- equal Bonferroni alpha_j = delta / 64;
- fixed-d tail bound:
  F_d(s) <= min(1, 2 exp(-m D(s/m || p_j(d)))).

## Scale reduction

For level j,

    p_j(d) = (1-exp(-d/(m*2^j)))/2.

Let y=d/2^j. For fixed observed count s, the KL crossing depends on y but not on
j. Therefore only one normalized threshold is needed per possible s.

For s<m/2, solve y_s such that

    2 exp(-m D(s/m || (1-exp(-y_s/m))/2)) = alpha.

For s where even p->1/2 cannot make the tail smaller than alpha, the level is
uninformative and gets an infinity sentinel.

Runtime then computes, for each level,

    U_j(s) <= ceil(2^j * y_s_upper)

using a conservatively rounded precomputed upper threshold.

The final bound is min_j U_j.

## Prototype representation

Generate Q32 upper thresholds:

    q_s = ceil(y_s_upper * 2^32).

For m=4096 only counts s=0..2047 require table slots. Initial analysis shows:

- 1853 finite thresholds;
- 195 infinity sentinels;
- every finite normalized threshold fits in u64 Q32.

The table is therefore 2048 u64 values = 16 KiB.

This table is static code/data, not per-meter state.

## Generator

research/strict_compact_lookup_q32.py:

- Python standard library only;
- binary64 bisection is used only to propose a nearby Q32 integer;
- 60-digit Decimal arithmetic independently checks the emitted inequality;
- integer repair moves upward until the candidate is conservative, then downward
  to the minimal Q32 value that still passes the high-precision check;
- fail if finite thresholds are not monotone;
- fail if any emitted Q32 value is not on the conservative side according to the
  high-precision evaluator;
- emit one-u64-per-line table plus a JSON provenance summary and SHA-256.

This is a prototype numerical generator, not yet the final proof certificate.
A public Proven API will require an independent conservative-rounding audit or
interval/rational certificate for every committed threshold.

## Rust lab

examples/strict_compact_lookup_lab.rs reuses the actual src/fpcsa.rs implementation
as a private example module and performs:

1. actual 4096x64 packed Parity state construction;
2. level-count extraction directly from the 4096 row-major u64 words;
3. Q32 lookup/shift/min inference using u128 intermediate arithmetic;
4. selected deterministic d fixtures;
5. hot-path timing with table generation/file parsing excluded.

No production library surface is changed.

## Evidence gates

Prototype PASS requires:

- generator emits exactly 2048 entries;
- table monotone up to the infinity region;
- every finite threshold fits Q32 u64;
- actual packed state is exactly 32,768 bytes;
- level counts are <=4096 and use no heap allocation;
- lookup bound uses integer arithmetic only;
- selected deterministic fixtures produce finite bounds where expected;
- research/Rust CI remains green.

Performance is descriptive at this slice; no hard nanosecond gate until exact-head
hosted evidence exists.

## Next after PASS

1. declared d-range closure for the 32 KiB profile;
2. independent table-certification method;
3. estimate-path CPU/allocation evidence on GitHub-hosted runners;
4. deterministic pseudo-oracle vs ideal-oracle contract decision;
5. only then propose a production/private Rust strict-bound API.

Snapshot v1, ParityDeltaMeter::estimate(), Coverage::Asymptotic and public profiles
remain unchanged.
