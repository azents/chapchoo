---
title: "Private conversation storage and source-bound acceptance"
---

# Private conversation storage and source-bound acceptance

The persistence library stores private Agent supply choices, accepted text inputs
and captured raw/normalized event pairs. These are internal completed Store
operations: no public conversation RPC, native launch, model execution or real
Chat UI is activated by this storage foundation. [Backend authentication](backend-authentication.md)
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
and event pairs before marking the new source loading. It does not remove Agent,
Computer, authentication, supply or native-context identities. Repeating the same
binding does not clear its already-ingested data.

Each ingestion stores opaque raw and service-normalized bytes together with its
explicit occurrence ID and capture ordinal, then updates the claimed source
cursor in the same transaction. Persistence does not parse those payloads or
invent normalized content. Ordinal must be the next retained capture position;
IDs are not sort order. An identical occurrence returns current progress without
a second publication; different content/order under that occurrence conflicts.
Old-source ingestion fails rather than continuing in another snapshot.

Completion names a finite captured boundary already fully ingested; later appends
continue without requiring a native idle period. No native replay, command retry,
outgoing dispatch journal or upstream-gap repair is performed by these operations.

## Bounded internal reads

Owner metadata reads are count-bounded and ordered by capture ordinal. An optional
after-position must match a known occurrence. Raw and normalized payloads have
separate source-qualified byte-continuation reads, bounded to 32 KiB per chunk.
A source change fails continuation rather than mixing content, and complete body
assembly remains the caller's responsibility. Raw/debug data is not a normal
browser DTO. These internal reads do not claim a functioning streaming UI or
measured latency.

## Schema preparation and cleanup

Forward migration 12 adds saved-supply/request, conversation/current-association,
input and event-pair tables to both engines. Existing accounts, Agent/authentication
identities, protected credential bytes and creation fingerprints are not rewritten.
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
- [Source ingestion and byte reads](../../../../rust/crates/persistence/src/conversations/records.rs)
- [Both-engine contracts and preserving upgrade](../../../../rust/crates/persistence/tests/conversations.rs)
