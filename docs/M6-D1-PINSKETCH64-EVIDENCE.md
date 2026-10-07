# M6-D1 — Pure-Rust PinSketch64 lab evidence

Status: **ACCEPT as a private correctness/performance reference; production backend remains NO-GO.**  
Issue: #23. Parent: #19 / #14. PR: #25.  
Evidence date: 2026-10-07.

## Decision

Retain the pure-Rust PinSketch64 implementation as a private lab reference.

Do not expose it as a public ExactSmallDelta backend yet.

The reference demonstrates all of the following:

- exact recovery for the audited small capacities when the true symmetric difference is within the explicit decode limit;
- exact full-u64 mapping without hashing or truncation;
- zero-key support through a separate XOR-composable bit;
- linear XOR sketch composition;
- very small serialized state;
- practical build/update cost;
- a substantial but reference-grade decode cost at d=4..8;
- catastrophic over-capacity false-success when stored capacity is also used as the decode limit;
- strong empirical rejection after reserving one extra 64-bit syndrome as a guard.

The final point changes the lab design: **stored syndrome capacity and maximum decoded elements are separate quantities.**

## Measured revision

Final code/evidence run:

~~~text
m6d1-lab #15 / 37523031317
head  7b610a1e99e49a08e8aaf923000b329ccfc6d3db
CPU   AMD EPYC 7763 64-Core Processor
Rust  1.99.0 / LLVM 23.1.1
runs  3
source set size 8192
~~~

Artifact: `m6d1-lab-37523031317-1`.

The later workflow-only commit does not change the measured implementation, tests, benchmark or summarizer.

The artifact records SHA256 for:

- `examples/m6d1_bench.rs`;
- `examples/support/m6d_pinsketch64.rs`;
- `tests/m6d1_pinsketch64.rs`;
- `research/m6d1_summary.py`.

## Representation

The lab uses the same GF(2^64) polynomial representation already audited by Energy.

For stored syndrome capacity `c`:

- nonzero u64 key -> identical nonzero GF(2^64) element;
- key zero -> separate XOR-composable presence bit;
- state -> odd power sums S1, S3, ..., S(2c-1);
- merge -> XOR every syndrome and the zero bit.

This mapping has no identifier collision or truncation.

The lab byte envelope is private and has no compatibility promise.

## Decoder

The reference reconstructs even syndromes through Frobenius squaring, runs Berlekamp-Massey, reverses the locator recurrence into the root polynomial, and factors it using deterministic absolute-trace splitting.

The test oracle covers:

- existing GF64 multiplication vectors;
- field inverse identities;
- polynomial division/GCD;
- fixed tiny syndrome/byte vectors;
- zero/high-bit/u64::MAX keys;
- capacities 1/2/4/8;
- multiple deterministic root sets at every in-bound d;
- symmetric-difference merge;
- exact local-membership direction classification;
- malformed lab encoding.

All ordinary Rust CI gates pass on the measured implementation.

## Stored capacity versus decode limit

The initial lab used:

~~~text
stored_capacity = max_elements
~~~

That configuration has no spare syndrome for over-capacity detection.

Measured over-capacity inventory showed:

~~~text
unguarded trials        896
unguarded rejected      590
unguarded false success 306
~~~

Examples:

- max=1: all tested d=2/3/5 cases false-success;
- max=2: 34/64 false-success at d=3, 28/64 at d=4, 39/64 at d=7;
- max=4: smaller but still nonzero false-success;
- max=8: no false-success in the sampled vectors, but that is not a guarantee.

Therefore decode success cannot be treated as exact when there is no independently justified bound on d.

## Guarded design

The accepted lab layout uses:

~~~text
stored_capacity = max_elements + 1
decode_limit    = max_elements
~~~

The candidate is reconstructed using only the recoverable budget and then must reproduce **all** stored syndromes.

This mirrors the upstream Minisketch distinction between stored capacity and `max_elements`. At 64 field bits and max_elements <= 8, upstream `ComputeCapacity(..., fpbits=64)` also selects one extra field element.

DeltaMeter does **not** yet adopt the upstream false-positive probability as its own theorem contract. The lab uses exact u64 values without a separate hashed/randomized input model, and adversarial semantics still require a dedicated decision.

Final guarded inventory deliberately includes weights beyond the simple BCH minimum-distance exclusion region:

~~~text
guarded trials        896
guarded rejected      896
guarded false success 0
~~~

The tested weights include:

- max=1 / stored=2: d=2,3,5;
- max=2 / stored=3: d=3,4,7;
- max=4 / stored=5: d=5,6,8,11;
- max=8 / stored=9: d=9,10,16,19.

This is **EMPIRICAL RESULT**, not a 2^-64 proof.

## State size

The private lab envelope is 16 bytes plus 8 bytes per stored syndrome.

With one guard syndrome:

| max elements | stored syndromes | lab bytes |
| ---: | ---: | ---: |
| 1 | 2 | 32 |
| 2 | 3 | 40 |
| 4 | 5 | 56 |
| 8 | 9 | 88 |

The raw algebraic sketch itself is only 8*c bytes; the table includes the lab-only 16-byte envelope.

The direct comparison harness sends an 8192-element raw u64 set:

~~~text
4 + 8192*8 = 65,540 bytes
~~~

That is not an entropy-optimal exact-set encoding. For random full-width u64 values, the information-theoretic set representation floor is materially smaller. D1 therefore does not claim a product-level compression ratio.

## Build and transport-state cost

Median hosted results:

| max | stored | cold build ns/key | approx 8192-key build | encode state | decode state |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2 | 197.588 | 1.62 ms | 25.6 ns | 33.2 ns |
| 2 | 3 | 262.756 | 2.15 ms | 24.8 ns | 33.1 ns |
| 4 | 5 | 400.205 | 3.28 ms | 26.0 ns | 34.5 ns |
| 8 | 9 | 667.323 | 5.47 ms | 25.7 ns | 33.4 ns |

Merge of two already-maintained sketches is about 17–20 ns in this harness.

These are diagnostic hosted-runner numbers, not product SLAs.

## Decode cost

Median guarded decode:

| max | d | decode |
| ---: | ---: | ---: |
| 1 | 1 | 8.86 us |
| 2 | 2 | 0.595 ms |
| 4 | 4 | 4.74 ms |
| 8 | 8 | 15.36 ms |

The local exact merge-scan over two 8192-key sorted sets is about 5.1 us.

Therefore the reference decoder is not competitive with co-resident exact sets on CPU once d grows past the smallest cases.

That is expected: the lab uses generic safe-Rust field multiplication and deterministic trace factorization, while optimized Minisketch uses specialized finite-field/root-finding implementations.

## Communication/CPU interpretation

For maintained remote state, the trade is different:

- max=8 guarded lab state: 88 bytes;
- raw exact-set payload in this harness: 65,540 bytes;
- guarded decode at d=8: about 15.36 ms.

Ignoring RTT and all other protocol costs, the byte savings offset that extra local decode time at roughly tens of Mbit/s scale for the max=8 case. Smaller d has a much lower CPU penalty.

This is only a diagnostic crossover calculation. It is not a production network benchmark.

## Why production remains NO-GO

1. The implementation is intentionally reference-grade, not optimized.
2. The guarded 0/896 result is empirical, not an adversarial false-positive theorem.
3. Rechecking the same syndromes is not independent final verification.
4. A generic Energy-first network workflow was already rejected by M6-A.
5. Direct exact transfer in D1 is not entropy-optimal.
6. Capacities above the audited small range are untested.
7. The lab encoding is not a public compatibility format.

Do not add:

- public ExactSmallDelta APIs;
- Coverage::Exact;
- snapshot-v1 tags;
- FFI/libminisketch;
- a claim that one guard syndrome proves 2^-64 failure for arbitrary DeltaMeter u64 inputs.

## D1 verdict

**ACCEPT the private PinSketch64 reference.**

It is now a trustworthy local oracle/reference for subsequent reconciliation research.

The original unguarded configuration is rejected.

The guarded `stored_capacity > max_elements` distinction becomes mandatory in future PinSketch experiments.

## Next decision

The strongest follow-up is **not** Energy + PinSketch.

Evaluate a guarded incremental prefix protocol:

~~~text
send k + guard odd syndromes
-> try decode up to k
-> on failure send additional syndrome words
-> retry with a larger explicit decode limit
~~~

Advantages:

- no 37–378 KB Energy snapshot just to estimate d;
- every additional syndrome is only 8 bytes at GF(2^64);
- prefixes are naturally nested/rate-compatible;
- source can precompute a maximum sketch or retain source data;
- failure/guard semantics stay explicit.

The next slice must measure repeated-decode CPU, incremental extension cost, retry/RTT tradeoffs and compare against direct exact transfer and Rateless IBLT. It must not silently become a public protocol.
