# DeltaMeter

DeltaMeter is a research-first Rust project for estimating the size of the symmetric difference between two sets.

For sets `A, B ⊆ [N]`:

```text
z = A XOR B
d = |A △ B| = ||z||_0
```

The equivalent integer `F2` representation remains useful, but the primary set-only model is GF(2) Hamming weight.

## Status

M0–M3 are complete. M3 adds the experimental/asymptotic ParityDeltaMeter wrapper.

Current implementation direction:

- **EnergyDeltaMeter** — first implementation and strict finite-sample backend.
- **PublishedFpcsaF2** — M2 faithful GF(2)-F-PCSA research reproduction.
- **ParityDeltaMeter** — M3 experimental/asymptotic wrapper over the published reproduction.
- Gaussian/chi-square — research oracle only.
- ExactSmallDelta — deferred.

Q1 is resolved for v0: there is currently **no accepted strict finite-sample Parity profile**. The published F-PCSA construction has been reproduced in M2; a `g(v)=1` set-specialized variant remains a separate unproved estimator.

## Documentation

- [docs/research/README.md](docs/research/README.md) — research index and authority order.
- [docs/research/Q1-FINITE-SAMPLE-PARITY.md](docs/research/Q1-FINITE-SAMPLE-PARITY.md) — current Q1 result.
- [docs/research/M2-FPCSA-REPRODUCTION.md](docs/research/M2-FPCSA-REPRODUCTION.md) — M2 source/implementation boundary.
- [docs/research/M3-PARITY-EXPERIMENTAL.md](docs/research/M3-PARITY-EXPERIMENTAL.md) — M3 API, profiles, guarantees and benchmark plan.
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

M3 comparison harness:

~~~bash
cargo run --release --example m3_compare -- 100000
~~~

Its timing output is diagnostic only; Energy and Parity have different guarantee contracts.

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
