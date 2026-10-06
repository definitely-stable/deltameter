# Audit of the attached F2-centric research report

Source: the attached report titled *“Доказуемый DeltaMeter: От теории F₂-оценки к верифицированному Rust-примитиву с односторонними гарантиями емкости симметрической разности”*.

The report is preserved as an important historical research input. It contains several correct derivations and useful references, but later analysis narrows some of its conclusions.

## 1. What remains valid

### F2 identity

For set difference represented as `x_i ∈ {-1,0,+1}`:

```text
|A △ B| = F2(x)
```

The AMS discussion and the role of fourth-order sign independence are useful baselines.

### One-sided capacity conversion

If a two-sided relative guarantee holds, the report's conversion

```text
C = ceil(d_hat / (1 - epsilon))
```

is valid on the estimator's good event.

### Need to distinguish statistical guarantee from engineering speed

The report correctly treats update cost, memory, confidence and mergeability as separate concerns.

### Adaptive robustness is not a free property

The report correctly identifies adaptive adversaries as a materially harder model than ordinary oblivious streaming.

### Formal verification can be useful later

The Verus/Alerus discussion is relevant as a long-term direction, especially if probability-budget code becomes complex.

## 2. Corrections and narrowing

### Correction A — F2 is not the only central model

The report frames DeltaMeter mainly as F2 estimation.

For true sets:

```text
z = A XOR B in GF(2)^N
|A △ B| = ||z||_0
```

This GF(2) Hamming/L0 formulation is strictly more specialized and changes the algorithmic search space.

Current decision: set-only DeltaMeter is modeled primarily as GF(2)-L0, with F2 retained as a valid reference path.

### Correction B — generic F2 lower bounds do not prove DeltaMeter optimality

The report treats broad F2 lower bounds as if they directly define the DeltaMeter frontier.

They apply to a larger class of vectors and streams than the Boolean/set-only promise.

Current decision: no “optimal DeltaMeter” claim is made from generic F2 lower bounds.

### Correction C — SSE vs CountSketch result was overgeneralized

The report recommends sparse sign embeddings and excludes CountSketch based on general oblivious embedding comparisons.

That does not directly settle the specialized energy estimator:

```text
T = sum_b c_b^2
```

for set differences.

Current decision: CountSketch-like Energy remains the proof-grade baseline because its specialized mean/variance are simple and explicit. Sparse JL/SSE remains a baseline, not the default.

### Correction D — oblivious input does not imply secret seed

The report states that a secret seed is required for the oblivious-input contract.

In the classical oblivious model, the input is fixed independently of the random seed. Public seed does not by itself invalidate the theorem.

Secret/post-commitment randomness is relevant to a stronger chosen-input-after-seed model.

Current decision: no secret-key infrastructure in v0.

### Correction E — asymptotic RSE is not strict finite-sample coverage

The report discusses high confidence mainly through generic median amplification and modern sequential inference.

For Parity/F-PCSA, the important unresolved point is narrower: obtain a useful finite-m, finite-N, all-d one-sided tail for the actual construction.

Current decision: Parity cannot expose strict `1e-6`/`1e-9` capacity until that exists.

### Correction F — SIMD should not be an architectural prerequisite

The report strongly pushes SIMD as mandatory.

Current decision: scalar stable Rust first, then profile. Compiler auto-vectorization or stable `std::arch` is added only if measurements justify it. Nightly-only APIs are not a baseline requirement.

### Correction G — formal verification is not a v0 gate

The report proposes Verus/Alerus as a major implementation feature.

For one developer, the initial ROI is better from:

- theorem documents;
- executable parameter generators;
- deterministic reference vectors;
- exact numerical oracles;
- ordinary Rust tests.

Formal verification remains post-v1 unless a concrete correctness risk justifies it.

## 3. What the report contributed to the current design

Despite the corrections, the report directly motivated several retained principles:

- keep the one-sided capacity contract explicit;
- distinguish point estimate from coverage;
- document adversary assumptions;
- keep linear/composable structure;
- use executable research instead of prose-only claims;
- do not rely on Monte Carlo as proof of very small failure probabilities.

## 4. Historical status

The attached report is **not deleted or treated as “wrong”**.

Its status is:

```text
historical research input
+ useful derivations and references
+ several superseded architectural conclusions
```

The current authority is [FOUNDATION.md](FOUNDATION.md) plus [DECISIONS.md](DECISIONS.md).
