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

The owner can continue composing and submitting while
[configuration refresh](agent-instructions.md#common-refresh-and-continuing-input)
waits for native work to finish. Native-action readiness does not disable server
input admission. Activity's System prompt link navigates to a separate
server-rendered page for the latest selected prompt, independently of current
saved settings and native history. It has no history selection or pagination.

## Conversation Work summary

The [work summary](conversation-storage.md#conversation-work) appears above the
unchanged composer only for open Work with an explicitly published plan.
The default reply objective has no placeholder, checklist item or bubble; a
direct reply may finish it without a summary ever appearing. A new input does
not replace a displayed plan. The single floating surface shows the title and
completed task count, and expands into the ordered task list with a straight
divider. It is not inferred from native text or foreground state. Stop does not
clear it, and terminal work removes it without adding a synthetic message.

Initial summary content is server-rendered without an entrance. Live appearance
grows in place; exit overshoots before contracting, retaining visible content
until clipped by the departing surface. Expansion and interaction keep the
composer anchored. The whole surface provides theme-aware press feedback, and
reduced motion omits animated transitions.

## Execution feedback

The transcript ends with a short localized Thinking status (`생각 중` in Korean)
while the current authorized observation reports native work. Its decorative
spark pulses and its text shimmers on a 2.4-second cycle. This is ephemeral
execution feedback, independent of published messages and token streaming; it
contains no Agent name or progress estimate. The existing content-size observer
keeps it above the composer while preserving a detached reader's scroll intent.

Display requires a ready conversation with `working` foreground, a successful
current activity read, current owner access, and a completed running Computer
intent with running power and a connected Connector. Idle, attention requests,
loading, failed/stale observation, disconnection and stopped/retiring lifecycle
withdraw the indicator. A pending, published or uncertain Stop withdraws it;
an unanswered live permission/form request also withdraws it even if the native
engine still reports `working`. Historical requests without current responder
authority and already answered requests do not suppress running feedback.
A rejected Stop does not erase positively observed running work. Independent
host-management connectivity does not erase native execution evidence.
The header's retained-state/refreshing presentation remains separate.

The non-interactive status uses polite screen-reader feedback. Reduced-motion
users receive static readable text and a stationary spark; high-contrast text
does not depend on a transparent gradient.

## Transcript transitions

New transcript messages, response requests, pending input and transient feedback
enter with a 320ms soft upward slide. Height, opacity and translation share
`cubic-bezier(.4,0,.4,1)` and the same duration for arrival and departure; spacing
is measured inside the animated item. Removed items remain only for visual exit,
are immediately inert and hidden from assistive technology, then unmount. Rapid
reappearance continues from the currently rendered geometry. Pending input and
its published receipt share one identity rather than producing duplicate bubbles.

Initial server-rendered content is immediately visible and hydrates unchanged.
Older receipt pages and full history do not replay arrival motion. Conversation
or native-source replacement clears retained visual exits; loss of page access
unmounts the chat. Reduced-motion and hidden-document updates settle immediately.
The existing scroll owner follows changing layout only while following latest;
explicit reading intent withdraws that pinning and preserves history anchors.

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
Full records include user and ordinary native/model text. An internal follow-up
input appears with its role and content as captured; it does not create a user
Chat message. Each chronological record has an
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
`server: "chapchoo"` and `tool: "send"` identity, and retains `send_message`
and `publish_text` identities in immutable records captured before replacement.
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
reading above it, Chat shows a localized Scroll down action. A genuinely newer
message receipt or actual live question changes it to View new messages; old pages
and updates to existing content do not create unread state. Full history
independently follows newer source-bound native records. The Chat action returns
to the latest receipt window and resumes following at the destination. User scroll
intent can interrupt that return. Reaching the bottom acknowledges unread state
and withdraws the action; its last displayed label remains unchanged through exit.
Reduced motion returns immediately, and preference changes preserve detached reading.

The return control uses the unchanged upper status-summary surface: translucent
profile tint, blur, border and shallow highlight/shadow. Its icon/text gap remains
fixed while measured label width transitions; outgoing and incoming labels do
not overlap. Initial server rendering adds no artificial return control.
Mobile Chat hides its scrollbar while retaining native touch/wheel/keyboard
scrolling. Desktop exposes the existing scrollbar only while hovering.

Desktop navigation is an inline fixed-width sidebar when expanded. A left
chevron collapses it and a menu button re-expands it, transitioning sidebar width
and contents while the chat takes the released space. Hidden navigation is inert.
Mobile retains its navigation drawer; resizing to desktop closes that drawer and
restores focus to the desktop toggle. Reduced motion removes sidebar transitions.

## User-facing execution diagnosis

A normal idle header shows only the Agent name. Meaningful work, action-required,
failure, stale and Computer states remain visible. Chat owns the viewport scroll
boundary. A conversation shorter than that boundary still has internal travel
above its resting latest position, and the viewport allows edge overscroll
instead of containing it. The document itself does not scroll.
The header and composer stay available while conversation content
scrolls internally. Activating Send keeps focus in the composer, including pointer
and touch activation, rather than transferring it to the button. An in-flight Send
keeps the editor focusable but noneditable until acceptance or failure,
preserving the submitted draft while disabling further Send activation; the container's
synchronous admission guard also rejects repeated Enter submission. A failed
request retains the draft and focus. Enter submits only outside active IME
composition; Shift+Enter inserts a plain-text newline. The composer uses a
`contenteditable="plaintext-only"` message editor with a localized accessible
label and unchanged localized placeholder, text input mode, sentence capitalization,
autocorrection and spellcheck. It does not disable autocomplete. Paste inserts
only clipboard plain text at the current selection, never clipboard HTML.
Unchanged text leaves the native caret/composition intact; accepted clearing and
other external draft changes synchronize through text content. The editor retains
the existing input dimensions and five-line scrolling boundary. Operating-system
keyboard predictions remain platform behavior, separate from DOM focus checks. The main transcript contains accepted user/Agent messages and
necessary questions or permission input. Tool, reasoning, plan, terminal, notice
and state records belong to the recent preview and full history, not duplicate
transcript disclosures. A successfully delivered user receipt remains the main
message after input cleanup or source replacement. Pending messages come
exclusively from queried server queue rows. The browser does not manufacture
sending/uncertain bubbles or clear drafts through receipt-nonce correlation.
A historical receipt that still has a matching queued operation is rendered
once, with pending styling and recovery attached to that exact input ID.
The return action distinguishes reading above latest from unread arrivals through
its label. Acknowledgement occurs at latest rather than implying that clicking
has already completed a smooth return.

The status distinguishes a conversation that has not started, missing saved AI
authentication, startup in progress, reported startup failure, an established
delivery failure, and a delivery result the server has not received. In the last
case, the status tells the user to check the chat before sending again rather
than reporting failure or labeling the missing observation. Known Computer off/start/stop/restart states take
precedence; replacement is restart to the user. During temporary read errors,
stale reads, unknown current power or Connector disconnection, the same-source
retained native foreground remains visible with only a spinner. Cold absence
has an empty label plus spinner, not an invented internal state or generic
checking caption. Retained presentation never restores account or native-control
authority. Ordinary text submission is independently admitted before native
readiness. Missing native
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
An operational consumer error or lost ownership after a possible-native grant
is an actionable failed queue item; it is not proof of physical non-delivery.
Safe pre-attempt readiness loss remains waiting rather than a failure.
A valid late native
success may still replace that retained item with its normal receipt.
Readers send at most 25 exact known-input IDs per continuation, rotating cached
IDs. A source-qualified absence removes that exact cached bubble;
page omission alone does not. Each read owner serializes reads, and applies
closures only to its admitted conversation/source without a permanent closure
ledger.

Stopped, starting, restoring and matching loading-source states do not disable
ordinary composition or substitute a queue-specific placeholder. Send retains
the normal Agent/message hint, editor behavior and acceptance-only draft clear.
Current account/private owner/Team, fresh reader authority, matching conversation,
deletion and missing selected AI authentication still fence submission. Native
answers and current-work Stop retain their own running/connected exact-target
admission; writable composition does not enable those effects. Cloud-only
automatic Start remains server-owned, not a browser poll or extra Send.

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

## Conversation model selection

The desktop overflow and mobile character/status controls open the same
**Activity** sheet, with its Agent link. Its **Model** row reflects the
current scoped saved choice or the harness's observed initial model.
The row opens **Change model** in the same sheet. The selected candidate
determines which reasoning levels and speeds are offered on that page. When
available, the single-select speed list distinguishes Model default, Standard,
Fast and Ultrafast. Changing models clears the draft tuning. One Save accepts
the tuple; Back and Cancel discard it. A confirmed save returns to Activity. A
conflicting save does not overwrite a newer choice,
and an ambiguous response can be checked by its original request ID without
repeating the mutation.

The setting is for new conversation turns. The interface distinguishes saved
preferences from the model, effort or tier actually used for an individual
turn. A qualified failure before start can
offer **Change model** in the same conversation without preselecting an
alternative; saving never resends the affected request.

Direct Codex conversations with reported permission modes show **Permissions**
directly below **Model**. Without a saved choice, the row shows the first
available mode in this order: **Approve for me**, **Full Access**, **Ask for
approval**. The permission picker saves through the same target and request
fences as the model picker, but can save without a model catalog. It neither
changes model settings nor sends a message.
The preference applies to new turns, not work already running.

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
