# DeltaGuard B1-B1B1-B1B1-B0 — frozen hot contract-comparison protocol

Research continuation of #109/#107 after D65 and merged PR #110.
**Frozen before any B1B1B1-B1B1-B0 measurement.**
No public Rust/Snapshot v1 changes; no system/product GO.

## Causal question and non-equivalent service contracts

Compare user-facing cost of 20 repeated questions on *the same
durable gen2 source snapshot* with 2 independent source owners.
The underlying question is **"does the B2A near-full test report
odd<=48, or is an exact result needed?"**, NOT recovering an exact
difference from a guard alone. Report observed B2A S and independent
true d, distinguish `guard_under_cutoff` from exact knowledge.
A guard's public keyed bitmap is not authenticated and
`S<=48` does NOT deterministically establish `d<=48`.

Three receiver-state contracts, NEVER conflate their costs:

- `guard_stateless`: source owners retain canonical durable
  source snapshots+WAL, rebuild the fixed B2A bitmap once per
  source process, receiver does NOT ingest/cache exact source
  lists. Each new guard query physically requests BOTH
  256B keyed bitmap payloads over TCP. UNKNOWN/strict exact
  reconstruction requires NEW physical FULL source transfers
  and all receiver ingestion cost must be charged.
- `hot_maintained_exact`: receiver physically bootstraps
  both gen1 canonical owner lists and persists them via
  the SAME atomic B1B1B1-B1B0 two-owner checkpoint/WAL;
  then it physically applies both source gen2 events
  and fsyncs one 256B receiver transaction plus 48B
  commit marker. It keeps canonical vectors materialized
  hot: repeated nonmutating exact-d questions need no
  source TCP transfer. Source WAL fsync and generation
  identity identical across contracts, but exact
  receiver's cold bootstrap, RAM and disk writes
  MUST be charged to exact, NOT guard.
- `direct_full`: receiver does not maintain state;
  on every query physically requests both full source
  inventories. It can supply full exact difference
  and pays every full transfer.

`guard_resolved`: first physical guard round; if S>48
(UNKNOWN), perform a second physical FULL two-owner
query and charge both. If S<=48, retain the B2A test
response ONLY; do not pretend that it produces exact
set members or a deterministic upper bound on d.
Full list requests may require additional auth and
disk cost in real deployment, not supplied by fixture.

## Preregistered experiment, immutable before timing

5 independent GitHub-hosted workers x N={256,65536}
x source symmetric difference d={48,57} x 3 fixture repeats,
fixed fixed-b11/T64 c48 and public fixture key,
one source generation 2 insert per owner, and
**20 alternating-mode paired queries** to the SAME
two long-lived source OS processes after full
reopen from private fsync'd source WALs.
This is **60 sessions** / **1200 pairwise sample rounds**,
each with physically measured guard and direct-FULL
two-owner request/frame/ACK exchanges, plus locally
computed maintained-exact response on genuinely hot
durably committed receiver vectors. Log per-query:
guard bytes, full bytes, guard value S, observed
under-cutoff bit, independent exact d, wall clocks,
local maintained exact wall clocks, sender total
VmHWM, real /proc/self/stat CPU ticks, source bitmap
construction wall time, exact cold FULL bootstrap,
shared generation2 physical event transfer,
receiver logical checkpoint+WAL bytes/actual fsync
operations and separate source metadata cost.

Rate=100Mbps and injected response delay=0ms
(**localhost application pacing**, NOT WAN);
no 1/10Mbps, 10/50ms or Q1/Q100 holdout claim.
Source owner process is reused across ALL 20 queries,
not reconstructed per query. Rotate guard/full
query order by worker/lane/fixture/round to limit
systematic one-sided timing bias. Execute all
data transfers and fsync calls, never fabricate
network byte totals from theoretical frames.

Independent exact oracle verifies both physically
transferred FULL lists match fsync source owner
tokens and receiver's hot exact lists at gen2.
B2A odd count verified against independently
hashed true symmetric difference. For d=57,
any observed S<=48 is explicitly an under-cutoff
instance at d>T and MUST NOT be called a proof of
small set difference. Report it without cherry
picking. Each query exact baseline accesses
hot vectors and computes actual exact symmetric
difference; not just checks a cached length.

## Fixed byte identities and decision gates

Physical request+ACK per owner 32+16B; 64B frame:
two-owner B2A bitmap per query =
2*(32+16+64+256)=**736B**.
Two-owner full gen2 per query =
2*(32+16+64)+8*(|owner1|+|owner2|)
=224+8*(2N+(d mod 2)+2) = **240+16N+8*(d mod 2)**.
Cold gen1 FULL physical bootstrap includes
two 1B owner hellos and identical request/ACK/
frame, so **226+16N+8*(d mod 2)**.
For odd d=57 the second source has N+1 elements: the existing\nfixture uses N-floor(d/2) shared elements and ceil(d/2) distinct\nelements. Each FULL frame therefore carries 8B more than d48.\nTwo receiver exact inventory snapshots combined add 16B\nto its cold serialized receiver write cost. This correctness\nerratum was added after the first incomplete CI run halted\non the d57 source-length assertion; **no d57 timing/result\nwas accepted before correction**, and matrix/threshold unchanged.\nPhysical gen2 WAL event/reconnection from two
fsync'd source children is **356B** (same
common event maintenance for exact/guard but
receiver WAL 304B logical + 3 sync calls is
needed only for hot exact). Owner seed/snapshot
disk writes included as common real source
provenance (do not present them as 'free').
Failure/resync rates, multi-epoch, hostile
authenticated peer, source crash after bitmap
initialization and power cut are OUT OF SCOPE.

Predeclared gate: all 5 workers, 60 sessions,
1200 sample rounds and every checksum/source
oracle PASS; per-worker fixture n20 nearest-rank
empirical p95, physical byte accounting,
strict B2A fixed SHA c9bdb9d185be29dd66335d5dfbe456fa562ccaf5c59a6f5f36a0d95f51d9e935,
source and receiver VmHWM/CPU clocks. ACCEPT only
`B1B1B1_B1B1B0_HOT_CONTRACT_COMPARISON_RESEARCH_ACCEPT`
for the particular warm service contract; **not**
SYSTEM_PRODUCT_GO. For product superiority require
>=10% all-in physical bytes vs the strongest
equally durable maintained-exact competitor AND
no per-worker n20 p95 regression for the actual
same application capability. These are explicitly
NOT comparable if one mode supplies only a
one-sided estimator and the other the exact set.

B1B1B1-B1B1-B1 under still-open #109 must add
Q1/Q10/Q100, 1/10/100 Mbps, 0/10/50ms,
full exact fallback on UNKNOWN, source/sink
fsync elapsed and true device logical I/O
and crash rates. #86 adaptive malicious
security and #69 product/public lifecycle
remain independent BLOCKERS.
