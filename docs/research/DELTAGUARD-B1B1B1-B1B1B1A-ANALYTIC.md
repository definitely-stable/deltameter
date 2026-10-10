# Analytical note — exact-capability guard-before-FULL cannot save B1 network bytes

Related [#109](https://github.com/definitely-stable/deltameter/issues/109),
frozen [B1B1B1-B1B1-B1A protocol](DELTAGUARD-B1B1B1-B1B1B1A-PROTOCOL.md).
This is **direct deterministic accounting**, not a new information-theoretic
theorem or an assertion about arbitrary other reconciliation algorithms.
It is independent of yet-to-be-observed hosted timing results.

## Scope and proof

Assume an immutable query generation, two independently framed owner TCP
sources, a receiver with NO stored exact owner lists, and the only available
exact reconstruction is a full canonical B1_FULL inventory frame from BOTH
sources. A B2A GUARD frame is a 2047-bit parity observation, padded to
256 bytes per owner; it does **not** output exact differing token IDs.
Hence, if the externally demanded answer is the actual exact set difference,
the protocol must still issue both FULL requests after any guard observation,
regardless of whether the observed odd count S is <=48.

For each query with source sizes \(|A|,|B|\):

- One physical guard round contains per owner
  32-byte request + 64-byte envelope + 256-byte keyed bitmap
  + 16-byte ACK = 368 bytes, hence **736 bytes** for both owners.
- One physical direct FULL round contains per owner
  32-byte request + 64-byte envelope + 8 bytes per element
  + 16-byte ACK. Hence
  \(F(A,B)=224+8(|A|+|B|)\).
- The implemented exact-output `GUARD; FULL` protocol
  sends \(G+F=736+F\), whereas `FULL` sends \(F\).
- Therefore \(B_{G\to F}(Q)-B_F(Q)=736Q>0\)
  for every integer query count \(Q>=1\).
  This strict positive byte overhead is unaffected by d,
  app delay, bandwidth, B2A false passes, or receiver
  crash protocol, as long as both paths use the same
  FULL output and the extra guard is actually sent.
  **No latency ordering** follows from these byte counts
  without measuring round-trips and scheduling.

The frozen fixtures have source1 length N+1 at gen2;
source2 length N+1 for even d, N+2 for odd d, so
\(F(N,d)=240+16N+8(d\bmod2)\). This also explains
why the odd d57 fixture has an additional 8-byte FULL payload.
Checksums/ACKs/requests are included, not theoretical compressed data.

### Illustrative exact fixed-width application totals

| N,d | Q | FULL physical bytes | GUARD then FULL bytes | overhead |
|---|---:|---:|---:|---:|
| 256,48 | 1 | 4,336 | 5,072 | +736 |
| 256,48 | 10 | 43,360 | 50,720 | +7,360 |
| 256,48 | 100 | 433,600 | 507,200 | +73,600 |
| 256,57 | 100 | 434,400 | 508,000 | +73,600 |
| 65,536,48 | 10 | 10,488,160 | 10,495,520 | +7,360 |
| 65,536,57 | 1 | 1,048,824 | 1,049,560 | +736 |

All values here are **format-derived**, not claimed to have already
been measured. The separate hosted workflow must verify actual
physical frames and independent source exact oracles. Rows not
present in the frozen physical matrix remain mathematical identities.

## Cold guard-only observation is DIFFERENT service quality

A one-round GUARD provides only S and a threshold flag, with
736B per query and no exact inventory. A maintained EXACT
receiver instead pays real cold FULL gen1 transfer and real
source gen2 WAL/receiver sync (initial network here
\(C(N,d)=226+16N+8(d\bmod2)+356\)), stores both
canonical lists, then answers unmodified hot exact queries
with no additional source transfer.

Ignoring receiver disk/CPU/repair costs for this narrow
network-only comparison, equal byte budget is at
\(Q=\lceil C(N,d)/736\rceil\):
- N256,d48: \(C=4,678\) bytes, first integer Q where
  GUARD-only costs at least cold maintained EXACT is **Q=7**.
- N65536,d48: \(C=1,049,158\) bytes, first such integer
  is **Q=1426**.

This is NOT a product break-even because outputs are
different and source construction/durable receiver CPU/disk,
crashes, exact fallback, authentication, latency and
workload-specific query information requirements are
not included. If the service MUST return actual differences,
a GUARD-only result is unusable and one must compare
`FULL`, `GUARD→FULL`, and a correctly maintained
exact alternative. If a service legitimately needs only
an S-observation, that separate screening niche remains
an open question with its own false-pass/security risks.

A more capable exact sketch could change the premise:
this accounting does **not** rule out fundamentally new
exact-capable reconciliation methods, decoder improvements,
or information-theoretic advances in other protocols.
It only tests the existing stateless B2A guard framing.
