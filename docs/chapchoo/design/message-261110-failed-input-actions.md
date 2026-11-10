---
title: "Failed message recovery actions"
created: 2026-11-10
implemented: 2026-11-10
tags: [chat, recovery, persistence]
document_role: primary
document_type: design
snapshot_id: message-261110
---

# Failed Message Recovery Actions

Authority: [Requirements](../requirements/message-261110-failed-input-actions.md) and [ADR](../adr/message-261110-failed-input-actions.md). Direct implementation was requested after the request-identity/body-deletion explanation.

## Design Authority

- Design revision: `1`

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Explicit confirmed-failure retry uses a new submission ID and existing acceptance/processing ownership | REQ-1, ADR-D2, conversation-storage | derived |
| M2 | Logical body removal plus retained request/result tombstone | REQ-2, ADR-D1 | decided |
| M3 | Owner/source/known-failure checked deletion; uncertain forwarding cannot be deleted as failed | REQ-1–2, ADR-D3 | required |
| M4 | Existing common user-bubble rendering and chronological new-input placement | REQ-3, conversation-web | existing |

## Architecture and State

Persistence owns deletion transactions. Add nullable body storage and a deletion marker using an additive migration; preserve request ID, target and outcome. Old IDs are rejected before deleted-body decoding. Bounded lists retain bodyless deleted-ID tombstones so browser caches can remove old entries; body reads exclude deleted content. Tombstones carry identity and deletion state, never the removed text. No new retry scheduler or replay path exists: the browser invokes ordinary explicit Send with copied retained text and a new ID after selecting a confirmed-failure action. New submission is subject to existing admission/source fencing, including new-ID receipt recovery on lost responses. The original failed entry remains independently deletable.

Expose DeleteInput on the private owner conversation service with exact scope and request ID. Delete requires reported setup or input failure and no possible/successful native delivery. Deletion is idempotent; it cannot cancel work. Reload and polling hide deleted entries using monotonic tombstone merging. An acknowledged delete also clears the exact source-qualified cached operation even when that row is outside the current page. After a lost delete response, read-only recovery checks that exact operation's deletion result; it never assumes success or replays the delete automatically. Read-only status recovery remains separate from retry.

## Migration and Rollout

Preserve all existing bodies, identities, foreign keys and outcomes. SQLite locally rebuilds the body column using a finite temporary copy, without rebuilding operation identities; PostgreSQL relaxes the body column and adjusts its nonempty constraint. A guard blocks downgrade while deleted bodies cannot be restored. No deletion backfill or production mutation is performed by the agent. New client actions require the updated server.

## Security and Recovery

Recheck private owner, Team and current source in the transaction. Setup evidence is not text-delivery evidence. An uncertain send is read-only recoverable, never a failed-send retry button. Same-ID stale arrivals after deletion cannot enter processing. Deletion is logical database content removal, not physical page/backup erasure.

## Removal and Replacement

| Existing behavior | Removal authority | Replacement | Boundary | Verification |
| --- | --- | --- | --- | --- |
| Failed entries without retry/delete | REQ-1–2 | Explicit adjacent actions | owner Chat | component tests |
| Non-null body assumption for every accepted operation | ADR-D1 | Nullable body plus deletion marker; used identity remains | persistence/read/admission | both-engine tests |
| Merge-only retained input cache that revives deleted rows | REQ-2 | Deleted-ID reconciliation in bounded read projection | web-core | unit tests |

## Test Strategy

Use the existing minimal chat E2E journey for cross-service acceptance; no extra long/live-model matrix. Both-engine persistence tests cover delete, idempotency, authorization, source change, unknown forwarding rejection, original-ID rejection and preserved setup evidence. RPC checks verify the generated owner method. Web-core tests cover deletion reconciliation and retry identity; component fixtures cover actions, progress, uncertainty and localized mobile/desktop rendering. No missing prerequisite is counted as passed. Run Rust/RPC and TypeScript gates, mandatory hooks and exact-head CI. Existing temporary live authentication is not needed.

## Feasibility

Existing request uniqueness, typed setup/delivery facts, private input scopes, generated RPC and browser mutation infrastructure provide the required boundaries. The migration can preserve all referenced operation rows; no existing native transcript deletion is needed.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Approved on: `2026-11-10`
- Approved Design revision: `1`
- Approved authority IDs: `M1, M2, M3, M4`
- Approved scope: direct request to implement explicit new-ID retry and body deletion retaining used request identity/results after that mechanism was explained. No new runtime/replay mode.
