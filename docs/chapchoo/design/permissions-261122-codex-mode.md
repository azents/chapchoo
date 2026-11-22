---
title: "Codex Conversation Permission Mode Design"
created: 2026-11-22
implemented: 2026-11-22
document_role: primary
document_type: design
snapshot_id: permissions-261122
tags: [conversation, codex, security]
---

# Codex Conversation Permission Mode Design

## Design Authority

Revision 1. Source: [permissions-261122/REQ](../requirements/permissions-261122-codex-mode.md);
decision record: [permissions-261122/ADR](../adr/permissions-261122-codex-mode.md).

| ID | Class | Mechanism | Authority |
| --- | --- | --- | --- |
| M1 | required | Codex picker below Model; evaluate available defaults in the requested order. | REQ-1, REQ-2 |
| M2 | decided | Resolve eligibility through Codex and transmit the full preset on a new turn, never on steer. | REQ-1, REQ-2, ADR-D1 |
| M3 | decided | Independent preference edits, one exact-scope save and start fence, explicit Connector capability. | REQ-1, ADR-D2 |
| M4 | existing | Keep Computer isolation, conversation access, and tool authorization unchanged. | REQ fixed constraints |

## Interfaces and lifecycle

The existing conversation settings owner renders the mode independently of
model catalog contents. Control stores one typed permission choice, using the
same request identity and exact launch checks as model edits. A permission-only
save updates just the mode and records a receipt without a model. A model-only
save preserves that choice. Migration 33 preserves existing rows and allows
the existing receipt and revision fields to represent a permission-only save.
Downgrade rejects incompatible rows rather than dropping preference history.

The authenticated launch reports native availability. A qualified fresh start
uses the retained choice or resolves the default from that launch's reported
options. An explicit unavailable choice or a Connector unable to apply it
does not authorize a different start. A retained disabled sandbox cannot be
treated as an implicit fallback after failed mode discovery. Attach the
complete native preset to `turn/start`; use the existing exact `turn/steer`
without settings changes. Save does not start or replay input.

## Removal and Replacement

The old assumption that every settings receipt must name a model is replaced
by the same fenced receipt supporting a permission-only operation. Verify
both operations and old retained rows through SQLite and PostgreSQL tests.
No old data, harness, or unrelated UI is removed.

## Test Strategy

- The mandatory browser conversation journey uses a seeded Codex App Server
  fixture; check the default below Model and Save without input, then require
  the complete selected preset, existing model and tuning on the next turn.
- Lower layers cover fallback order, no mode changes on steer, saving without
  a catalog, preservation across independent edits, legacy-owner rejection,
  and original receipt recovery on both databases.
- The separate opt-in released Codex loopback run uses inert credentials,
  confirms exact resume and manual tool approval, and does not substitute
  for mandatory deterministic CI. No production account is needed.

## Design Approval

- Mode: direct implementation under requester-confirmed product scope.
- Product scope decision owner: requester; confirmed 2026-11-22.
- Recorded technical revision: 1; authority IDs: M1, M2, M3, M4.
- Scope: the three permission modes, specified placement and fallback order;
  no new operating policy. The requester did not separately review this
  technical record.
