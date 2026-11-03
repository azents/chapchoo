---
title: "Docker Server Computers Decisions"
created: 2026-11-03
tags: [computer, docker, provider, architecture]
document_role: primary
document_type: adr
snapshot_id: docker-261103
---

# Docker Server Computers Decisions

- Snapshot: `docker-261103`
- Mode: Collaborative; the requester owns material decisions.
- Requirements: [docker-261103/REQ](../requirements/docker-261103-server-computers.md), confirmed on 2026-11-03.
- The following topic agreements were separately made during Requirements
  refinement and confirmed as a whole before this ADR was created. They do not
  authorize implementation or imply complete Design approval.

## Decision Map

- D1 accepted: CLI-selected connection by default, explicit connection override,
  startup selection pinned and no alternate-host fallback.
- D2 accepted: existing outbound Connector path, optional container-reachable
  Control address override, operator-prepared networking.
- D3 accepted: shared Docker host with exclusive Provider-reserved resource
  management and an explicit native-volume conditional-deletion limitation.
- Fixed outcomes: SYSTEM/server ownership; private User/Team admission; current
  request/connection/execution fencing and uncertain-effect handling; original
  saved-environment preservation; confirmed cleanup and retained force-removal
  residuals. These are not new voting items.
- Local implementation details: crate/helper/file names and equivalent internal
  adapter structure. These do not create new state, authority or runtime modes.
- Feasibility review must identify any additional material mechanism not derived
  from the accepted sources before the primary Design can be approved.

## D1. Connection selection follows Docker usage with an explicit override

- Reference: `docker-261103/ADR-D1`
- Accepted: 2026-11-03
- Authority: REQ-2; requester agreement and subsequent complete Requirements confirmation.

### Question

Should a Provider use independent explicit daemon settings, or reuse the Docker
CLI connection selected for the executing account?

### Considered alternatives

1. Always require explicit endpoint/authentication configuration. This is clear
   for services but duplicates the normal developer Docker selection.
2. Only use CLI-selected configuration. This is convenient for local testing but
   makes an explicit service/test target unnecessarily indirect.
3. Use CLI selection by default and allow coherent explicit configuration to
   override it. This combines the requested developer workflow with deliberate
   service configuration.

### Decision

Use alternative 3. Resolve the executing account's normal Docker CLI selection
rules, including standard environment/context precedence, unless Provider
connection settings explicitly override that source. Treat explicit endpoint and
its authentication as one source; do not silently fill holes from another
context. Resolve once at startup. A later context change does not retarget the
process. Failure of the selected connection is an error, not permission to try
another context or the default socket.

### Consequences

CLI configuration resolution must be implemented and tested deliberately; an SDK
constructor that ignores contexts does not satisfy the requirement. Service
accounts need access to the intended Docker configuration or an explicit
override. Remote reachability does not imply Computer traffic reachability.
Original resources cannot move to a different daemon merely because a selection
name or endpoint string is reused. Supported transports/platforms and daemon
identity evidence remain explicit feasibility obligations in Design.

## D2. The operator supplies the Computer-to-Control network path

- Reference: `docker-261103/ADR-D2`
- Accepted: 2026-11-03
- Authority: REQ-5; requester agreement and complete Requirements confirmation.

### Question

Should the Provider create networking/tunnels to bridge container traffic to
Control, or use an operator-provided reachable path?

### Considered alternatives

1. Provider-owned tunnels/relays and automatic connectivity changes. This may
   simplify one-machine setup but creates additional process lifetime, security
   and recovery authority and is not needed for the bounded server Provider.
2. Operator-prepared connectivity with an explicit runtime address override.
   This matches the existing outbound Connector and works with a remote daemon
   without conflating its host with the Provider machine.

### Decision

Use alternative 2. Preserve outbound Connector connections. Default to the
Provider's configured Control address and allow a separate container-reachable
Control address. The operator supplies the appropriate Docker network,
listener reachability and firewall policy. Do not automatically create a tunnel,
expose container ports publicly or fall back to host networking.

### Consequences

Provider-to-daemon and container-to-Control are separate validation gates. A
loopback address is not automatically rewritten into a portable host address.
Docker Desktop-specific host names and Linux host-gateway mappings require
explicit compatible operator setup, not a universal fallback. Existing Control
transport/authentication policy remains unchanged; this decision does not
authorize public exposure of a trusted-network listener.

## D3. Provider-reserved resources have exclusive management on a shared host

- Reference: `docker-261103/ADR-D3`
- Accepted: 2026-11-03
- Authority: REQ-3 and REQ-4; requester agreement and complete Requirements confirmation.

### Question

Which native volume-deletion safety boundary is acceptable when Docker local
volumes lack a Kubernetes-style immutable-UID conditional delete?

### Considered alternatives

1. Allow unrelated workloads on the daemon but reserve Provider-managed objects
   to exclusive Provider management, explicitly accepting the weaker native
   conditional-delete guarantee.
2. Require stronger storage/removal authority than ordinary Docker local volumes,
   or leave cleanup unresolved rather than accepting this operational boundary.
   This would change the storage/cleanup design and increase local-test setup.

### Decision

Use alternative 1. Unrelated containers/volumes may share the host, but external
operators/tools do not create, delete or reuse Provider-reserved objects/names.
Computer workloads may write saved files; that is not native object management.
Verify reserved ownership and original environment evidence, reject conflicts,
and clean up exact individual resources. Never use daemon-wide pruning.
Docker host administration is trusted operator authority, not a tenant boundary
provided by labels.

### Consequences

Container full IDs support replacement-safe container targeting. Local volume
removal remains name-based; inspect-then-delete does not become an atomic UID
precondition. The approved boundary excludes concurrent external name recreation,
not every Docker failure. Missing allocated environments are never silently
replaced. Conflict, disconnection or insufficient evidence remains unknown or
blocked rather than successful cleanup. The Design must preserve application
fencing and disclose that metadata is not proof against a hostile daemon admin.

## Further Decision Discovery During Design

The first full draft and independent source/authority review identified two
additional material lanes. D1-D3 remain accepted and are not reopened.

- **D4 pending, not accepted:** native recovery after an unexpected process exit.
  Current `rust/crates/kubernetes/src/lib.rs` constructs `restartPolicy=Always`
  and checks it on collision recovery. Docker restart disabled with Control
  retirement/new execution is a different failure/recovery policy, not an
  automatic consequence of parity. The proposed choice will be reviewed with
  the requester before final native terminal predicates are accepted.
- **D5 pending, not accepted:** initial supported connection/platform/security
  envelope, including verified TLS/SSH prerequisites and stable single-daemon
  endpoint trust. A CLI-default selection rule alone does not promise every
  native transport, insecure context setting or volume driver. Released SDK
  evidence and the exact public support boundary must agree.

Implementation details that introduce no new authority (column/helper names,
behavior-preserving task extraction, concrete typed schema and migration
preservation) stay with the design author. No new daemon-rebind/reset authority,
private-registry credential service or privileged volume helper is presumed.

## D4. Native restart within the current execution after unexpected exit

- Reference: `docker-261103/ADR-D4`
- Accepted: 2026-11-03, after the pending discovery above.
- Authority: REQ-3 and requester choice of same-container native restart.

### Question and alternatives

After unexpected process exit, restart the process in the same native container
or disable native restart and retire/create a fresh execution through Control?
Current Kubernetes code uses and verifies Pod `restartPolicy=Always`; native
same-container restart is the closer parity choice. Control-only replacement
would change recovery dependence, native identity and Connector session lifetime.

### Decision

Use Docker restart policy `always` for unexpected process failure within a
current execution. Retain the same container/execution and its Connector session;
ordinary reconnect still checks current session and credential authority. Native
restart does not bypass retirement or revive a retired execution. Keep automatic
container removal disabled so lifecycle evidence remains observable.

Owner-requested Stop, Restart and Delete retain their existing lifecycle meaning:
retire/remove the exact old container, with original saved storage preserved for
an authorized successor. A temporary exited/restarting state under `always` is
not terminal quiescence and cannot authorize execution replacement or cleanup.
Before declaring retirement, establish the accepted stop/removal boundary and
rule out pending Start or native resurrection. Daemon restart/live-restore and
ambiguous stop results must be covered without treating API disconnection as
stopped evidence. Docker `always` behavior is not claimed identical to K8s
across every daemon-outage boundary; native acceptance is a feasibility gate.

### Consequences

The earlier pending `restart=no` draft is superseded, not accepted authority.
Update native inspect predicates and shutdown/removal sequencing accordingly.
If exact no-resurrection retirement cannot be proven, retain uncertainty rather
than create a successor or erase original volume evidence. No new auto-recovery
ledger or reset API is authorized by this decision.

## D5. Linux and native macOS Provider workflows

- Reference: `docker-261103/ADR-D5`
- Platform scope accepted: 2026-11-03.
- Authority: amended REQ-2 and REQ-6; explicit requester instruction to support
  Docker Desktop for Mac after discussion of native Provider execution.

### Question and decision

Limit the first Provider to current Linux executables, or include native macOS
execution against Docker Desktop for Mac? Include native macOS alongside Linux.
Connecting from Linux to a remote Mac is not an equivalent delivery of the
requested local developer workflow. Computer execution remains Linux containers
inside the selected Engine/Desktop daemon. Do not promise macOS guest Computers
or a macOS-native backend/web installation as incidental Provider support.

### Consequences and unresolved remainder

Plan native macOS executable build/startup/shutdown, CLI-context and Unix-socket
selection, platform/architecture-compatible image selection, Docker Desktop
restart behavior, named-volume persistence and container-to-Control acceptance.
No Linux-only simulation or cross-compile proves the real-Mac lifecycle.
Supported Mac architecture/minimum OS must be stated from verified build and
Docker Desktop prerequisites rather than inferred from Docker's product name.

This accepts the platform workflow, not every candidate transport/security
setting discussed in the earlier D5 discovery. The Unix/verified-TLS/SSH support
and rejection policy remain proposed until specifically agreed. No Windows
native Provider, named-pipe transport or insecure TCP/TLS-bypass mode is
authorized by the Mac request.

### D5 clarification: connect to an already running Desktop

On 2026-11-03 the requester clarified the intended workflow: Docker Desktop for
Mac is already running, and the Provider connects naturally through the normal
Docker CLI context. Native Provider build/start/shutdown concerns the Provider
process only, not Desktop lifecycle management. The Provider never installs,
launches, restarts or configures Desktop/Engine, switches context, or tries to
repair an unavailable daemon by launching a host application. Desktop restart
acceptance means observing an operator-initiated restart, not a product action.
This clarifies D1/D5 and adds no Desktop supervisor or setup wizard.

## D6. Respect the user's Docker connection security configuration

- Reference: `docker-261103/ADR-D6`
- Accepted: 2026-11-03.
- Authority: amended REQ-2; requester instruction to respect user defaults.

### Question and alternatives

Apply a separate Provider allowlist requiring verified TLS or SSH for remote
connections, or preserve the Docker connection/security settings selected by the
user? A stricter allowlist would reject contexts that the user's Docker CLI
deliberately accepts. Preserving the selected policy provides the requested
ordinary Docker experience but inherits its trust risks.

### Decision

Honor the selected connection's endpoint, authentication source, TLS mode,
certificate-verification setting and SSH configuration. In default mode follow
the user's Docker CLI configuration; explicit Provider connection settings
override it as the coherent source under D1. A deliberately configured plaintext
TCP connection or TLS verification bypass is not rejected by a new Provider
security policy. Do not silently enable either, turn verification back on,
borrow another context's credentials or fall back to another target.

SSH follows the user's existing authentication and host-key policy rather than
inventing an independent stricter/weaker policy. No new Desktop/daemon listener,
host configuration change, tunnel or trust-on-first-use automation is introduced.
An unimplemented transport/configuration is a visible compatibility limitation,
not a reason to substitute a different connection or claim the chosen policy
was faithfully supported.

### Consequences

The operator owns the selected daemon endpoint's reachability and security
trust. When settings disable transport/server verification, daemon-ID checks and
labels do not provide authenticated protection against interception; document
this without changing settings or blocking them solely on security-policy
grounds. Do not print credentials or raw context contents in diagnostics.
Validate selected modes against the released SDK's supported public APIs and
test exact effective configuration. A technical compatibility blocker returns
as evidence, not an invented blanket secure-only product requirement.

This completes the connection-security remainder discovered with D5; it does
not add Windows Provider support or arbitrary volume-driver support.

### D5 implementation-scope correction

On 2026-11-03 the requester clarified that respecting Docker CLI context and
avoiding unnecessary host-OS coupling should let a later Mac build work
naturally. This does not authorize building, packaging or distributing a Mac
executable now. The earlier D5 native-build/real-Mac release-gate decomposition
is superseded for this effort. Keep the selected socket/context, already-running
Desktop prerequisite and no-setup/no-fallback behavior; remove Mac-only CI and
archive work. Future Mac distribution needs its own broader delivery design.
Missing Mac infrastructure is not a blocker for completing this Docker feature,
and Linux evidence is still not advertised as a verified Mac support result.

## Evidence

- Existing Kubernetes ownership/retirement contract: [computer-kubernetes](../spec/domain/computer-kubernetes.md).
- Current shared authority: [computer-storage](../spec/domain/computer-storage.md),
  [computer-directory](../spec/domain/computer-directory.md) and
  [computer-control-rpc](../spec/domain/computer-control-rpc.md).
- Docker [contexts](https://docs.docker.com/engine/manage-resources/contexts/),
  [daemon protection](https://docs.docker.com/engine/security/protect-access/),
  [volumes](https://docs.docker.com/engine/storage/volumes/) and
  [Engine API](https://docs.docker.com/reference/api/engine/).
