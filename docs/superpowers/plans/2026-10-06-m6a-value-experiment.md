# M6-A — Estimator-assisted value experiment plan

Status: implementation in progress. Issue: #16. Parent: #14.

## Goal

Measure whether a DeltaMeter snapshot can make a useful distributed admission decision before an expensive exact-set exchange.

This slice does **not** recover missing elements and does not add a public network API.

## Experiment arms

For the same immutable peer set B and source set A:

1. **direct-exact** — send canonical sorted A, peer computes exact |A △ B|.
2. **snapshot-control** — send an Energy snapshot, receive an estimate, then send the identical exact A regardless of the estimate.
3. **snapshot-admission** — send the same snapshot first:
   - if the predeclared point-estimate policy admits, send A and require the exact oracle result;
   - if it rejects, stop before exact transfer and record a rejected decision, never a successful reconciliation.

The control arm proves the unavoidable overhead when a snapshot does not change the action.

## Guarantee boundary

CI workloads use deterministic custom Energy rows so runs are reproducible. They are **not** theorem draws and the admission decision is empirical.

M6-A therefore records exact-oracle false-admit/false-reject outcomes instead of presenting the policy as Coverage::Proven.

A theorem-backed production admission policy would require a separately trusted uniform-row provenance source and a predeclared finite query/error budget.

## Private experiment protocol

The examples use bounded length-prefixed stdin/stdout framing.

Every message carries:

- protocol magic/version;
- message kind;
- session id;
- dataset id;
- source generation;
- peer generation;
- config id;
- strictly increasing request id.

The peer rejects:

- wrong session/dataset/peer generation/config identity;
- stale source generation;
- replayed/non-increasing request ids;
- oversized frames;
- malformed or non-canonical exact sets;
- snapshot/config mismatch.

Responses echo request identity. A reject/error is never encoded as success.

## Fixture boundary

The harness owns exact oracle data outside the measured decision path.

The peer loads its immutable canonical B set from a local fixture file before protocol measurement. This file is harness setup, not network traffic.

Two peer modes:

- direct: exact set only;
- sketch: exact set plus maintained custom Energy sketch.

Peer startup reports setup time so estimator-maintenance cost is visible.

## Workloads

Fixed, nonadaptive workloads use full-width u64 keys and cover:

- d = 0, 1, 2, 8, 32, 128;
- above-threshold d;
- large d;
- disjoint sets;
- source/peer cardinality asymmetry.

The initial exact-admission threshold is declared in the harness output before observations.

## Accounting

Record per arm/scenario:

- source and peer cardinalities;
- exact d;
- estimator point;
- admission decision;
- exact oracle success/reject;
- request/response application bytes;
- round trips;
- local elapsed time;
- source sketch build time;
- peer setup time;
- exact bytes avoided on reject;
- false-admit / false-reject classification against the declared exact threshold.

RTT is reported as round-trip count, not hidden inside local subprocess timing.

## Resource and process safety

- maximum frame length is fixed before allocation;
- EOF and malformed input fail closed;
- harness response waits are bounded;
- timeout kills and reaps the child;
- Drop cleanup prevents orphan peers;
- fixture files are removed after each run.

## Acceptance for the foundation slice

- direct and snapshot-control exact results equal the harness oracle;
- an admitted snapshot path must also finish with the exact result;
- a rejected path never claims exact completion;
- replay/stale/wrong-identity tests fail closed;
- timeout path kills/reaps a deliberately stalled peer;
- physical application bytes and round trips are deterministic;
- no public library API, snapshot format, estimator math or coverage class changes;
- GitHub-hosted CI only.

## Decision gate

M6-A GO requires a named workload where the snapshot changes an action and the saved rejected-work cost exceeds snapshot/setup overhead under a declared reuse/RTT model.

Otherwise record NO-GO for the workflow and prefer estimator-only scope or the independently audited exact lane.
