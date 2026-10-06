# Historical research snapshot — attached F2-centric report

Date imported: 2026-10-06.

Source title:

> Доказуемый DeltaMeter: От теории F₂-оценки к верифицированному Rust-примитиву с односторонними гарантиями емкости симметрической разности

This file preserves the report's structure and major conclusions in repository-readable form. It is historical evidence, not the current normative design.

For corrections, see [../SOURCE-AUDIT.md](../SOURCE-AUDIT.md).

## Report thesis

The report starts from

[
d=|A\triangle B|=F_2(x)
]

for (x_i\in\{-1,0,+1\}), then develops DeltaMeter as a linear F2 sketch with one-sided capacity guarantees.

Its main proposed direction is:

```text
Sparse Sign Embedding
+ sparse updates
+ SIMD
+ median-of-means / modern confidence methods
+ explicit adversary contract
+ optional formal verification
```

## Section 1 — F2 theory and lower bounds

The report reviews AMS:

[
Z=\sum_i r_i x_i,
\qquad
Z^2
]

as an unbiased estimator of F2.

With fourth-order sign independence it uses the standard bounded-variance reasoning and median-of-means amplification.

The report then argues for a memory floor of roughly

[
\Omega(\varepsilon^{-2}\log(1/\delta))
]

and treats this as the effective DeltaMeter lower-bound target.

Later research narrows this claim: it is relevant to generic F2, but is not automatically tight for the GF(2) set-only Hamming-weight problem.

## Section 2 — One-sided capacity

The report highlights the useful conversion

[
U=
\left\lceil
\widehat d/(1-\varepsilon)
\right\rceil.
]

On a valid two-sided ((1\pm\varepsilon)) event this is a safe upper capacity.

For (arepsilon=0.1), the successful-event worst multiplicative overprovision is about 1.2222.

This remains part of the current design.

The report also discusses median-of-means, empirical Bernstein, Catoni-style robust means and anytime-valid inference as possible confidence machinery.

Current narrowing: the v0 implementation uses the simplest finite-sample path first; advanced sequential inference is not a bootstrap dependency.

## Section 3 — Fast projections and SIMD

The report argues for Sparse Sign Embeddings over CountSketch, based on general embedding comparisons, and recommends sparse updates plus SIMD.

Current narrowing:

- general SSE-vs-CountSketch embedding evidence does not directly settle the specialized Energy estimator;
- scalar stable Rust comes first;
- SIMD is an optimization decision after profiling.

## Section 4 — Practical comparisons

The report compares:

- SSE;
- CountSketch;
- BJKST/distinct counting;
- Minisketch;
- related sketch systems.

It correctly identifies Minisketch as a reconciliation decoder rather than a difference-size estimator, and therefore as a consumer that could benefit from a DeltaMeter-style capacity oracle.

The conclusion “exclude CountSketch” is superseded by the specialized Energy analysis.

## Section 5 — Adversarial robustness

The report distinguishes oblivious from adaptive inputs and recommends deferring strong adaptive robustness.

That separation remains correct.

The report additionally requires a secret seed even for the oblivious-input contract. That is superseded: secret/post-commitment randomness belongs to a stronger chosen-input-after-seed model, not the baseline oblivious theorem.

## Section 6 — Rust API and formal verification

The report proposes:

- insert/remove;
- estimate;
- upper_bound;
- Minisketch capacity helper;
- linear add/subtract;
- Verus/Alerus probability-budget verification.

Current narrowing:

- set-only Parity should expose toggle/XOR semantics rather than silently reuse generic insert/remove;
- Minisketch coupling is deferred;
- formal verification is optional post-v1;
- strict capacity is exposed only where finite-sample coverage is justified.

## Historical value

The report remains useful for:

- AMS/F2 derivations;
- one-sided capacity framing;
- references;
- explicit confidence/adversary discussion;
- the idea of keeping guarantees visible in the API.

Its architecture has been simplified by later research.
