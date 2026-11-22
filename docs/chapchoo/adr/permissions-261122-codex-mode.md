---
title: "Codex Conversation Permission Mode Decisions"
created: 2026-11-22
document_role: primary
document_type: adr
snapshot_id: permissions-261122
tags: [conversation, codex, security]
---

# Codex Conversation Permission Mode Decisions

Authority: [permissions-261122/REQ](../requirements/permissions-261122-codex-mode.md).

## D1. Use native availability and a coherent preset at the turn boundary

The requester fixes the default order and excludes new operating policy. An
independent Chapchoo allowlist would add a second policy and could disagree with
Codex. Resolve eligibility from the connected Codex profile catalog, enabled
reviewer feature, and native config requirements. Apply profile, policy and
reviewer together to a newly started turn. Persist neither the inferred default
nor partial native settings. This also prevents a saved narrower preset from
inheriting a previous Full Access sandbox.

## D2. Keep each edit independent while reusing start ordering

A separate permissions dispatch lane could race the model preference
reservation. Instead, keep the existing compare-and-set, retained receipt and
start fence, with separate model and permission edits. Allow a permission-only
receipt without a model. The Connector must signal that it accepts permission
settings; protobuf presence alone does not keep an older Connector from
ignoring a newly added field. Refuse such a start rather than changing the
saved permission semantics.
