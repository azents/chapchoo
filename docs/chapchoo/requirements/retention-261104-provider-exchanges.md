---
title: "Provider exchange history retention requirements"
created: 2026-11-04
tags: [provider, authentication, retention]
document_role: primary
document_type: requirements
snapshot_id: retention-261104
---

# Provider exchange history retention requirements

## Problem and Primary System Outcome

Five-minute Provider reauthentication permanently accumulates credential mutation
receipts. Bound live Provider history by an explicit retry horizon without
reinterpreting a deleted old request as fresh authority.

## Requirements

- **REQ-1:** Permit exact retry recovery for 15 minutes from request creation.
  After that boundary, reject the request even when its receipt is absent.
- **REQ-2:** Allow request creation times at most one minute ahead of server time.
  A request timestamp is admission metadata, never authentication evidence.
- **REQ-3:** On the next successful credential exchange, atomically remove that
  Provider's receipts whose retry period ended. No periodic cleanup worker is
  required. Inactive Providers retain their last receipts until activity resumes;
  deleting a Provider removes its associated history.
- **REQ-4:** Preserve exact token and original issuance/expiry for eligible retries,
  changed-intent conflicts, fresh authentication, ownership and native-effect
  checks. An expired request can be replaced only by a fresh authenticated request.
- **REQ-5:** Preserve current operational keys, canonical Provider/Computer/native
  and Connector identities during cutover. Connector policy does not change.

## Non-goals and Constraints

No UI, root-key rotation, periodic sweeper, configurable retention, rate-limit
policy or backwards-compatible opaque Provider request ID fallback. Bounded time
does not promise a fixed maximum number of receipts under arbitrarily high traffic.

## Requester Confirmation

The requester approved time-bearing request IDs, expiry-before-recovery,
cleanup during the next successful exchange and the inactive-Provider behavior,
then explicitly requested implementation. On 2026-11-04, the requester confirmed
the 15-minute retry horizon and one-minute maximum future clock tolerance.
