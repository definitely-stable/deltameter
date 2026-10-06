# DeltaMeter

DeltaMeter is a research-first Rust project for estimating the size of the symmetric difference between two sets.

For sets (A,B\subseteq[V]), the set-only model is:

```text
z = A XOR B
d = |A △ B| = ||z||_0
```

That is the primary model for this repository. The equivalent integer (F_2) view is useful as a reference, but it is intentionally not treated as the only or automatically optimal formulation.

## Status

Pre-implementation. The repository currently contains the research harness and initial design notes. No production Rust API is frozen yet.

The current implementation direction is deliberately small:

- **EnergyDeltaMeter** — proof-friendly reference/strict backend.
- **ParityDeltaMeter** — fast GF(2) research backend; finite-sample strict coverage is not claimed yet.
- Gaussian/chi-square constructions remain test or research oracles only.

We are explicitly **not** starting with a large framework, multiple crates, adaptive-adversary machinery, secret-key infrastructure, FFI, or hand-written SIMD. Those are only added if measurements or a concrete product requirement justify them.

## Repository layout

- [docs/RESEARCH.md](docs/RESEARCH.md) — mathematical conclusions and open research.
- [docs/DESIGN.md](docs/DESIGN.md) — minimal v0 design constraints.
- [docs/ROADMAP.md](docs/ROADMAP.md) — implementation order for one developer.
- [research/](research/) — dependency-free research scripts.
- [research workflow](.github/workflows/research.yml) — reproducible GitHub-hosted research run.

## Run the research harness

Requires Python 3. No third-party packages are needed.

```bash
python research/run_all.py --out research-out
```

The run writes machine-readable JSON for the conservative Energy profiles and finite-(d) F-PCSA level-moment checks.

## Project principles

1. Keep set semantics explicit: parity/toggle is not generic insert/remove semantics.
2. Do not label asymptotic or calibrated intervals as proven finite-sample coverage.
3. Prefer a small correct baseline over speculative infrastructure.
4. Stable Rust first; scalar code first; optimize after profiling.
5. Public GitHub-hosted CI is part of the normal development path, not a scarce resource.
6. Persisted formats and compatibility rules are frozen only after the state layout earns them.

## License / release

Not decided in this bootstrap slice.
