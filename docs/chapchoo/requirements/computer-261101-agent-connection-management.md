---
title: "Agent and Computer Connection Management Requirements"
created: 2026-11-01
updated: 2026-11-01
tags: [agent, computer, product, infrastructure]
document_role: primary
document_type: requirements
snapshot_id: computer-261101
---

# Agent and Computer Connection Management Requirements

## Current Planning Baseline

This unimplemented snapshot is retained as the earlier requirements record.
Use [computer-261102/REQ](computer-261102-agent-computer-foundation.md) and its
matching ADR for current planning. They consolidate the confirmed standalone,
service-group, direct-delivery and internal-network trust foundation and identify
the remaining open decisions. Older open assumptions here are historical, not
the current checklist.

- Snapshot: `computer-261101`
- Document reference: `computer-261101/REQ`
- Review revision: 11 (post-removal retry, completed visibility and Redis exclusion)

## Problem

An account holder cannot yet create a personal Agent with a service-provided
computer, distinguish its actual service connection from its execution state,
or control that computer's lifecycle. Account and Team functionality alone
does not provide these outcomes.

## Primary Context

### Primary Actor

An authenticated individual using a personal Agent.

### Primary Scenario

The user opens their Agent list, selects an available service-provided cloud
and an Agent name, creates the Agent and its dedicated computer, and observes
the computer's actual connection. The user can stop, start or restart the
computer without resetting its saved environment, revisit the same Agent,
and eventually delete the Agent and computer.

## Supporting Scenarios or Effects

- An installation infrastructure administrator configures an authorized
  Kubernetes computer provider through infrastructure deployment settings.
- An authorized installation operator observes registered providers and their
  management connection, ability to create computers and managed Computer
  inventory, including cleanup after an Agent has been force-deleted.
- A user distinguishes unavailable resources, preparation failure, observation
  failure and an operation whose result is not yet known.

## Goals

- Deliver personal Agent and dedicated cloud-computer connection management.
- Make connection, power and management observations truthful and independent.
- Provide manual power control without resetting saved files and settings.
- Provide normal deletion and an explicit immediate force-delete escape path.
- Preserve existing account, authorization and deployment behavior outside
  this capability.

## Non-Goals

- Codex authentication, harness execution, model responses, conversations,
  channels, persona or template selection.
- Tool integration, file-management UI, memory features, process snapshots,
  backups or continuous restoration of in-memory execution.
- User-owned computer attachment, Team Agents or Team-provided clouds.
- Pricing, billing, computer-size or region selection.
- Inactivity-based automatic shutdown, forced power-off or cancellation of
  an already accepted power operation.
- Web provider registration, provider-key entry, deployment-setting editing,
  or an independent web switch allowing new computer creation.
- Creating an administrator-only computer independently of an Agent. Tracking
  a previously assigned Computer left after force deletion is in scope.

## Terminology

- **Agent:** the user's personal managed entity, not a promise that model
  execution or conversation is ready.
- **Computer:** the environment in which CC runs. This describes an execution
  environment, not necessarily a physical host. In this milestone, each
  provisioned cloud computer is dedicated to one Agent; that scope restriction
  is not the general definition of Computer.
- **Computer Connector (CC):** the component running in a Computer
  that connects that Computer to the Chapchoo server. The broader product role
  also provides/manages harnesses in that environment. The connection role
  and harness-provider purpose describe the same component, not two newly
  introduced components. This milestone implements the connection boundary;
  harness execution and the user-owned-machine flow remain out of scope.
- **Computer Provider (CP):** provides Computers and manages their lifecycle.
  This is the requester-confirmed name replacing the earlier HO shorthand.
  Orchestration describes how the role is fulfilled.

Computer Provider and Computer Connector are confirmed component names;
package, executable and protocol names are not selected. Naming preserves these
responsibility boundaries without selecting a protocol, process layout or
trust mechanism. Ordinary user-facing surfaces use computer and cloud-computer
terminology.

## Requirements

### REQ-1. Infrastructure-configured provider registration

An authorized Kubernetes provider registers and connects through
infrastructure configuration and server-to-server authentication without a
prior web registration action.

**Acceptance criteria**

- Configuring and deploying the provider through Helm or equivalent
  infrastructure settings can make it visible as an authorized provider.
- Reconnection or redeployment of the same logical provider does not create
  duplicate selectable providers.
- An unauthenticated or unauthorized component is not accepted as an
  available provider by its own claim.
- Provider registration does not itself create an Agent or its computer.

### REQ-2. Read-only provider observation and availability

The installation operator can observe registered providers. An authorized,
connected provider that can create computers is offered to users without an
additional web enablement switch.

**Acceptance criteria**

- Provider management connection and computer-creation capability are
  distinguishable; connected does not necessarily mean creation is possible.
- A registered provider remains observable when its management connection
  is unavailable.
- No usable provider and failure to retrieve the provider list are distinct.
- The web provider-management surface is read-only; configuration and trust
  setup remain infrastructure responsibilities.
- Provider unavailability does not imply that existing computers or valid CC
  connections have stopped.

### REQ-3. Personal Agent creation and ownership

The user creates a named personal Agent and one dedicated computer from an
available service-provided cloud.

**Acceptance criteria**

- The journey starts at the user's Agent list, collects a provider and name,
  and leads to that Agent's computer detail after request acceptance.
- Each Agent has one dedicated computer; another Agent does not share it.
- Only the owner can discover, inspect or control their personal Agent through
  the user-facing Agent capability. The provider inventory in REQ-11 exposes
  authorized infrastructure observations without granting access to private
  Agent content or user lifecycle controls.
- An invalid name or unavailable selected provider does not discard entered
  work; an empty or whitespace-only name cannot be submitted.
- Preparation, disconnected and deletion-in-progress items remain observable
  until normal completion or explicit force deletion removes the Agent.
- An Agent name is display data, not evidence that two resources are the same.

### REQ-4. Actual connection and truthful state observation

The product distinguishes computer execution/power, CC service connection,
CP management connection and browser observation failure.

**Acceptance criteria**

- A running computer is not reported as service-connected until its
  authenticated CC connection is established.
- Loss of CC connection alone does not establish that the computer is off.
- Loss of CP connection does not erase an independently valid CC connection.
- Computer execution with no CC connection is distinguishable from a
  connected computer and from a confirmed powered-off computer.
- Preparation failure, unavailable observation and lost connection are not
  collapsed into a success state or indefinite unexplained preparation.
- Retained observations are identified as historical rather than current
  when the current state cannot be verified.
- CC connection is not represented as Codex or conversation readiness.

### REQ-5. Manual start, stop and restart

The owner can start, stop and restart their computer when its state and
management capability support the requested action.

**Acceptance criteria**

- A confirmed stopped computer can be started; a confirmed running computer
  can be stopped or restarted.
- A valid management path can support stop/restart even when CC is not
  connected. CC disconnection is not itself a prohibition on management.
- Stop/restart discloses process interruption and requires user
  confirmation; cancelling that confirmation submits no power operation.
- Request acceptance, power-operation progress, confirmed execution outcome
  and CC connection are distinguishable.
- CC disconnection alone does not prove successful stop; successful start
  or restart execution alone does not prove CC reconnection.
- Unavailable management or unknown required power state prevents an
  unsupported action and explains the unavailable capability.
- Closing a browser page does not request shutdown. No inactivity shutdown
  is introduced by this capability.

### REQ-6. Saved-environment continuity across power control

Stopping, starting and restarting preserve the Agent and saved environment
without promising continuity of live execution.

**Acceptance criteria**

- The same Agent name, ownership and dedicated-computer relationship remain.
- Saved files and settings remain after stop/start and restart.
- In the initial Kubernetes environment, the saved environment consists of
  content stored on the Computer's PVC. Files and settings outside that
  persistent storage are not preserved when execution resources are replaced.
  The same coverage applies to automatic execution recovery.
- OS/system changes in the container's writable layer are outside the
  preservation guarantee. No full root-filesystem snapshot is required.
- Stop is not deletion; restart is not an environment reset.
- Running processes, memory and open connections are not required to survive
  the power transition. CC connection is established and observed again.
- This requirement does not add backup, memory-snapshot or file-management UI.

### REQ-7. Safe operation progress, uncertainty and re-entry

The user can revisit accepted work and recover from observation failures
without unintentionally duplicating external effects.

**Acceptance criteria**

- Repeated submission, refresh and revisit do not create duplicate Agents,
  duplicate dedicated computers or repeated accepted power commands.
- Replaying an accepted Agent-creation request after that Agent was removed
  does not create another Agent. A new creation requires a distinct explicit
  user request.
- Accepted progress remains associated with the same Agent across re-entry.
- Conflicting power controls are unavailable while an operation is pending.
- A confirmed not-executed failure is distinguishable from an unknown result.
- An uncertain mutation is not automatically replayed as a new command.
  Recovery first determines the original operation's result.
- Observation retry does not create or restart a computer.
- Confirmed preparation failure has a recovery path associated with the
  same Agent; connection delay alone does not request another computer.

### REQ-8. Normal deletion

The owner can delete an Agent and its CP-created dedicated computer.

**Acceptance criteria**

- The normal path confirms dedicated-computer deletion before removing the
  Agent. No separate CC data-cleanup prerequisite is imposed on this
  CP-created environment.
- User-visible progress is a single deleting state, not internal data-cleanup
  or computer-deletion substeps.
- Preparation, failure or disconnection does not remove the deletion path.
- Deletion progress prevents conflicting power actions and remains
  observable across refresh/revisit.
- Shared provider infrastructure, other Agents and account credentials are
  not deletion targets.
- For the initial Kubernetes provider, completion concerns removal of its
  managed Computer resources, including the Computer's storage claim.
  Underlying storage reclamation follows the Kubernetes operator's policy
  and is not an additional Chapchoo deletion-completion condition.

### REQ-9. Immediate explicit force deletion

The user can finish Agent deletion without waiting for external cleanup by
explicitly selecting and confirming force deletion.

**Acceptance criteria**

- Force delete is available immediately during deletion progress, without a
  timeout, failure threshold or automatic escalation.
- Confirmation explains that the computer may remain after the Agent is
  removed. Cancelling confirmation does not force delete.
- Confirmed force deletion finishes user-visible Agent deletion without
  requiring an external completion response.
- Agent removal is not reported as confirmed computer or data cleanup.
- External cleanup behavior and guarantees belong to the individual CP.
  The initial Kubernetes CP continues best-effort background deletion of
  the remaining computer, without guaranteeing completion.
- Manual external cleanup is not a mandatory user step.
- Late CC connection, power completion or deletion completion does not
  resurrect the removed Agent or attach to another Agent.
- A remaining Computer continues to be identifiable and its cleanup observable
  through its provider inventory after user-visible Agent deletion.

### REQ-10. Secure, understandable and accessible interaction

The capability preserves account authorization and presents actionable
feedback without exposing secrets or confusing independent failures.

**Acceptance criteria**

- Two independent users cannot access each other's personal Agent
  information, computers or lifecycle operations.
- Errors do not expose keys, another user's information or internal cluster
  resource identifiers to ordinary users.
- Unavailability, preparation failure, observation failure and unknown
  mutation results are distinguishable.
- Primary actions work with a keyboard; confirmation supports cancellation
  and focus return.
- State is not communicated by color alone, and narrow layouts do not clip
  status, confirmation content or essential actions.
- Existing supported display language, time-zone, regional-format and
  appearance behavior remains applicable to the new surfaces.

### REQ-11. Per-Provider Managed Computer Inventory

An authorized provider observer can open a Computer Provider's managed Computer
list and track remaining Computer cleanup after an Agent is force-deleted.

**Acceptance criteria**

- Each provider has a page listing the Computers managed through that provider.
  In the current system-owned scope, this is an installation-operator surface.
- Force-deleting an Agent does not hide its remaining Computer or lose the
  association with the responsible Computer Provider.
- The page distinguishes an existing Agent association from an Agent already
  removed, without recreating that Agent or exposing its private content.
- Computer target, observed resource/connection state and cleanup progress
  remain distinguishable. Agent removal is not evidence of Computer removal.
- Provider disconnection or inventory-query failure is not reported as an
  empty provider or successful cleanup. Historical observations are marked
  as such until current observations are available.
- Confirmed removal of provider-managed Computer resources can be distinguished
  from cleanup still pending or not currently observable. Storage-backend
  reclamation governed by operator policy is outside this inventory's scope.
- The page is observational within the existing read-only provider-management
  boundary. This requirement does not add manual force-cleanup, retry buttons,
  resource adoption, provider configuration editing or incident-response tools.
- The inventory is scoped to authorized managed Computers, not an unrestricted
  listing of all cluster resources. It does not grant private Agent access.
- Confirmed completed cleanup remains visible with minimal non-secret completion
  information for 24 hours, after which completed tracking details are removed.
  Pending or uncertain cleanup does not expire merely because it is old.
  A permanent inventory audit/history archive is not required.

### REQ-12. Current Target and Observation Convergence

Computer lifecycle management keeps one current target and current observations,
and converges authorized resources toward that target rather than treating
connection events or a permanent command history as the Computer's state.

**Acceptance criteria**

- Requested running/stopped intent, observed resource state, provider
  connectivity and connector connectivity remain separate facts.
- Refresh/revisit and process restart preserve the current intent and do not
  turn the same request into a fresh destructive operation.
- Connection loss alone does not establish resource absence or authorize
  replacement of an otherwise existing Computer.
- Current observations and request/result correlation support recovery;
  replaced connections or stale results cannot overwrite current authority.
- Public lifecycle presentation and action availability use a consistent
  server interpretation across Agent and provider-inventory surfaces.
- Deletion has a distinct terminal intent. Force deletion does not discard the
  remaining Computer cleanup target or allow a late running report to revive it.
- Normal convergence preserves saved files/settings. It never silently
  escalates to data reset or destructive removal as a recovery fallback.

### REQ-13. Conditional Automatic Execution Recovery

When a Computer is still intended to run, unexpected loss of its execution
resources is automatically recovered for the same Computer only when continuity
and safe replacement can be established.

**Acceptance criteria**

- Automatic recovery requires a current running target, confirmed absence of
  prior execution, a usable existing saved environment and no conflicting
  lifecycle work. It does not require a new user recovery request in that case.
- The same Agent, Computer assignment, saved files and settings are preserved.
  Recovery does not allocate another logical Computer or reset its environment.
- Provider or connector disconnection, observation failure, or uncertainty
  about potentially live prior execution does not authorize replacement.
- Missing or unverified saved storage or uncertain resource ownership blocks
  automatic recovery rather than creating an empty replacement environment.
- Stop/deletion intent prevents recovery toward running; an active explicit
  restart follows its own accepted progress rather than launching a competing
  recovery.
- Recovery progress and an inability to verify its prerequisites are observable.
  Confirmed execution recovery and Computer Connector reconnection remain
  separate facts.
- This is execution recovery, not a backup, data-loss recovery or live
  process/memory continuity guarantee.

## Fixed Constraints

- This capability does not use Redis. Connection coordination, lifecycle
  handling and deployment must work without a Redis service or optional
  Redis-backed mode.
- Kubernetes storage policy, including StorageClass reclaim behavior, belongs
  to the Kubernetes operator. Chapchoo does not override that policy, track
  retained backing storage after its managed claim is removed, or require
  its erasure to complete Computer deletion.
- Connector authorization is revocable at the authentication-session boundary.
  Invalidating a session withdraws its connector authority across issued keys
  and prevents refresh from restoring that authority. This defines the
  authorization boundary, not a requirement to build credential-leak
  detection, incident-response automation or a session-management UI now.
- Start with a system-owned, server-level Kubernetes CP and personal Agents
  only. CP ownership has user-owned and system-owned forms. The corresponding
  owner authentication and credential rotation must inform the design;
  recognizing user ownership does not add its product flow to this milestone.
- Public deployment artifacts remain reusable and environment-independent;
  private installation values and credentials are not product defaults.
- Existing account and current authorization behavior remains intact.
- SQLite and PostgreSQL remain supported persistence modes; this snapshot
  does not authorize dropping an existing mode.
- Schema preparation remains an explicit operator responsibility rather
  than an implicit side effect of normal application startup.
- Existing display settings apply; translated examples are not exact
  protocol or English-copy contracts.
- Product scope approval is not authorization to provision infrastructure,
  deploy changes or implement unapproved technical mechanisms.

## Open Assumptions

- The selected deployment can supply a computer lifecycle that preserves
  saved files and settings across power transitions. Technical design must
  validate this outcome without weakening REQ-6 implicitly.
- Existing provider-independent terminology does not require implementing
  another CP in this snapshot.
- Observation timing and operational measurement thresholds require an
  explicit technical design and verification plan; no numerical SLA is
  established by these requirements.
- Confirmed component names do not authorize additional product behavior or
  implementation mechanisms.
- Confirmed interaction outcomes have not been validated by user research
  or live Kubernetes tests.

## Confirmation

The requester confirmed review revision 1 on 2026-11-01 KST, with the explicit
clarification that the then-current HO/HP names must be decided first. Review revision 2 records
that naming exception without changing the confirmed user-visible outcomes.
Review revision 3 records the requester's further clarification on the same
date: Computer is the environment where CC runs, and CC connects that Computer
to the server. The cloud Agent dedication constraint remains milestone-scoped.
Review revision 4 records the requester's confirmation of Computer Provider
(CP) as the name replacing HO. At that point the computer-side name remained open:
the requester noted that Harness Provider is inconsistent with the clarified
Computer-centered relationship. Its responsibilities and milestone scope
remained unchanged while alternatives were evaluated.
Review revision 5 records the requester's acceptance of Computer Connector
(CC), replacing HP, on 2026-11-01 KST. Computer Provider / Computer Connector
is the confirmed component pair.

On 2026-11-01 KST the requester additionally clarified session-wide connector
authority invalidation and explicitly excluded building credential-leak
response features at this stage. User-created connector sessions describe
the general authentication model; the existing system-owned Kubernetes
delivery scope is unchanged.

On 2026-11-01 KST the requester accepted the current-target/observation
convergence direction after the existing lifecycle research, and explicitly
requested a per-provider managed Computer page to track deletion after Agent
force deletion. Review revision 7 records REQ-11/REQ-12 and reconciles the
infrastructure-observation boundary with private Agent ownership.

On 2026-11-01 KST the requester approved conditional automatic restoration
after unexpected execution-resource loss. Review revision 8 adds REQ-13:
current running intent, confirmed absence of prior execution, preserved saved
environment and no conflicting lifecycle work are required. Disconnection,
uncertainty or missing storage does not authorize a fresh empty environment.

On 2026-11-01 KST the requester confirmed individual storage claims per Computer
and clarified that StorageClass policy belongs to the Kubernetes operator.
Review revision 9 defines the managed-resource deletion boundary and excludes
operator-governed backing-storage reclamation from Chapchoo completion and
inventory tracking. The storage allocation mechanism is recorded in D17.

On 2026-11-01 KST the requester confirmed that only PVC contents are preserved.
Review revision 10 clarifies REQ-6 accordingly. This does not select concrete
mount paths or require preservation of changes outside persistent storage.

On 2026-11-01 KST the requester confirmed post-removal creation deduplication,
independent Computer tracking and 24-hour completed-cleanup visibility.
The requester subsequently excluded Redis entirely. Review revision 11 records
these outcomes and the deployment constraint; concrete mechanisms remain in
the ADR.

This confirmation permits technical design against these requirements.
It does not approve any material technical mechanism, implementation,
infrastructure provisioning or deployment.
