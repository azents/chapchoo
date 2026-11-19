---
title: "Implicit Conversation Reply Work Decisions"
created: 2026-11-19
tags: [backend, engine]
document_role: primary
document_type: adr
snapshot_id: reply-261119
---

# ADR-D1. Extend existing Conversation Work at input acceptance

Use the existing conversation-scoped Goal and Work lifetime, rather than
creating a reply Goal alongside a separately settled plan. The input
transaction opens or resumes that Work. Plan publication is independent of
existence; a Work row with no published title has no summary and projects
only its bounded internal objective.

Serialize settlement with input acceptance. Use the existing ordinary-input
queue to prevent `finish`, `ignore`, or `request_input` from closing or
pausing Work while a user input still awaits delivery. Send requested Chat
output in the same transaction, without attempting to infer which already
delivered messages were semantically answered from their text.

Keep eligibility separate from Work existence. A successful input pickup
qualifies the reply objective; a blocked wait or settlement consumes prior
readiness without dropping the remaining input. This prevents failed delivery
from creating an actionable reply context. Reuse the launch-local input report
fence to order ending admission after reports of dispatched input, without
waiting on inputs parked behind that ending.

This implements `reply-261119/REQ-1`–`REQ-3` using the existing private
conversation authority and user-input priority, without changing tool inputs,
RPC, or native packages.
