---
title: "Standalone and Distributed Agent Computer Requirements"
created: 2026-11-02
tags: [agent, computer, product, deployment, architecture]
document_role: primary
document_type: requirements
snapshot_id: computer-261102
---

# Standalone and Distributed Agent Computer Requirements

- Snapshot: `computer-261102`
- Document reference: `computer-261102/REQ`
- Review revision: 4, future Mac compatibility without a current Mac delivery scope.
- Current planning baseline. This unimplemented snapshot carries forward the
  applicable outcomes of [computer-261101/REQ](computer-261101-agent-connection-management.md)
  and incorporates the subsequently confirmed standalone/distributed foundation.
- The matching [ADR](../adr/computer-261102-agent-computer-foundation.md) contains
  current technical decisions and the bounded remaining decision map.

## Problem and Primary Context

### Primary System Outcome

Design a common Agent/Computer system in which the Mac itself can be the Computer
for a future standalone Mac application, alongside managed cloud Computers in a
distributed installation. The local mode is not merely a client for remote cloud
Computers. Simplicity of deployment and shared product/domain behavior are primary
constraints. The current work preserves that future integration boundary; it
does not deliver the local Mac runtime or design its concrete enrollment and
authentication workflow. Mac UI and packaging are also outside this scope.

### Supporting Scenarios

- An owner creates a named personal Agent, observes its Computer, controls its
  power and eventually deletes it without inadvertently resetting saved data.
- An operator deploys an authorized Computer Provider and observes its managed
  Computers, including residual cleanup after Agent force deletion.
- An operator combines service groups in one instance or separates selected
  groups for independent deployment and scaling.

## Goals

- Preserve ownership, truthful lifecycle observation and saved-environment
  continuity through supported power operations.
- Support standalone operation without a separately deployed product backend.
- Treat the local Mac as a first-class Computer rather than requiring Kubernetes
  or a remote cloud Computer to supply the local execution environment.
- Support initial distributed deployment without Redis or a mandatory message
  broker, while keeping deployment-specific address discovery outside product logic.
- Separate persistent product intent from transient request delivery guarantees.

## Non-Goals

- Harness/model execution, Codex login, conversations, tools, file-management UI,
  memory, Team Agents, pricing, sizing/region selection, reset and batch recreation
  remain outside the original Computer-management milestone.
- No inactivity shutdown, live process/memory continuity, snapshots or backups.
- No web Provider enrollment/key entry or independent creation-enable switch.
- No manual inventory cleanup/retry/adoption controls or administrator-only
  Computer creation independently of an Agent.
- No mandatory permanent command history, message archive or event replay system.
- No initial guarantee of NAT traversal or communication between disconnected
  clusters. The directly reachable single-cluster scope is defined below.
- Local Mac implementation, concrete local enrollment/authentication workflow,
  UI technology selection, UI implementation, application packaging and
  distribution are outside the current scope. Preserve the common system
  boundaries and confirmed local cardinality for later Mac integration; do not
  make a working Mac runtime or its detailed workflow a current delivery gate.

## Requirements

### REQ-1. Infrastructure-Configured Provider Registration

- Deploying and authenticating the initial cloud system-owned Kubernetes Computer
  Provider makes it available without prior web registration.
- Reconnection/redeployment of the same logical Provider does not create another
  selectable Provider. Its name or self-asserted identity does not grant authority.
- Registration alone does not allocate an Agent or Computer.

### REQ-2. Read-Only Provider Observation

- Authorized operators can inspect Provider connectivity and creation capability.
- An authorized connected Provider capable of creation is offered without an
  additional web enablement switch.
- Management disconnection does not imply its Computers stopped or independently
  authenticated Computer Connector connections disappeared.
- This surface does not expose editing, key entry or new creation-policy controls.

### REQ-3. Personal Agent Creation and Ownership

- The owner starts from the Agent list, selects an available Provider and enters
  a nonempty name, then reaches that Agent's Computer detail after acceptance.
- Every cloud Agent has its own dedicated Computer, and every cloud Computer
  serves only that Agent. Cloud 1:1 dedication is an ongoing product rule,
  not an initial-milestone restriction.
- A local Mac Computer can be used by multiple Agents of the same owner.
  Each Agent is associated with one current Computer; adding another local
  Agent does not register another Computer for the same Mac.
- Only the owner can discover or control the private Agent through the user
  capability. Infrastructure observation does not grant private Agent access.
- Validation/Provider-unavailability errors preserve entered work. Preparation,
  failure, disconnection and deletion remain observable until Agent removal.
- Names are display data, not proof that two resources are identical.
- Provider selection and dedicated provisioning describe the managed cloud
  journey. They do not require a separate execution environment or Pod/PVC for
  every Agent using the same local Mac.

### REQ-4. Truthful State and Connection Observation

- Request acceptance, target convergence, execution/power, Provider management,
  Connector connection and browser observation failure remain distinct.
- A running Computer is not service-connected until its authenticated Connector
  connection is established. Neither peer disconnection alone proves powered off.
- Historical observations are distinguishable when current state is unverified.
- Failure or unknown outcome is not presented as success or unexplained endless
  preparation. Connector connection does not imply harness/conversation readiness.

### REQ-5. Manual Power Control

- The owner can start a confirmed stopped Computer and stop/restart a confirmed
  running Computer when management capability supports the action.
- Connector disconnection does not by itself prohibit Provider power control.
- Stop/restart disclose process interruption and require confirmation; cancelling
  the confirmation submits nothing.
- Unavailable management or an unknown required power state explains why an action
  is unavailable. Closing the browser does not request shutdown.
- Execution completion and Connector reconnection are separate observations.
- Capability handling must distinguish the local Mac from a provider-created
  cloud environment. Cloud Pod deletion is not implicit authority to shut down
  the host Mac or remove unrelated local files.

### REQ-6. Saved-Environment Continuity

- Agent identity, ownership and current Agent-to-Computer association survive
  stop/start, restart and authorized execution recovery.
- In the initial Kubernetes environment, preservation covers the Computer's PVC
  contents only. Container writable-layer/OS changes outside it are not preserved.
- Power control is not environment reset or deletion. Processes, memory and open
  connections need not survive, and Connector connectivity is re-established.

### REQ-7. Non-Duplicating Re-entry and Honest Uncertainty

- Repeated submission, refresh and revisit do not intentionally allocate duplicate
  Agents/Computers or resubmit an accepted power action as a new action.
- Replaying an accepted creation after its Agent was removed does not recreate it;
  a genuinely new creation requires a distinct explicit user request.
- The current product target and relevant lifecycle progress remain associated
  with the same Agent. Conflicting power actions are unavailable during pending work.
- Confirmed non-execution differs from an unknown result. Timeout is not proof
  that a side effect did not occur; uncertain effecting calls are not blindly repeated.
- Observation retry has no create/restart side effect. Preparation failure has a
  same-Agent recovery path; connection delay does not allocate another Computer.
- This does not require every message to survive a process crash or every unknown
  call to be resolved automatically. Minimum lifecycle recovery state remains a
  Design obligation, distinct from permanent message history.

### REQ-8. Normal Deletion

- For a dedicated cloud Computer, confirm its managed resource removal before
  removing the Agent.
- Show a single deleting state without a separate Connector data-cleanup stepper.
- Preparation/failure/disconnection does not remove the deletion path. Deletion
  blocks conflicting power actions and remains observable on revisit.
- Other Agents, shared infrastructure and account credentials are not targets.
- Removing one local Agent does not remove the shared Mac Computer or interrupt
  another Agent's association with it. Host shutdown or shared-resource deletion
  is not inferred from the cloud Agent-deletion path.
- Kubernetes completion covers managed execution resources and the dedicated PVC;
  operator-governed backing-storage reclamation is not an extra completion gate.

### REQ-9. Immediate Explicit Force Deletion

- Force deletion is immediately available during deletion progress with explicit
  confirmation that the Computer may remain. No timeout/failure threshold is needed.
- It removes the visible Agent without an external completion response and does
  not claim the Computer/data was cleaned up.
- The Kubernetes Provider continues best-effort cleanup without guaranteeing
  completion or requiring manual cleanup as a mandatory user step.
- Late reports never restore the Agent, cancel terminal intent or reassign the
  remaining Computer to another Agent.

### REQ-10. Secure, Accessible and Consistent Interaction

- Enforce current account/ownership permissions and isolate independent users.
- Do not expose credentials, private Agent contents or internal infrastructure
  identifiers to ordinary users through errors or public DTOs.
- Use distinguishable unavailable/failed/unknown states, keyboard-operable controls,
  cancellable confirmations, appropriate focus handling and narrow-screen layouts.
- Existing display language, time-zone, regional-format and appearance settings
  apply to General and Admin surfaces.

### REQ-11. Provider-Scoped Computer Inventory

- Authorized operators can inspect each Provider's managed Computers and track
  residual cleanup after the live Agent is removed.
- Preserve provider association, resource identity, terminal intent and useful
  observations without retaining private Agent content for operator access.
- Disconnection/query failure is not an empty inventory or cleanup success.
- Confirmed completed cleanup remains visible with minimal non-secret information
  for 24 hours, then completed tracking details are removed. Pending/unverified
  cleanup does not expire just because it is old.
- No permanent inventory archive or new cleanup/retry/adoption controls are required.

### REQ-12. Durable Current Intent and Consistent Interpretation

- Persist current running/stopped/deleting intent across Control or app restart.
- Keep intent, execution observations and both component connections independent.
- Loss of a connection or process-local history does not prove resource absence
  or authorize another execution or empty saved environment.
- Stale connections/results cannot overwrite current authority. General detail
  and Provider inventory use one consistent server interpretation of state/actions.
- Force deletion preserves residual terminal intent, not a second UI-owned queue.
- When a Computer Connector connects, its accepting Control owns that Connector's
  reconciler and continuously compares current desired state with observations.
- On connection loss, the previous Control no longer owns that connection's
  reconciliation. When the Connector reconnects to another Control, that Control
  creates its reconciler from current desired state and current observations.
- Continuity does not require transferring the old reconciler's process memory or
  restoring a durable history of delivered commands. Message loss/timeout follows
  REQ-17/18 while desired-state convergence continues under the current owner.
- Initial cloud provisioning and Provider-supported stop/deletion remain possible
  without a healthy Connector, as required by REQ-1/5/8. Do not introduce a
  connection prerequisite that prevents creating or managing that environment.

### REQ-13. Conditional Automatic Execution Recovery

- Automatically restore execution of the same Computer only with current running
  intent, confirmed prior-execution absence, usable existing saved environment,
  established ownership and no conflicting lifecycle work.
- Unknown observation, potentially live prior execution or missing/unverified
  saved storage blocks unsafe replacement; never silently create an empty fallback.
- Stop/deletion intent and explicit restart progress take precedence. Recovery
  progress and unmet prerequisites remain observable; it is not backup recovery.

### REQ-14. Standalone and Initial Distributed Deployment

- A Mac application can run the required product backend locally without a
  separately deployed product server, with the Mac itself acting as a Computer.
  The architecture must support this future app. Its concrete local registration,
  authentication workflow, runtime implementation and UI/packaging are deferred;
  they are not deliverables or approval prerequisites for this system design.
- Initial distributed support covers one Kubernetes cluster in which the relevant
  server instances can directly reach one another.
- Deployment supplies reachable addresses. Product logic must not require
  Kubernetes Pod/Service APIs as its application identity/address authority.
- Preserve SQLite and PostgreSQL support; do not impose Redis or an external
  broker on standalone or distributed operation.

### REQ-15. Composable Service Groups and Independent Client Targets

- Select Client, Admin and Control service groups at execution time, including
  all groups together or subsets for separate deployment/scaling.
- Split broad existing API surfaces by domain where needed. The group is a
  composition boundary, not a requirement for a dedicated port/process.
- Each client configures its required group targets independently. Equal addresses
  may be configured; missing/failed targets are not inferred from another group.
- Service grouping does not bypass per-operation account/component authorization.

### REQ-16. External and Internal Trust Boundaries

- Separate client-facing APIs from server-to-server relay APIs by listener.
  Client-facing includes Provider/Connector APIs and need not be Internet-public.
- Initial Kubernetes deployment does not publicly expose the internal listener
  and uses enforced network policy to admit only authorized server workloads.
- Trust that restricted internal server network without a separate application
  peer credential or mandatory mTLS deployment. Compromise of an allowed server
  is within that trust boundary and can reach internal APIs.
- Standalone does not run an unnecessary cross-Control internal listener.
- User/component authentication, current target/connection checks and business
  authorization remain mandatory. This does not select all TLS/encryption settings.

### REQ-17. Targeted Request/Response Delivery

- Reach the actual current owner of a Provider or Connector connection; do not
  assume these two component connections reside on the same server.
- Support a general request/response foundation rather than treating every
  command/query as a desired-state change or background job.
- Permit bounded owner re-resolution after an authoritative rejection before
  target forwarding, preserving request identity and the original deadline.
- After possible forwarding, timeout/disconnect yields uncertainty rather than
  automatic replay of an effecting command. No live owner yields unavailability,
  not an unbounded offline command reservation.

### REQ-18. No Mandatory Message History or Crash-Proof Command Transport

- Delivery may use memory/buffering but is not required to persist each message.
- Do not require a permanent command archive, message replay ledger or guaranteed
  eventual execution after every process failure.
- Persist product intent separately from transport messages. An expired call or
  lost response must not be presented as evidence of non-execution.
- Retention for explicit product records, such as creation deduplication and
  Computer cleanup visibility, is distinct from a universal command archive.

## Fixed Constraints

- No Redis, optional Redis mode, or increasing application generation/version
  numbers. This does not ban time limits or provider-native object metadata.
- Preserve the existing user/account authentication system. Provider/Connector
  authentication remains distinct from internal server trust.
- Session-wide Connector revocation withdraws issued-key authority and refresh
  eligibility. No compromise-detection or session-management UI is added.
- StorageClass reclamation belongs to the Kubernetes operator. Do not track or
  directly remove operator-retained backing assets after managed-claim removal.
- Kubernetes deployment/security/resource policy is administered by the cluster
  operator. Supply sound configurable Helm defaults consistent with confirmed
  product/trust boundaries; do not turn every operational setting into another
  product-design approval request.
- Existing schema preparation remains explicit; normal startup does not silently
  migrate. Standalone setup must be designed consistently with this boundary.
- Keep short database-only transactions; no Kubernetes/gRPC/filesystem operation
  occurs while an application database transaction is open.
- Reusable distribution artifacts contain no private deployment values or secrets.
- No implementation, infrastructure provisioning or deployment is authorized by
  this requirements consolidation.

## Remaining Design Work

Mac-as-Computer inclusion, connection-owned desired-state reconciliation and
operator-owned Kubernetes policy are confirmed, not open product questions.
Preserve the future local Computer boundary and confirmed sharing relationship
without designing a concrete Mac enrollment/authentication workflow now. Current
cloud lifecycle, common data and component credential mechanisms still require
design, without a universal command ledger.

The matching ADR tracks remaining directory/ownership details, relay contracts,
domain API composition and data/verification work. Helm defaults are agent-owned
implementation work within the confirmed constraints. New product behavior or
relaxation of a constraint still requires requester confirmation.

## Confirmation and Continuity

The requester confirmed the original Computer outcomes and subsequent foundation
choices during 2026-11-01 and 2026-11-02 KST, then requested their consolidation
into repository documents on 2026-11-02. This document records those confirmations;
it does not infer approval of additional product behavior or a complete Design.
The previous snapshot remains decision history, not the current planning checklist.
Future accepted changes must be recorded in repository artifacts in the same
working turn, not only in session continuity notes.

On 2026-11-02 KST the requester clarified that the Mac itself being a Computer is
the core local scenario, Mac UI/packaging is out of scope, the Connector's current
Control owns its reconciler and recreates it on reconnection from desired state,
and Kubernetes policy belongs to the operator with reasonable Helm defaults.
Revision 2 records these confirmations and closes the corresponding broad
questions rather than requesting them again.

The requester corrected the scope wording on 2026-11-02 KST: cloud Agent/Computer
dedication remains 1:1 beyond the initial milestone; multiple same-owner Agents
may use one local Mac Computer, and each Agent uses one current Computer.
Revision 3 records this established relationship rather than reopening it as
a new decision.

On 2026-11-02 KST the requester reiterated that local Mac implementation is not
current work. Revision 4 makes the existing scope boundary explicit: future Mac
compatibility remains required, while concrete local enrollment/authentication
and Mac runtime delivery are deferred. This does not remove standalone service
composition, SQLite support or the confirmed same-owner local sharing model.
