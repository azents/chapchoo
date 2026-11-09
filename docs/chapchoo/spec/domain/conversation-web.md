---
title: "Private conversation reader and controls"
---

# Private conversation reader and controls

The General Chat surface presents source-bound native observations and retained
product operations. It is not a local transcript simulator, a durable browser
feed or evidence that credentials or a model work.

## Initial rendering and active reads

Exact-Team/private-owner route admission loads both the Agent and initial
conversation region through the request-owned server session. The HTML renders
that region and hydrates the same values. Reads do not create a conversation,
power on a Computer, prepare an installation or start native work.

One client read owner continues on a one-second cadence while the document is
visible and the account remains authorized. It coalesces concurrent refreshes,
allows at most one read in flight, and rereads on focus/online activation.
Hydration does not issue an immediate duplicate read. Hidden views stop scheduling
new reads. Unmounted owners discard late responses. Request authentication,
Origin/POST and cookie disposition remain owned by the existing web boundary.

The client sends bounded known-record/update hints, not a durable cursor.
Ordinal, paging index and accepted-time request values use canonical int64 decimal
strings so browser query keys stay JSON-safe; server ingress validates syntax and
range before restoring exact bigint values. This is not floating-point coercion. It
assembles complete field bytes before UTF-8 decoding, validates the empty final
suffix under the same source authority, and rejects changing field revisions.
Recent/older records and retained input pages use source-bound anchors. The
current plan has its own read even outside the recent region. Selecting history
in the activity sheet scrolls to its record after the sheet's exit/focus boundary.
Read-only retry never resubmits input, an answer or Stop.

Conversation identity and source snapshot are compared independently. A replaced
source clears old live content and paging before the new source refills. Ordinary
loading/error feedback does not fabricate a new conversation or native context.
Same-source updates retain original record order and update the matching native
message. Viewport anchoring preserves an existing record's screen offset while
the reader is detached; Jump to latest restores following.

## Text, requests and retained operation facts

User content is whitespace-preserving plain text. Assistant utterances use
Markdown without raw HTML or automatic external-image loading. Supported media
have explicit previews and inspectable original base64. Terminal pieces are
assembled as bytes before text decoding and retain exact base64 independently
of display replacement characters. Plans, tools, reasoning, locations, notices
and terminal results remain inspectable activity rather than invented prose.

Send uploads exact UTF-8 pieces through the generated owner client. Acceptance,
preparation, possible forwarding, inserted native identity, steering, known
failure and missing evidence remain different observations. Original submitted
text survives navigation through the backend. An exact inserted native message
ID can associate a visible user record with its retained input details; textual
similarity is not identity. Steering does not manufacture a native message ID.

Permission/form requests render inline. Original choices, their descriptions,
schema/field titles and supported adjacent constraints remain inspectable.
Permission is not preselected. Form drafts distinguish omission, empty strings,
false, empty selections and exact int64 values. Native defaults may seed a draft;
only explicit submission accepts it. Standard format feedback is advisory; server
and native validation remain authoritative, including bounded offline string
patterns and supported formats. Unsupported required fields cannot be answered
as an ordinary form. Password-format collection is rejected.

A historical request is not responder authority. Only the matching current live
responder enables a new answer, and native dequeue revalidates it. The retained
original typed answer remains discoverable after responder closure, bound to
source and occurrence. Original choices and selected answers stay expanded rather
than being replaced by a summary. Lost mutation responses retain their exact
operation/source and use read-only result recovery; there is no automatic effect
retry or browser offline queue.

Current-work Stop is beside the execution summary and uses the exact native
source/session/work token. Possible forwarding, cancellation publication and
rejection remain distinct; publication does not prove termination. It neither
powers off the Computer nor retires the harness. Computer power controls retain
their existing eligibility and independent confirmation contracts.

## Owner supply settings

The activity sheet has a separate Codex setup control. Its supplementary read is
activated when the owner opens it, so settings availability does not replace the
primary server-rendered conversation. Managed preparation is the server default;
existing mode preserves the exact absolute executable path. The owner can save a
choice with the original expected selection Request ID while a Computer is off
or native work is running. Saving changes only the retained owner selection.

The UI separately shows the saved choice and the actual-start choice reported by
the execution-local native owner under the current source. Missing applied facts
remain unobserved. A saved/applied difference names next-process-start application,
not next-message adoption, successful credentials or current process liveness.
Saving never powers a Computer, starts/stops/restarts a harness or refreshes native
authentication. A retained capture can span physical native starts; the native
owner freezes its choice for each live process while newer ordered observations
may describe a later start.

Known rejection preserves the draft and rereads current saved state. An uncertain
save freezes editing and exposes read-only selection recovery. Only an observed
matching accepted Request ID confirms that operation; a different current choice
is not proof that the earlier save never occurred. A post-save read failure does
not revoke an acknowledged save. No mutation is automatically replayed.

## Connections

- [Conversation storage and acceptance](conversation-storage.md) — backend ownership, byte bounds, original answers and source fencing.
- [Computer owner/operator web](computer-web.md) — real power observations, navigation and separate confirmation flows.
- [Primary rendering](web-rendering.md) — resolved server data and client continuation ownership.
- [Web sessions](web-session.md) — request credentials, account withdrawal and cookie authority.
- [Native lifetime](native-harness.md) — actual preparation/session ownership and whole-scope retirement.
- [Verification boundaries](acceptance.md) — component, deterministic protocol fixture and real-model evidence are distinct.

## References

- [`Chat server loader`](../../../../typescript/apps/web/src/features/computers/chat/server/loadChat.ts)
- [`Conversation read owner`](../../../../typescript/apps/web/src/features/computers/chat/containers/useConversationReader.ts)
- [`Conversation renderer`](../../../../typescript/apps/web/src/features/computers/chat/components/ConversationBody.tsx)
- [`Generated-client boundary`](../../../../typescript/packages/web-core/src/conversation.ts)
