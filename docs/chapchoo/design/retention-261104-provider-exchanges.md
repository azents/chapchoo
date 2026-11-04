---
title: "Provider exchange history retention design"
created: 2026-11-04
implemented: 2026-11-04
tags: [provider, authentication, retention]
document_role: primary
document_type: design
snapshot_id: retention-261104
---

# Provider exchange history retention design

Requirements: [retention-261104/REQ](../requirements/retention-261104-provider-exchanges.md).
Decisions: [retention-261104/ADR](../adr/retention-261104-provider-exchanges.md).

## Design Authority

Design revision: **1**.

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Time-bearing ID, 15-minute retry horizon, one-minute future tolerance | REQ-1/2, ADR-D1 | required |
| M2 | Fresh authority plus expiry-before-receipt lookup; preserve exact recovery and conflict behavior inside horizon | REQ-4, ADR-D1 | required |
| M3 | Indexed per-Provider expired receipt cleanup in successful credential mutation transaction; existing deletion cascade | REQ-3, ADR-D2 | required |
| M4 | Forward migration removes obsolete opaque receipts, preserves current keys and canonical data; update Provider caller/fixtures | REQ-5, ADR-D3 | derived |

## Architecture and State

A domain value parses a canonical nonnegative millisecond Unix timestamp and
opaque nonce. The wire representation is `<created-at-ms>:<nonce>`. Runtime
callers reuse their existing cryptographically random ID generator for the nonce;
no UUID dependency or independent clock service is required. Equivalent encoding
details are local implementation choices, not new authentication authority.

Storage validates the syntax and acceptance instant for Provider rotation and
revocation. Creation later than `now + 60_000` rejects. `now >= created + 900_000`
rejects before reading receipts. Overflow or negative instants reject. Every
eligible request still authenticates and rechecks owner/native binding. Within
the horizon, superseded or revoked results reject and changed intent conflicts.

Receipts carry their absolute retry expiry with an index on Provider and expiry.
Successful rotation (including exact recovery) and revocation remove only that
Provider's expired receipts in their repository-owned transaction. Current-key
overlap, operational expiry and Connector behavior are unchanged.

Provider retries keep one ID for an uncertain exchange. New exchange attempts
after the retry horizon use a new authenticated request; no uncertain native
lifecycle operation is replayed. Runtime creates the next pending ID only when
exchange is needed, not five minutes before its first send.

## Migration and Operations

Migration 8 replaces the receipt table and adds the expiry index on SQLite and
PostgreSQL. Existing opaque receipts cannot be admitted by the replacement
contract; current keys and all other tables are preserved. Rolling back schema
cannot restore removed receipts, so down migration refuses rather than enabling
old callers without replay protection. Prefer forward rollout; old callers fail closed.
Clock skew beyond one minute is an operator configuration error, not a fallback
to unrestricted IDs. Retention bounds history duration, not authenticated traffic.

## Removal and Replacement

| Existing behavior | Authority | Replacement | Boundary | Verification |
| --- | --- | --- | --- | --- |
| Indefinite opaque Provider request history | REQ-1/3, ADR-D1 | Time-scoped receipt admission and cleanup | Domain, storage, Provider caller | Stale ID after physical deletion still rejects |
| Receipt rows without expiry/index | ADR-D3 | Expiry-bearing table | Forward migration 8 | Both engines preserve live keys and canonical data |
| Timeless fixture requests and acceptance client | M1/M4 | Explicit deterministic time-bearing IDs | Provider credential tests only | Connector and lifecycle IDs unchanged |

## Test Strategy

Primary integration evidence is actual RPC authentication/recovery against both
SQLite and PostgreSQL plus the existing disposable kind Provider lifecycle lane
in CI. Unit/domain and storage tests own precise clock boundaries and physical
row pruning; no 15-minute wall-clock sleep or additional browser E2E is needed.
Use injected acceptance instants and isolated DB fixtures, not production secrets.
Test exact recovery, changed payload, superseded/current/expired IDs, future
boundary/overflow, cross-Provider isolation, atomic rollback, and deletion
cascade. Migrate populated version-7 databases and compare live key/native data.
Run Rust/RPC/chart gates and current-SHA CI; skipped live checks are not passes.

## Authority and Feasibility

Feasible: Provider IDs are opaque strings today, callers own stable retry IDs,
repository operations already own transactions and both engines support the
indexed receipt replacement. A timestamp prefix reuses existing nonce generation
without changing token cryptography. No unapproved scope or alternate authority
is introduced.

## Design Approval

- Mode: Collaborative.
- Decision owner: requester.
- Approved on: 2026-11-04.
- Approved Design revision: 1.
- Approved authority IDs: M1, M2, M3, M4.
- Approved scope: requester-approved time-bearing ID retry/cleanup flow and
  explicit confirmation of 15 minutes / one minute, followed by implementation.
  Forward obsolete-receipt cutover and local encoding follow those fixed outcomes.
