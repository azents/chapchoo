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

The current transcript read owner continues on a one-second cadence while the
document is visible and the account remains authorized. Full history mounts its
own source-bound read owner, seeded from the current admitted region, so paging
older history does not replace current-head polling. Each owner coalesces refreshes,
allows at most one read in flight, and rereads on focus/online activation.
Hydration does not issue an immediate duplicate read. Hidden documents stop scheduling
new reads. Unmounted owners discard late responses. Request authentication,
Origin/POST and cookie disposition remain owned by the existing web boundary.

The client sends bounded known-record/update hints, not a durable cursor.
Ordinal, paging index and accepted-time request values use canonical int64 decimal
strings so browser query keys stay JSON-safe; server ingress validates syntax and
range before restoring exact bigint values. This is not floating-point coercion. It
assembles complete field bytes before UTF-8 decoding and validates the empty final
suffix under the same source authority. One initial coherent field metadata view
selects each finite extent, so appends can continue without starving the observed
complete body. The returned known ordinal includes the initial field ordinals,
not later updates. New fields wait for the next poll. Replacing an unread extent
still rejects the read; a final field read rechecks owner/source authority.
Recent/older records and retained input pages use source-bound anchors. The
current plan has its own read even outside the recent region. Both the Chat header
portrait and status control open the same activity sheet. The activity sheet
provides a character/name identity row linking to the current Agent's canonical
Team-scoped detail page. Its top bar and close action are outside the native body
scroll viewport. The scrollbar starts below that bar, and browser-native body
edge overscroll does not move the bar. Dragging over recent-record controls pans
the body; an intentional tap still opens the selected record. Hover highlighting
is limited to fine hover pointers rather than remaining selected after touch.
The Activity sheet locks document overflow through its mounted CSS scope instead
of a JavaScript wheel/touch boundary canceler. Closing it removes that scope;
focus trapping and nested control dialogs retain their existing ownership.
The sheet previews five recent records in chronological order, with a transparent
top fade when older records exist. Preview and full history reuse the same compact
row design. Selection enters a separate full-height history view at that record and keeps
keyboard focus on the selected entry after the activity sheet finishes closing.
Opening history without a selection also leaves focus inside that view. Closing
the sheet back to the transcript restores the status control.
User and assistant conversation is included; records expand from a short opening
preview to their full content. Older-page insertion preserves the visible reading
anchor, and latest-follow can be restored explicitly. The history reader unmounts
on close and the activity sheet remains the return destination. Sheet headings
and close actions sit outside their internal scrolling content. Read-only retry
never resubmits input, an answer or Stop.

Conversation identity and source snapshot are compared independently. A replaced
source clears old live content and paging before the new source refills. Ordinary
loading/error feedback does not fabricate a new conversation or native context.
Same-source updates retain original record order and update the matching native
message. Viewport anchoring preserves an existing record's screen offset while
the reader is detached; Jump to latest restores following.

## User-facing execution diagnosis

A normal idle header shows only the Agent name. Meaningful work, action-required,
failure, stale and Computer states remain visible. Chat owns the viewport scroll
boundary. A conversation shorter than that boundary still has internal travel
above its resting latest position, and the viewport allows edge overscroll
instead of containing it. The document itself does not scroll.
The header and composer stay available while conversation content
scrolls internally. The main transcript contains user/assistant conversation and
necessary questions or permission input. Tool, reasoning, plan, terminal, notice
and state records belong to the recent preview and full history, not duplicate
transcript disclosures. A confirmed native user message is the visible message
in both the transcript and full history; Chat does not repeat a separate
delivery receipt beside it. Pending input disappears at actual native
Inserted/Steered acceptance independently of when native history appears.
There is no ID association, text matching or message morphing between them;
a short natural display gap is allowed.
The down arrow signals new content that arrived while the owner was reading
above latest, rather than a detached position alone. Reaching latest or selecting
the arrow acknowledges it.

The status distinguishes a conversation that has not started, missing saved AI
authentication, startup in progress, reported startup failure, delivery failure,
unconfirmed delivery. Known Computer off/start/stop/restart states take
precedence; replacement is restart to the user. During temporary read errors,
stale reads, unknown current power or Connector disconnection, the same-source
retained native foreground remains visible with only a spinner. Cold absence
has an empty label plus spinner, not an invented internal state or generic
checking caption. Retained presentation never restores Send, answer or Stop
authority. Missing native
observations alone do not prove a failed start. Reported native rejection or
pre-launch failure names the harness (currently Codex) as failing to start; it
does not invent token-expiry or network diagnoses. An unavailable saved source
and uncertain retirement retain their distinct user consequences.

The read owner preserves typed setup results beside retained input operations.
Connection-startup preparation is also carried in the owner header before an
input exists. It projects starting or the reported failure without synthesizing
a user message. A startup failure remains visible during source loading and when
the first accepted input has no independent setup fact. Native foreground still
wins; saved-authentication absence remains its own actionable condition.
A separate latest-input observation stays current even when the user pages older
inputs. It follows the newest non-deleted operation for the exact source, so
deleting a newer failure does not hide an earlier accepted or preparing
operation. The current native foreground observation takes precedence over past
startup failures. New local submissions supersede older failure feedback without
replaying it. Authentication setup links to the Agent detail page; read recovery
only refreshes observations. The composer is disabled when authentication must
first be selected. Input-level reported startup failures remain readable on
reentry rather than being reduced to generic delivery failure. Possible setup
forwarding does not replace known delivery uncertainty in an input label.

Pending and failed text uses the same right-aligned user-bubble presentation as
native user messages: a localized author label above a padded Markdown body.
Pending input alone has opacity 0.6 and no delivery/lifecycle caption. Explicit
failure recovery actions remain separate. Paragraph default margins do not add
empty space inside the user bubble.
Retained inputs precede newer local
submissions, so a pending message does not jump above earlier failures.

## Text, requests and retained operation facts

User content and assistant utterances use Markdown without raw HTML or automatic
external-image loading. The body uses the small text token; the composer retains
the normal input size. Assistant authors include the selected Agent profile.
Links use the common theme color. Disclosure transitions respect reduced motion.
Supported media
have explicit previews and inspectable original base64. Terminal pieces are
assembled as bytes before text decoding and retain exact base64 independently
of display replacement characters. Plans, tools, reasoning, locations, notices
and terminal results remain inspectable activity rather than invented prose.
Original tool input and output are independent typed fields, including tool-only
updates without generic content. Their JSON is displayed opaquely with exact
int64 values; clearing one field does not clear or substitute the other.
Text plans render their own typed text rather than assuming generic content.

Send uploads exact UTF-8 pieces through the generated owner client. Acceptance,
preparation, possible forwarding, known failure and missing evidence remain
different pending observations. Original submitted text survives navigation
until actual Inserted/Steered pickup ends that input. Native history is not
fabricated or correlated to its deleted input. Steering does not manufacture a
native message ID.
Recovery of a retained Request ID removes its matching local uncertain entry.
An owner may then intentionally send identical text as a new operation; recovery
itself never submits another effect or treats text equality as request identity.
Readers send at most 25 exact known-input IDs per continuation, prioritizing
locally accepted IDs and rotating cached IDs. A source-qualified absence removes
that exact local/cached bubble even if pickup preceded its first list poll;
page omission alone does not. Each read owner serializes reads, and applies
closures only to its admitted conversation/source without a permanent closure
ledger.

Confirmed failed outgoing inputs expose intentional retry and deletion actions.
Retry copies retained text into ordinary Send with a new Request ID; it does not
replay an uncertain operation or change the old outcome. Delete clears the
original submitted body and hides the entry after an authoritative response,
while retaining the operation evidence described in
[conversation storage](conversation-storage.md). It does not require a running
Computer. Pending or lost delete responses stay read-only recoverable through
the exact input result. A successful acknowledgement clears the source-qualified
cached operation even outside the current page. Tombstones merge monotonically,
so late bounded reads cannot restore deleted text or its latest-failure status.

Permission/form requests render inline. Original choices, their descriptions,
schema/field titles and supported adjacent constraints remain inspectable.
Permission is not preselected. Form drafts distinguish omission, empty strings,
false, empty selections and exact int64 values. Native defaults may seed a draft;
only explicit submission accepts it. Standard format feedback is advisory; server
and native validation remain authoritative, including bounded offline string
patterns and supported formats. Native patterns are not compiled or executed as
unbounded browser regular expressions. A definitive pre-admission validation
rejection preserves an editable draft for manual correction with a new Request
ID. Lost/unknown responses keep the responder frozen pending read-only recovery.
Unsupported required fields cannot be answered
as an ordinary form. Password-format collection is rejected.

A historical request is not responder authority. Only the matching current live
responder enables a new answer, and native dequeue revalidates it. The retained
original typed answer remains discoverable after responder closure, bound to
source and occurrence. Original choices and selected answers stay expanded rather
than being replaced by a summary. Lost mutation responses retain their exact
operation/source and use read-only result recovery; there is no automatic effect
retry or browser offline queue.
Once the retained answer is discovered, it supersedes stale local uncertainty
warnings and its obsolete read-recovery control without reopening the responder.

Current-work Stop is beside the execution summary and uses the exact native
source/session/work token. Possible forwarding, cancellation publication and
rejection remain distinct; publication does not prove termination. It neither
powers off the Computer nor retires the harness. Computer power controls retain
their existing eligibility and independent confirmation contracts.
Stop availability is independent of input upload/sending and general conversation
reread activity. Current account/Computer admission and the exact observed target
still apply, and the native owner remains final authority over that work token.

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

The setup view also shows the reported preparation freshness and exact Node,
adapter and Codex versions. Cached preparation after a transient lookup failure
is distinguished from a fresh selection at start. Current saved authentication
is reread through owner conversation reads, not frozen initial Agent-page data.
Its comparison with the physically applied record distinguishes a saved future
selection from the current process; historical unknowns stay unknown. Same-record
token-file updates do not change that identity, and record deletion is not proof
of immediate offline native revocation. Credentials and source fingerprints are
not exposed in this view.

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
