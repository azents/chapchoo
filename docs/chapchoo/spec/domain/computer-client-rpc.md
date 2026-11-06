---
title: "Computer owner and installation inventory RPC"
---

# Computer owner and installation inventory RPC

Owner operations and redacted installation inventory are separate capabilities over current account authority.

# Computer Client and Inventory RPC

The Client service union registers Agents, Computers and ProviderCatalog. Admin
registers read-only ProviderInventory. These use the existing complete storage
operations and current account session authority, not a caller-supplied user ID.
Every generated message and outcome oneof has field-free Debug output.

## Owner operations

- Agents.Create accepts required Team, Request and Provider IDs, name, final
  handle, catalogue image ID and palette. The atomic creation
  receipt returns the original Agent on replay or its removed-creation outcome;
  replay after removal never creates a new Agent/Computer.
- Agents.List/Get and Computers.Get expose only the authenticated owner's private
  Agent and dedicated Computer projection. Installation administrator status does
  not grant access to another user's private Agent detail.
- Agents.List requires a Team ID and a nested bounded Page. It returns only the
  caller's Agents in that current membership, including a personal Team. Agent
  DTOs include their required Team ID. Detail and lifecycle admission require
  current membership in the Agent's Team as well as individual ownership;
  missing or inaccessible scopes return not-found. Creation Request ID reuse
  across Teams conflicts rather than allocating or returning another context.
- Computers.RequestLifecycle accepts Agent ID, Request ID, expected current
  Request ID and a typed Start/Stop/Restart/Delete action. Missing, unspecified,
  unknown and Create action values are rejected. This accepts durable desired
  intent; it does not wait for external Kubernetes convergence.
- Computers.RetryPreparation requires owner authentication, a fresh Request ID
  and the expected current lifecycle Request ID. It accepts only an explicitly
  blocked, proven-safe preparation with no possibly accepted Pod submission.
  It retains the same Agent, Computer, execution and allocated storage, starts
  a new preparation observation interval and returns current progress on exact
  retry. Unknown outcomes and stale contexts reject; no execution identifiers
  are exposed in the owner DTO. `preparation_retry_available` is server-derived.
- Agents.ForceRemove requires the exact current deletion Request ID. It removes
  the live Agent immediately while preserving the independent residual Computer,
  resources and terminal cleanup intent. Creation deduplication survives.
- ProviderCatalog.List returns authenticated creation choices, not credentials,
  ownership internals or management inventory.
- Agents.ListProfileImages returns authenticated bounded catalogue pages.
  Agents.CheckHandle requires an accessible Team and exact valid candidate,
  returns only availability and does not reserve the handle.
- Agents.GetInTeam accepts a resolved Team ID and exact Agent handle. Current
  membership and individual ownership are both required; global handle
  uniqueness does not bypass the Team scope.
- Agents.UpdateProfile accepts stable Agent ID and complete name/image/palette.
  It changes saved presentation only, retains the immutable handle and performs
  no native effect.
- Agents.CreationResult reads the authenticated owner's original Request ID
  without a mutation payload. It returns present assigned Agent/Team IDs,
  a removed assigned ID, or an absent observation. The latter is not evidence
  that an uncertain concurrent Create stopped.

Agent DTOs optionally carry the [saved backend authentication reference](backend-authentication.md),
without secret material or Computer-application status.
They carry the persisted handle, image ID, selected local asset key and palette in addition to name,
Team and Computer. Missing final creation fields reject rather than invoking a
server default or legacy mode. [Profile contracts](agent-profile.md) own grammar,
uniqueness, catalogue validation and full-payload recovery.

The Computer response separates desired intent, current lifecycle, current power,
last-known power, management connectivity, Connector connectivity, saved
environment and power observation time. Current power uses the same source-qualified
storage projection as lifecycle admission. Start/Stop/Restart action availability
uses domain capability/current lifecycle/management/power checks. Start additionally
requires verified saved environment from the current Provider connection. Delete
and force-remove availability are separate. Initial-start delay derives from the
existing first-dispatch timestamp, never a browser timer or mutation.

Connector connectivity is looked up through current DB directory/session
eligibility. It is not evidence of power-off, storage deletion or harness
readiness. Read failures remain failures rather than an empty/disconnected success.

## Read-only installation inventory

Every ProviderInventory method requires a currently authenticated installation
administrator within its complete DB operation. It has no mutation, retry,
resource adoption, cleanup trigger or private Agent-detail method.

- ListProviders returns stable Provider ID/name, capabilities and current
  management connectivity, including disconnected Providers.
- ListComputers is bounded within one Provider and reads Computers independently
  of live Agent associations. It uses the same power/evidence projection as user
  detail and includes residual/cleanup-completion facts. Its wire DTO has no User
  ID, private Agent ID/name, request credentials, mutation actions or file data.
- ListResources paginates the independently retained managed resource inventory
  for one Computer by exact kind/namespace/name, with optional original UID and
  execution binding, presence and observation timestamp. It never includes Secret
  values, bootstrap keys or user files.

All lists default to twenty rows and accept limits one through twenty. Agent,
Provider and Computer pages use stable opaque ID cursors. Resource pages use a
lexicographic kind/namespace/name cursor. A full page supplies its last key as the
next cursor; the following page may be empty. Frames remain bounded at 64 KiB.

## Verification boundary

Generated real gRPC memory clients exercise owner isolation (including Admin),
creation replay, invalid action/limits, exact current projection, management versus
Connector state, resource pagination, force removal and residual retention.
Both-engine storage fixtures cover Admin admission, Computer pagination and
residual visibility. These checks are not browser or actual Kubernetes lifecycle
acceptance; those remain separate integration gates.

## Connections

- [Agent handles and saved profiles](agent-profile.md) — immutable handles, final creation fields and bounded catalogue/availability.
- [Repository identity](repository-identity.md) — typed opaque IDs cross RPC without consumer-side decoding.
- [Private Agent ownership and Team deletion](agent-ownership.md) — Team membership selects context while individual ownership remains the private Agent boundary. Context deletion accepts cleanup intent, not completed native deletion.
- [Computer lifecycle admission and Provider keys](computer-lifecycle.md) — Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.
- [Computer owner and operator web](computer-web.md) — Server-rendered owner controls and redacted administrator inventory project backend authority rather than creating browser lifecycle state.
- [Computer schema and native identities](computer-storage.md) — Durable Computer state records authority, exact resource identity and cleanup obligations; schema records do not themselves perform external effects.

## References

- [`proto/chapchoo/computers.proto`](../../../../proto/chapchoo/computers.proto)
- [`rust/crates/rpc/src/computers.rs`](../../../../rust/crates/rpc/src/computers.rs)
- [`rust/crates/rpc/tests/computers.rs`](../../../../rust/crates/rpc/tests/computers.rs)
