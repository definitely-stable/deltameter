# M6-D3 — Incremental Berlekamp–Massey state reuse

Status: implementation/evidence complete in PR #29; **performance NO-GO for Candidate A**. Issue: #28. Parent: #19 / #14.

## Goal

Reduce D2's cumulative retry CPU without changing the D2 protocol.

D2 is frozen as the baseline:

```text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
cumulative payload   17  25  41  73 bytes
attempts / RTTs       1   2   3   4
```

Candidate A changes decoder computation only.

## Algorithmic basis

Berlekamp–Massey is iterative over the syndrome sequence. Its state after processing the first n terms is sufficient to continue with term n+1.

Retain:

- connection polynomial C;
- previous correction polynomial B;
- linear complexity L;
- shift m;
- previous nonzero discrepancy b;
- processed term count n;
- reconstructed full syndrome sequence used for discrepancy evaluation.

When a larger guarded prefix arrives, reconstruct only the newly available full syndrome terms and continue BM from the existing state.

## Sequence growth

For explicit decode limit k:

```text
nonzero_limit = k - zero_present
target full syndrome terms = 2 * nonzero_limit
```

Odd terms come from the D1 odd-syndrome prefix. Even terms are reconstructed by Frobenius:

```text
S_(2j) = S_j^2
```

The zero-presence bit is immutable across D2 prefix extension.

## Candidate A boundary

Candidate A may optimize only BM continuation.

Keep unchanged:

- deterministic trace root factorization;
- GF(2^64) representation/reduction polynomial;
- one guard syndrome per stage;
- exact-u64 mapping and out-of-band zero bit;
- candidate revalidation against every stored syndrome;
- D2 stage schedule, bytes and RTT accounting.

Do not add cached factorization, specialized root finding or public APIs in the same evidence run.

## Correctness gates

For each D2 stage and deterministic workload:

1. incremental reconstructed syndrome sequence equals fresh reconstruction;
2. incremental BM connection polynomial equals fresh BM;
3. incremental linear complexity equals fresh BM degree;
4. incremental candidate equals D1 fresh candidate whenever D1 succeeds;
5. every d<=8 success equals the exact symmetric-difference oracle;
6. d=9/10/16 remains reject in the frozen matrix;
7. zero/high-bit/u64::MAX cases remain valid;
8. extending with a changed historical syndrome fails closed.

## Measurement

Hosted release evidence reports separately:

- fresh syndrome reconstruction CPU;
- incremental syndrome append CPU;
- fresh BM CPU;
- incremental BM append CPU;
- root-factor CPU;
- full D1/D2 reference decode CPU;
- full D3 candidate decode CPU;
- cumulative retry CPU by d;
- D2 bytes/attempts/RTTs as invariant controls.

Use the same source cardinality 8192 and d matrix:

```text
d = 0,1,2,3,4,5,8,9,10,16
```

## Decision gate

Candidate A **ACCEPT** requires:

- all correctness gates pass;
- D2 payload/RTT values are byte-for-byte unchanged;
- cumulative decoder CPU is materially reduced for retry cases;
- no meaningful regression for one-stage d=0/1.

If BM reuse is a small fraction of total retry CPU, record **NO-GO Candidate A** and retain the implementation only as diagnostic evidence.

Candidate B, only after Candidate A verdict:

- cache reconstructed even syndromes and/or reuse root-finding work.

No root-finding specialization is allowed before Candidate A is measured.

## Boundaries

Private lab only; stable safe Rust; no unsafe; no FFI; no runtime dependency; no snapshot-v1 or Coverage changes; GitHub-hosted runners only.


## Completion record

Measured implementation head: `872c4ee818777fcc26d722e68de74a2aed2750c6`.

Hosted evidence: `m6d3-incremental-bm #5 / 37567202464`.

Canonical evidence: [M6-D3 incremental BM evidence](../../M6-D3-INCREMENTAL-BM-EVIDENCE.md).

Verdict:

- incremental BM correctness/equivalence: ACCEPT;
- Candidate A as a performance optimization: NO-GO;
- d=2..8 end-to-end change: approximately -0.3% to +0.6%;
- BM share at d=8: about 0.07% of candidate decoder CPU;
- cached-even-syndrome Candidate B: skipped by measurement because reconstruction is ~1.3–1.4 us versus ~100 ms root-factor/verification;
- next permitted target: isolated safe-Rust trace/root-factor specialization with D2 bytes/RTTs frozen.
