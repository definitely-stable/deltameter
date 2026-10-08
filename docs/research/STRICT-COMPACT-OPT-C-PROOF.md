# STRICT-COMPACT OPT-C — simultaneous level coverage and adaptive stopping

Issue #74, parent #70, prerequisite OPT-A PR #76 (merged).
Status: **research proof note, not a new probabilistic theorem or public contract**.

## Fixed objects and validity boundary

Fix a *single immutable* pair of u64 token sets A,B, its symmetric difference
D=A Δ B of unknown cardinality d, one secret oracle key supplied independently
of the token set, one keyed BLAKE3 v1 mapping, one m=4096 / J=52 / LEVEL_MAJOR
parity bitmap, and the certified table generated with alpha=delta/64.
Let `U_j(S_j)` be the Q32 calibrated one-sided upper confidence endpoint for
level j of this frozen sketch, with the full-domain fallback
`2^64` represented as u128.

For each predeclared level j, the *ideal-oracle* finite-sample theorem gives

    P_d[U_j >= d] >= 1 - alpha_j

under the original per-level theorem's assumptions.
Every level's statistical budget is **fixed in advance**, and
`sum_{j=1..J} alpha_j = J*delta/64 <= delta`.
We must not redistribute alpha in response to observed states or query traffic.

## Simultaneous event and selection

Let E_j = {U_j >= d} and E = intersection_{j=1..J} E_j.
The elementary union bound, without any independence between levels, yields

    P_d(E) >= 1 - sum_j P_d(E_j^c)
           >= 1 - sum_j alpha_j >= 1-delta.

Define for any (possibly adaptively selected) received subset L:

    U(L) = min_{j in L} U_j,  if L != empty
           2^64,            if L == empty.

On event E, `U_j >= d` for every predeclared j. Consequently the minimum
over ANY subset L remains >= d, including a subset chosen after seeing
previously received level data/bounds, after a stopping event, or after the
receiver requests additional levels. Thus

    P_d[U(L) >= d] >= 1-delta

for every stopping/selection policy that only selects among those already
predeclared level statements **for the same immutable snapshot**.
The claim is uniform over all such selection policies because the one E
event suffices for all L simultaneously. No per-request alpha spending.

Also `U(L union R) <= U(L)`, even when R is requested adaptively, because
adding candidate endpoints to a minimum cannot increase it.
This is an algorithmic monotonicity property, not a promise of narrower
q95 width for every prefix. A partial prefix often returns the saturated
domain ceiling, which is correct but uninformative.

## Explicit limitations and non-claims

1. **No adaptive-token guarantee:** the keyed BLAKE3 PRF transfer retains
   its declared computational/nonadaptive input model. A peer choosing new
   tokens from disclosed intermediate states is not covered by this note.
2. **No mutable-state mixing:** combining one level from state(epoch0) with
   another from state(epoch1) is not the fixed-D sketch above. Senders must
   freeze or copy a consistent snapshot; recipients must reject mismatched
   epoch/generation identifiers and profile/key/config binding.
3. **No meaning for missing levels:** an unreceived level is UNKNOWN, not
   a known all-zero bitmap. A received all-zero bitmap *is* informative and
   must remain distinguishable in frame/presence metadata.
4. **No automatic integrity guarantee:** a CRC/unkeyed digest detects
   accidental corruption, not a capable malicious sender. Peer authenticity
   and replay protection require an external authenticated channel or
   separately designed authentication. Never send the secret oracle key.
5. **No oracle replacement or public Proven guarantee:** this is exactly
   the original ideal statistical theorem plus a union-bound observation,
   under the same computational PRF assumption already declared by DeltaMeter.

## Receiver-observable stopping

The receiver does not know d. It can use `U(L) <= T` for a predeclared
application tolerance T, a max transmitted bytes budget, or a deadline.
The criterion `U(L)/d <= 1.5` cannot be a real receiver stop condition,
because true d is not observed. It is only an offline performance/quality
metric for synthetic known-d experiments.

If a policy stops with U(L) > T, it returns
`ExceededOrUncertain` (never false-safe claim). Additional levels
can tighten the bound, but no unreceived level may be treated as present.

## Wire design consequences

A progressive research frame needs:
- protocol version and exact profile/J and table/oracle identity;
- nonsecret key/config binding (never the key);
- one immutable generation/epoch identifier across all level frames;
- explicit unique level index and received-level bitmap;
- exactly 512 B of canonical LEVEL_MAJOR data per received level;
- corruption detection and documented trust model.

If received level IDs are out of order, duplicates are equal-only or rejected
by frozen policy; conflicting duplicates and mixed generations fail closed.
A separate mandatory timeout/error state is needed rather than treating
missing levels as zeros or valid final convergence.

This note justifies *testing* progressive transfer. It does not determine
whether additional headers/round trips make it better than a full one-shot
transfer. That is the evidence gate of #74.
