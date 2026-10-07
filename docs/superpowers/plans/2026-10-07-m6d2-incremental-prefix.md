# M6-D2 — Guarded incremental PinSketch64 prefix experiment

Status: implementation and evidence complete in PR #27; **LAB-GO private only, production/public NO-GO**. Issue: #26. Parent: #19 / #14.

## Goal

Test whether the guarded PinSketch64 odd-syndrome family can self-stage exact-small reconciliation for unknown small d without the rejected Energy-first sizing step.

This remains a private lab protocol. It does not add a public network API, snapshot backend, or Coverage variant.

## Frozen stage schedule

Initial schedule:

~~~text
decode limit k        1   2   4   8
stored syndromes      2   3   5   9
new syndrome words    2   1   2   4
~~~

Each stage stores exactly one syndrome beyond the explicit decode limit.

The prefix is nested:

~~~text
S1, S3
S1, S3, S5
S1, S3, S5, S7, S9
...
~~~

An extension sends only the newly required 64-bit syndrome words. Already transmitted words are never resent in the incremental arm.

## Application-byte model

Count protocol payload separately from framing:

- first stage: one zero-presence byte plus two 64-bit syndrome words;
- each extension: only new 64-bit syndrome words;
- cumulative incremental payload at final k=8: 1 + 9*8 = 73 bytes;
- fixed known-k payload: one zero byte plus (k+1)*8;
- naive resend payload: resend the full guarded prefix at every attempted stage;
- direct exact baseline: u32 count plus all canonical u64 elements.

Framing/authentication are excluded from the first algebraic value experiment and must be added before any production protocol claim.

## Arms

For the same immutable source sets:

1. **direct-exact** — canonical exact set payload plus exact merge-scan oracle.
2. **fixed-guarded** — caller already knows the smallest schedule k >= d; send one guarded prefix; one decode attempt.
3. **incremental-prefix** — start at k=1; on guarded decode failure extend to 2, then 4, then 8; count each syndrome word once; every attempt is timed.
4. **naive-resend** — same stage schedule and decoder, but resend the entire guarded prefix on every retry; control arm for the value of nesting.

For d > 8, fixed/incremental/resend finish as a guarded reject, never exact success.

## Workload matrix

Deterministic full-width u64 sets with d = 0,1,2,3,4,5,8,9,10,16.

Include at least one case where key zero is in the symmetric difference. Source cardinality baseline: 8192 elements, matching D1.

## Required measurements

Per scenario and arm:

- exact d and final exact/rejected outcome;
- final stage k, decode attempts and modeled RTT count;
- cumulative syndrome payload bytes;
- naive-resend, fixed-known-k and direct-exact bytes;
- cumulative and per-attempt decode CPU;
- local prefix-extension/merge CPU;
- cold full-sketch build cost;
- maintained-state cost excluding build;
- exact oracle equality for every successful result.

## Correctness contract

- D1 exact u64 mapping remains unchanged;
- zero remains one out-of-band XOR-composable bit;
- every decode names an explicit max_elements;
- every stage has one additional stored syndrome guard;
- a failed guard cannot fall back to unguarded decode;
- all successful in-bound candidates must equal the exact oracle;
- over-bound completion is reject;
- syndrome revalidation is not independent final verification;
- the one-guard empirical result is not promoted to a theorem.

## Incremental implementation contract

The sender may precompute a maximum stored-capacity-9 lab sketch. The receiver accumulates a remote prefix monotonically.

Extension must prove:

- old syndrome words are byte-for-byte unchanged;
- only suffix words are added;
- zero-presence metadata is sent once and cannot change mid-session;
- extending a prefix to capacity n equals taking the n-word prefix of the original full sketch;
- merged difference prefixes equal the corresponding prefix of a full merged sketch;
- no source-set rescan is required for maintained-state extension.

## CPU interpretation

D1's decoder is deliberately reference-grade. D2 measures cumulative repeated-decode cost before attempting decoder optimization.

Do not optimize Berlekamp-Massey/root finding in the same evidence run. If repeated decode dominates, open a separate candidate for incremental BM-state reuse or cached even syndromes.

## Rateless IBLT comparison boundary

Primary external comparator:

- Practical Rateless Set Reconciliation, SIGCOMM 2024;
- official implementation: yangl1996/riblt;
- pinned implementation commit: 4afa6bc06cb2237d9ea273a51d97a7e05b3f573b.

Do not import it as a dependency in D2.

D2 records the coded-symbol count model, the symbol payload fields needed for an apples-to-apples byte comparison, and semantic differences in direction/recovery/failure handling.

Do not compare 1.35d symbols directly to 8d bytes without accounting for symbol representation.

## Decision gate

LAB-GO requires all of:

1. prefix extension is exact/nested;
2. all d<=8 workloads reach exact-oracle completion;
3. d>8 workloads reject in the fixed deterministic matrix;
4. incremental bytes are strictly below naive resend bytes whenever retries occur;
5. a named bandwidth/RTT regime exists where maintained incremental prefix beats direct exact transfer;
6. repeated decode CPU is reported explicitly rather than hidden.

NO-GO if nesting provides no meaningful value after retry CPU/RTT accounting.

Even LAB-GO does not authorize a public ExactSmallDelta protocol.


## Completion record

Measured implementation head: `c5e8603be0d396dddfe1c43894d6acb10ac423d6`.

Hosted evidence: `m6d2-prefix #10 / 37564857480`.

Canonical evidence: [M6-D2 incremental prefix evidence](../../M6-D2-INCREMENTAL-PREFIX-EVIDENCE.md).

Verdict:

- nested prefix semantics: ACCEPT;
- incremental bytes equal ideal fixed-known-k bytes;
- naive resend: REJECT as wasteful control;
- repeated reference decode CPU: material retry tax;
- private maintained-state protocol primitive: LAB-GO;
- public/production reconciliation API: NO-GO;
- next optimization target: decoder-state reuse without changing D2 bytes/RTTs.
