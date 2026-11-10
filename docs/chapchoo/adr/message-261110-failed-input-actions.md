---
title: "Failed message recovery actions"
created: 2026-11-10
tags: [chat, recovery, persistence]
document_role: primary
document_type: adr
snapshot_id: message-261110
---

# Failed Message Recovery Actions

Authority: [message-261110/REQ](../requirements/message-261110-failed-input-actions.md).

## D1. Remove content while preserving request identity

The requester accepted logical body deletion with minimal request identity/outcome retention on 2026-11-10. Keep the accepted operation and setup/delivery outcome as an immutable replay barrier; null the submitted body and mark it deleted. Listing and body reads omit deleted messages. Repeated submission of a deleted request is rejected before decoding its deleted body.

Physical operation-row deletion would erase the unique identity and setup evidence, allowing a late duplicate to be admitted and weakening first-context recovery constraints. Merely hiding the body would not satisfy deletion. Database-page erasure and backups are not promised.

## D2. Explicit retry is a new fenced operation

The requester accepted new request identities for deliberate retries. The browser exposes retry for confirmed failures and submits their text through ordinary Send with a new identity. Existing server acceptance rechecks current private conversation/source in its transaction; retry gains no authority beyond a new explicit Send. Existing accepted processing ownership alone dispatches the new operation. Browser uncertainty performs read-only result recovery, not another submission. The old failed message remains as the retry anchor until its explicit deletion; the new attempt appears below in chronological order. This preserves both operation outcomes and avoids changing native transcript identity.

## D3. Destructive mutation is limited to settled non-delivery

Only reported setup failure or reported delivery failure, with no possible/delivered input outcome, permits deletion/retry. Missing ownership or absent delivery evidence alone is insufficient. Deletion is idempotent for the exact current scope and rechecks private owner/Team authority. It neither cancels work nor revokes credentials.
