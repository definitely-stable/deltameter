# Source snapshot — Qwen post-M3 strict-Parity report

Date received: 2026-10-06
Source form: user-provided text
Canonical status: evidence only; see ../../../STRICT-PARITY-POST-M3.md.

## Main claims

The report concludes NO-GO STRICT PARITY for the published W_i estimator while keeping a narrow ParityLevelCounts research path.

Strong mathematical contributions include:

- exact full-state Walsh/Fourier formula;
- exact one-level PGF and moments;
- exact Poissonized independent-cell/independent-row law;
- explicit fixed-d coefficient-extraction identity;
- individual-row W_i stochastic monotonicity;
- direct argument that W_i is not sufficient;
- whole-sketch finite-J truncation budget;
- small-d zero-coefficient floor;
- conditional union-bound construction using per-level parity counts S_j.

## Accepted contributions

Most of the report's mathematical decomposition is adopted after notation/indexing normalization.

Especially useful:

~~~text
published W_i strict track -> stop
ParityLevelCounts -> narrow research candidate
Energy -> only current Proven backend
~~~

The report correctly refuses to turn asymptotic RSE, Monte Carlo, or Poisson-only inference into fixed-d Proven coverage.

## Corrections

### Zero-observation floor is off by one in the examples

The theorem-implied requirement is:

~~~text
U(0) >= max { d : 2^-d > delta }.
~~~

For delta 1e-3, 1e-6, 1e-9 this gives 9, 19, 29 respectively.

Values 10, 20, 30 are conservative round-ups, not the exact forced floor.

### Full-statistic monotonicity remains open

The report correctly avoids promoting row monotonicity to the full statistic.

That cautious status is adopted.

### Alternative-backend conclusions stay conditional

The report does not justify an immediate REPLACE decision.

Simple Set Sketching, Minisketch and other candidates remain separate future audits.

## Architectural effect

This report provides the main shape of the canonical post-M3 synthesis:

- stop the published-W_i strict track;
- preserve exact Poissonized and truncation research artifacts;
- allow a narrow S_j / ParityLevelCounts research track;
- keep M4 unblocked.
