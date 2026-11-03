---
title: "Docker Server Computers Requirements"
created: 2026-11-03
updated: 2026-11-03
tags: [computer, docker, provider, lifecycle]
document_role: primary
document_type: requirements
snapshot_id: docker-261103
---

# Docker Server Computers Requirements

- Snapshot: `docker-261103`
- Document reference: `docker-261103/REQ`

## Problem

The implemented Computer Provider requires Kubernetes. Local product testing and
other operator-managed container-host environments need a Computer substrate
without a Kubernetes cluster while retaining the current Computer contract.

## Primary Context

### Primary System Outcome

An operator can run a server Computer Provider on a machine with access to a
Docker host. That Provider supplies Docker containers from its connected host
as Computers through the existing server-managed Agent/Computer experience.
The Provider machine and Docker host need not be the same machine.

## Supporting Scenarios or Effects

- Exercise the Computer lifecycle locally without Kubernetes prerequisites.
- Run the Provider natively on macOS against Docker Desktop for Mac's Linux
  containers, as well as the Linux Provider workflow.
- Retain the saved environment across execution replacement.
- Continue safe observation and cleanup after Provider reconnection or visible
  Agent removal, within the established authority and evidence boundaries.

## Goals

- Add a Docker-backed server Provider while preserving existing Kubernetes behavior.
- Match current Computer ownership, observable lifecycle and recovery semantics
  wherever the native substrate permits them.
- Make necessary Docker-specific security and operational differences explicit.

## Non-Goals

- User-owned/local-machine enrollment, attaching an existing user container, or
  changing the current server-managed Computer dedication model.
- Docker Swarm, Kubernetes replacement, automatic host discovery or scheduling
  across unrelated Docker hosts.
- Installing, launching, restarting or configuring Docker Desktop/Engine, or
  switching the user's Docker CLI context.
- Automatic migration of saved environments between Docker hosts or between
  Kubernetes and Docker.
- A new Agent harness, user-facing command shell, deployment or live host changes
  as part of this design task.

## Requirements

### REQ-1. Server Provider ownership

The Docker Provider is initially a server Provider operated under the existing
trusted installation-operator authority, not a user-enrolled Provider.
The execution process may run outside the system under a trusted third-party
operator. Server ownership does not require execution inside the backend
installation.

**Acceptance criteria**

- Authorized server Provider registration and current communication authority
  remain distinct from private Agent ownership and Team membership.
- Selecting the Provider does not make the Agent or its Computer visible to
  another Team member or installation administrator as private user content.
- Existing Kubernetes Providers and their saved environments remain usable.
- Both Kubernetes and Docker Providers operate without direct product-database
  access or installation encryption-root credentials. Product authorization and
  persistence remain server responsibilities.

### REQ-2. Connected-host selection and scope

By default, the Provider uses the Docker connection selected for its execution
account by the normal Docker CLI configuration rules. An explicit Provider
connection configuration overrides that default.

**Acceptance criteria**

- A Provider process can manage a host it can access without requiring that the
  Docker daemon run on the Provider machine.
- macOS native Provider execution with Docker Desktop for Mac is in scope.
  Remote access from a Linux-only Provider is not a substitute for this local
  developer workflow. Computers still use Linux containers, not macOS guest OS
  containers.
- Docker Desktop is already running and reachable as an operator prerequisite.
  The Provider naturally connects using the selected CLI context; it does not
  launch or manage Desktop. If the selected daemon is unavailable, report that
  condition without starting Desktop or selecting another target.
- Explicit connection configuration is authoritative as one coherent endpoint
  and authentication source; missing pieces are not silently borrowed from a
  different context.
- Honor the user's selected Docker connection and its authentication, TLS
  verification and SSH policy. Do not impose a separate Provider-only rule that
  rejects a deliberately configured unencrypted TCP connection or disabled
  certificate verification, and do not silently strengthen or weaken that
  configuration. Explicit Provider overrides remain authoritative when supplied.
- Connection selection is fixed for the process lifetime. Changing the current
  CLI context does not retarget a running Provider, and connection failure does
  not select a different context or a default socket.
- An unavailable or changed target is not treated as permission to allocate a
  replacement saved environment on another host.
- Supported connection and host-platform combinations are explicitly declared
  by the resulting Design and verified independently of CLI configuration.

### REQ-3. Existing lifecycle and saved-environment semantics

Create, start, stop, restart, normal deletion and force removal retain the
established Computer meanings rather than introducing a separate Docker UX.

**Acceptance criteria**

- A stopped/restarted execution does not promise process or memory persistence;
  the original saved environment remains available to an authorized successor.
- A previously allocated environment that is missing or unverified is not
  silently recreated as an empty environment.
- Exact repeated requests recover their original result; ambiguous effects are
  not automatically replayed as new mutations.
- Normal deletion confirms completion only after required native retirement
  and owned-resource cleanup evidence. Force removal does not assert cleanup
  completion and retains unresolved cleanup responsibility.

### REQ-4. Bounded resource and credential authority

Provisioning and cleanup remain scoped to reserved, owned resources and the
current authorized execution and Provider connection.

**Acceptance criteria**

- Foreign objects and conflicting native identities are not adopted as an
  existing Computer or knowingly removed as its resources.
- Unknown effect outcomes, disconnected daemons and insufficient retirement
  evidence remain distinguishable from confirmed stopped or deleted state.
- Docker-host management credentials and socket access are not supplied to the
  Computer workload or exposed in ordinary user/operator messages.
- Any native safety limitation relative to Kubernetes is disclosed and accepted
  before final Design approval; labels are not described as an authorization
  or atomic conditional-deletion boundary.
- The Docker host may contain unrelated containers and volumes. Provider-reserved
  objects have an exclusive management boundary: external tools do not create,
  delete or reuse their names. Computer workloads may write their saved files;
  that is not external resource management.
- Cleanup targets individually reserved resources, never host-wide pruning.
  The Docker host administrator is a trusted operator. Ownership metadata is
  not isolation from that administrator.
- Under this boundary, the lack of immutable-UID conditional deletion for local
  Docker volumes is an accepted limitation, not a Kubernetes-equivalent atomic
  guarantee against external same-name replacement.

### REQ-5. Independently reachable Computer connection

A Computer's management availability and its own connection to Control remain
independent observations.

**Acceptance criteria**

- Docker API reachability alone does not establish Computer Connector reachability.
- A container that cannot reach Control is not presented as a connected Computer.
- Operator prerequisites distinguish the Provider-to-host management path from
  the container-to-Control path, including remote-host cases.
- The existing outbound Connector path is retained. The Provider's Control
  address is the default, with an explicit container-reachable address override
  when the container network needs a different address.
- The operator prepares a suitable network, listener reachability and firewall
  policy. The Provider does not automatically add a tunnel, publicly expose a
  container port or switch to host networking as a fallback.

### REQ-6. Docker-backed validation without Kubernetes

The new Provider is verified against an explicitly selected real Docker host,
not only simulated responses or image-packaging commands.

**Acceptance criteria**

- A minimal end-to-end lifecycle verifies creation, connection, stop/start with
  preserved saved data and confirmed normal removal without a Kubernetes cluster.
- Lower-layer tests own ambiguous outcomes, foreign-resource conflicts and
  connection/execution fencing without duplicating the full E2E matrix.
- Missing Docker prerequisites are reported as unavailable validation, not a
  successful or silently skipped lifecycle test.
- macOS Provider build/startup, selected Docker Desktop context/socket,
  Connector reachability and original-volume lifecycle receive real-Mac
  validation. A Linux fixture or cross-compile alone does not establish this
  supported workflow.

## Fixed Constraints

- Providers may be operated externally by trusted third parties. Neither native
  Provider is a database client or requires backend database credentials.
- Preserve existing private owner/Team authority and server Provider admission.
- Preserve current request, connection and execution fencing and uncertain-effect
  handling; external IO does not move into short database transactions.
- Preserve Kubernetes data during any required forward migration.
- Initial scope is Docker container Computers, not user-owned LOCAL Computers.
- This is design work only; implementation requires a later request.

## Open Assumptions

- The initial supported Docker connection transports and host-platform
  combinations still need a concrete feasibility statement. The default
  CLI-selection and explicit-override operator workflow is requester agreed.
- The operator supplies a valid container-to-Control network path under REQ-5;
  concrete transport/network prerequisites still require feasibility validation.
- The accepted exclusive-management boundary for Provider-reserved resources
  must be reflected in operational prerequisites and failure handling. Native
  behavior and ownership/absence evidence still require feasibility validation.
- Extra Docker-specific capabilities are not implied by native daemon features.

## Confirmation

On 2026-11-03, after implementation authorization, the requester explicitly
clarified that both Kubernetes and Docker Providers are external processes that
must not access the product database, and may be run by trusted third parties
outside the system. This confirms the external process boundary, not open
self-enrollment or a new third-party management product.

The requester confirmed the complete Requirements on 2026-11-03 before formal
ADR and Design drafting. On the same date the requester agreed to the REQ-2 operator
workflow: CLI-selected connection by default, explicit configuration override,
target fixed at startup and no automatic fallback to another host. The requester
also agreed to the REQ-5 network workflow: existing outbound connection, optional
container Control address override and operator-prepared networking without
automatic tunnels or public-port exposure. The requester also
agreed to the REQ-4 resource-management boundary: unrelated workloads may share
the Docker host, Provider-reserved resource identities are managed exclusively
by the Provider, host-wide pruning is excluded, and local-volume conditional
deletion is explicitly weaker than Kubernetes UID-precondition deletion.
The complete Requirements confirmation covers these agreements and REQ-1
through REQ-6. Detailed Design approval and implementation authorization remain
separate; neither is implied by Requirements confirmation.

On 2026-11-03 the requester explicitly extended the supported workflow to Docker
Desktop for Mac, in response to the question about native macOS Provider
execution. This scope amendment includes local native Provider execution and
Mac-specific verification, not only a remotely accessible Docker host.

The requester subsequently confirmed that Desktop is already running and the
Provider simply follows CLI context, then selected respecting the user's Docker
defaults for connection security on 2026-11-03. This includes deliberately
configured TCP/TLS-verification/SSH policy. It does not authorize changing host
security configuration, exposing a new listener or falling back to a different
target when the selected connection fails.
