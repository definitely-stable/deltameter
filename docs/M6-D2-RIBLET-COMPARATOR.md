# M6-D2 — Rateless IBLT comparator boundary

Status: external comparator only; not a DeltaMeter dependency.

## Pin

Official implementation:

- repository: https://github.com/yangl1996/riblt
- commit: `4afa6bc06cb2237d9ea273a51d97a7e05b3f573b`
- paper: Practical Rateless Set Reconciliation, SIGCOMM 2024.

The pinned repository is unchanged at this commit as of the D2 start.

## What is comparable

The official implementation and README describe an infinite/incremental coded-symbol sequence. A receiver consumes a prefix and retries decoding until the symmetric difference is recovered.

That is the same high-level product question as D2:

~~~text
unknown d
+ progressive transmission
+ reuse prior transmitted information
+ retry decode
~~~

Rateless IBLT is therefore the strongest external protocol comparator for D2.

## What is not directly comparable

The paper/README headline is about **coded-symbol count**. The official Go implementation defines:

~~~text
CodedSymbol {
    Symbol
    Hash  uint64
    Count int64
}
~~~

The serialized byte size therefore depends on:

- the source Symbol representation;
- whether Hash and Count are transmitted as fixed-width fields;
- framing/serialization choices;
- any protocol compression.

Do not translate the asymptotic ~1.35d coded-symbol count into bytes by multiplying by eight merely because DeltaMeter keys are u64.

D2's PinSketch measurements are raw algebraic application payload bytes:

~~~text
one zero-presence byte
+ 8 bytes per GF(2^64) odd syndrome
~~~

The two quantities remain separate until an explicit RIBLT u64 wire model is frozen.

## Semantic differences

### PinSketch64 D2

- deterministic exact recovery when d is inside the explicit decode limit, subject to the D1 field/reference correctness;
- one extra syndrome is currently an empirical over-capacity guard, not an adopted adversarial theorem;
- symmetric difference is recovered without direction;
- direction requires exact local membership;
- staged schedule is discrete: 1 -> 2 -> 4 -> 8.

### Rateless IBLT

- rateless coded symbols are generated incrementally;
- the required coded-symbol count is probabilistic and approaches a linear coefficient near 1.35d in the paper/implementation description;
- decoder state distinguishes remote/local elements;
- source symbols require a non-homomorphic hash;
- implementation documentation assumes trusted workload for production use.

## D2 comparison rule

D2 may report:

- PinSketch cumulative bytes, RTTs and repeated decode CPU;
- Rateless IBLT asymptotic/empirical coded-symbol counts from the primary source;
- implementation symbol fields and qualitative failure/direction semantics.

D2 may not claim a byte winner against Rateless IBLT until a pinned u64 wire representation is implemented or reproduced.

No Go/C++/Rust RIBLT implementation is imported into the DeltaMeter crate in this slice.
