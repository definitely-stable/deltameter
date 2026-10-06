# Historical research snapshot — GF(2)/DeltaMeter synthesis

Date: 2026-10-06.

This snapshot captures the later synthesis that corrected the earlier F2-only framing.

## Executive conclusion

For true sets,

[
d=|A\triangle B|
=F_2(1_A-1_B)
=\|1_A+1_B\pmod2\|_0.
]

Therefore DeltaMeter is not merely “AMS in Rust”.

The strongest candidate product shape became:

```text
ParityDeltaMeter
    GF(2)-L0
    compact / fast

EnergyDeltaMeter
    F2 / CountSketch-like energy
    proof-friendly / strict reference
```

with Gaussian/chi-square used only as a golden statistical oracle.

## Key correction

Generic F2 lower bounds cannot be presented as the exact lower bound for a GF(2)-linear, Boolean set-difference estimator.

The narrow model must be audited separately.

## Parity candidate

GF(2)-PCSA/F-PCSA was identified as the strongest published fast building block because it naturally supports XOR composition and compact bit state.

Published asymptotic RSE:

[
\approx1.638/\sqrt m.
]

For (V=2^{32}), memory is approximately (4m) bytes.

The key unresolved issue is not point estimation but strict finite-sample high-confidence capacity.

## Energy candidate

For

[
T=\sum_b c_b^2,
]

the specialized set-difference variance is

[
\operatorname{Var}(T)=2d(d-1)/B.
]

This creates a transparent theorem path and a simple conservative high-confidence configuration through independent tables plus median amplification.

A useful systems observation is that (T) can be updated incrementally:

[
T' = T + 2c\Delta + \Delta^2.
]

## Capacity

Given a valid relative-error event,

[
C=\left\lceil\widehat d/(1-\varepsilon)\right\rceil
]

is safe on the same event.

For (arepsilon=0.1), good-event overprovision is at most about 22.2% before rounding.

## Why MinHash/HLL are not the core

MinHash estimates resemblance/Jaccard and becomes poorly conditioned for small symmetric difference.

HLL estimates large cardinalities; obtaining a small difference by subtracting large noisy values also becomes poorly conditioned when the sets are nearly equal.

Neither is the natural subtractable DeltaMeter primitive.

## Small-d observation

At (d=0,1,2,\ldots), relative error is awkward.

An exact-small-d lane may later be valuable, but the synthesis recommended postponing it until the primary backends are measured.

## Adversary conclusion

Three models were separated:

1. oblivious/public-seed input;
2. chosen-input after seed reveal, where secret/post-commitment randomness can matter;
3. adaptive-query attacks, where an ordinary fixed linear sketch should not claim robustness.

v0 chooses model 1.

## Implementation conclusion

The final pre-implementation recommendation was:

1. keep research executable;
2. implement Energy first;
3. reproduce Parity second;
4. only then decide whether strict Parity and exact-small-d are worth adding;
5. avoid speculative security, FFI, formal-verification and SIMD infrastructure.
