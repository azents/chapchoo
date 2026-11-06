---
title: "AI connections and saved Agent authentication"
---

# AI connections and saved Agent authentication

General manages the [backend authentication catalogue](backend-authentication.md)
and each Agent's saved reference. This surface performs no credential delivery,
Computer application, refresh, Codex launch or model inference. Stored credentials,
a saved selection and actual provider access are distinct facts.

## Scope, navigation and rendering

`/teams/my/connections` selects the current User's personal records;
`/teams/{teamHandle}/connections` selects only that collaborative Team's records.
AI connections is a same-context sidebar destination. The ordinary workspace
switcher and [cross-context navigation](workspace-navigation.md) remain unchanged.
There is no personal/Team section selector when the page has only one owning
scope. Team Owners/Managers manage Team records; members can consume eligible
records for their independently owned Agents but do not get management controls.
Current backend authority remains final if membership or role changes.

The route authenticates before scope resolution and server-renders its required
bounded catalogue snapshot. Pagination uses the `after` URL parameter and pushes
history; `connection` selects a management target in the URL and survives reload.
The client hydrates the same scope/page; changing scope never borrows another
catalogue as placeholder data. Unavailable reads remain errors, not empty lists.
Bounded forward pages offer an explicit First page action rather than claiming
to reconstruct the immediately preceding cursor.
SSR snapshots are bound to their initiating User ID and are hidden synchronously
when identity is withdrawn or replaced. Successful writes invalidate related
catalogue and Agent queries.

Cards identify alias, Codex/API kind and known provider email without inventing
missing account/plan metadata or an association count. ChatGPT registration is
labelled registered; API keys remain usage-unverified. Record management supports
rename, same-account reconnect, API-key replacement and in-use-protected deletion.
A deletion does not claim external revocation. Replacement discloses that all
Agents selecting the shared record refer to the replaced stored key and that its
API billing project may change.

## In-place registration and recovery

API-key entry has a labelled masked field with a temporary visibility toggle,
the official OpenAI issuance link and separate API-billing notice before Save.
The browser holds the raw key only in the transient controlled draft. Secret
mutations use the imperative generated transport rather than query/mutation
cache variables; success, cancel and uncertain submission clear the input.
The server copies UTF-8 input into a bounded Buffer, forwards it once and clears
the Buffer in `finally`. Catalogue/detail/result DTOs construct approved fields
explicitly and never return the key. Storage remains usage-unverified.
Registration/management uses a desktop modal and a bounded mobile bottom sheet;
the active attempt owns dismissal and focus independently of the outer picker.

Device registration collects an alias and starts one backend attempt. Active
interaction shows the actually issued code, fixed official verification link,
remaining validity, explicit advance/status actions and cancellation. Opening
the external page does not report success. Poll advances once; an uncertain
advance switches to status observation rather than replaying it. Provider
success returns directly to the originating selection/management task without
account review or a second Register action. Cancellation, expiry, failure and
supersession stay distinct. Expired/failed attempts expose a new connection
action without reusing the expired attempt. Same-account reconnect preserves its
record and requires the captured expected Request ID.

Uncertain catalogue writes retain the original Request ID and offer request-result
observation, not a fresh operation. No observed result is not cancellation proof.
Observed accepted device identities resume status; observed record/deletion
identities finish their original task. A known accepted write is not reclassified
as rejected when its subsequent catalogue read fails. An accepted record that is
no longer available can return to the owning picker without selecting a replacement
or claiming that the original write failed. An uncertain cancellation
with no accepted receipt can still observe its retained attempt ID through
DeviceStatus, including registration or a terminal result, without replaying the
cancellation or advancing provider work. Modal dismissal cannot discard ownership
of an active or uncertain attempt. Generation/identity fences ignore stale
completions. Invalid sessions withdraw shared identity for both ordinary queries
and imperative secret mutations; business denial or provider unavailability does
not sign out the User. The existing POST/Origin/cookie/safe-error boundary applies.
An explicit conflict from a newly issued catalogue/selection mutation is rejected
before its receipt, not an unknown transport outcome. The UI refreshes related
state and permits a deliberate new action with a new Request ID; it does not
replay the rejected write. Original uncertain requests still use their read-only
receipt recovery, and known accepted responses retain accepted-outcome handling.

## Creation and existing-Agent selection

New creation proceeds environment → Codex authentication → existing profile.
It offers only registered eligible cloud environments, keeps the cloud icon and
selected card, and requires an explicit credential choice. Back/Next retain the
environment, credential, image, name, handle and palette without retaining secret
input. No first-record/default fallback is added. Selection can use the caller's
personal catalogue and, for a collaborative Agent, that exact Team's catalogue.
Registration opens in place and returns the new eligible record selected.
The complete immutable creation payload includes authentication ID and retains
the [original uncertain-request recovery](agent-profile.md#final-creation-and-client-candidates).

Existing detail keeps profile and Computer controls and adds a compact Codex row
showing saved alias or no selection. Its picker uses the same scoped catalogue;
unchanged selection is disabled. Selection compares the captured credential ID
and last accepted selection Request ID; it changes no other Agent or shared
record. An uncertain save is recovered by its exact request result, never a
blind repeated write. In-place registration selects the accepted record for that
Agent. Cancel leaves its prior reference unchanged. The saved reference is SSR
primary data; the unused picker catalogue is supplementary and is queried only
when opened, so its availability does not block Computer detail or chat.

## Verification boundary

Real generated Node/Rust RPC verifies catalogue contracts on both databases;
Store fixtures cover atomic creation, receipt recovery and credential deletion
races. The assembled browser journey registers synthetic keys, preserves drafts
across steps, changes only the current Agent's saved reference and checks its SSR
projection while retaining existing Computer lifecycle and account-isolation
journeys. Owning component tests exercise key/device presentation, direct Device
completion and uncertain-advance status recovery through a browser transport
fixture. This fixture does not call OpenAI or bypass production issuer/TLS policy.
Service/RPC injected-provider tests own actual acquisition acceptance. Rendered
component checks cover responsive layout, fonts and native-resolution evidence;
these are not real OAuth, paid inference, native application or rollout evidence.

## Connections

- [Backend authentication](backend-authentication.md) — durable authority, scoped receipts and direct device completion.
- [Agent profiles](agent-profile.md) — final creation fields and frozen-payload recovery.
- [Computer web](computer-web.md) — independent Computer observation and controls.
- [Workspace navigation](workspace-navigation.md) — canonical Team context and Profile separation.
- [Web session](web-session.md) — request-local credentials, cookie disposition and identity withdrawal.
- [Primary rendering](web-rendering.md) — SSR handoff and URL-owned selection.

## References

- [`authentications.ts`](../../../../typescript/apps/web/src/shared/server/authentications.ts)
- [`external-authentication.ts`](../../../../typescript/packages/web-core/src/external-authentication.ts)
- [`useConnectionPanel.ts`](../../../../typescript/apps/web/src/shared/authentications/useConnectionPanel.ts)
- [`useAgentAuthentication.ts`](../../../../typescript/apps/web/src/shared/authentications/useAgentAuthentication.ts)
- [`useAgentCreation.ts`](../../../../typescript/apps/web/src/features/computers/containers/useAgentCreation.ts)
- [`computers.spec.ts`](../../../../typescript/apps/e2e/src/computers.spec.ts)
