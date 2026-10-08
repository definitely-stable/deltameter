# STRICT-COMPACT OPT-C C2/C3 accounting addendum

Issue #74; PR #78 merged. PRE-REGISTERED BEFORE C2 MEASUREMENTS.

## Crucial wire correction

Receiver-side `U<=T` does NOT save sender bytes in an unidirectional
push stream. Compare five genuinely different transport semantics:
COMPLETE (one full bitmap), PUSH_ALL (52 frames, no cancellation),
BOUNDED_PREFIX (sender knows policy T before sending and stops at first
certified sufficient prefix), ZERO_ELISION (sender-certified zero levels
encoded in explicit presence/zero masks), and INTERACTIVE with receiver
request batches of 1,2,4,8 levels. For BOUNDED_PREFIX, charge sender-side
Q32 prefix computation. For INTERACTIVE, charge every request and RTT.
No mode may quietly assume unsent bytes disappear after receiver stopping.

## Frozen application byte accounting

C1 session content 92 B + CRC32 4 B = 96 B.
Receiver request (predeclared T:u64) = 8 B, charged in every mode.
Plan header (mode:u8, count:u8, requested_mask:u64,
declared_zero_mask:u64) = 18 B.
One canonical level frame: 16 B bound session tag + 1 B level ID +
512 B bitmap + 4 B CRC32 = 533 B.
Extra interactive batch request = 17 B, one RTT per request.
Full bitmap payload = 26,624 B + 4 B CRC32.

Therefore COMPLETE = 26,750 application bytes;
PUSH_ALL = 27,838 bytes regardless of local receiver stopping.
BOUNDED_PREFIX = 122 + 533*n for n transmitted levels.
Elided-zero levels must be declared explicitly and reconstructed as
zero, never confused with absent/unknown levels. CRC and public tags
are not authentication; no secret key is transmitted.

## Corpus and stopping semantics

Preserve already-frozen d/T scenarios:
(4096,8192), (65536,131072), (1048576,2097152).
Synthetic u64 tokens are fixed and nonadaptive to the keyed oracle.
Use 32 independent deterministic unique token-range offsets per
scenario on each of 5 GitHub-hosted workers.

Compare ascending, descending, and deterministic T-centered level
order. T-center is round(log2(T/4096))+1, clamped to [1,52],
tie break to smaller j. Only externally predeclared T may influence
the order, never hidden true d. Evaluate COMPLETE once and seven
candidate variants per order (PUSH_ALL, BOUNDED_PREFIX, ZERO_ELISION,
INTERACTIVE_1/2/4/8), 22 observations per sample, 10,560 overall.

The sender and receiver must use identical frozen source epoch/config/Q32
IDs. Reconstruct the exact full bitmap on COMPLETE; for all other
modes reconstruct exactly the declared level subset with a correct
presence mask and bound, and never claim success if U>T after all levels.

## Measurement/accounting

Output exact application bytes, frames, requests/RTTs, receiver U
as decimal string, success vs ExceededOrUncertain, sample provenance,
sender immutable copy cost, sender prepare/serialize cost, and receiver
processing cost. Fail on incomplete/duplicate/invalid evidence.

Model the 1/10/100 Mbit/s x 0/10/50/150 ms RTT grid. Distinguish
modeled serial-link latency from measured end-to-end latency.
DO NOT label an analytical RTT calculation as a real transport benchmark.
Keep the earlier C3 >=25% p95 bytes and no-latency-regression numeric
gates fixed, but an analytical network model only authorizes a
MODELED_C3_CANDIDATE/NO_GO preliminary verdict, not a final
PROGRESSIVE_PASS. Final acceptance requires controlled actual
network/latency evidence and complete in-flight/snapshot-copy accounting.

No public API, Snapshot v1 change, or guarantee promotion.
