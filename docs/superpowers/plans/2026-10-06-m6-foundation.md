# M6 Foundation Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task. Broader M6 work is tracked separately in issues #16–#19.

**Goal:** Make snapshot optimization reviewable through frozen-byte fixtures and paired hosted-runner evidence, then evaluate single-buffer encoding.
**Architecture:** Keep the root API and snapshot v1 unchanged. A private envelope writer owns one output buffer; public encoders calculate checked payload lengths and append directly. CRC, hash functions, decoders and profiles are unchanged in this slice.
**Tech Stack:** One stable-Rust crate, std only; GitHub Actions ubuntu-latest; Python stdlib for fixture derivation/evidence handling if needed.
**Spec:** [M6 design](../../M6-RECONCILIATION-AND-OPTIMIZATION.md).

## Global Constraints

- M6-0 preserves snapshot v1 bytes and interpretation; the user permits a separately justified format revision in a later slice.
- Energy remains conditional Proven; Parity remains Asymptotic.
- No unsafe, FFI, runtime dependency, profile or public API change.
- GitHub-hosted runners only; no local performance/acceptance substitution.
- No absolute timing CI gate or unmeasured acceleration claim.

## Review Focus

- Negative counters and continued updates after decode preserve bytes and estimates.
- Partial Parity words retain canonical padding, including nonempty multiword shapes.
- Invalid internally declared payload lengths fail before a malformed snapshot escapes.
- Identical benchmark binaries/workloads run for base/head; ordering and raw observations are recorded.
- Timing includes encode allocation/drop and decode allocation/drop; initialization is separately identified, not silently moved out of an end-to-end claim.

## Task 1 — Authority and compatibility guards

Files: docs/M6-RECONCILIATION-AND-OPTIMIZATION.md, docs/ROADMAP.md, docs/DESIGN.md, docs/research/OPEN-QUESTIONS.md, docs/research/DECISIONS.md, README.md, tests/snapshot_v1.rs.

- [ ] Link issues #14–#19, retain historical research and state exact candidate gates.
- [ ] Add independently specified nonempty Energy negative-counter and multiword Parity fixed byte vectors, verify canonical decode/reencode and continuation.
- [ ] Add all-profile snapshot length and continuation coverage with explicitly labelled deterministic fixtures (not empirical theorem evidence).
- [ ] Run existing and new compatibility tests in hosted CI. These are preservation tests and are expected to pass on the baseline; do not manufacture a failing correctness test for a performance-only change.

## Task 2 — Serialization measurement

Files: examples/snapshot_bench.rs, .github/workflows/snapshot-perf.yml.
Interface: cargo run --release --example snapshot_bench -- [samples] [repeats]; output stable CSV-like raw sample records, bytes, shapes and diagnostic contract.

- [ ] Measure encode/decode for Energy default-shape custom rows and a smaller profile shape, Parity Standard and non-J64/padded custom shape.
- [ ] Include empty, sequential and deterministic full-width keys; values are reproducible fixtures, not independent-uniform theorem draws.
- [ ] Black-box both input and output, warm up, emit every sample; fail on decode errors.
- [ ] Build the same harness source for pinned base/head, keep separate binaries and run AB then BA on the same hosted runner. Persist CPU/rustc, SHAs and raw data even if a later step fails.
- [ ] Validate format/clippy/tests/example compile through hosted Rust CI.

## Task 3 — Single-buffer candidate

Files: src/snapshot.rs, src/energy.rs, src/parity.rs.
Private interface: encode_envelope(backend: u8, payload_len: usize, write_payload: impl FnOnce(&mut Vec<u8>)) -> Result<Vec<u8>, SnapshotError>.

- [ ] Add tests for exact/incorrect closure payload length and usize::MAX envelope overflow, observing the missing-interface failure in hosted CI before implementation (compile-stage RED, not a behavioral assertion failure).
- [ ] Check total length and u64 conversion, reserve one output buffer, write header/payload, validate actual payload length and append unchanged CRC32C.
- [ ] Calculate Energy payload as 16+48R+8BR and Parity as 16+8*word_count with checked operations; write unchanged fields directly into the envelope.
- [ ] Run whole Rust suite, rustdoc, lint, examples and paired snapshot workflow. Preserve byte fixtures and decode behavior.
- [ ] Retain only with explicit review of resource reduction and latency evidence; revert candidate if a repeatable material regression appears. A noisy/neutral timing result is not a speedup claim.

## Task 4 — Review and durable handoff

- [ ] Independent branch review against spec, raw diff and compatibility tests.
- [ ] Address important findings; rerun affected hosted gates.
- [ ] Record exact candidate SHA, CI links, benchmark limitations and accepted/rejected result in docs/M6-0-EVIDENCE.md.
- [ ] PR closes #15 only; parent #14 and later slices stay open. Do not imply M6-A is implemented.

## Execution notes

2026-10-06: user explicitly requested audit, publication in issues/docs and implementation in this session. Execute the bounded foundation now without an additional approval loop. Local checkout is an isolated connector-fetched inspection copy; remote Git tree/commit/ref operations preserve actual upstream ancestry. Tests and timing run in GitHub Actions. Source formatting and diff inspection may run locally.
