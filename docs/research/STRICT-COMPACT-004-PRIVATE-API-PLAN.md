# STRICT-COMPACT-004 — private computational coverage API plan

Issue: #67. Parent: #59. Depends on merged #66.

Status: **private/research contract prototype only**.

## Why this slice exists

STRICT-COMPACT now has three independent pieces of evidence:

1. exact/certified finite-sample upper-bound mathematics in the ideal independent
   sampling model;
2. a continuous q95 width certificate for every integer d in
   [4096, 2^64-1];
3. a practical keyed BLAKE3 oracle whose update cost passed all five hosted workers.

The remaining risk is API semantics. A public type that calls the result simply
`Coverage::Proven { failure_probability_upper_bound: 1e-6 }` would be wrong.

The implemented system has:

~~~text
ideal statistical failure <= 1e-6
+
computational PRF replacement assumption
~~~

and DeltaMeter does not claim a concrete numeric bound for the BLAKE3 PRF
distinguishing term.

## Frozen candidate

Do not generalize:

~~~text
token domain       u64
rows               4096
stored levels      64
primary state      32,768 B
lookup table       16,384 B static
ideal delta        1e-6
useful width       q95 U/d <= 1.5
useful d range     [4096, 2^64-1]
domain cardinality 2^64
oracle             keyed BLAKE3 v1 research contract
~~~

## Private estimate shape

The prototype must keep these fields conceptually separate:

~~~text
upper_bound: u128
ideal_statistical_failure_upper_bound: 1e-6
coverage_model: KeyedPrfConditional
useful_range_min: 4096
domain_cardinality: 2^64
q95_width_ratio_upper_bound: 1.5
~~~

The domain ceiling is `2^64`, not `u64::MAX`, and therefore the estimate's
upper bound is `u128`.

A saturated/uninformative sketch returns the explicit domain-cardinality ceiling
rather than wrapping/truncating it.

## Key contract

The caller supplies 32 secret bytes. The prototype:

- derives the frozen oracle subkey;
- retains enough secret config in memory to compare merge compatibility exactly;
- never prints secret material in Debug;
- exposes a non-secret 128-bit key/config fingerprint only as a diagnostic ID;
- documents that this fingerprint is not an authenticator;
- rejects XOR merge if the actual derived oracle keys differ.

Key rotation means rebuild.

No snapshot API is implemented. Snapshot v1 remains untouched and never carries
the secret.

## Coverage model wording

Private research descriptor:

~~~text
KeyedPrfConditional {
    primitive: "BLAKE3 keyed hash",
    context_version: 1,
    input_model: NonAdaptiveIndependentOfSecretKey
}
~~~

Interpretation:

- in the ideal random-function replacement experiment, the certified one-sided
  statistical failure is <=1e-6;
- the real keyed-BLAKE3 implementation additionally relies on a computational PRF
  assumption;
- no numeric total failure probability is claimed;
- adaptive token choice based on observations under the same key is outside the
  current transfer argument.

## Runtime path

The lab must use:

- the exact keyed oracle mapping accepted in STRICT-COMPACT-003;
- the actual 4096 u64 packed state;
- the exact generated/certified Q32 table;
- integer-only level-count extraction / lookup / scaling;
- no per-toggle allocation.

It must test:

1. deterministic keyed mapping;
2. double-toggle cancellation;
3. same-key XOR merge;
4. different-key merge rejection;
5. Debug redacts secret;
6. table digest and finite/sentinel counts match the certified artifact;
7. upper bound is never above 2^64 and can represent exactly 2^64;
8. saturation/uninformative state returns 2^64;
9. known packed-state fixtures produce expected bounds;
10. no public exports or snapshot changes.

## Decision

### GO_PRIVATE_API_FREEZE

The private contract is coherent, fail-closed and uses the already-certified
state/table/oracle without changing theorem wording.

### GO_REDESIGN_CONTRACT

The product remains viable but existing estimate/config vocabulary is misleading.

### STOP_COMPACT_PRODUCT

Honest key/coverage semantics make the API too burdensome for the product value.

## Boundary after this slice

Even GO_PRIVATE_API_FREEZE does **not** authorize a public API.

The next separate decision would need to choose whether DeltaMeter is willing to
ship:

- a runtime BLAKE3 dependency;
- explicit secret-key provisioning;
- a new computational-coverage vocabulary;
- and eventually a new snapshot/config lifecycle.

Only then should public API design begin.
