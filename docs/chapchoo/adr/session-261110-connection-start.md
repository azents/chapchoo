---
title: "Connection-Time Session Start"
created: 2026-11-10
document_role: primary
document_type: adr
snapshot_id: session-261110
tags: [conversation, connector, harness, architecture]
---

# Connection-Time Session Start

- Snapshot: `session-261110`
- Document reference: `session-261110/ADR`
- Requirements: [Connection-Time Session Start Requirements](../requirements/session-261110-connection-start.md)

## Fixed outcomes

The requester selected an empty session so harness preparation can finish before the first message, and directed immediate implementation. The same briefing fixed no synthetic prompt, no historical resend, no fresh replacement after a failed restore, authentication-required behavior, and idle retention until the Computer stops or the Connector ends. That confirmation is product scope, not a revision-bound Design approval.

Repository evidence closes the open cardinality assumption: each Agent has at most one selected conversation, and production insertion creates that conversation only with its selection pointer. A Computer may still have more than one Agent. No second selected conversation exists to choose.

The outcomes below follow [session-261110/REQ](../requirements/session-261110-connection-start.md) and the current Specs. A synthetic input or a process-only warmer cannot seal the required permission, and waiting for setup inside Connector stream admission cannot return the stream. Those alternatives are not viable. This record does not treat that conclusion as a separate requester approval of every implementation mechanism.

### session-261110/ADR-D1 — Sealed startup without an input

Connection-time preparation is a conversation operation with the same sealed supply, execution, fresh-context, and retained-source facts as input setup. It is not a user input and does not submit text.

A synthetic input cannot satisfy the empty-conversation and no-model-work requirements: accepted input text must be nonempty, and an input is a user-visible product operation. A process-only warmer cannot seal fresh or retained permission, and a stream callback cannot invent that permission. Historical startup rows are required because the fresh barrier and retained-source fence are computed from prior attempts, not from the current row alone.

### session-261110/ADR-D2 — Reconciliation dispatch after the stream is active

The Client group admits and dispatches startup only after a current Connector route exists and the Computer is desired running. Dispatch uses the existing reconciliation authority and proves the current Connector owner. It does not create a user session, borrow the Connector credential, or wait for setup inside Connector stream admission.

Setup queue admission is separate from native completion. The Connector stream must already be installed before any setup call is awaited. Stopping the Computer or ending the Connector keeps the existing native-owner retirement path; startup adds no second lifetime.
