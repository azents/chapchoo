---
title: "Private conversation storage and source-bound acceptance"
---

# Private conversation storage and source-bound acceptance

The persistence library stores private Agent supply choices, independent chat
receipts, temporary accepted text inputs and captured raw/normalized event pairs.
These are internal completed Store
operations and authenticated component transfer. The client service exposes
current observations, saved supply, text upload/acceptance/delivery observations,
typed answer upload/results, live responder references, chat metadata/body reads,
normalized record reads and exact-work Stop. The [Chat reader](conversation-web.md) consumes these owner
contracts without exposing raw captured RPC envelopes.
No model execution is established
by these storage or routing contracts. [Backend authentication](backend-authentication.md)
still owns saved credential records separately.

## Private admission and saved supply

Owner reads/mutations authenticate the current account session, require the
individual Agent owner and current Team membership, then lock/revalidate the
Computer context. Team management or installation administration alone does not
grant access to another owner's Agent. User, Team and Computer admission remains
inside the completed database operation, including lock-wait revalidation.

An absent supply row reads as managed preparation without writing a new owner
choice. Explicit existing mode retains an absolute executable path; syntax is not
file usability or publisher verification. Saving compares the expected accepted
selection Request ID and retains an exact request result. Repeating an old accepted
save returns its receipt without restoring an old selection. A save does not
change the Computer request or launch/stop/apply anything to a running process.

The current native observation separately retains an optional actual-start supply
choice. Its additive child table belongs to that execution/source-qualified
native state and is removed with source replacement. Migration 15 preserves all
existing identities and leaves old observations unknown instead of copying the
saved choice. Downgrade refuses nonempty applied-supply state. The native owner
freezes the choice for one live process; a later ordered state can describe a new
physical start even when its captured transcript is retained. Older observations
cannot overwrite newer ones, and an equal captured state cannot change a known
applied choice. An unknown existing observation may be enriched with its first
explicit applied fact without changing its work/source meaning.

## Conversation identity and product input

Conversations and accepted inputs use [repository-issued identities](repository-identity.md)
with `CV` and `CO` DTO prefixes. A separate current-conversation association
selects one initial ongoing conversation; the schema does not impose a permanent
unique conversation on every Agent. Reading an absent current conversation
returns absence and does not create one.

If connection startup has not already selected an empty conversation, first
input admission creates the conversation and association in the same transaction
as its outgoing input. Enqueue commits no ordinary Chat message. Admission
failure rolls back the queue item and any conversation creation. Exact
retained-input recovery adds no second queued item or consumer dispatch.
A connected desired-running Computer can instead prepare
the selected empty conversation before the first input through the sealed
[startup boundary](#connection-time-preparation). The exact submitted text is preserved. Request
recovery compares its original source target and text while the pending input
exists; changed input under that request conflicts. Receipt recovery never
dispatches native work or establishes native acceptance. Missing/replaced
evidence does not prove non-delivery or authorize automatic resend.

`NOT_STARTED`, `LOADING` and `READY` source states have distinct admission rules.
A not-started input can bind to initial setup; a bound input retains its exact
snapshot target. When no input is currently pending, a not-started input may join
a startup-owned loading source or its matching ready source without changing
its original target. Another concurrent pending not-started input cannot use
that exception. Other loading sources reject new input, while an already
accepted receipt may still be read. After consumed-input removal this is not
a historical first-request counter: another explicit submission can be admitted,
but setup still uses the exact independently retained source and cannot grant a
fresh native context from input absence.

Native forwarding has a separate durable boundary. The Store revalidates private
owner admission and the exact resolved Connector route without requiring its
bearer secret. One transaction records possible forwarding and returns a single
execution/session-bound grant. Concurrent or later attempts recover existing
evidence rather than another grant. A lost waiter or server restart cannot turn
that committed fact into known non-delivery. Native results retain an actual
inserted message ID, steering without an invented ID, or failure. Current
authenticated source admission fences result publication. Inserted or Steered
ends the pending input: one transaction appends its ordinary user Chat message
and deletes its content, acceptance and dependent delivery/preparation rows.
An already committed user receipt from a pre-queue installation is preserved
without a second message. Actual native session/captured history remains
independent. A reported failure retains the outgoing item and its error; no
automatic retry follows. Unresolved physical delivery is not proof of rollback.

Queue PENDING/ERROR is a consumer-attempt observation, separate from native
delivery evidence. Only newly admitted processing renews the existing row's
database-time lease, using the configured component validity and heartbeat.
Existing recovery never renews or dispatches. Processor return/error closes
the attempt, while panic or process loss stops its co-owned renewal and an
independent reaper expires orphaned PENDING rows. A renewed lease cannot
resurrect an expired/error row. Exact input identity isolates old finalization
from a manual replacement. Connector AttemptError requires authenticated current
source/target and the operation's existing forwarding fact; it records queue
error without manufacturing native nonexecution. Valid late success can still
consume a retained errored row. No model-task deadline or automatic retry is
derived from consumer liveness.

An authenticated same-source successful report for an already absent operation
acknowledges closure, not reconstructed delivery success. This permits the
Connector to release a known native result after a lost report ACK; subsequent
body pulls and forwarding cannot use a removed input. Independent chat receipts
preserve successfully delivered text but are not completed-input recovery records.
ReportInput separately acknowledges whether that normal user receipt exists,
using its existing conversation/source-input provenance. A closure after explicit
deletion returns false rather than inventing a receipt. The
[native publication fence](server-tools.md#user-receipt-publication-order) uses
this distinction without adding another history table.
Consequently an arbitrary explicit old owner
submission is no longer recoverable forever and may be a new operation if current
admission allows it. Missing results never automatically resubmit.

Migration 22 first installs a database terminal guard, then removes legacy
successful rows only when their exact native source binding independently
preserves session authority. An old connected writer completing after the
pre-upgrade sweep is consumed in its own successful write transaction too;
backend rollout timing cannot leave a new completed receipt behind. Failed,
unresolved and differently bound historical data is preserved. Downgrade removes
the guard but cannot resurrect consumed content; older preserving migration
guards still apply.

## Independent chat receipts

Chat belongs to the product conversation, not its native source, current
selection or temporary delivery operation. Immutable `USER` and `AGENT` messages
retain exact UTF-8 text, database receipt time and a positive int64 receipt order.
New messages have repository-issued `CM` identities. A locked conversation-local
allocation head serializes both roles until transaction commit; rollback restores
the head too. Receipt order, not timestamps, native event order or DTO sorting,
defines the shared stable order.

User receipts retain the original input ID, request nonce and acceptance time
as provenance without an owning input foreign key. The nonce identifies the
original accepted request; it is neither a permanent
deduplication key nor authority to recover or replay an absent input. Another
explicit admission with a consumed nonce can produce a new input and message.
Successful pickup creates the ordinary user receipt. Failed-item deletion and
native-source replacement leave existing Chat receipts unchanged. Captured
assistant/model output never appends chat.

Owner `Messages` and `ReadMessage` calls revalidate the current private Agent
owner, Team membership and exact product conversation on every read. No native
session or snapshot is required. Metadata pages contain at most 25 entries in
ascending receipt order: latest and before select the newest matching suffix;
after selects the oldest matching prefix. A returned `through` head bounds
newer-page traversal against concurrent appends. Exact message body reads return
at most 32 KiB per chunk, including arbitrary byte boundaries; consumers assemble
bytes before UTF-8 decoding. Metadata never loads full bodies.

The Store/service performs explicit Agent message publication. It checks
the current authenticated Connector execution/route and actual Agent/conversation
ownership independently of native readiness, native IDs or the UI's selected
conversation. The component-only [server-tools boundary](server-tools.md)
exposes this operation as intentional `send_message` to the fixed native owner.
There is no owner-selected recipient or publication retry key. Ordinary native
output does not append independent chat receipts. The
[Chat browser](conversation-web.md) reads these messages separately from actual
native activity and full-record history.

Explicit schema preparation transfers retained original non-deleted inputs into
a deterministic prefix ordered by original acceptance time and stored input
identity. Migration 23 preserves original bytes, input identity, request nonce
and acceptance time, using the original UUID under the new `CM` projection for
the transferred message. It neither reconstructs native history nor resends
input. Already consumed or deleted text is not recreated. Forward migration 24
removes the input-time receipt guard so new enqueue and delivery are separate.
Existing messages remain immutable. Preparation is atomic
under the existing installation binding; downgrade refuses to discard nonempty
chat. Native event capture and full-record browser presentation remain separate
from message-only Chat.

Migration 24 also removes obsolete bodyless input rows left by previously
completed explicit deletions, including their dependent input facts. Existing
ordinary Chat receipts and retained outgoing bodies are preserved.

## Internal input owner and setup facts

The Store distinguishes newly accepted input from exact receipt recovery in its
completed admission transaction. `ConversationInputService` reserves a bounded
process task and byte budget before that admission. New input returns a small
acceptance reference before its composition-supplied processor completes; the
full submitted body is not part of that response. Browser waiter loss cannot
cancel the task. Existing receipts, including those read by a fresh server owner,
do not invoke the processor again. Lookup reports local ownership without
dispatching or claiming non-delivery from its absence.

The owner bounds each input to 16 MiB and aggregate retained input bytes to
32 MiB, independently of task capacity. Capacity survives short acceptance and
is released through completed-task collection. Admission handoff reads wait only
for their already-existing finite batch, not later unrelated admissions. Explicit
drain retains infrastructure failures through cancellation and reentry. This
service is the owner-facing text-submission entrypoint. The RPC frame cap is
64 KiB, not a text truncation limit: clients send pieces of at most 32 KiB with
the exact Agent/conversation/request/target and total byte count on every piece.
Only complete, valid UTF-8 enters product admission. Partial uploads have a
30-second lifetime, four slots and 32 MiB aggregate capacity; their completion
retains those permits through the input owner's handoff. Duplicate pieces must
match retained bytes and identity. Losing temporary upload state does not accept
partial text or grant native replay. Clients query the small acceptance reference
after uncertainty; recovery of an existing pending receipt cannot dispatch twice.

Input-result reads select acceptance metadata without loading the submitted body.
Source-bound input discovery uses accepted-time/request keyset pages, including
the exact not-started boundary. Separate bounded body reads retain complete
original bytes and revalidate the same private source on every continuation.
Separate exact-presence hints accept at most 25 operation IDs and return only
those absent under that private source. A missing item in a keyset page is not
absence evidence. A concurrent successful pickup can end a listed body before
its next read; the owner observes actual absence instead of inventing uncertainty.
First native binding preserves the initial input's original not-started target.
Local processing ownership and durable native delivery observations are separate:
absence of either is missing evidence, not proof of non-delivery.
The same result includes the frozen requested supply and separate setup
possible/session-ready/failure evidence. A known setup failure is not collapsed
into absent text-delivery evidence; session-ready still does not prove current
native liveness, completed source catch-up or text consumption.

An owner can delete a reported failed outgoing input. The completed transaction
checks private owner, Team, exact conversation and current source, then physically
deletes the row and its dependent preparation/delivery records. No bodyless
deleted-item result or separate safety-history store is created. Repeated Delete
of an absent item is idempotent. It is not native cancellation and does not erase
already recorded Chat or native transcript records.

Explicit Retry names the original item and a fresh request ID. In one transaction
it verifies the original failure, reads its exact text, deletes the old item and
registers a fresh operation. A failed insertion rolls back the deletion too.
The replacement is handed to the same bounded consumer only for fresh admission;
recovery of the still-present replacement returns its acceptance without another
dispatch. Competing replacements for one original yield at most one new item.
The optional replacement-origin ID belongs only to the new pending row and
disappears with it; it is not a retained result for the removed item.

Bounded lists query the outgoing table and expose the newest current item
independently of the page anchor. Exact absence hints close removed rows even
outside the current page. Queue text remains readable while its exact native
source is loading, independently of native history readiness. Downgrade to
admission-time Chat refuses outgoing bodies without their matching old receipts.

The concrete internal Control processor uses current owner/Computer authority and
PeerDelivery, obtains setup admission once, then sends only the sealed small setup
intent. It observes actual setup results and finite source readiness before
obtaining the separate exact text-delivery grant. The process-owned preparation
observation budget is five minutes; expiry preserves durable uncertainty and does
not cancel or replay native work. This is not a browser response deadline or a
model-completion wait. Server admission withdrawal ends observation without
inventing non-delivery. The backend composes this processor with the shared local
PeerDelivery when it owns Control, or ordinary remote delivery otherwise. Client
service shutdown closes admission and drains input/Stop tasks under the existing
backend cleanup lifetime.

## Connection-time preparation

The Client composition owns a background reconciler independently of Connector
stream admission. It reads desired-running Computers in keyset pages of 20, keeps
at most 20 Computer tasks, and revalidates each current authenticated Connector
route before admission. The completed transaction locks the affected Teams and
Computer, rechecks owner membership and creates the selected empty conversation
when absent. It inserts no input or prompt. Without a saved authentication
selection it retains only the empty conversation and does not grant native setup.
Owner reads never run this admission.

A startup operation seals execution, requested supply, fresh/retained permissions
and any exact retained snapshot/native target in `conversation_startups`.
Migration 21 adds this independent source-local history without rewriting existing
inputs, credentials or source bytes; downgrade refuses nonempty startup history.
It participates in the same fresh-context barrier and native-rejection fence as
input preparation. Source replacement clears startup facts with the other
source-local operation state. Retained intent cannot follow a moved source.

Delivery uses reconciliation authority with proof of the current Connector owner;
Setup is the only command. Queue admission is not native readiness. The worker
waits for an already-started peer attempt when closing, stops adding subsequent
work and drains its owned tasks. A panic releases only that task's Computer slot.
An attempted Computer is reconsidered after 15 seconds. Current unresolved
operations retain their identity; Connector's invocation lease and native setup
registry recover identical operation results rather than admitting another native
launch. A missing route or stopped Computer withholds dispatch without proving
that an earlier attempt had no effect. Native setup dequeue rechecks desired
running, exact execution, supply and the sealed source target.

A previous-execution unresolved startup is settled as `RETIREMENT_UNKNOWN`, not
redispatched or treated as unused fresh permission. READY records also retain the Connector connection that reported them. A new
current connection is not that witness, even within the same Computer execution:
it obtains another retained-only Setup for the exact source. An already-live
native owner reuses its session; a replacement process resumes it. A matching
ready source may also be resumed on a new execution; missing or failed restoration
cannot create a replacement. First input processing observes the current startup until its exact
source is ready and then obtains the ordinary text-forwarding grant. It does not
send another Setup. Known non-admission/pre-launch failures may use a new ordinary
setup operation under the combined history barrier.

`OwnerConversation.startup` exposes optional preparation evidence independently
of accepted input and current native foreground. Authentication absence,
preparation in progress and known failures remain distinguishable before any
input. A session-ready startup is not current liveness or text consumption.

## Observed native work and exact Stop

The actual native owner publishes foreground state with the captured occurrence
that established it and an execution-local work token. Ingestion must already
contain that occurrence. Within one execution, older observations cannot overwrite
newer state; an equal occurrence with different work/state conflicts. These
observations are not reconstructed from historical requests and do not guarantee
current physical process liveness. A new execution fences old observation targets.

Stop admission rechecks the exact source/session/work under current private owner
authority, records possible forwarding and returns a small operation receipt.
Duplicate request recovery never dispatches again. A process-owned task survives
browser waiter loss, routes the small exact-work command and rechecks its sealed
server admission at Connector dequeue. The native actor checks the real current
work immediately before cancellation and retains operation idempotence across
server-link changes. Later results distinguish `CancelPublished` from exact
`Rejected`; publishing cancellation is not work termination. There is no
confirmation step, Computer power change or harness retirement.

Client-only `chapchoo.conversations.Conversations` provides current source/native
observation, managed/existing saved supply reads/changes, complete text upload,
input-result lookup, independent message metadata/body reads, normalized record
region/field reads, Stop and Stop-result
lookup. Admin and Control-only groups do not register this owner service.
Component native/request reports are separate execution-authenticated operations.
Current reads do not start native work; supplier changes remain next-start only.
Every continuation/mutation keeps private ownership and current Team checks.

Owner record regions return at most 25 typed metadata entries, newest first,
with an explicit older-record anchor. Field metadata identifies the field update
and complete byte length; body continuations read at most 32 KiB, preserving
newline-delimited normalized values and explicitly cleared empty fields. A
continuation binds the same source snapshot, record and captured field extent.
Later appends preserve that finite body view; replacement or source changes
conflict rather than mixing two bodies. Neither the
raw captured RPC envelope nor a durable browser feed is exposed by these APIs.

Latest-plan discovery is a separate bounded metadata read under the same private
source, so the plan need not remain within the recent record page. It confers no
native authority and uses the ordinary field continuation for complete content.

Setup has separate durable possible-forwarding and outcome facts, available
before a captured source becomes ready. One setup grant binds the current route/
execution and atomically copies the saved requested supply choice. Later saved
selection changes do not alter that record. The requested choice is not proof of
an applied installation. An actual session-ready result is neither current native
liveness, source catch-up completion nor text consumption. Known failures and
changed/late outcomes remain distinct.

Fresh-context permission is sealed at setup admission, not recovered from missing
local state. An unresolved earlier setup, a known native context, uncertain
retirement or a creation-capable source failure denies a new fresh context.
Known non-admission, pre-launch failure or explicit native rejection after
successful scope retirement can allow an explicit new operation. A settled
source-lookup failure from an attempt denied fresh creation does not itself
establish a new creation barrier; any earlier unresolved attempt still does.
Any historical native rejection also seals a retained-source requirement in the
new grant, since rejection may have occurred while restoring an existing session.
The native owner may use that retained source but cannot create a replacement if
it disappears before dequeue. Both creation and preservation permissions are
persisted and revalidated against the original grant. An existing empty, unbound
capture permits a first context; a bound native session is resumed, never replaced.
The additive migration preserves old grants and refuses to downgrade while a
retained-source fence would be discarded. None of these outcomes grants another setup
dispatch for the original operation. Initial source binding must
agree with any already-known setup session/source. Source replacement clears
these setup facts with temporary accepted inputs; conversation-owned chat
receipts remain. It does not replay work.

## Live requests and original answers

Captured request history does not make a question answerable. The current native
owner explicitly publishes a responder token, kind and captured occurrence under
its exact execution/source/session. That occurrence must already be ingested.
One occurrence cannot acquire a second responder token. Closing a token is
irreversible even if an older open observation arrives later.

Answer admission preserves original typed values independently of capture:
optional omission, false, empty text/arrays, exact integers, decline, cancel and
offered permission identity remain distinct. Non-finite numbers and oversized
payloads are rejected. The Store fences one effective answer per responder;
an identical request recovers its receipt, while changed payload or a competing
request conflicts. New admission requires complete source loading, an observed
open request and its still-current execution. The native owner must recheck the
actual responder and schema again before sending.

The internal conversation service first reads the original normalized question
under current private/source authority. It decodes the typed permission or
supported form, checks the bound native session and validates the original
offered option, required presence, property types, ranges, string lengths,
choices and the shared offline string-pattern/format rules. Pattern compilation
and backtracking are bounded. Email, URI, date and date-time formats are checked;
unknown formats remain text and password-format collection is rejected. Unsupported optional values cannot pass as validated fields. Pure
schema validation happens outside DB locks; final Store acceptance rechecks the
same immutable captured occurrence and live responder. The original question
read is internal service data, not a raw browser/debug endpoint.

Answer forwarding separately records possible forwarding before returning a
single exact-route grant. The original typed answer is not rewritten by delivery
results. A local SDK responder accepting the response is not remote consumption
or task completion. Exact closed-target rejection is retained separately; a lost
acknowledgement remains possible forwarding. Reading or recovering an answer never
grants a second native response.

The client-only owner API accepts complete Protobuf `TypedAnswer` uploads in
pieces of at most 32 KiB. Its finite upload assembly binds private caller, Agent,
conversation, request, exact native source and responder token. Missing content,
an empty object, false, empty text/arrays, exact integers and explicit
decline/cancel have separate typed representations; duplicate field names,
non-finite values and missing variants fail ingress. `ConversationAnswerService`
owns the admitted operation beyond a cancelled browser waiter and returns a
small acceptance reference before routing finishes. Existing acceptance never
invokes its processor again. Answer-result lookup reads metadata rather than
loading the large original answer.

Retained-answer content reads return complete Protobuf `TypedAnswer` bytes in
pieces of at most 32 KiB, binding both source snapshot and immutable operation
identity. They remain available after the live responder closes and preserve the
original typed values independently of delivery status. Transcoding uses one
shared reader permit and the bounded original answer; status and live-reference
reads do not perform that full-body materialization. Source replacement or an
unknown operation rejects a continuation rather than reading a different answer.

Read-only request-answer discovery binds the exact source and captured request
occurrence, including after the live responder closes. It returns the retained
acceptance reference without loading the body or reopening response authority.

Live-request lookup rechecks the exact current source and returns at most 32 open
responders belonging to the current execution. Its occurrence links to the
normalized original `REQUEST` record and `REQUEST` field, including the original
question/schema. An existing answer Request ID lets a reader recover that
answer's status without making the question answerable again. Closure removes
the live reference, not the historical question or original answer.

The concrete Control processor obtains a separate one-time answer-forwarding
grant, sends a small reference and does not wait for native task completion.
Connector pulls the complete original typed answer in bounded 32-KiB pieces,
checks consistent responder/occurrence/kind/length on every piece and performs a
final empty-suffix authority check. It converts through the supported ACP SDK
response types before the native owner rechecks the actual target. The
execution-local registry binds operation identity to source, occurrence, token,
kind and original typed values; duplicate known results recover and uncertain
operations never dispatch again. Later authenticated result publication retains
local responder acceptance or exact rejection without rewriting original values.
Production root shutdown closes and drains input, answer and Stop processing and
retains multiple cleanup failures. Real successful native responses and
authenticated model work are not established by model-free wire fixtures.

## Connector source and event publication

Source mutations admit the current Connector credential, exact Control/connection,
non-draining live Control, unrevoked session, unretired current execution and
Computer. The Agent must belong to that Computer and its owner must still have
Team membership. Row locking and fresh source reads fence mutations that completed
while admission waited; a detached earlier connection is not authority. The
Control's shared row lock remains held through commit, so a concurrent drain
cannot withdraw that authority before the source operation finishes.

Initial source binding preserves the input that initiated setup. A different
snapshot must name the expected previous snapshot and retain the same bound
native conversation identity. Replacement atomically clears the stored inputs
and event pairs, normalized records/fields, request/answer state and independent
delivery evidence before marking the new source loading. It does not remove Agent,
Computer, authentication, supply or native-context identities. Repeating the same
binding does not clear its already-ingested data.

Each ingestion stores opaque raw and service-normalized bytes together with its
explicit occurrence ID and capture ordinal, then updates the claimed source
cursor in the same transaction. Persistence does not parse those payloads or
invent normalized content. Ordinal must be the next retained capture position;
IDs are not sort order. An identical occurrence returns current progress without
a second publication; different content/order under that occurrence conflicts.
Old-source ingestion fails rather than continuing in another snapshot.

Typed record ingestion additionally requires the normalized bytes to match the
canonical record mutations. It stores the source pair, materialized fields and
cursor in that same transaction. An identical occurrence cannot append its
content twice. Record identity is source-local kind plus native entity ID;
assistant, user, thought and activity identity spaces stay separate. First-seen
capture ordinal and within-event index order records, not lexical native IDs.
Field omission preserves content, replacement clears or replaces it and append
adds only the actual new content. Original raw events remain separately retained.

The internal conversation service decodes native events through the public ACP v2
schema and checks the DB-bound native session. It normalizes typed utterances,
activity, formal content, foreground and request facts without inventing live
response authority from history. Unsupported required requests retain an error,
not a fabricated user refusal or browser consent URL. Credential protocol traffic
does not enter this ingestion path. Source authority is revalidated after pure
normalization and before final publication.

Completion names a finite captured boundary already fully ingested; later appends
continue without requiring a native idle period. No native replay, command retry,
outgoing dispatch journal or upstream-gap repair is performed by these operations.

## Bounded internal reads

The component-only `chapchoo.conversations.ConnectorCapture` service exposes
current-source discovery, exact expected-previous binding, bounded event pieces
and finite completion. Every call authenticates the exact execution/Control/
connection; the listener's client and Admin groups do not register it. Current
discovery does not create product state.

Event pieces carry snapshot, occurrence/ordinal, transfer identity, total bytes
and exact offset. A piece is at most 32 KiB, an event at most 16 MiB and incomplete
assemblies share four slots and 32 MiB per service owner. Partial assemblies have
a fixed 30-second lifetime rather than an extendable per-piece timeout. Exact
repeated bytes recover accepted offset; changed identity/bytes or gaps conflict.
Complete bytes retain their capacity through normalization and DB publication.
Incomplete, expired or lost assembly never publishes a pair or advances a cursor.
Repeating retained observations after response loss is pair-idempotent, not a
native command retry. These bounds keep the existing 64-KiB wire frame limit.

Owner metadata reads are count-bounded and ordered by capture ordinal. An optional
after-position must match a known occurrence. Raw and normalized payloads have
separate source-qualified byte-continuation reads, bounded to 32 KiB per chunk.
A source change fails continuation rather than mixing content, and complete body
assembly remains the caller's responsibility. Raw/debug data is not a normal
browser DTO. These internal reads do not claim a functioning streaming UI or
measured latency.

Normalized region reads return at most 25 metadata records using first-seen
keyset anchors. Per-record fields are separately bounded by the supported field
set. Content continuations read at most 32 KiB and bind the exact snapshot,
record/field and an existing captured ordinal. The current-body extent index
records the server-derived byte end at each append in the same ingestion
transaction. A read finishes at its observed end even while later appends arrive.
Replace/clear removes earlier extents, and source replacement cascades them;
those changed body identities fail continuation rather than mixing bytes.
Only current body bytes are retained by this projection, not an archive of
replaced bodies. Extents reuse captured ordinals, not a new revision counter,
reader token, authorization grant or durable browser event cursor. Each unary
Store read holds the existing Computer authority lock through its own coherent
query, serializing source mutations for that call without changing global
isolation. A multi-chunk caller holds no database transaction between calls.

Tool calls retain independent normalized `TOOL_INPUT` and `TOOL_OUTPUT` fields
for actual native raw arguments/results, even when the tool has no content blocks.
An omitted patch preserves the prior field; explicit null clears it; a supplied
value replaces that field alone. Ordinary owner field reads return bounded
normalized JSON values rather than raw protocol envelopes or debug traffic.

## Schema preparation and cleanup

Forward migration 12 adds saved-supply/request, conversation/current-association,
input and event-pair tables to both engines. Existing accounts, Agent/authentication
identities, protected credential bytes and creation fingerprints are not rewritten.
Migration 13 adds normalized record/field projection, independent setup/input delivery,
native state and Stop operations,
and live-request/original-answer tables without rewriting the existing identities,
source pairs or accepted inputs. Migration 16 extends the typed projection field
set for tool input/output. SQLite copies all existing identities, bytes and update
ordinals transactionally into the extended constraint; PostgreSQL changes only
the check constraint. Both downgrade paths refuse retained new tool fields.
Migration 17 adds source-qualified native preparation and applied authentication
details without rewriting earlier observations. Preparation carries freshness
and exact Node, adapter and Codex versions; authentication retains the record
identity without a catalogue foreign key or credential material. A deleted
record cannot erase an observed application or block approved catalogue deletion.
Unknown historical details are never inferred from saved settings. Within one
execution/source, stale reports are ignored; equal positions reject changed known
facts but permit the first explicit fact and preserve known details when omitted.
A newer captured position may describe a later physical start, even when the
capture snapshot is reused. Source replacement clears these child observations.
The paired downgrade refuses retained details rather than discarding them.
Migration 18 adds/backfills the derived current-body extent index from existing
field bytes and update ordinals without rewriting those values. Its downgrade
removes only that index, leaving every original field byte and ordinal intact.
[Explicit preparation/readiness](account-storage.md) includes the new migration
and product-table binding checks. Downgrades reject loss of original new facts;
derived extent metadata can be removed without discarding their underlying bytes.
Agent removal cascades its owned conversation/supply rows through foreign keys.
Source replacement is narrower and cannot erase saved supply or credentials.

## Connections

- [Private Agent authority](agent-ownership.md) — ownership, membership and terminal cleanup.
- [Account storage](account-storage.md) — preserving preparation and root binding.
- [Connector credentials](connector-credentials.md) — exact execution credentials and withdrawal.
- [Control lifetime](computer-control-rpc.md) — transient calls are not native effect receipts.
- [Backend authentication](backend-authentication.md) — saved credential selection, not conversation execution.

## References

- [Domain values](../../../../rust/crates/domain/src/conversation.rs)
- [Completed conversation operations](../../../../rust/crates/persistence/src/conversations.rs)
- [Source admission](../../../../rust/crates/persistence/src/conversations/authority.rs)
- [Supply selection](../../../../rust/crates/persistence/src/conversations/supply.rs)
- [Input acceptance](../../../../rust/crates/persistence/src/conversations/inputs.rs)
- [Durable forwarding evidence](../../../../rust/crates/persistence/src/conversations/delivery.rs)
- [Live request and original answer admission](../../../../rust/crates/persistence/src/conversations/answers.rs)
- [Source ingestion and byte reads](../../../../rust/crates/persistence/src/conversations/records.rs)
- [Normalized projection and continuations](../../../../rust/crates/persistence/src/conversations/projection.rs)
- [Native event normalization](../../../../rust/crates/service/src/conversation/normalize.rs)
- [Component capture transfer](../../../../rust/crates/rpc/src/conversation_capture.rs)
- [Both-engine contracts and preserving upgrade](../../../../rust/crates/persistence/tests/conversations.rs)
