---
title: "Provider exchange history retention decisions"
created: 2026-11-04
tags: [provider, authentication, retention]
document_role: primary
document_type: adr
snapshot_id: retention-261104
---

# Provider exchange history retention decisions

Requirements: [retention-261104/REQ](../requirements/retention-261104-provider-exchanges.md).

## D1 — Time-bearing request admission replaces indefinite opaque-ID history

Accepted by the requester on 2026-11-04. Retain exact request fingerprints only
within a 15-minute creation-time horizon, accept at most one minute of future
clock skew, and reject expired IDs before receipt lookup.

Indefinite arbitrary opaque-ID rejection needs unbounded membership history;
deleting those receipts alone would allow stale mutations to become fresh
issuance. Tombstones merely relocate growth. A time-bearing ID makes stale
rejection independent of retained rows. Its timestamp is not proof of identity:
each request still authenticates and revalidates canonical authority.

## D2 — Activity-owned database cleanup

Accepted by the requester on 2026-11-04. A successful Provider credential exchange
cleans that Provider's expired receipts in the same database transaction. No
timer or worker is introduced. Provider deletion keeps its existing cascade.
Failed authentication does not delete history. Inactive rows remain until the
next successful exchange or Provider deletion.

## D3 — Forward cutover preserves live authority, removes obsolete receipts

Derived from REQ-1 and REQ-5. Existing opaque request IDs become invalid under the
new admission contract, so their receipts are obsolete and removed by a forward
migration. Current keys and canonical product/Connector records remain intact.
An old Provider binary must be updated; no legacy receipt/admission path is kept.
