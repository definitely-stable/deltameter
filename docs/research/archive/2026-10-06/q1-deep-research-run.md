# Snapshot — latest Deep Research Q1 run

Date: 2026-10-06  
Topic: finite-sample GF(2)-PCSA capacity.

## Reported approach

The run attempted to turn the published GF(2)-PCSA asymptotic RSE

[
1.638/sqrt m
]

into one-sided capacity profiles by:

1. treating the implied variance as approximately (2.683d^2/m);
2. applying Chebyshev/Cantelli-style bounds;
3. grouping independent copies;
4. using a median/binomial tail to drive the failure probability down to (10^{-3},10^{-6},10^{-9}).

It produced illustrative configurations in the thousands-to-tens-of-thousands of replicas/registers.

## Useful output

- reinforced the value of independent-copy amplification;
- reinforced that (d=0) and small (d) need separate attention;
- produced an engineering comparison point for memory cost;
- kept Energy as a fallback.

## Audit status

**Exploratory, not proof-grade.**

The decisive gap is that the published RSE is asymptotic. Substituting that asymptotic variance into Chebyshev or Cantelli does not create a finite-sample theorem.

Therefore the numeric profiles from this run must not be committed as `Coverage::Proven` configurations.

See [../../Q1-FINITE-SAMPLE-PARITY.md](../../Q1-FINITE-SAMPLE-PARITY.md).
