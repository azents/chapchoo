---
title: "Computer Client and Inventory RPC"
created: 2026-11-02
tags: [agent, computer, grpc, security, inventory]
spec_type: domain
domain: computer
code_paths:
  - proto/chapchoo/computers.proto
  - rust/crates/protocol/src/lib.rs
  - rust/crates/rpc/src/computers.rs
  - rust/crates/rpc/src/groups.rs
  - rust/crates/persistence/src/computers.rs
  - rust/crates/service/src/computers.rs
  - rust/crates/rpc/tests/computers.rs
  - typescript/packages/rpc-client/src/generated/computers.ts
last_verified_at: 2026-11-02
spec_version: 1
---

# Computer Client and Inventory RPC

The Client service union registers Agents, Computers and ProviderCatalog. Admin
registers read-only ProviderInventory. These use the existing complete storage
operations and current account session authority, not a caller-supplied user ID.
Every generated message and outcome oneof has field-free Debug output.

## Owner operations

- Agents.Create accepts a Request ID, Provider ID and name. The atomic creation
  receipt returns the original Agent on replay or its removed-creation outcome;
  replay after removal never creates a new Agent/Computer.
- Agents.List/Get and Computers.Get expose only the authenticated owner's private
  Agent and dedicated Computer projection. Installation administrator status does
  not grant access to another user's private Agent detail.
- Computers.RequestLifecycle accepts Agent ID, Request ID, expected current
  Request ID and a typed Start/Stop/Restart/Delete action. Missing, unspecified,
  unknown and Create action values are rejected. This accepts durable desired
  intent; it does not wait for external Kubernetes convergence.
- Agents.ForceRemove requires the exact current deletion Request ID. It removes
  the live Agent immediately while preserving the independent residual Computer,
  resources and terminal cleanup intent. Creation deduplication survives.
- ProviderCatalog.List returns authenticated creation choices, not credentials,
  ownership internals or management inventory.

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
