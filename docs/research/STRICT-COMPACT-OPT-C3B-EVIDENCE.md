# OPT-C C3-B — genuine two-party TCP lifecycle evidence

Issue #80 / #74, PR #82, prerequisite merged C3-A PR #81.
Frozen protocol: [STRICT-COMPACT-OPT-C3B-PROTOCOL.md](STRICT-COMPACT-OPT-C3B-PROTOCOL.md).

## Frozen gate result

**C3B_TWO_PARTY_NO_GO** for the explicit 2-RTT request-first
two-source progressive transport compared with concurrently served
dual-full state, under the predeclared 10Mbps and emulated
RTT 0/10/50ms gate. The measurement itself is
**C3B_CORRECTNESS_AND_PHYSICAL_ACCOUNTING_PASS**.

Source SHA:
`d0480354e372d1d81d0e03263cd375467b680031`.
GitHub-hosted five-worker run:
[37816319786](https://github.com/definitely-stable/deltameter/actions/runs/37816319786).
Five workers PASS, full source-HEAD and completeness aggregator PASS,
**1,440 checkpoint records**.

This is actual **localhost TCP** application writes/reads under
application-layer 10Mbps pacing and RTT sleeps — **not WAN,
kernel-shaped TCP or end-to-end secure networking**.

## Genuine two-source functionality

Two independently held J52 parity sketches, distinct immutable epoch
IDs, shared keyed oracle/config/table. Both senders have separate
TcpListener/TcpStream connections. A receiver requests identical
level IDs and retains two partial parity states, XORs per-level bytes
and evaluates Q32. It adds three levels after the first and then
48 levels in the final request. The computed upper bound is monotone,
never below the direct complete XOR bound; after all 52 levels,
full reconstructed bitmaps and Q32 upper bound match exactly.
Mixing a different epoch is rejected.

The control runs two concurrently served complete bitmap exchanges,
decodes both from socket frames with CRC, XORs the two reconstructed
word vectors and checks every word against the original canonical
bitmaps. Every sender's successful send byte count matches the
receiver's successful read count, and vice versa.

## Exact cumulative application traffic

| Functional checkpoint | Two senders total | Relative to dual full |
|---|---:|---:|
| Dual complete canonical states | 53,500 B | baseline |
| One retained level per sender | 1,344 B | 97.49% fewer |
| Four retained levels per sender | 4,576 B | 91.45% fewer |
| All 52 retained levels per sender | 55,778 B | **4.26% more** |

After the full fallback, physical framing and three rounds negate
the bytes advantage. A partial upper bound and full interchange
must never be treated as equivalent.

### Worker 1, application RTT target 10ms, 10Mbps, p95

| d / T | Complete p95 | Retained 1 p95 | Retained 4 p95 | Retained 52 p95 |
|---|---:|---:|---:|---:|
| 4,096 / 8,192 | 32.541ms | 21.872ms | 33.864ms | 69.092ms |
| 65,536 / 131,072 | 32.502ms | 21.810ms | 33.784ms | 69.006ms |
| 1,048,576 / 2,097,152 | 32.472ms | 21.802ms | 33.752ms | 68.999ms |

For all worker-1 synthetic seeds at this RTT, both full and
retained-one result satisfy the predeclared receiver threshold
`U<=T` (8/8 per scenario). Across **all five workers and
RTT 0/10/50ms**, the original **no p95 wall-time regression**
condition fails for every scenario. The fail-closed aggregator
reports `C3B_SCENARIOS []`.

This is a real negative product-gate observation for this
request-first protocol. **Do not weaken the gate after measuring.**

## Trace and artifact provenance

GitHub artifact ZIP SHA256:

| Kind | ID | Digest |
|---|---:|---|
| summary | 11567088310 | 1ea87645ccb0d531bdef669bf6e9b783819eef825f3b574d4b18ff3e920559a3 |
| worker1 | 11567925100 | ebecf92bd1ace0d73a72dc2ef8254241b447948e27ebf9da68709e38bb5391f2 |
| worker2 | 11566369376 | fc743b55119d3466dab9c46cc6f9602f3a13b43a74b700d0d6b7ff02ef657741 |
| worker3 | 11567421955 | 0504b0c2e15369b29b3c3decdbd31ee3d2d835979454e98bc7665a505870eeb1 |
| worker4 | 11567536437 | 4e063ea990ad5139dc961c2be977a08a84ef265451888e270ea909de70cb32db |
| worker5 | 11566948542 | b5e3d79f400f21a95f938d22d54b2976adc1b8704c07650c969949c94b55ded3 |

## Next evidence-gated hypothesis (new protocol, not reclassification)

A different design may remove the **initial extra request RTT**:
after receipt of the receiver's externally predeclared T, each
independent sender immediately transmits a fixed **T-centered
one-level prefix** without waiting for the receiver to request the
first level. The receiver validates both same-key parity levels and
computes merged U. If not sufficient, it explicitly asks for more,
charging all additional request bytes/RTT and fallback.

This is a genuinely different protocol, **not a reinterpretation
of C3-B results**. Its order is decided from publicly known T
before observing the two states. It still may not justify a public
API under key/epoch lifecycle and high-load unknown distributions.

The C3-B failure does not falsify the simultaneous coverage theorem
nor rule out another transport design. Keep issue #80/#74 open and
public freeze #69 blocked pending a separate preregistered test
of the one-shot prefix hypothesis or STOP decision.
