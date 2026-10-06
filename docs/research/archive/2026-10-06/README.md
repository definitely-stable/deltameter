# 2026-10-06 source register

This register records the latest research inputs that were incorporated into the canonical documents.

These are **source snapshots**. When they conflict with current decisions, [../../DECISIONS.md](../../DECISIONS.md), [../../STRICT-PARITY-POST-M3.md](../../STRICT-PARITY-POST-M3.md), and [../../FOUNDATION.md](../../FOUNDATION.md) win.

## Inputs

### 1. Latest Deep Research Q1 run

Topic: finite-sample GF(2)-PCSA upper-capacity bounds.

Important content:

- explored moment bounds, Cantelli/Chebyshev and independent-group amplification;
- proposed practical profile sizes;
- reinforced the distinction between point-estimation asymptotics and strict capacity.

Audit result:

- useful exploratory work;
- strict profiles are **not accepted**, because the run substituted asymptotic variance/RSE into finite-sample inequalities.

Snapshot: [q1-deep-research-run.md](q1-deep-research-run.md).

### 2. “Доказательное DeltaMeter: Архитектурный план для v0 через строгую верификацию и оптимизацию CI”

Important content:

- finite-sample Parity remains unsolved;
- de-Poissonization should be treated explicitly;
- published F-PCSA and `g(v)=1` set-specialized parity must be separated;
- proposes Minisketch for exact small differences;
- proposes two-level CI.

Snapshot: [proof-deltameter-v0-plan.md](proof-deltameter-v0-plan.md).

### 3. “From Theory to Practice: Resolving Implementation Barriers for the DeltaMeter v1 Backend”

Important content:

- Energy-first backend recommendation;
- Parity secondary pending Q1;
- exact-small-d deferred;
- minimal non-cryptographic randomness direction;
- PBT + deterministic CI.

Snapshot: [theory-to-practice-v1.md](theory-to-practice-v1.md).

### 4. “Минимальный контракт случайности v0”

Important content:

- public/non-secret seeds under oblivious input;
- pairwise/universal hashing discussion;
- persistence/reproducibility requirements;
- explicit warning against language-default hash identity for persisted data.

Snapshot: [minimal-randomness-contract-v0.md](minimal-randomness-contract-v0.md).

### 5. “Строгие верхние границы для GF(2)-PCSA: От теоретической возможности к практической реализации без асимптотической нормальности”

Important content:

- surveys concentration/test-inversion approaches;
- highlights dependence, finite-m bias and truncation;
- considers median wrappers and alternative estimators;
- ends with a GO recommendation.

Audit result:

- research directions retained;
- GO conclusion not accepted because no completed finite-sample theorem supports it.

Snapshot: [strict-gf2-pcsa-upper-bounds.md](strict-gf2-pcsa-upper-bounds.md).

### 6. “От Эвристики к Доказательству: Теоретическое Обоснование Доверительных Границ для GF(2)-F-PCSA при Конечных Параметрах”

Important content:

- strongly rejects asymptotic RSE, empirical tails and median amplification as substitutes for a finite-sample theorem;
- prioritizes exact small-parameter distributions, truncation, de-Poissonization, test inversion, and interval width/power;
- highlights de-Poissonization with explicit remainder control as a promising direction.

Audit result:

- the high-level proof agenda is useful;
- the report often analyzes classical PCSA instead of the exact published F-PCSA state/statistic;
- notation overloads m and d;
- the GF(2^V) interpretation is speculative;
- truncated multivariate-normal statistics are not a direct model for finite-level F-PCSA censoring;
- two-sided exact-test machinery is more than the one-sided capacity contract requires;
- binary-search inversion is unsafe until monotonicity is proved.

Snapshot: [heuristic-to-proof-fpcsa.md](heuristic-to-proof-fpcsa.md).

### 7. Gemini post-M3 strict-Parity report

Important content:

- NO-GO for strict published Parity;
- exact Poissonized independence;
- finite-J truncation analysis;
- Simple Set Sketching hybrid recommendation.

Audit result:

- NO-GO and Poissonized decomposition retained;
- whole-sketch truncation formula corrected;
- full-statistic monotonicity inference rejected;
- Simple Set Sketching kept as a future candidate rather than immediate Coverage::Exact.

Snapshot: [gemini-post-m3-strict-parity.md](gemini-post-m3-strict-parity.md).

### 8. Qwen post-M3 strict-Parity report

Important content:

- exact full-state Walsh law;
- exact one-level PGF/moments;
- W_i non-sufficiency;
- exact Poissonized law;
- narrow ParityLevelCounts proposal.

Audit result:

- this report provides most of the accepted post-M3 mathematical decomposition;
- zero-observation floor corrected from 10/20/30 to the exact theorem-implied 9/19/29;
- alternative-backend decisions remain conditional.

Snapshot: [qwen-post-m3-strict-parity.md](qwen-post-m3-strict-parity.md).

### 9. DeepSeek post-M3 strict-Parity report

Important content:

- NO-GO for published W_i;
- hybrid Energy/IBLT/Minisketch recommendation;
- cancellation/monotonicity claims;
- adaptive-attack context.

Audit result:

- NO-GO retained;
- generic cancellation is not accepted as a proof about the actual published scalar;
- mean monotonicity is not stochastic monotonicity;
- IBLT finite-sample-confidence claim requires primary-source verification;
- Energy memory numbers are inconsistent with the repository profile oracle;
- adaptive attacks remain outside the oblivious v0 theorem model.

Snapshot: [deepseek-post-m3-strict-parity.md](deepseek-post-m3-strict-parity.md).

## Why the original PDFs are not normative

The reports contain mutually contradictory implementation recommendations and several claims at different evidence levels.

The repository therefore stores their findings as structured Markdown research snapshots and keeps a separate reconciliation layer.

See [../../STRICT-PARITY-POST-M3.md](../../STRICT-PARITY-POST-M3.md) and [../../NEW-INPUTS-AUDIT.md](../../NEW-INPUTS-AUDIT.md).
