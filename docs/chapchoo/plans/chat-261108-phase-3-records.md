---
title: "Codex Conversation Phase 3: Capture, Recovery and Conversation Services"
created: 2026-11-09
tags: [agent, conversation, codex, implementation, protocol, persistence]
---

# Phase Execution Plan

- Phase: 3/6, captured sources, normalized state and exact conversation operations.
- Branch/base: `feature/chat-text-3-records` → `feature/chat-text-2-native` (PR 98, `d7369a779e90717585eaaf383fd85d432f070982`).
- PR boundary: retained local source and finite catch-up, atomic server ingestion/projection and recovery, typed owner/Connector wire and service operations. Credential publication and the real browser Chat remain phases 4 and 5.
- Inputs: [Requirements r24](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md), [approved Design r9](../design/chat-261108-text-conversation.md), [implementation plan](chat-261108-implementation.md), completed phase 1 private Store and phase 2 native ownership.
- Approved Design mechanisms: M01, M07–M13, M18–M20, M22–M24, M26 (current source/selection authority portions; credential publication remains phase 4).
- Authority references: REQ-1–12/14–16/18; ADR-D10–D13/16–D23/29–D31/33; current private Agent ownership, completed transaction, exact Control admission, generated protocol and request-owned browser authentication contracts.
- Design delta: None.
- Owner: primary agent `/root` for implementation, integration, local checks and corrections.
- Exact independent reviewer: `/root/chat-implementation-reviewer`, read-only after the complete six-PR stack exists. No additional reviewer or implementation agents.

## Deliverables and Fixed Interfaces

1. Retain one complete current captured JSONL source, snapshot identity and ordered
   occurrence identities in the existing saved-storage class. Preserve actual
   event occurrence IDs where supplied, assign independent IDs otherwise, and
   never substitute native entity IDs or JSON-RPC correlation IDs. Append is
   independent of server connectivity. Malformed/missing local source fails
   explicitly without silently requesting native gap recovery.
2. New local capture requests native retained history from `start` exactly once.
   Existing capture survives server-link/Connector replacement and does not ask
   for history again. Make replay intent explicit at the phase 2 native setup
   boundary; existing native context restore failure never creates fresh context.
   Do not classify all pre-response updates as historical. Keep the entire
   current source through server ACK; no pruning, rotation or hidden archive.
3. A catch-up request resolves a retained occurrence within its exact snapshot,
   captures a complete-record boundary, transfers ordered records through that
   boundary and reports completion separately. Later live append uses the same
   path. Byte-bound pieces retain source/event/body identity; incomplete transfer
   does not publish a pair or advance a cursor.
4. Normalize actual typed native events into faithful utterance, formal content,
   activity, foreground state and supported request records. Append chunks to
   the same native entity; full content replaces it. Tool/thought/plan output is
   activity rather than assistant prose. Unknown source fields remain in raw
   capture; ordinary owner DTOs never expose raw protocol/debug data.
5. Extend preserving storage only as needed for independent product delivery facts,
   original typed responses and bounded current read projection. Publish raw,
   normalized payload, source cursor and any derived projection in a completed
   transaction. Store rechecks current owner/source/Control authority at mutation
   boundaries; PostgreSQL coherent reads do not assume global snapshot isolation.
6. Preserve the distinction between first source binding and replacement. Keep the
   initial accepted setup input on first binding. Replacement clears every prior
   session pair/projection/input/answer/delivery fact, marks incomplete loading,
   refills bounded pairs and completes only at the stored finite boundary. Agent,
   native context, account/authentication and supply identities remain unchanged.
   Missing old facts never grant replay; stale source writes/answers are fenced.
7. Product text submission persists exact input and short acceptance before native
   processing. The server operation owner retains admitted work beyond a cancelled
   browser waiter and records possible-forwarding evidence before crossing that
   boundary. Application operation identity and transient Control call identity
   remain separate. Exact duplicate payload recovers existing facts; changed
   payload conflicts. Lookup/read never dispatches work.
8. Execution-bound Connector operations use short local admission and later
   observation rather than waiting for preparation or a model inside a Control
   waiter. Preserve bounded processing facts across streams, with no persistent
   local outgoing journal. EOF, timeout, cancelled waiter or server restart cannot
   become proof of non-delivery. Capture replay never resends commands.
9. Running text uses the producer's supported steering operation with
   `idleBehavior: promptRequired`. Only an explicit `noRunningTurn`/non-consumption
   outcome permits ordinary prompt. Preserve exact native insertion correlation
   when available; never merge an input receipt and echo by text or timing.
10. Typed flat forms and permissions retain original questions/options and exact
    typed answers, including omitted, false, empty, declined and cancelled values.
    Unsupported required fields/modes are errors, not simulated refusal or URL
    credential fallback. Server response admission fences one effective answer;
    the native owner checks the actual still-current session/responder again.
    Historical/reconstructed requests do not become live response targets.
11. Exact current-work Stop has no confirmation and does not change Computer power
    or retire the harness. Preparation, process alive, initialization, applied
    authentication, native readiness and work completion remain distinct facts.
    Reads/connectivity neither start native work nor power on a Computer.
12. Generate typed Rust/Node contracts from proto for private owner region/body/
    operation/supply APIs and exact Connector operations. Respect existing 64-KiB
    frames with bounded transfer/assembly and complete-content continuation.
    Owner reads recheck Team/private ownership on every request and bind body
    continuation to the same source/body identity. Do not add a browser event
    feed, broker, durable browser cursor or indefinite authentication context.

## Owned Workstreams

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Local source and native integration | `/root` | `rust/crates/harness/src/**` and its owning tests/examples | phase 2 owner/storage/SDK | retained source, explicit replay policy, bounded exact operations/observations and steering | deterministic SDK/process/source fixtures, no model calls |
| Domain/storage/projection | `/root` | `rust/crates/domain/src/conversation*`, `rust/crates/persistence/src/conversations/**`, preserving paired migrations/preparation and owning tests | phase 1 admission/pairs | product evidence, original responses, coherent bounded projection and atomic publication | SQLite/PostgreSQL contracts, race barriers, migration/downgrade preservation |
| Service/Control/wire composition | `/root` | `proto/chapchoo/**`, `rust/crates/{protocol,service,rpc}/**`, `rust/apps/chapchoo/src/**`, source-generated Node RPC output and canonical tooling if needed | capture and Store boundaries | accepted operation owner, exact current calls, source transfer and typed owner reads | generated-client verification, deterministic transports, live Node/Rust both engines |
| Current contracts/evidence | `/root` | this phase plan and affected semantic Spec pages | integrated implementation | current-only claims and exact remaining live gates | semantic lint, docs/hooks; review after complete stack |

- Integration order: captured-copy model and replay policy → typed normalized facts
  and preserving Store operations → generated bounded wire → source catch-up and
  server operation owner → native exact dispatch/steering/response/Stop → coherent
  owner region/body reads → focused/canonical local gates → phase PR → phase 4.
- Removal obligations: replace observe-only assumptions in owning Connector command
  dispatch where conversation operations activate. Clear old affected conversation
  state at source replacement. Keep Computer power, native context, credentials
  and sample Chat until phase 5 activates the real UI replacement.
- Absence verification: no command replay from retained history or missing evidence;
  no live authority from a historical request; no old-source independent input/
  answer/delivery facts after replacement; no raw/debug owner DTO, transcript
  credential traffic, upstream repair, outgoing WAL, browser feed or new broker.
- Non-goals: central OAuth renewal/file publication, production UI replacement,
  active model usage, generic versioning/event-sourcing, new idle policy, scope
  privilege relaxation, automatic Computer restart or merge/deployment.

## Validation and Context Checkpoint

Root owns focused tests plus final `scripts/check-rust.sh`, `scripts/check-rpc.sh`,
document validation and commit hooks. Exercise coherent source/record continuation,
large UTF-8 body/event transfer, concurrent replacement/admission, exact duplicate
and changed-payload conflict, cancellation after admission/possible forwarding,
recording failures, initial binding versus replacement, finite catch-up while
append continues, source loss, delayed/identical echoes, request closure and
steering non-consumption. Use explicit barriers and owned state, not sleeps as
ordering proof. Keep these combinations below browser E2E.

Phase 2 is open as PR 98 and its final local hooks passed. Model-free restricted
Docker acquisition/cleanup and strict-v2 initialization were verified; native
session creation requires human authentication (`-32000`). Leave only dependent
live tests pending, conserve model usage and continue all independent work. No
remote CI/review inspection or requests until all six phase PRs exist. At phase
boundaries check missing approved behavior and unauthorized additions in both
directions; return only actual new material authority conflicts to design.

### Current Internal Checkpoint

The retained CaptureLog, explicit captured-versus-initial replay policy, native
foreground/request identities, supported steering and bounded input processing
facts are implemented. Numeric/string RPC request IDs remain distinct and fresh
response targets bind their actual captured occurrences. Missing or uncertain
operation evidence never dispatches another native effect. Native protocol/capture
termination reuses whole-scope retirement; task completion and server-link loss do
not. No user credential or model prompt is involved in these deterministic tests.

Preserving migration 13 adds normalized record/field projection. Source pair,
projection and cursor publish atomically; source replacement removes all of them.
Owner metadata/keyset and field continuations recheck private authority and bind
the same source/content identity. The service normalizes supported typed ACP v2
utterances and formal activity, retains original requests, and represents
unsupported required requests as errors rather than user refusal or consent URL.
The full local Rust gate passed for that earlier internal checkpoint, including
both engines; later activation changes have their own focused evidence and
still require the final phase gate.

Durable input forwarding and original typed answer storage are now implemented
internally. One exact-route forwarding grant persists possible-delivery evidence
before the native boundary; duplicates only recover facts. Explicit live request
observations bind captured occurrences and single-use responder tokens; historical
projection alone grants no response authority, closure cannot be undone by a late
open observation, and concurrent answer admission fences one effective response.
Source replacement deletes these independent facts too. Native form submission
validates types, required presence, ranges and choices without consuming the
responder on invalid input. Local responder acceptance is not remote consumption.
The server conversation service reads the immutable normalized original question,
validates typed answers and exact native scope outside locks, then revalidates
atomic live-answer admission. Focused service tests and both-engine storage
contracts pass for this boundary.

Component-only capture wire and its generated Rust/Node clients are now connected
to the actual Connector read worker. It discovers current state, resolves exact
source cursors, assembles bounded byte pieces, publishes atomic pairs and completes
finite passes independently of native work. A model-free real Connector process
test transfers a saved UTF-8 source larger than one frame and verifies exact raw
bytes and completion; library/actual-RPC tests cover partial/conflicting pieces,
capacity ownership, duplicate observations and replacement fencing.

The internal input service now owns accepted work beyond the browser waiter,
returns small acceptance references before processing completes, skips processing
on duplicate/new-server receipt recovery, and retains bounded count/byte capacity
and drain failures. Store setup facts separately freeze the requested saved supply
and possible forwarding before source readiness; repeated grants are denied and
fresh context requires positive sealed admission rather than absent temporary
state. Native operation handles share one physical owner without retirement
authority. The generated Control operation union now includes setup and input
references. Setup dequeue revalidates the sealed Store intent; local queue admission
and later authenticated setup/input result publication remain separate. Full input
uses 32-KiB authenticated byte pulls, exact final source revalidation and native
processing facts rather than embedding oversized bodies in Control frames.

The concrete `ConversationControlProcessor` drives these boundaries using existing
PeerDelivery and current owner authority. It waits for setup observations and finite
source readiness within a bounded processing budget, then obtains a distinct text
delivery grant. It does not wait for model completion or replay missing outcomes.
The actual child Connector fixture composes this processor with the input service
and verifies independent short acceptance and later restricted-profile rejection.
Root client-service lifetime composition is connected to the existing backend
shutdown and shared local/remote PeerDelivery. Actual native observations/current
request references and exact Stop admission/check/result routing are implemented.
The client-only owner service exposes current observation, saved supply,
Stop/result, bounded complete input upload and small acceptance/delivery lookup.
Recent/older typed record metadata and source/update-bound full normalized field
reads use 32-KiB pieces without returning raw captured RPC envelopes. Incomplete
owner uploads have finite count/byte/lifetime admission and never become product
inputs; complete handoff enters the existing process-owned input service. Status
reads select metadata rather than loading full submitted text. Public UI
activation and authenticated model success are not claimed.

Typed response routing is connected through a cancellation-independent answer
owner, one-time durable grant, exact Control reference, complete bounded
Connector pull, SDK response conversion, native fingerprint/responder check and
later authenticated result. The production root composes and drains this owner.
Owner typed-answer upload and metadata-only result lookup are registered.
Current live responder references link to original normalized question/schema
fields and expose an accepted answer's Request ID without restoring liveness.
Model-free tests cover retained originals, exact duplicate/result conflicts,
large UTF-8 answer upload/pull and process-owned short acceptance; successful
real native responder consumption is not claimed.

Source/operation-bound full retained-answer reads now expose complete typed
Protobuf bytes with bounded pieces and one shared transcoding reader; they do not
depend on a still-live responder.

Known-result capacity release now follows successful durable publication.
One local invocation lease spans native result, report and ordered actor release;
the same command-checking loop enqueues release so an already-checked delayed
duplicate still encounters the held lease. Uncertain or different results and
facts behind refused reports remain retained. Tests exceed the retained bound with acknowledged outcomes and verify
that ordinary duplicate admission remains blocked before ordered release.

The actual child Connector fixture now transfers original large typed answer
bytes, routes exact Stop, publishes known missing-native-owner rejection for both
and completes owned shutdown. This does not establish real model-side response
consumption or cancellation. Owner input-result projection includes independent
setup supply/outcome evidence so known setup failure remains distinct from
missing text delivery.

Final integrated local validation passed on the frozen phase diff:
`scripts/check-rust.sh` (workspace format/Clippy/private docs, complete tests with
both database engines and dependency boundaries), `scripts/check-rpc.sh`
(canonical generated-client verification and 129 actual Node/Rust checks per
database), documentation metadata and 18 validator tests, whitespace and
byte-identical approved Requirements/ADR/Design comparisons.
The actual Connector fixture additionally exercises answer/Stop rejection and
owned shutdown; the known-result release tests exceed the 256 retained-fact bound
without evicting uncertainty. No real credential or model call was used.

Implementation and local integration are complete. Commit hooks and the stacked
phase PR remain the delivery boundary before phase 4 begins; remote CI and the
single independent review remain deferred until the complete six-PR stack.
Existing sample
Chat and credential-delivery boundaries remain unchanged until their owning
phases. Design delta: None.
