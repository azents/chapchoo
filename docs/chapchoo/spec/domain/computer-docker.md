---
title: "Computer Docker Native Adapter and Provider"
created: 2026-11-03
tags: [computer, docker, runtime, security]
spec_type: domain
domain: computer-docker
code_paths:
  - rust/crates/docker/src
  - rust/crates/docker/tests
  - rust/apps/chapchoo/src/provider.rs
  - rust/apps/chapchoo/src/provider/control.rs
  - rust/apps/chapchoo/src/provider/docker_worker.rs
  - rust/apps/chapchoo/src/provider/effects.rs
  - rust/apps/chapchoo/src/config.rs
  - rust/apps/chapchoo/src/main.rs
  - rust/apps/chapchoo/tests/provider_cli.rs
  - rust/apps/chapchoo/tests/provider_cli/docker.rs
last_verified_at: 2026-11-04
spec_version: 1
---

# Computer Docker Native Adapter and Provider

`chapchoo docker-provider` is a separate external foreground process. It loads
Provider owner credentials and selected native configuration, not backend DB
configuration or the installation root. SYSTEM is its scoped server ownership
category; a trusted third party may run the process outside the installation.
The server's immutable kind/daemon binding, current-source authority, typed
reservations and durable preparation/retirement fences remain authoritative.
This source-level runtime implementation is not proof of real Docker Engine,
native Mac/Desktop, SSH or remote security-mode acceptance.

## Selected connection and native IO

Connection selection resolves once from the execution account's Docker CLI
configuration/environment or a complete explicit Provider override. Explicit
host/TLS/API-version settings do not borrow another context's material. Metadata
and credential files are bounded; TLS bytes are captured at startup and their
Debug output is redacted. A selected file/endpoint failure does not select another
context, socket or host. Docker daemon IDs are opaque bounded evidence, not
Kubernetes namespaces or endpoint-hostname attestations.

The public Bollard 0.21.1 SDK owns Engine routes, serialization, native operations
and response decoding. A custom Hyper HTTP/1 byte transport supplies Unix/TCP/TLS
or OpenSSH streams, without request retries or redirects. TLS uses selected CA/
client credentials; a verification-disabled verifier is constructed only when
selected, not after a verification failure. SSH does not inject host-key,
known-host-file or identity-file policy; selected remote socket arguments are
quoted as literal remote CLI words. Local OpenSSH and remote Docker dial-stdio
are transport prerequisites, not alternative native lifecycle commands.
One process-lifetime authenticated SSH master freezes the selected target and
authentication source. Independent SDK channels reuse that private mux socket
without rereading account configuration. Missing master/socket fails closed,
never direct-reauthenticates or selects another target. Drain ends the master
after its connection drivers finish.

Effective Docker TCP `HTTP_PROXY`/`HTTPS_PROXY` routing is currently an explicit
compatibility error before direct connection. `NO_PROXY` host/port, domain,
IP/CIDR and wildcard exclusions plus Go's unconditional loopback/localhost
bypass are honored. Unix/SSH ignore HTTP proxy routing. A proxy-required target
is never silently reached directly; no proxy credential is logged.

At most 64 connection drivers are owned through response-body completion, not
merely response headers. Drain stops new transport admission and joins owned
connections. SDK request deadlines and explicit total-operation timeouts are
separate from Control's ten-second response waiter. Image pull consumes its
entire bounded stream and embedded errors. A native mutation timeout is uncertain.

Startup reads actual `/info`, accepts only the required Linux architecture and
reported seccomp/CPU/memory-limit capability, then registers the daemon identity
through scoped Control. Fresh native checks and Provider heartbeat checks compare
that identity and workload policy. API unavailability is not stopped power.
A stable trusted single-daemon endpoint remains an operator prerequisite; the
separate info/effect operations are not atomic target attestation.

## Original saved volume

Only a reserved local named volume with local scope and no driver options is
accepted. Exact Provider/Computer labels, reserved name, daemon and original
creation/label identity must match. Host-bind/NFS/device options or foreign labels
are not adoption authority. A bound missing/changed original is never recreated,
including during its initial execution. An already-confirmed absence cannot
adopt a surprisingly present object.

A fresh admitted initial preparation may submit one volume-create operation.
Present exact results can be observed and reported without repeating create.
Only explicit rejection at the create API boundary qualifies as definite native
non-acceptance; inspect, transport, timeout and server ambiguity do not. The
Provider waits for Control's allocation report ACK before later stages.

Removal is by the individually reserved name with force disabled, after native
ownership checks and the server's exact never-submitted cleanup-suffix admission.
Docker provides no volume UID-conditional delete; exclusive management of these
reserved objects is part of the trusted shared-daemon boundary. One remove is
followed by bounded read-only absence observation, never prune or repeated delete.

## Container profile and staged effects

The execution is a native container, not a Pod/Secret identity. Its accepted full
64-character ID, exact daemon/Provider/Computer/execution ownership, supported
image/command, original saved-volume mount and immutable configuration anchor
observations and effects. Collision recovery during Prepare compares the original
Connector session/key/endpoint, not newly allocated bootstrap credentials.

The profile uses UID:GID 10001:10001, read-only root, dropped capabilities,
no-new-privileges, constrained daemon seccomp and bounded CPU/memory settings.
It has private PID/IPC, no privileged mode, host binds, devices, Docker socket,
extra anonymous volumes or published ports. Image `EXPOSE` declarations are
advisory metadata; the Provider forbids `PublishAllPorts` and every nonempty
`PortBindings` configuration independently. Only the saved named volume is
mounted. Initial allocation permits image-directory population; successors use
no-copy. More narrowly, only an opaque proof from a positively completed fresh
volume creation permits that initial population. Recovered/existing originals,
even under an initial-allocation retry, always use no-copy. Image preparation
captures the actual SHA-256 identity and verifies Linux architecture; container
creation uses that immutable digest, and collision checks compare actual image
identity rather than a mutable tag. The image/mount must already provide writable non-root storage; the
Provider supplies no privileged helper or recursive ownership repair.

Preparation separates container create from Start. Native allocation is reported
and ACKed before Start, followed by another exact Control check and original
volume/daemon verification. Opaque consumed tokens and a bounded 256-entry
execution-stage record prevent replay/continuation after an unknown native stage.
The cache is transient correlation, not a new durable lifecycle ledger.
If that bound rejects a new record before any container-create submission, it
is a distinct local admission rejection. After earlier allocations have settled
and been ACKed, the Worker reports the existing safe PreparationBlocked fact,
not native uncertainty. Owner retry may reuse a positively retired cache slot
and the same original volume (still no-copy). Actually submitted/unknown stages
remain fenced and cannot be evicted or relabeled as rejected.

Actual Running requires coherent running status, true Running, false paused/
restarting/dead flags, positive PID and coherent timestamps. Start must also have
positively settled locally or have the exact prior server Running-completion fact
without a conflicting same-process unknown stage. A bare running snapshot after
Start ambiguity is not permission to report completion, stop or start again.
Docker `always` keeps unexpected exits in the same execution; transient exited,
paused, restarting or contradictory observations do not authorize restoration.

Intentional retirement consumes the exact admitted current call in stages:
inspect ownership/quiescence, one update to restart policy `no`, inspect effective
policy, one graceful Stop if required, coherent exited/known-never-started proof,
one non-force full-ID Remove with anonymous-volume deletion disabled, then exact
same-daemon absence. Each later mutation requires a fresh Control check. Grace is
30 seconds with a 45-second Stop request budget; other writes are bounded to
30 seconds, proof observation to 90 seconds and common process drain to 120 seconds.
No unpause, kill, force-delete or repeated mutation is introduced as recovery.

The original native task/permit survives a lost call waiter or stream incarnation.
A failed authority check/report ACK stops later effects; old-source results cannot
report under a new connection. A native uncertain result does not return a token
for the next stage. Correlation loss permits observation, not reconstructed Start,
Update, Stop, Remove or volume cleanup. The current server-side M16 manifest
independently fences initial dispatch and coordinated Delete suffix progression.

## Validation boundary

Unit and supported-SDK HTTP fixtures cover connection selection/redaction, TLS
policy, body lifetime, original-volume identity, no replay after ambiguous effects,
non-force stage order, saved-volume preservation, bootstrap/security/mount/ID
mismatches, and truthful power/terminal predicates. Actual external child-process
tests reach authenticated scoped Control with absent or poisoned product DB/root
configuration and no native mutation before registration.
The real-Control/native-SDK fixture additionally exercises allocation ACKs before
the next effect, normal stop/start/restart/delete and original-volume retention,
source replacement before volume ACK, and an accepted-but-lost Start result
across authenticated stream reconnect without mutation replay.
TLS handshake fixtures exercise required exact client authentication and frozen
credential material. An opt-in `ssh-fixture` feature exercises a real local
OpenSSH server/client with disposable keys, multiple mux channels, changed/lost
configuration and missing mux/master failure without reauthentication.
These checks do not establish real writable-volume population, actual cgroup/
seccomp enforcement, remote Engine security-mode acceptance, Engine restart behavior or
native Mac/Desktop compatibility. Platform/image packaging and real acceptance
remain separately validated delivery boundaries.

## Platform and required acceptance delivery

The standard Linux image stages its saved directory as UID:GID 10001:10001.
The Provider respects CLI context/socket selection and avoids unnecessary
host-OS coupling so a later Mac build can use it naturally. This effort adds
no Mac-only build, packaging, distribution or real-Mac completion gate.

The separate required Linux integration gate builds the Computer image from
the same-run native binary artifact and runs one browser-free Docker lifecycle.
An explicitly authorized disposable socket is mandatory. The fixture selects
the existing bridge and supplies a container-reachable fixture Control address,
not a guest port/tunnel/host-network workaround. The product journey verifies
non-root initial saved writes, Stop/Start with original data, successor no-copy,
emptied saved storage across Restart, exact normal cleanup and unrelated canaries.
The integration app invokes generated public owner RPC against actual backend/
Control/Provider/Connector. Support uses public Docker SDK calls and bounded
test-only exec, never a product exec API or guest Docker credentials. The browser
suite remains unchanged; no Docker-specific UI journey or Chromium is required.

Missing required prerequisites fail; no synthetic fallback or skip counts as a
real lifecycle pass. A workflow or cross-platform archive alone does not establish
an executed Linux, native Desktop or remote transport result. Executed Linux
acceptance does not imply verified native Desktop or remote transport behavior.
