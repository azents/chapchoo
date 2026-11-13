---
title: "Direct Conversation Sender Requirements"
created: 2026-11-13
document_role: primary
document_type: requirements
snapshot_id: sender-261113
tags: [tools, mcp, harness]
---

# Direct Conversation Sender Requirements

- Snapshot: `sender-261113`
- Document reference: `sender-261113/REQ`
- Scope: the independent sender correction within the wider
  [operation-result requirements](outcome-261113-bounded-operation-results.md).

## Problem and Primary System Outcome

The essential conversation sender must use the approved name and be available to
the native model without tool discovery. A production-name correction must not
make existing persisted conversation history lose its intentional-publication
meaning.

## Requirements

### REQ-1. Approved production identity

The sole production conversation-publication tool is `send_message`. Catalog,
validation, dispatch, fixtures and new captured records agree on that identity.
The incorrect name is not an executable compatibility alias.

### REQ-2. Initial direct model exposure

The Chapchoo tool namespace is present directly in the initial model tool list on
actual New and exact Resume. Other integration namespaces keep their normal
policy. The existing scoped MCP transport, fixed recipient, authorization and
explicit-publication boundary remain intact.

### REQ-3. Preserve existing records

Persisted records from before the correction remain readable as intentional
publications, with their original input/result available. Read-only recognition
of historical evidence does not permit invocation under an old name or rewrite
stored records.

### REQ-4. Honest verification

Owned tests verify the contract and actual adapter configuration propagation.
Transport/catalog readiness, controlled native fixtures and real model-side use
are reported as different evidence. No live user conversation is invoked merely
to test the correction.

## Non-Goals and Fixed Constraints

No new recipient routing, native restart, fresh-context fallback, execution alias,
credential store, persistent result API, merge or deployment. Existing conversation
bytes, private authority and historical implemented snapshots are preserved.

## Confirmation

On 2026-11-13 KST the requester reaffirmed the previously approved `send_message`
identity and explicitly requested implementation of the earlier feedback,
including the immediately preceding direct-only namespace policy proposal. The
requester additionally required meaningful independent PRs. REQ-3 preserves the
existing readable-history contract and data-preservation constraint; it is not a
request to keep an executable alias. This document records that existing
instruction rather than claiming that the wider state-machine work is complete.
