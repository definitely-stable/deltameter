# M6-D — ExactSmallDelta/hybrid audit plan

Status: source/failure-semantics audit in progress. Issue: #19. Parent: #14.

## Goal

Decide whether DeltaMeter should add an exact-small-difference lane, and if so which construction is worth prototyping without weakening the current guarantee boundary.

M6-A already recorded a NO-GO for a generic public snapshot-admission workflow. M6-D therefore evaluates the exact lane as an independent primitive first; it must earn production scope on its own.

## Candidate set

Primary candidates:

1. PinSketch / bitcoin-core Minisketch.
2. Simple Set Sketching (Houen, Pagh, Walzer).
3. classical IBLT.
4. Practical Rateless IBLT.

Current frontier comparisons:

5. CertainSync.
6. Self-Sizing IBLT.
7. XYZ-Sketch (2026).

The frontier items may change the architectural choice, but freshness is not evidence of production maturity.

## Required questions

For every construction record separately:

- what is deterministic versus high-probability;
- what is guaranteed when d is within the configured capacity;
- what happens when d exceeds capacity;
- whether a decoder can return a plausible but wrong set;
- how failure probability is parameterized;
- whether successful decode needs an independent verifier;
- key-domain restrictions and collision/mapping semantics;
- treatment of u64 zero;
- whether symmetric-difference direction is recovered directly;
- update, merge and decode complexity;
- communication/state size;
- whether capacity can grow without rebuilding from source data;
- whether the original set must remain available;
- adversarial-input / decode-DoS considerations;
- implementation language, unsafe/FFI implications and license;
- maturity of the available implementation.

## DeltaMeter invariants

A candidate must not silently change:

- the public u64 set domain;
- snapshot-v1 Energy/Parity semantics;
- Energy Coverage::Proven;
- Parity Coverage::Asymptotic;
- the no-false-success interpretation of an eventual exact result.

Do not add Coverage::Exact. Exact recovery belongs to a separate result/verification contract.

## Verification model

Separate these events:

1. estimator bound failure;
2. exact-sketch decoder failure;
3. decoder false success;
4. u64-to-backend mapping collision/loss;
5. final verification collision/failure;
6. transport/session failure.

A union bound is valid over individually justified event bounds; no independence assumption is required. A decoder reporting success is not itself final verification.

For the research lab, an exact oracle may verify recovered differences. A production verifier needs an independently justified contract and must not merely recompute the same syndrome that admitted the candidate solution.

## Decision gate

Phase 1 may conclude:

- NO-GO: no exact lane is worth further work;
- LAB-GO: one or more candidates deserve a private pure-Rust reference implementation;
- PRODUCTION-GO is explicitly impossible in this audit alone.

A LAB-GO must name the exact domain mapping, failure semantics, verifier, capacity range and benchmark comparison required before any public API.

## Current implementation constraints

- one stable-Rust crate;
- no unsafe/FFI/new runtime dependency in accepted production code;
- GitHub-hosted runners only;
- no public exact backend until a later measured GO;
- source pins and current-paper dates must be recorded.
