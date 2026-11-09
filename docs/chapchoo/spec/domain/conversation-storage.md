---
title: "Private conversation storage and source-bound acceptance"
---

# Private conversation storage and source-bound acceptance

The persistence library stores private Agent supply choices, accepted text inputs
and captured raw/normalized event pairs. These are internal completed Store
operations and authenticated component transfer. The client service exposes
current observations, saved supply, text upload/acceptance/delivery observations,
typed answer upload/results, live responder references, normalized record reads
and exact-work Stop. The real Chat UI remains unactivated.
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

## Conversation identity and product input

Conversations and accepted inputs use [repository-issued identities](repository-identity.md)
with `CV` and `CO` DTO prefixes. A separate current-conversation association
selects one initial ongoing conversation; the schema does not impose a permanent
unique conversation on every Agent. Reading an absent current conversation
returns absence and does not create one.

First input admission creates the conversation and association in the same
transaction as its retained input. The exact submitted text is preserved. Request
recovery compares its original source target and text; changed input under that
request conflicts. Receipt recovery never dispatches native work or establishes
native acceptance. Missing/replaced evidence does not prove non-delivery.

`NOT_STARTED`, `LOADING` and `READY` source states have distinct admission rules.
A not-started input can bind to initial setup; a bound input retains its exact
snapshot target. Loading rejects new input, while an already accepted receipt may
still be read. A prior target cannot be silently reinterpreted against a later
source after its old receipt has been removed.

Native forwarding has a separate durable boundary. The Store revalidates private
owner admission and the exact resolved Connector route without requiring its
bearer secret. One transaction records possible forwarding and returns a single
execution/session-bound grant. Concurrent or later attempts recover existing
evidence rather than another grant. A lost waiter or server restart cannot turn
that committed fact into known non-delivery. Native results retain an actual
inserted message ID, steering without an invented ID, or failure; changed results
conflict. Current authenticated source admission fences result publication.

### Internal input owner and setup facts

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
after uncertainty; complete receipt recovery still cannot dispatch twice.

Input-result reads select acceptance metadata without loading the submitted body.
Local processing ownership and durable native delivery observations are separate:
absence of either is missing evidence, not proof of non-delivery.
The same result includes the frozen requested supply and separate setup
possible/session-ready/failure evidence. A known setup failure is not collapsed
into absent text-delivery evidence; session-ready still does not prove current
native liveness, completed source catch-up or text consumption.

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
input-result lookup, normalized record region/field reads, Stop and Stop-result
lookup. Admin and Control-only groups do not register this owner service.
Component native/request reports are separate execution-authenticated operations.
Current reads do not start native work; supplier changes remain next-start only.
Every continuation/mutation keeps private ownership and current Team checks.

Owner record regions return at most 25 typed metadata entries, newest first,
with an explicit older-record anchor. Field metadata identifies the field update
and complete byte length; body continuations read at most 32 KiB, preserving
newline-delimited normalized values and explicitly cleared empty fields. A
continuation binds the same source snapshot, record and field update. A changed
source/content identity conflicts rather than mixing two bodies. Neither the
raw captured RPC envelope nor a durable browser feed is exposed by these APIs.

Setup has separate durable possible-forwarding and outcome facts, available
before a captured source becomes ready. One setup grant binds the current route/
execution and atomically copies the saved requested supply choice. Later saved
selection changes do not alter that record. The requested choice is not proof of
an applied installation. An actual session-ready result is neither current native
liveness, source catch-up completion nor text consumption. Known failures and
changed/late outcomes remain distinct.

Fresh-context permission is sealed at setup admission, not recovered from missing
local state. An unresolved earlier setup, a known native context or a failure
without explicit pre-launch/non-admission evidence denies a new fresh context.
A known pre-launch failure can allow an explicit new operation; it never grants
another setup dispatch for the original operation. Initial source binding must
agree with any already-known setup session/source. Source replacement clears
these setup facts with the independently accepted input; it does not replay work.

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
offered option, required presence, property types, ranges, string lengths and
choices. Unsupported optional values cannot pass as validated fields. Pure
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
record/field and last content-update ordinal; source/content change fails the
continuation. Content identity is not a durable browser event cursor. Owner
admission holds the existing Computer authority lock while these coherent reads
complete, serializing source mutations without changing global isolation.

## Schema preparation and cleanup

Forward migration 12 adds saved-supply/request, conversation/current-association,
input and event-pair tables to both engines. Existing accounts, Agent/authentication
identities, protected credential bytes and creation fingerprints are not rewritten.
Migration 13 adds normalized record/field projection, independent setup/input delivery,
native state and Stop operations,
and live-request/original-answer tables without rewriting the existing identities,
source pairs or accepted inputs.
[Explicit preparation/readiness](account-storage.md) includes the new migration
and product-table binding checks. Downgrade rejects any retained new state.
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
