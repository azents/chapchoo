---
title: "Conversation Model Selection Decisions"
created: 2026-11-20
tags: [agent, conversation, harness]
document_role: primary
document_type: adr
snapshot_id: model-261120
---

# Conversation Model Selection Decisions

Confirmed product authority: [model-261120/REQ](../requirements/model-261120-agent-model-preference.md).

## Fixed boundaries

The connected harness chooses a new session's initial model. Chapchoo keeps
explicit selections within their same-session, same-connection scope. Recovery
is manual. An unknown outcome cannot authorize replay, substitution, or a
preference reset. External harnesses remain unmodified and use their supported
interfaces. See `model-261120/REQ-1`, `REQ-3`, `REQ-4`, and `REQ-5`.

## Open decision map

| Topic | State | Consequence to resolve |
| --- | --- | --- |
| Scoped preference authority | Accepted — `model-261120/ADR-D1` | The Chapchoo server owns the explicit choice, scoped to proved continuity of the native session and applied harness/account connection. |
| Selectable options and failure evidence | Accepted — `model-261120/ADR-D2` | Use the prepared session's advertised options and require separate proof of model unavailability and non-start for the dedicated recovery. |
| Save and new-execution ordering | Accepted — `model-261120/ADR-D3` | The server orders saves against one operation's model binding and actual native start, including a prompt held after publication. |
| First-integration execution scope | Resolved by `model-261120/REQ-4` | Keep native commands unchanged; guarantee the selected model for ordinary conversation and Chapchoo work continuation. |

Storage shape, API names, feature-local helpers, and deterministic fixture
composition are local details once the material boundaries are decided.

## Evidence for the open decisions

### Session and connection identity

[Connection-time preparation](../spec/domain/conversation-storage.md#connection-time-preparation)
may prepare an empty conversation, reuse its existing native owner, or resume
its exact retained source under another Computer execution or Connector route.
Failure to resume does not grant creation of a replacement. The
[captured source](../spec/domain/native-harness.md#retained-captured-source)
retains snapshot and native-session identity independently of a live process.
The [applied authentication boundary](../spec/domain/backend-authentication.md#internal-applied-execution-boundary)
is distinct from the latest saved authentication selection; rotation of
material for the same record does not replace that record's identity. ChatGPT
same-account reauthentication proves account continuity; replacing an API key
retains its record ID without verifying that the replacement represents the
same account. Record equality alone cannot prove the latter. The launch
operation, Computer execution, and Connector route remain authority fences
but cannot on their own identify the durable preference scope.

A server-owned record could reuse private owner transactions and request
recovery while relying on the native owner for the facts about the prepared
session. A Computer-owned record alongside its retained native context could
serialize writes with local prompt publication, but needs a separate durable
write/receipt/read path through the Connector and still needs server-side user
and Team admission. The released harness is not a durable authority: the
same session resumed by a fresh process reported A even after B was selected
and used. Reapplying B before the first new prompt made that prompt use B.

### Selectable options and confirmed failure

The [Codex ACP adapter](https://github.com/agentclientprotocol/codex-acp/blob/ad09e0ead18511a65a0761cac832aa3087219b10/src/CodexAcpServer.ts)
uses session-held options when setting a model. ACP v2 exposes a model option
at setup and subsequent whole-set configuration observations; a successful
setting acknowledges configuration, not account entitlement or prompt use.
The tested missing-model config request returned generic invalid parameters.
A missing choice and a confirmed model-specific reason for non-start are not
interchangeable.

The [native model-list schema](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server-protocol/schema/json/v2/ModelListResponse.json)
reports data and a cursor, not refresh provenance. Unmodified Codex 0.165.0
and 0.171.0 returned a complete-looking list without a remote-only model when
the controlled upstream instead returned HTTP 503. Completing pagination
does not prove account-level absence. A configured gateway allowlist may
qualify only that gateway; adding a gateway or a second credential path is
not needed to interpret the current contracts.

The outage was also reproduced at the ACP boundary: an identical controlled
connection exposed and accepted a fixture model while upstream returned HTTP
200, but omitted and rejected it with generic `-32602` when upstream returned
HTTP 503. Neither configuration attempt sent a prompt. In a separate probe,
configuration of a listed model succeeded, then an actual provider request
returned HTTP 404 `model_not_found`. The adapter had already reported Running
and input insertion; it then reported an error and Idle. An exact downstream
model error is therefore not necessarily a pre-start refusal. Neither probe
used a real account.

### Execution ordering

[Native operation ownership](../spec/domain/native-harness.md#session-protocol-and-operation-ownership)
allows a new ordinary prompt only after the harness explicitly did not consume
steering. An undecided normal ending defers that path; resumption revalidates
the original forwarding grant. Internal continuation instead has a one-use
grant and publishes against the exact live Idle ending and current route.
Both paths are distinct from a steer that joins active work. A database read
followed by awaited configuration and a prompt cannot by itself exclude a
save being confirmed before that prompt is sent.

Local enqueue is not native start. The ACP v2 SDK can leave prompt acceptance
pending until input insertion while the harness holds a sent prompt. A local
enqueue acknowledgement or a route expiry cannot authorize a newer save if
the earlier prompt can still begin with the older selection. Any proposed
serialization must account for the actual start boundary, or exact evidence
that future start is impossible, without holding a database transaction open
across native I/O.

A loopback Responses fixture with unmodified Codex 0.165.0 and codex-acp
2.8.0 observed A throughout a held turn that accepted a steer. After normal
ending, the same-session ordinary prompt after explicit non-consumption
and reconfiguration used B. A direct prompt after a subsequent normal ending
used A after reconfiguration. Together with the fresh-process resume probe,
this establishes released primitives, not Chapchoo save/dispatch ordering,
continuation-grant integration, or current account availability. The
[managed loader](../../../rust/crates/harness/src/preparation.rs) has a wider
accepted supply range than the adapter version tested here; compatibility
must be evaluated on supported capabilities rather than inferred from the probe.

## D1. The Chapchoo server owns the scoped explicit choice

- Decision: `model-261120/ADR-D1`
- Accepted: 2026-11-20 (KST), by the requester
- Authority: `model-261120/REQ-1`, `REQ-2`, `REQ-3`, `REQ-4`

The Chapchoo server is the sole durable authority for an owner-confirmed
explicit model selection. Owner reads, writes, and result recovery preserve
existing private Agent and current Team admission. The native owner reports
facts about the exact prepared session and its applied harness/account
connection; a harness observation is not a save receipt. There is no duplicate
durable authority on the Computer and no saved initial model.

A selection applies only while the native session and applied harness/account
connection are proven continuous. Reopening, a transient Connector or process
change, and an exact retained-session restore do not by themselves change the
scope. An actual new session or actual binding change does not inherit the
selection. Unknown continuity is neither proof of continuity nor proof that a
replacement may be started. The chosen store must not use newer saved
authentication intent to describe an unchanged live native owner.

Persisting the preference on the Computer could put the save closer to prompt
publication, but would add a Computer-dependent save and receipt authority
behind the same required server admission and would need a separate recovery
path. The server aligns reads and receipts with existing ownership and
durability. This decision does not equate stored preference with effective
in-flight model, determine the availability evidence, or settle the
save-to-start ordering protocol.

## D2. Use session options and reserve dedicated recovery for proved pre-start failures

- Decision: `model-261120/ADR-D2`
- Accepted: 2026-11-20 (KST), by the requester
- Authority: `model-261120/REQ-1`, `REQ-2`, `REQ-5`

Use the prepared and correctly scoped harness session's supported model
configuration observations as the source of advertised model choices. A
reported current model and advertised alternatives describe that session;
neither asserts upstream entitlement or actual model use in a later turn.
Do not use a separate gateway, provider credential path, or synthetic session
to fetch a second catalog. A failed observation is not a verified empty set.

The dedicated recovery requires both model-specific unavailability evidence
for the exact session/connection and evidence that the affected request did
not start. An omitted option, generic configuration rejection, or provider
failure after start does not satisfy that pair. Report failures or uncertainty
through the existing truthful conversation states when the pair cannot be
proved, preserve the saved choice, and retain ordinary access to the picker.
When both facts are proved, use the manual recovery specified in `REQ-5`.

The first Codex integration can deliver selection, save, and application
through supported released interfaces without claiming that its ordinary
connections can trigger the dedicated recovery. Requiring that trigger for
all supported Codex connections would defer delivery pending new supported
evidence. An incorrect recovery banner would misstate what happened and could
encourage resending work that had already started. Neither is required for the
confirmed conditional recovery.

## D3. The server orders owner saves against actual new execution starts

- Decision: `model-261120/ADR-D3`
- Accepted: 2026-11-20 (KST), by the requester
- Authority: `model-261120/REQ-3`, `REQ-4`, `REQ-5`; `model-261120/ADR-D1`

Under the same server authority that confirms a save, reserve at most one
unsettled start for the exact scope and newly starting operation. Resolve
the currently committed explicit choice, or its absence, in that reservation.
Ordinary input participates only after explicit steering non-consumption;
continuation participates only once the exact live ending is eligible.
Existing route, non-consumption, and single-use ending checks still apply.

A competing save committed first wins. If the start reservation won, a
newer save does not commit until exact evidence establishes the operation's
actual start with the model already bound, or establishes that the
operation can no longer start. During unresolved start, a save may return
confirmed non-save and preserve the draft. Save never waits for a confirmed
active execution to finish. It does not mutate live configuration, cause
retirement, or start or replay any input.

Local prompt enqueue, a generic activity observation, timeout and route
loss are not settlement. Preserve exact pending-start authority across
server restart and transport loss, and settle only with exact evidence. Do
not hold a database transaction across native I/O. With a Computer-owned
sequencer the same guarantee would need exclusive owner handoff and would
make saves depend on the live Computer even outside an unsettled start.
Server ordering keeps serialization and the durable save authority together.

## Feasibility finding after D3: Codex command paths can select another model

In isolated strict-v2 runs against unmodified Codex 0.165.0 and codex-acp
2.8.0, configuration acknowledgement selected B before any inference
request. A plain prompt sent B, while `/review`, `/compact`, and
`/goal <objective>` sent A. The compact fixture established the outgoing
model but did not complete a valid compaction. Compact and goal also
acknowledged exact input insertion before reporting Running. Input
acknowledgement is therefore neither a general model binding nor a general
start witness. Codex review can additionally use an independent review
model. None of these cases used a live account.

The existing Chapchoo path forwards the submitted text verbatim. The tested
adapter sends the model explicitly for a plain prompt but performs separate
native operations for these commands. Its ACP configuration setter does not
update the underlying model for those native operations. A server reservation
can order the model choice, but cannot make a native command use it. Other
supported interfaces could only close the gap with evidence of the exact
operation, model binding, ownership and start order.

The requester confirmed the narrower first-integration scope on 2026-11-20
(KST). The confirmed `REQ-4` preserves native command pass-through and
applies D3 to ordinary conversation and Chapchoo work continuation. Existing
command authority does not take on any new model-setting authority. This
clarification does not replace D1–D3 or require a separate technical choice.

## D4. Order saves against accepted submissions, without claiming model use

- Decision: `model-261120/ADR-D4`
- Accepted: 2026-11-21 (KST), by the requester
- Authority: revised `model-261120/REQ-4`
- Supersedes: D3's actual-start settlement boundary; retains server-owned
  serialization, operation identity, and uncertainty handling.

Codex can attach a plain prompt to a turn that started independently after the
idle check. The supported ACP prompt receipt identifies the input that was
inserted, but does not distinguish a newly started turn from such an attachment.
Do not derive model use from that receipt or generic foreground observations.
Use the released model configuration operation before prompt submission so
the approved model is requested if the prompt starts a new turn. Close the
ordering reservation on exact insertion acknowledgement, not on a claim about
either execution or inference. A later save can then succeed without waiting
for the turn to finish. An input already steered has no model setting effect.

Unknown insertion remains uncertain and is not replayed or treated as
non-start. Non-publication and exact physical retirement retain their separate
meanings. D2's dual evidence requirement for unavailable-model recovery is
unchanged. This uses unmodified external harness releases with no shadow
catalog, model override, or inference inspection.
