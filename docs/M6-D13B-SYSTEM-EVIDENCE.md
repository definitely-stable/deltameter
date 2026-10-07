# M6-D13-B — maintained-state system comparison evidence

Issue #56; parents #19/#14. Baseline:
`8b5421709965e88dfbb74cd61eb742321d738efa` (merged D13-B0
MEASUREMENT_READY).

Verdict: **STOP_SYSTEM_PRODUCT**.

This is a private system-research verdict under D13-A protocol v2 and D13-B
contract v2. It is not a theorem over all possible reconciliation protocols and it
does not authorize a public ExactSmallDelta backend.

## Provenance and validity

The first hosted attempt, run `37650626996`, is excluded as **INVALID**. All five
workers stopped at the mandatory odd d=9 boundary because the reused untimed B0
ready-validator assumed equal A/B cardinality. No complete raw worker artifact from
that attempt is eligible for evidence.

Protocol v2 froze the validator-only correction before replacement timing:
pair readiness checks the controller-known `|A|` and `|B|` separately. Native
workers, native timers, data matrix, wire model and decision gates were unchanged.

Canonical measurement-source head:

`3161119129bb033c39ad5c403ca46de14b7664b9`

Hosted run:

- M6-D13-B system comparison #6 / `37651049535` — **SUCCESS**;
- Research #405 / `37651049572` — **SUCCESS**;
- five independent hosted timing workers — **5/5 SUCCESS**;
- three deterministic replicate families per worker;
- raw observations — **3,900**;
- named modeled network/amortization cells — **360**;
- D11 qualifying cells — **0**;
- RIBLT stream-lower-bound qualifying cells — **0**.

Canonical artifacts:

~~~text
summary
  id      11497205069
  sha256  ace4c7e31351aec929e224f1095c2b5d3ac162793c965defa83b9b06edeec07a

worker 1
  id      11497090116
  sha256  78915c3b33fac540a82df6779d38749ba3d2d6fd67334392b3563c9626dbbaca

worker 2
  id      11496870388
  sha256  06bd471b7c5f536479304c5b38aec2430669b8e02f1747b7ac18559958110744

worker 3
  id      11496104114
  sha256  3d457d5640d831ca2ee9f987c2ca9eb78fbc50922ed8021cc734916a50ebec93

worker 4
  id      11495814188
  sha256  83772995b4ef489b8dbc72164f71448cd20bacd41432477daa600e8e3d2f8c49

worker 5
  id      11497165108
  sha256  22fd3f85d63149bff3294d83cacf90c71afb3a97a25ef76262323938a4692366
~~~

All timing jobs verify the frozen measurement-source manifest before measurement.

## Frozen decision result

The predeclared private-candidate gate requires a candidate to reduce modeled total
cost by at least 10% versus direct exact on **every** hosted worker for the same
named `(N,U,S,RTT,bandwidth)` cell.

No D11 cell passes. No optimistic `riblt_stream_lb` cell passes.

The most favorable cell for both alternatives is already a loss:

~~~text
N                  65,536
updates/session    1
sessions/build     100
RTT                0 ms
bandwidth          1 Mbps

D11:
  minimum worker improvement    -0.0424%
  median worker improvement     -0.0356%

RIBLT stream_lb:
  minimum worker improvement    -0.4352%
  median worker improvement     -0.2978%
~~~

Negative improvement means slower/more expensive than direct exact. Therefore the
result is not a near miss against the 10% gate: even the best named candidate cell
fails to beat direct at all on every worker.

At high bandwidth/zero RTT, compute cost makes the separation much larger. For
example at N=65,536, U=8, S=100, RTT=0, 1 Gbps, the median D11 modeled regression
is about 72%, while the optimistic stream-lower-bound regression is about 920%.

## Why direct wins

Under the frozen exact-verification contract, a sketch-derived result is
provisional. It must send its reconstructed canonical list back for independent
verification. Direct exact already sends that canonical list once as the
synchronization representation.

Consequently, even a successful sketch session has no network-byte advantage over
direct:

- direct: one full list plus one request/response header pair;
- D11/RIBLT: sketch/coded-symbol traffic **plus** one full list for verification;
- sketch paths also require additional feedback RTTs.

The hosted evidence confirms this structural cost instead of merely relying on the
accounting formula.

### N=65,536 boundary

Median over 15 hosted process/family observations:

~~~text
d=8
                 native        app bytes   rounds
direct           0.363 ms       524,392      1
D11              2.889 ms       524,849      5
RIBLT pull      48.889 ms       525,256      6
RIBLT stream_lb 40.867 ms       524,776      2

d=9
direct           0.365 ms       524,400      1
D11              6.204 ms       524,857      5   (15/15 exact fallbacks)
RIBLT pull      50.743 ms       525,264      6
RIBLT stream_lb 43.117 ms       524,880      2
~~~

D11 therefore crosses 8→9 safely but not cheaply. d=8 had no fallback; d=9
fell back in all 15 observations, exactly as the capacity contract requires.

## Core maintained-state evidence

Representative median native per-session values across workers:

~~~text
N=8,192
U=1
  direct             ~0.009 ms
  D11                ~0.014 ms
  RIBLT stream_lb    ~1.186 ms

U=8
  direct             ~0.011 ms
  D11                ~2.700 ms
  RIBLT stream_lb    ~4.557 ms

U=64
  direct             ~0.036 ms
  D11                ~5.614 ms
  RIBLT stream_lb    ~9.208 ms

N=65,536
U=1
  direct             ~0.073 ms
  D11                ~0.060 ms
  RIBLT stream_lb   ~11.604 ms

U=8
  direct             ~0.094 ms
  D11                ~2.606 ms
  RIBLT stream_lb   ~39.514 ms

U=64
  direct             ~0.274 ms
  D11                ~6.219 ms
  RIBLT stream_lb   ~83.520 ms
~~~

The N=65,536/U=1 lane is informative: D11 native compute can be slightly below
direct in a tiny-d maintained case. It still loses the full modeled system cost
because it must send the full verification list plus sketch bytes and pays another
RTT. This separates a useful small-d primitive from a useful end-to-end protocol.

Across the core matrix D11 records 300 expected U=64 over-capacity fallbacks and no
performance-matrix false candidates. The deterministic B0 d=8/k=1
false-candidate witness remains binding evidence that the extra-syndrome guard is
empirical, not a theorem; its absence here is not a universal guarantee.

## One-million-key scaling guard

Median over 15 hosted observations:

~~~text
N=1,048,576

d=128
  direct             3.222 ms     ~35.9 MiB HWM
  D11               10.186 ms     ~34.2 MiB HWM   15/15 fallback
  RIBLT stream_lb 3342.365 ms    ~260.6 MiB HWM

d=512
  direct             3.288 ms     ~34.1 MiB HWM
  D11                9.712 ms     ~34.2 MiB HWM   15/15 fallback
  RIBLT stream_lb 4946.501 ms    ~266.5 MiB HWM

d=1024
  direct             3.328 ms     ~34.2 MiB HWM
  D11               10.391 ms     ~34.2 MiB HWM   15/15 fallback
  RIBLT stream_lb 5329.364 ms    ~260.3 MiB HWM   15/15 cap fallback
~~~

The pinned RIBLT comparator pays full source import/hash/scheduler work every
session, as required by the upstream lifecycle. A different maintained rateless
implementation could change that cost, but it would be a new implementation and a
new evidence program. The current optimistic streaming transport already removes
avoidable pull RTTs and still does not produce a qualifying cell.

## Decision

**STOP_SYSTEM_PRODUCT** for the evaluated reconciliation-backend program.

Specifically:

- do not productionize D11/ExactSmallDelta from M6;
- do not add a public reconciliation/network API from this evidence;
- do not pivot M6 to the pinned RIBLT implementation;
- do not resume algebraic D11 optimization: D12's STOP remains binding;
- retain D11 and its accepted optimizations as private research/reference code and
  evidence;
- retain direct exact as the system baseline for the current exact-verification
  contract;
- close M6-D after this result.

This does not prove that all future set reconciliation is useless. A materially
different product contract could reopen the question, especially a compact
cryptographic/probabilistic verification tier with an explicit collision/error
budget, or a genuinely maintained rateless implementation that avoids per-session
source reimport. Either would be a new protocol and must not inherit M6's Exact or
performance claims.

Production/public ExactSmallDelta remains **NO-GO**.
