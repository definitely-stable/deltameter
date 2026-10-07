# RECON-VERIFY-001 — compact-verification revalidation plan

Issue: #60

Status: bounded product-validation plan. No decoder optimization.

## Question

M6-D13-B stopped the current system contract because a provisional sketch result
was independently verified by sending a full reconstructed canonical set. That
structurally erased the bandwidth advantage of compact reconciliation.

This task asks: if verification becomes a compact cryptographic digest with an
explicit computational collision assumption, does small-d reconciliation become
useful, and is there any reason to own DeltaMeter's D11 instead of Minisketch?

## Prior-art correction

The D13 staged false candidate is a BCH/PinSketch miscorrection/aliasing phenomenon,
not a new class of failure. Minisketch already exposes false-positive sizing through
compute_capacity / compute_max_elements and explicitly treats over-capacity decode
as probabilistic false-positive behavior.

No new guard theorem is sought.

## Candidate verification contract

For reconstructed candidate set B', both peers compute a domain-separated digest of
the canonical set representation, conceptually BLAKE3-256 over:
- protocol domain string;
- generation/session identity;
- canonical sorted set bytes.

Receiver sends only the digest. Sender compares it with its digest of A. Digest
mismatch triggers terminal direct exact fallback.

This is computationally verified reconciliation, not mathematically exact.

The model must charge digest CPU on both peers, canonicalization if needed, 32
digest bytes plus framing, every failed attempt, and exact fallback.

## Comparison arms

1. direct exact;
2. current private D11 staged 1->2->4->8 + digest;
3. current private D11 one-shot k=8 + digest;
4. pinned Minisketch for the same small-d envelope + digest;
5. a Minisketch fpbits-sized lane if distinct from arm 4.

Use the same declared nonzero 64-bit token domain. Any identifier-to-field hashing
must have its own collision boundary.

## Error budget

Keep distinct:
- BCH/Minisketch over-capacity false-positive probability;
- D11 guarded over-capacity behavior;
- BLAKE3 digest collision assumption;
- identifier-to-field mapping collision if introduced.

Digest verification can turn a wrong decoder candidate into a safe fallback under
the hash assumption. It does not make the decoder mathematically exact.

## Workload

Start with a cheap analytical/native model:
N = 8K, 64K, 1M
d = 0,1,2,4,8,9,16,64
RTT = 0,1,10,50,100 ms
bandwidth = 1,10,100,1000 Mbps

Judge one-shot k=8 primarily at d<=8. d=9/16/64 are fallback/miscorrection guards.

Only spend a full five-worker measurement budget if the byte/RTT/digest model shows
a meaningful possible win.

## Structural change from M6

At N=65,536 M6 full-list verification was about 524 KiB. Compact digest verification
turns a successful k=8 path into small syndrome payload + framing + 32 digest bytes,
but adds O(N) digest CPU. The experiment must measure that CPU rather than treating
hashing as free.

## Ownership gate versus Minisketch

A win versus direct is necessary but not sufficient. If pinned Minisketch is
materially faster or equal-size with equivalent/stronger failure semantics, prefer
USE_MINISKETCH_NOT_OWN_D11 and stop productizing D11 permanently.

Safe Rust / no FFI is only a differentiator if it produces concrete product value;
it is not by itself a reason to maintain a second PinSketch implementation.

## Outcomes

GO_SMALL_D_PRODUCT:
owned D11 materially beats direct for declared d<=8 cells, remains competitive with
Minisketch, and has an explicit acceptable computational verification contract.

USE_MINISKETCH_NOT_OWN_D11:
compact verification makes reconciliation useful, but Minisketch is the better
primitive.

STOP_RECON_PRODUCT:
compact verification still fails to create a compelling regime, or contract
complexity is not worth the gain.

## Non-goals

No BM/root/GF micro-optimization, no new BCH theorem, no generic rateless backend,
no public API, and no rewriting M6. This is a materially different verification
contract.
