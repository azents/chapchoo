---
title: "Private conversation reader and controls"
---

# Private conversation reader and controls

The General Chat surface renders accepted user messages and intentional Agent
publications from the independent
[chat receipt store](conversation-storage.md#independent-chat-receipts).
Ordinary and final native/model text remains captured full-record history, not
an automatic Chat message or a fallback when no publication exists. Actual live
permission/form requests and delivery-recovery controls remain independently
available. This is not evidence that real model credentials work.

## Initial rendering and active reads

Exact-Team/private-owner route admission loads the Agent, initial receipt-message
region and actual activity controls through the request-owned server session.
The HTML renders those values and hydrates exactly that state. Reads do not create a conversation,
power on a Computer, prepare an installation or start native work.

Message and activity read owners continue independently on a one-second cadence
while the document is visible and the account remains authorized. Activity reads
actual live request occurrences, latest plan and input operations, without scanning
general native transcript metadata or tool fields to discover Chat messages.
Every retained outgoing item's original text is read there for its pending
bubble, with unchanged cached bodies reused on subsequent reads.
Opening Activity starts a separate full-record read for its recent preview.
Full history mounts its own source-bound reader from an actually loaded recent
region or an explicit initial full-record fetch; an activity-only region is not
treated as complete history. Closing either consumer withdraws its read owner.
Each owner coalesces refreshes, allows at most one read in flight, and rereads on
focus/online activation. Primary hydration does not issue an immediate duplicate
read. Hidden documents stop scheduling new reads, and withdrawn/unmounted
lifetimes discard late responses. Request authentication,
Origin/POST and cookie disposition remain owned by the existing web boundary.

Both the main Chat viewport and full history automatically request older pages
near the top, including filling an underfull viewport. Page insertion retains the
visible reading anchor. Chat pages use product-conversation receipt order, not a
native snapshot/ordinal or retained-input cursor. Newer catch-up freezes a finite
`through` head and continues each bounded page before observing a later head.
Older reads cannot advance that frontier or discard unfinished catch-up. A
latest-window selection supersedes an in-flight older response and replaces the
window rather than merging a disconnected prefix across a gap.
Full history separately preserves its native-record oldest cursor. It reads
only actual captured records, not temporary input bodies or local drafts as
synthetic user records. Current delivery/error recovery remains in Chat's
explicit operation controls.
A page admitted during an in-flight poll is queued. An unchanged cursor does not
trigger repeated automatic page requests. Failed older reads yield to current-head
freshness and then retry their exact cursor on the read cadence. Truthful read
errors do not authorize effect replay; Chat also exposes read-only recovery.
These reads never replay Send, an answer, deletion or Stop.
Recent-record errors in Activity and full-history errors expose a compact
record-specific status with an adjacent read-only retry. Retained records stay
visible; retry refreshes that reader rather than reloading the app or issuing
an effect. An absent optional native Notice description is a valid record, not
a history-read error. Its title and other retained fields remain available.
Paging progress/errors are separate from current-head freshness: a failing older
page remains visible as a read error without withdrawing Send authority established
by successful current-head observation and the existing admission checks.

Native-history clients send bounded known-record/update hints, not a durable
cursor. Receipt order, native ordinal, paging index and accepted-time request values use canonical int64 decimal
strings so browser query keys stay JSON-safe; server ingress validates syntax and
range before restoring exact bigint values. This is not floating-point coercion. It
assembles complete message/field bytes before UTF-8 decoding and validates the empty final
suffix under the same source authority. One initial coherent field metadata view
selects each finite extent, so appends can continue without starving the observed
complete body. The returned known ordinal includes the initial field ordinals,
not later updates. New fields wait for the next poll. Replacing an unread extent
still rejects the read; a final field read rechecks owner/source authority.
Message pages contain at most 25 metadata entries; complete bodies use 32-KiB
byte chunks, including an empty final suffix check under current private
conversation authority. Missing or changed authorized bodies are explicit read
failures, not empty messages or native-output substitution. Immutable receipt
identity/order governs merging; equal text or reused request nonces never merge
distinct messages. A native-source replacement does not reset Chat receipts.
Recent/older native records and retained input pages use source-bound anchors. The
current plan has its own read even outside the recent region. Both the Chat header
portrait and status control open the same activity sheet. The activity sheet
provides a character/name identity row linking to the current Agent's canonical
Team-scoped detail page. Its top bar and close action are outside the native body
scroll viewport. The scrollbar starts below that bar, and browser-native body
edge overscroll does not move the bar. Dragging over recent-record controls pans
the body; an intentional tap still opens the selected record. Hover highlighting
is limited to fine hover pointers rather than remaining selected after touch.
Activity and full history share the design-system Sheet frame and native body
viewport. Chrome stays outside scrolling content; body padding is inside that
viewport rather than creating a second clipping inset below the header. The
mounted Sheet's CSS scope locks document overflow without a JavaScript wheel/touch
boundary canceler. Closing it removes that scope; focus trapping and nested
control dialogs retain their existing ownership. Activity and full history keep
the complete viewport readable up to the header.
The sheet previews five recent records in chronological order, with a transparent
top fade when older records exist. Preview and full history reuse the same compact
row design. Selection enters a separate full-height history view at that record and keeps
keyboard focus on the selected entry after the activity sheet finishes closing.
Opening history without a selection also leaves focus inside that view. Closing
the sheet back to the transcript restores the status control.
User and ordinary native/model text is included in full records. Each chronological record has an
independent single-line summary with a left chevron, ACP-kind icon, title, plain
opening preview and separate status. Opening a record adds its full content
below without changing the summary geometry. There is no event aggregation.
Tool icons use the native ACP kind, with a safe generic icon for missing or unknown
kinds. Recognized structured arguments and native locations provide meaningful
subjects; unspecified or malformed input remains available as exact raw JSON.
Read/search/command output, retained before/after edits, locations and other
typed content remain inspectable without inferred counts or fabricated results.
User rows have a user icon; intentional publication has a paper plane; ordinary
native/model text has a text icon and localized Output label without the Agent
name. User/publication rows omit visible sender-name labels while retaining
accessible role semantics. Both collapsed rows preview the normalized beginning
of the actual body, not a generic sent-action title. Their rail follows the
current scoped Mantine primary filled color.

Publication classification decodes the captured MCP input's exact
`server: "chapchoo"` and `tool: "send_message"` identity, and retains the
`publish_text` identity in immutable records captured before the correction.
Historical read-only recognition adds no callable alias or record rewrite.
A matching display title,
generic kind or body text is insufficient. The pinned adapter supplies this
structured identity rather than a separate ACP tool name. One captured tool
item displays readable publication text and its actual result/error; nested
original tool input/output remains inspectable. Invocation intent or a missing
result does not prove physical publication. No synthetic assistant row is added.
Preview text strips Markdown through
its syntax tree; expanded text retains Markdown. Adjacent streaming text chunks
concatenate verbatim rather than inserting whitespace.
The entire expanded left gutter, including panel/viewport-edge padding, is a
native collapse button with a dashed guide. Noninteractive expanded tool content
also closes its disclosure; links, copy controls, text selection and native
scrolling remain usable. Closing restores focus to the opener.
The history reader unmounts on close and the activity sheet remains the return
destination. Sheet headings and close actions sit outside their scrolling content.

Product conversation identity and native source snapshot are compared independently.
A replaced native source clears source-bound activity/history and paging before
refill, without removing accepted receipt messages. A changed product conversation
replaces the receipt window too. Ordinary
loading/error feedback does not fabricate a new conversation or native context.
Same-source updates retain original record order and update the matching native
message. Viewport anchoring preserves an existing record's screen offset while
the reader is detached. Both viewports follow new events at the bottom. While
reading above it, a genuinely newer message receipt or actual live question
exposes Chat's new-events-below action. Full history independently follows newer
source-bound native records. Prepending old pages, updating existing streaming content,
or merely leaving the bottom does not. Selecting that action resumes following;
there is no always-visible latest action.

## User-facing execution diagnosis

A normal idle header shows only the Agent name. Meaningful work, action-required,
failure, stale and Computer states remain visible. Chat owns the viewport scroll
boundary. A conversation shorter than that boundary still has internal travel
above its resting latest position, and the viewport allows edge overscroll
instead of containing it. The document itself does not scroll.
The header and composer stay available while conversation content
scrolls internally. The main transcript contains accepted user/Agent messages and
necessary questions or permission input. Tool, reasoning, plan, terminal, notice
and state records belong to the recent preview and full history, not duplicate
transcript disclosures. A successfully delivered user receipt remains the main
message after input cleanup or source replacement. Pending messages come
exclusively from queried server queue rows. The browser does not manufacture
sending/uncertain bubbles or clear drafts through receipt-nonce correlation.
A historical receipt that still has a matching queued operation is rendered
once, with pending styling and recovery attached to that exact input ID.
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
startup failures. New queried queue items supersede older failure feedback without
replaying it. Authentication setup links to the Agent detail page; automatic read
recovery only refreshes observations. The composer is disabled when authentication must
first be selected. Input-level reported startup failures remain readable on
reentry rather than being reduced to generic delivery failure. Possible setup
forwarding does not replace known delivery uncertainty in an input label.

Queried outgoing text uses the same right-aligned user-bubble presentation as
ordinary user messages: a localized author label above padded Markdown.
Both waiting and failed outgoing bubbles have opacity 0.6; completed messages
retain normal contrast. Waiting has no lifecycle caption. Failure feedback is
below and outside the bubble, with its own ordinary contrast. Retry and Delete
are icon-only 28-by-28 CSS-pixel controls, including on mobile, with 18-pixel
icons, zero inter-button gap, localized accessible names and hover/focus
tooltips. This is scoped to these controls, not a global touch-target change.
The refresh icon retains its standard arrowhead geometry.
Paragraph default margins do not add empty space inside the user bubble.

## Text, requests and retained operation facts

Receipt content and retained native text use Markdown without raw HTML or automatic
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
different pending observations. Queue acceptance clears only an unchanged
submitted draft. A failed enqueue request leaves the draft and an adjacent
composer error notice, without an optimistic bubble. If its response was lost,
later server queries may still discover the accepted item; the browser does
not infer nonexecution. Original input text survives until actual Inserted/Steered
pickup commits the ordinary user message and removes the queue item, or explicit
deletion removes a failed item. Native history is not fabricated from queue rows.
Steering does not manufacture a native message ID.
An operational consumer error, including owner loss, is also an actionable
failed queue item; it is not proof of physical non-delivery. A valid late native
success may still replace that retained item with its normal receipt.
Readers send at most 25 exact known-input IDs per continuation, rotating cached
IDs. A source-qualified absence removes that exact cached bubble;
page omission alone does not. Each read owner serializes reads, and applies
closures only to its admitted conversation/source without a permanent closure
ledger.

Confirmed failed outgoing inputs expose intentional retry and deletion actions.
Retry calls the atomic server delete-then-requeue contract with a new Request ID.
It does not create a browser-local substitute row or automatically retry a failed
request. Delete physically removes the failed outgoing item and its dependent
records, without requiring a running Computer. Query invalidation and the next
server queue read reconcile both actions. Successful acknowledgement removes the
source-qualified cached item, including outside the current page. The reader
discards responses from its pre-mutation snapshot and immediately queries again,
so an earlier in-flight read cannot restore that item. It retains no bodyless
deleted IDs, deletion-result verification mode or deleted-item log.

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
- [Native-owner server tools](server-tools.md) — intentional publication and actual captured invocation identity.
- [Verification boundaries](acceptance.md) — component, deterministic protocol fixture and real-model evidence are distinct.

## References

- [`Chat server loader`](../../../../typescript/apps/web/src/features/computers/chat/server/loadChat.ts)
- [`Message read owner`](../../../../typescript/apps/web/src/features/computers/chat/containers/useMessageReader.ts)
- [`Activity and history read owner`](../../../../typescript/apps/web/src/features/computers/chat/containers/useConversationReader.ts)
- [`Conversation renderer`](../../../../typescript/apps/web/src/features/computers/chat/components/ConversationBody.tsx)
- [`Generated-client boundary`](../../../../typescript/packages/web-core/src/conversation.ts)
- [`Message client boundary`](../../../../typescript/packages/web-core/src/conversation-messages.ts)
