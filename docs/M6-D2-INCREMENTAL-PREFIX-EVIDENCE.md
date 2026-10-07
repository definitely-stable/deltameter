# M6-D2 — Guarded incremental PinSketch64 prefix evidence

Status: **LAB-GO for the private nested-prefix protocol; production/public protocol remains NO-GO.**  
Issue: #26. Parent: #19 / #14. PR: #27.  
Evidence date: 2026-10-07.

## Decision

Retain the guarded incremental odd-syndrome prefix as the next private reconciliation research primitive.

The experiment demonstrates that unknown small d does not require an Energy sizing snapshot:

- previously transmitted syndrome words are reused exactly;
- cumulative incremental payload equals the ideal fixed-known-k guarded payload;
- retries cost RTT and repeated decode CPU, not retransmission of prior syndrome words;
- every d <= 8 workload reaches the exact oracle;
- every sampled d > 8 workload is rejected;
- no false-success occurred in the frozen deterministic matrix.

This remains a lab result. The one-extra-syndrome guard is still empirical and is not independent final verification.

## Measured revision

Hosted run:

~~~text
m6d2-prefix #10 / 37564857480
head  c5e8603be0d396dddfe1c43894d6acb10ac423d6
CPU   AMD EPYC 9V74 80-Core Processor
Rust  1.99.0 / LLVM 23.1.1
runs  3
source set size 8192
~~~

Artifact: `m6d2-prefix-37564857480-1`.

Later PR commits before this record are documentation/comparator-only; the measured implementation, tests, harness and summarizer are the head above.

The artifact hashes:

~~~text
m6d2_bench.rs
2951d44e8f6c1799b3db59267589ee5aa64c0c86d32985a1834fffbe1412f86f

m6d_pinsketch64.rs
0f9466efc092debcaee97ad9fc5e4434044caa3760a3d68f2e63828ea7ccc1f4

m6d2_prefix.rs
a81d31303c885e9717d9ef4747e174aa30b17cdf0fcd45211fb2ac8d1416ed87

m6d2_prefix test
7ad1903a80429327584e78fbce4da64bcfefcae4d2f23a99c6485026bfaef0e1

m6d2_summary.py
13de6ffaa0858f2c9aac8aebeea9b4494a0f73fb4ffefdbf1b1d83a5c4ba46f5
~~~

Fail-closed summary result:

~~~text
logical_gate=PASS
false_success_total=0
~~~

## Frozen schedule

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
new syndrome words    2   1   2   4
~~~

The first stage also transmits one zero-presence byte.

The maintained receiver extends the existing remote prefix. It does not resend previous syndrome words and does not rescan the source set.

## Correctness result

The isolated D2 adapter proves:

- a prefix is exactly the first n D1 odd syndromes;
- extension preserves all already received words byte-for-byte;
- zero metadata cannot change during extension;
- merged prefixes equal the same prefix of a full merged sketch;
- every decode names an explicit max-elements limit;
- failed stages do not fall back to unguarded decode.

Frozen workloads:

~~~text
d = 0, 1, 2, 3, 4, 5, 8, 9, 10, 16
~~~

The d=1 case includes key zero.

Results:

- d <= 8: exact-oracle completion;
- d > 8: guarded reject;
- false-success: 0 in the frozen matrix.

This does not replace D1's wider over-capacity inventory or create an adversarial theorem.

## Communication result

Application payload excludes framing/authentication.

| d | final k | attempts / RTTs | direct exact | fixed guarded | incremental | naive resend |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 1 | 1 | 65,540 B | 17 B | 17 B | 17 B |
| 1 | 1 | 1 | 65,548 B | 17 B | 17 B | 17 B |
| 2 | 2 | 2 | 65,540 B | 25 B | 25 B | 42 B |
| 3 | 4 | 3 | 65,548 B | 41 B | 41 B | 83 B |
| 4 | 4 | 3 | 65,540 B | 41 B | 41 B | 83 B |
| 5 | 8 | 4 | 65,548 B | 73 B | 73 B | 156 B |
| 8 | 8 | 4 | 65,540 B | 73 B | 73 B | 156 B |
| 9/10/16 | 8 | 4 | ~65.5 KiB | 73 B reject | 73 B reject | 156 B reject |

Retry byte savings versus naive resend:

- d=2: 40.476%;
- d=3/4: 50.602%;
- d>=5: 53.205%.

The important invariant is stronger: **incremental cumulative bytes equal ideal fixed-known-k bytes.**

Unknown d therefore has no extra syndrome-byte cost under this nested schedule.

## CPU result

Median maintained-state decode CPU:

| d | fixed one-shot | incremental cumulative |
| ---: | ---: | ---: |
| 0 | 0.96 us | 0.28 us |
| 1 | 0.43 us | 0.14 us |
| 2 | 0.678 ms | 0.694 ms |
| 3 | 2.05 ms | 3.38 ms |
| 4 | 11.86 ms | 53.87 ms |
| 5 | 15.61 ms | 108.29 ms |
| 8 | 38.70 ms | 89.57 ms |

Prefix extension itself is negligible here (sub-2 us cumulative). Repeated reference decoding dominates the retry tax.

The exact merge-scan baseline over the 8192-key sorted sets is about 8.7 us.

Cold construction of both stored-capacity-9 source sketches is about 12.3 ms in this harness. Cold one-shot use therefore remains a different and less attractive workload than already-maintained state.

## Idealized bandwidth / RTT crossover

For maintained state, compare:

~~~text
direct:
    direct_bytes / bandwidth
    + exact_scan_cpu
    + 1 RTT

incremental:
    prefix_bytes / bandwidth
    + cumulative_prefix_cpu
    + attempts * RTT
~~~

Framing, authentication and scheduling overhead are still excluded.

Approximate maximum link bandwidth below which the measured incremental path wins:

| d | RTT 0 ms | RTT 10 ms | RTT 50 ms |
| ---: | ---: | ---: | ---: |
| 2 | 764 Mbps | 49.0 Mbps | 10.3 Mbps |
| 3 | 155 Mbps | 22.4 Mbps | 5.07 Mbps |
| 4 | 9.73 Mbps | 7.09 Mbps | 3.41 Mbps |
| 5 | 4.84 Mbps | 3.79 Mbps | 2.03 Mbps |
| 8 | 5.85 Mbps | 4.38 Mbps | 2.19 Mbps |

For d=0/1 the measured maintained prefix path also uses less local CPU than the full exact merge-scan and one RTT, so the raw-payload model favors it across the tested bandwidth range.

These crossover values are diagnostic, not product claims. The direct baseline is a raw canonical u64 list rather than an entropy-optimal set code.

## Rateless IBLT comparison

Pinned external comparator:

- Practical Rateless Set Reconciliation, SIGCOMM 2024;
- official `yangl1996/riblt`;
- commit `4afa6bc06cb2237d9ea273a51d97a7e05b3f573b`.

The official implementation's coded symbol contains:

~~~text
Symbol
Hash  uint64
Count int64
~~~

Therefore its asymptotic ~1.35d coded-symbol count is not directly a byte count for DeltaMeter u64 keys.

Rateless IBLT has two protocol advantages over D2:

- naturally fine-grained rateless growth rather than 1/2/4/8 stages;
- decoder distinguishes remote/local recovered elements.

D2 PinSketch has a different correctness profile:

- deterministic exact recovery inside the explicit admitted limit;
- exact full-u64 identity mapping with no hash collision;
- current over-capacity guard remains empirical.

Canonical comparison boundary: `docs/M6-D2-RIBLET-COMPARATOR.md`.

## Interpretation

D2 answers the narrow question positively:

**a nested guarded exact-sketch prefix can replace Energy-first sizing for the audited small-d lab regime.**

It does not establish a public reconciliation product.

The main remaining weakness is CPU, not communication. The reference factorizer pays the cost of every failed stage again.

## D2 verdict

**LAB-GO** for the private incremental-prefix primitive.

Keep:

- frozen 1 -> 2 -> 4 -> 8 schedule as the reference control;
- one guard syndrome per stage;
- application-byte accounting;
- explicit RTT count;
- D1 exact oracle and full-u64 mapping.

Do not add:

- public ExactSmallDelta/reconciliation API;
- snapshot-v1 integration;
- Coverage::Exact;
- a theorem claim for the one-syndrome guard;
- Rateless IBLT as a crate dependency.

## Next experiment

The next optimization question is decoder reuse, not more protocol design.

Evaluate one candidate at a time:

1. reuse Berlekamp-Massey state across stage growth;
2. cache reconstructed even syndromes;
3. only after those measurements consider root-finding specialization.

The optimization must preserve D2 payload/RTT behavior exactly and compare cumulative retry CPU against the frozen D2 reference.

A separate future experiment may implement a pinned u64 Rateless IBLT wire model for direct byte/CPU comparison.
