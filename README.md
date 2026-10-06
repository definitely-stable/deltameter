# DeltaMeter

DeltaMeter is a research-first Rust project for estimating the size of the symmetric difference between two sets.

For sets `A, B ⊆ [N]`:

```text
z = A XOR B
d = |A △ B| = ||z||_0
```

The equivalent integer `F2` representation remains useful, but the primary set-only model is GF(2) Hamming weight.

## Status

M0 research bootstrap and M1 EnergyDeltaMeter are merged. M2 published GF(2)-F-PCSA reproduction is active.

Current implementation direction:

- **EnergyDeltaMeter** — first implementation and strict finite-sample backend.
- **PublishedFpcsaF2** — M2 faithful GF(2)-F-PCSA research reproduction.
- **ParityDeltaMeter** — later M3 wrapper, experimental/asymptotic.
- Gaussian/chi-square — research oracle only.
- ExactSmallDelta — deferred.

Q1 is resolved for v0: there is currently **no accepted strict finite-sample Parity profile**. The published F-PCSA construction will be reproduced first; a `g(v)=1` set-specialized variant is treated as a separate unproved estimator.

## Documentation

- [docs/research/README.md](docs/research/README.md) — research index and authority order.
- [docs/research/Q1-FINITE-SAMPLE-PARITY.md](docs/research/Q1-FINITE-SAMPLE-PARITY.md) — current Q1 result.
- [docs/research/M2-FPCSA-REPRODUCTION.md](docs/research/M2-FPCSA-REPRODUCTION.md) — M2 source/implementation boundary.
- [docs/research/FOUNDATION.md](docs/research/FOUNDATION.md) — canonical mathematical/engineering synthesis.
- [docs/research/DECISIONS.md](docs/research/DECISIONS.md) — current decisions.
- [docs/research/NEW-INPUTS-AUDIT.md](docs/research/NEW-INPUTS-AUDIT.md) — reconciliation of the latest reports.
- [docs/research/OPEN-QUESTIONS.md](docs/research/OPEN-QUESTIONS.md) — remaining research.
- [docs/research/archive/2026-10-06/README.md](docs/research/archive/2026-10-06/README.md) — latest Deep Research + attached-report register.
- [docs/DESIGN.md](docs/DESIGN.md) — minimal v0 design.
- [docs/ROADMAP.md](docs/ROADMAP.md) — implementation sequence.
- [research/](research/) — dependency-free executable research scripts.

## Run the research harness

Requires Python 3. No third-party packages are needed.

```bash
python research/run_all.py --out research-out
python research/energy_profiles.py --check-csv research/energy_profiles.csv
```

The committed CSV is the machine-readable bridge between the theorem-derived Python profile generator and Rust profile tests.

## Project principles

1. Set/toggle semantics stay explicit.
2. `Proven`, `Asymptotic` and `Empirical` are different contracts.
3. Prefer a small correct baseline over speculative infrastructure.
4. One crate, stable Rust, scalar first.
5. Public GitHub-hosted CI is used normally; no self-hosted runner requirement.
6. No default crypto/key-management subsystem.
7. No FFI/exact-recovery subsystem until a concrete need is demonstrated.
8. Persisted formats are frozen only after estimator state/API stabilization.
9. Historical research is preserved, but current decisions have an explicit authority order.

## License / release

Not decided in this bootstrap slice.
