---
title: "Codex Harness Cutover Decisions"
created: 2026-11-21
tags: [architecture, backend, migration]
document_role: primary
document_type: adr
snapshot_id: cutover-261121
---

# Codex Harness Cutover Decisions

## Confirmed scope

[cutover-261121/REQ](../requirements/cutover-261121-codex-harness.md)
requires one full Codex cutover, retaining exact identity and history for
existing conversations, leaving ACP available to other harnesses, and exposing
candidate-specific preferences. The requester confirmed direct App Server
implementation and clarified that Codex may dispatch already-admitted input.
The identity of the component sending the final start call is not a product
requirement. Admission, source ownership, and no uncertain replay still apply.

## cutover-261121/ADR-D1 — Use direct App Server for Codex

**Decision:** Use the official release over local stdio for both managed and
owner-supplied Codex. Preserve the owned workspace, native state, capture, and
execution admission. Use exact native resume; never create a substitute thread
when resume fails. Retain the ACP adapter and legacy capture interpretation.

Staying with the released ACP Codex bridge would keep its existing behavior
but could not provide the requested candidate-specific capability matrix.
Attaching a second App Server to the bridge's child has no established public
interface. A separately launched owner would break the single-owner boundary.
Released App Server preserves native history without translating stored
threads or introducing a second inference credential.

## Dispatch and preference decision map

- The branch reuses the existing Chapchoo input admission and delivery lane.
  This is an implementation choice under the existing publication contract,
  not a requirement that Chapchoo control native dispatch. The tested native
  queue persists entries and can dispatch an entry after an explicit start
  was rejected. Adopting that queue would require an explicit delivery and
  recovery mapping; adding it alongside the existing publisher risks a
  duplicate submission.
- Preserve the existing server-owned preference and insertion fence
  ([model-261120/ADR](model-261120-agent-model-preference.md)).
  Extend the draft and receipt together rather than creating a separately
  mutating effort or speed setting. Bind the tuple at the already-authorized
  publication and serialize it with the new-execution request. A start reply
  can describe an existing active turn; do not use that reply alone as evidence
  of new execution or of effective preferences.
- The exact migration rehearsal, retirement evidence, and Design revision
  review are still separate from authorization to implement this draft.
