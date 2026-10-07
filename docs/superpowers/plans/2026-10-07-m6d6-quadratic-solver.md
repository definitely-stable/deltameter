# M6-D6 — Deterministic quadratic root solver

Status: implementation/evidence complete in PR #35; **ACCEPT private, factor-tree dependent**. Issue: #34. Parent: #19 / #14.

## Goal

Replace only degree-two root factorization in the accepted D4 decoder and measure
whether the D5 profiling opportunity converts into real end-to-end speedup.

D5 measured degree-two local work as material:

~~~text
d=2  ~62.8% of D4 control
d=3  ~51.7%
d=4  ~40.9%
d=5   ~3.6%
d=8  ~27.5%
~~~

Verification is negligible at d=8 (~0.055%).

## Algebra

For a monic quadratic over GF(2^64):

~~~text
x^2 + a*x + b = 0
~~~

with a != 0, substitute x = a*y:

~~~text
y^2 + y = c
c = b / a^2
~~~

If y solves the Artin-Schreier equation, the two roots are:

~~~text
x0 = a*y
x1 = x0 + a
~~~

The candidate must verify both roots by direct polynomial evaluation before returning them.

## Artin-Schreier solver candidate

The field basis and reduction polynomial remain unchanged.

The linear map:

~~~text
L(y) = y^2 + y
~~~

is GF(2)-linear.

Its kernel is {0,1}; column zero is therefore the kernel basis element. Fix coefficient
bit zero of y to zero and solve for bits 1..63.

Candidate A uses deterministic binary Gaussian elimination over the fixed 64x63
matrix induced by L in the current polynomial basis.

Initial implementation may build/eliminate the matrix per solve. If this already
produces material end-to-end benefit, do not add caching in the same evidence slice.

A later isolated candidate may precompute the linear transform only if D6-A evidence
shows solve setup is the remaining degree-two cost.

## Frozen boundaries

Keep unchanged:

- degree >=3 trace splitting;
- D4 characteristic-2 trace-square;
- polynomial/GF(2^64) representation and 0x1B reduction;
- deterministic high-degree split coefficients;
- D2 1/2/4/8 stage schedule;
- 17/25/41/73-byte cumulative payloads;
- out-of-band zero;
- one guard syndrome;
- candidate all-syndrome verification;
- public API, snapshot-v1 and Coverage.

## Correctness gates

Quadratic unit/differential:

1. generate deterministic distinct nonzero root pairs;
2. construct monic polynomial (x+r0)(x+r1);
3. D6 solver returns exactly {r0,r1};
4. direct polynomial evaluation of both roots is zero;
5. compare against D4 specialized factor result.

Edge/failure:

- a == 0 repeated-root case fails closed;
- inconsistent Artin-Schreier RHS fails closed;
- zero roots are rejected by final candidate semantics;
- high-bit/u64::MAX coefficients covered;
- direct GF arithmetic reference vectors retained.

Full decoder:

- d=0/1/2/3/4/5/8 equals accepted D4 and exact oracle;
- d=9/10/16 remains reject;
- invalid-limit/error precedence unchanged;
- guard mutation still rejects;
- bytes/RTTs unchanged.

## Measurement

Three hosted processes, four balanced AB/BA samples per scenario.

Report:

- D4 complete decoder;
- D6 complete decoder;
- paired reduction;
- quadratic solver standalone latency over deterministic split quadratics;
- optionally Artin-Schreier setup/solve split if it can be measured without redesign;
- d=0/1 controls;
- d=2/3/4/5/8 exact cases;
- d=9/10/16 reject cases.

Use the same 8192-key maintained-state workload shape.

## Decision

ACCEPT only if:

- all correctness gates pass;
- d=2/3/4/8 show reproducible material improvement;
- d=8 end-to-end benefit is meaningful relative to D5's ~27.5% opportunity;
- no meaningful regression appears in d=0/1 or d=5;
- reject semantics remain unchanged.

NO-GO is valid.

Do not add matrix precomputation, unsafe/SIMD/CLMUL, alternative field representation
or another root specialization in the same evidence slice.


## Completion record

Canonical measured head: `1b982b9288f36a8a2e858ba9da846989a552360d`.

Hosted gates:

- M6-D6 #6 / `37586264293`;
- Rust #245 / `37586264300`;
- Research #258 / `37586264319`.

Canonical evidence: [M6-D6 quadratic evidence](../../M6-D6-QUADRATIC-EVIDENCE.md).

Verdict:

- deterministic quadratic solver correctness: ACCEPT;
- private decoder optimization: ACCEPT;
- performance is factor-tree dependent, not a universal fixed percentage;
- d=8 corpus medians: ~3.7%, ~15.3%, ~27.1%;
- d=2 stable ~39.6%, d=4 stable ~36.5-41.9%;
- per-solve Gaussian implementation ~10.2 us and not the next bottleneck;
- next step: post-D6 multi-corpus residual factor profile before any further solver specialization.
