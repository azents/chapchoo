---
title: "Codex Conversation Final Feasibility Boundaries"
created: 2026-11-09
tags: [agent, conversation, codex, acp, architecture, security, testing]
---

# Codex Conversation Final Feasibility Boundaries

This supporting source review grounds the
[primary Design](../design/chat-261108-text-conversation.md) for
[chat-261108/REQ](../requirements/chat-261108-text-conversation.md) revision 24
and [chat-261108/ADR](../adr/chat-261108-text-conversation.md) D1–D33.
It does not authorize new policy, implement conversation behavior or certify live
acceptance. No credential exchange, model invocation, native installation,
container provisioning or production change was performed for this review.

`feasible` below means a concrete reusable source-level path exists. `conditional`
means implementation and the specified acceptance evidence are still required.
Neither label means the unimplemented product feature already works.

## 1. Browser Reads and Session Lifetime

### Source findings

- The General app's `RpcProvider` uses `httpLink` with POST method override.
- The tRPC route uses `writableRejection`, unbatched ordinary request handling,
  a fresh context and `privateResponse` after execution. Streaming response
  headers are deliberately unsupported. Same-Origin and POST checks remain.
- `RequestSession` coalesces renewal within one request. Its outgoing cookie is
  finalized by that response; SSR `readOnlyIdentity` does not rotate credentials.
- Store session renewal locks/revalidates the current session and has existing
  token rotation/overlap handling. This is not proof that arbitrarily delayed
  concurrent browser responses cannot expose a cookie-ordering problem.

### Design consequence

M10's bounded unary active-region reads fit this boundary without a streaming
cookie redesign. Every protected read rechecks current private Agent/Team
admission. Hydrate the actual SSR data, then schedule active-view updates; do not
turn mounting into a duplicate initial query. Maintain one in-flight read for the
current region, coalesce its successor, and ignore responses belonging to an old
Agent/anchor/component lifetime. Older-page results cannot overwrite newer
already-loaded records. Scope read keys to the actual content, not a global
browser event cursor.

### Acceptance

**Source-level fit: feasible. Product responsiveness/session/load: conditional.**
Use deterministic delayed responses for hydration, Agent navigation, old-page
versus live reads, reconnect and token rotation/overlap. Check actual response
cookies, no-store headers and current owner admission. Separately measure visible
update latency and bounded request/backend work under the supported profile.
Record cadence, payload/region bounds, concurrency and results; no latency or
throughput measurement was performed here. A failed session/load check is an
implementation issue to resolve within current authority, not permission to add a
durable browser feed or claim polling has already passed acceptance.

## 2. Central Refresh and Credential Delivery

### Source findings

The current `chapchoo-openai-auth` adapter implements device acquisition only.
Its direct-HTTP exception does not automatically cover a new refresh operation.
The OpenAI Node SDK API surface inspected for this review exposes no matching
Codex ChatGPT refresh operation. The pinned Codex login source uses a private
`oauth` module; calling that internal module is not public SDK reuse.

An established public Rust OAuth client has a concrete supported refresh path:
`oauth2` 5.0.0 `Client::exchange_refresh_token`, public token response/extra-field
interfaces, and public `AsyncHttpClient`. It constructs the refresh grant and
parses the response. With request-body client authentication it can carry the
existing public client ID without a client secret. A bounded HTTP implementation
can execute that SDK-produced request without reconstructing provider grants.

The native 0.165.0 source at
`360319d8dee62d87e2d59fc0c21805d79519cfed` uses
`request_chatgpt_token_refresh` with the same public client identity as the current
device adapter, but its private helper selects JSON encoding and accepts optional
replacement fields. The public OAuth library uses standard form encoding.
Existing device authorization-code exchange already uses form encoding at the
fixed token route. This is credible interoperability evidence, not proof that a
refresh token acquired through the exact existing flow has passed the public
SDK's request/response contract.

### Design consequence

Use a supported OAuth SDK public refresh operation as the implementation path.
Do not copy the native private helper, convert a generated form request into an
unapproved hand-written JSON grant, or silently extend the device direct-HTTP
exception. Keep the existing fixed official issuer/client/token boundary; this
review does not authorize dynamic discovery, registration, a different client,
new OAuth scopes or a different sign-in product. Contemporary discovery metadata
is not authority to replace the route used by the pinned credential flow.

Use the existing verified-TLS/no-redirect/no-automatic-retry transport restrictions
with bounded successful and error bodies before SDK parsing. Sanitize SDK errors
at the adapter boundary: parse errors can retain provider bodies and must not be
logged, formatted into public responses or attached as ordinary source chains.
Do not claim dependency-owned allocations are comprehensively zeroized.

The server admits one refresh attempt for the actual protected credential source,
coalesces eligible consumers and owns the external work beyond a browser waiter.
Use existing exact request/source/attempt identities and Store revalidation rather
than a generic credential-version platform. Recheck current source, account,
attempt and authority before result publication. Same-record reauthentication,
record deletion or scope loss fences late results. No secret resurrection.

Deliver only the current eligible short-lived payload to the exact execution's
applied record. Serialize file publication for that native owner and coalesce
superseded updates to the latest admitted source; old stream/owner responses
cannot overwrite the new owner. Reconnect obtains current eligible credentials,
not replay permission for an old delivery or native command. Publication is not
native adoption. D33 does not block deletion for applied-only use and does not
promise immediate revocation of material already forwarded.

### Acceptance

**SDK operation and server ownership plan: source-backed. Exact provider/native
interoperability: conditional.**
Before shipping refresh, validate the SDK-generated form grant and typed response
against the exact existing device-acquired credential flow. Verify optional
rotated refresh/ID fields, actual account identity, expiry and rotation semantics;
do not invent missing metadata or discard a valid unchanged refresh credential
merely because a successful standard response omits its replacement.

A possibly consumed refresh whose response is lost, malformed or cannot be
published is uncertain; do not automatically repeat it using the old credential.
Retain explicit reauthentication recovery, not an invisible provider retry loop.
Fixture tests cover one admitted exchange for shared consumers, replacement and
deletion races, stale delivery, reflected-secret errors and restart uncertainty.
Live checks require separately authorized credentials and cannot be certified by
fixtures. If the supported SDK cannot express or parse the confirmed provider
contract, stop this implementation lane and return the concrete gap for a scoped
exception/design decision; no direct-call fallback is approved by this note.

## 3. Supported Runtime and Native Acceptance

The current image runs UID/GID 10001 and its base installs CA certificates and
`libgcc-s1`; it does not establish the complete ABI prerequisites of downloaded
Node/Codex. The Computer image/preparation implementation must supply the actual
supported runtime libraries, without changing the non-root/read-only-rootfs,
capability-drop, no-new-privileges or saved-storage boundaries. Writable native
HOME/state/TMP must be explicit under the allowed saved environment.

Docker pins the Connector entrypoint/arguments and rejects a nonempty PID-mode
override. Kubernetes creates a restricted one-container Connector Pod and rejects
host PID, but its current collision predicate does not establish that
`shareProcessNamespace` is disabled. Intended configuration is not runtime proof.

D1/D14 require a dedicated owned process scope. Before native admission, establish
the actual Connector/PID1/non-shared namespace and visible process-scope conditions
on which retirement relies. Owner death can then rely on namespace-init teardown;
normal controlled retirement still needs bounded cooperation, scope-wide forced
termination and actual quiescence/reaping evidence. A process group alone cannot
contain detached/reparented descendants. Do not silently restart/power-cycle the
Computer as a substitute for proving retirement. Ambiguous scope or retirement
blocks native/successor admission.

**Source-backed enforcement direction and launch plan: conditional.**
Actual ABI/sandbox/tool behavior, detached descendants, owner death, normal
retirement, adapter-internal replacement, exact steering and native file-token
adoption need supported-image tests. Keep these out of a long browser E2E matrix.
No successful native launch or process cleanup is claimed. If a supported profile
cannot meet this boundary, report the failed gate; do not relax security or adopt
an uncertain predecessor.

## 4. Source Replacement and Admission

D18/D19/D30 already select snapshot-bound complete replacement. Implement its
read/input fence using the same conversation/source authority as pair ingestion:

1. Distinguish initial no-cursor binding from loss of a prior snapshot; preserve
   the accepted input that initiated first setup.
2. In one DB-only source replacement operation, clear the affected conversation
   data and mark the new source incomplete. Recheck that source/readiness in the
   Store operation that admits new effect-bearing input/answers.
3. Persist each raw/normalized pair and its claimed cursor atomically. A bounded
   transfer carries one source identity and complete-record boundary. Incomplete
   chunks neither advance progress nor appear as a complete body.
4. Complete replacement only when that captured finite boundary is ingested.
   Later appends continue on the same capture path; continuous output does not
   require waiting forever for an idle harness or enabling native replay.
5. Read content and source/readiness coherently; PostgreSQL's existing ordinary
   transaction helper does not by itself make separate SELECTs repeatable-read.
   Chunk continuation and final assembly remain bound to the same source/body.
6. Reject late old-source ingestion/dispatch and suppress stale old-view results.
   Keep exact known current-work Stop/shutdown separate from history readiness.
   Work forwarded before replacement is not retroactively undone or made safe to
   replay by deleting its old product evidence.

**Coherent implementation plan: feasible; product/concurrency proof: conditional.**
Both-dialect deterministic tests must cover initial binding, replacement versus
admission, incomplete reads, finite-boundary completion with ongoing append,
partial transfer, source changes during body reads and late old-source results.
Use ordinary conversation loading/error presentation with preserved draft input
and reachable recovery; internal reconstruction terms remain diagnostic only.
Do not add staging, an old-data archive or a durable browser cursor.

## Evidence Entry Points

- [Browser client](../../../typescript/apps/web/src/shared/rpc/provider.tsx)
- [Web route](../../../typescript/apps/web/src/app/api/trpc/[trpc]/route.ts)
- [Web transport boundary](../../../typescript/packages/web-core/src/boundary.ts)
- [Request session](../../../typescript/packages/web-core/src/session.ts)
- [Stored session renewal](../../../rust/crates/persistence/src/sessions.rs)
- [Device-only provider adapter](../../../rust/crates/openai-auth/src/lib.rs)
- [Authentication deletion](../../../rust/crates/persistence/src/external_auth/catalogue.rs)
- [Image prerequisites](../../../images/chapchoo/Dockerfile)
- [Docker profile](../../../rust/crates/docker/src/containers/profile.rs)
- [Kubernetes profile](../../../rust/crates/kubernetes/src/lib.rs)
- [Pinned native refresh](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/login/src/auth/manager.rs#L1695-L1747)
- [Private native OAuth module](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/login/src/lib.rs)
- [Public OAuth refresh implementation](https://github.com/ramosbugs/oauth2-rs/blob/5.0.0/src/token/mod.rs)
- [Public OAuth transport interface](https://github.com/ramosbugs/oauth2-rs/blob/5.0.0/src/endpoint.rs)
- [Native authentication feasibility](chat-261108-native-auth-feasibility.md)
- [Execution and retirement grounding](chat-261108-execution-lifetime.md)
