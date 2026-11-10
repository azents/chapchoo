---
title: "Codex Execution and Retirement Grounding"
created: 2026-11-08
tags: [agent, codex, acp, architecture, runtime]
---

# Codex Execution and Retirement Grounding

This note develops topic 2, execution and retirement, against
[chat-261108/REQ](../requirements/chat-261108-text-conversation.md) and accepted
[chat-261108/ADR](../adr/chat-261108-text-conversation.md) D1, D3, D8 through D15.
It is not a completed primary Design, implementation approval or live acceptance
report. Product-record authority, delivery recovery and credential materialization
remain separate topics.

## Policy Already Decided

- The Connector lifetime owns the managed ACP/native connection. A replaceable
  server stream is not that lifetime. A successor Connector does not adopt the
  predecessor's live harness.
- In an available authorized Computer, actual user text starts a harness when
  needed. UI reads and Computer/Connector availability alone do not eagerly
  start it or power on a Computer.
- Existing live work is reused. Task completion does not retire the harness, apply
  pending supply settings or introduce an idle timer.
- New starts apply the saved owner selection and managed artifact check. Active
  configuration remains unchanged while the current harness survives.
- Lifetime shutdown has bounded cooperation/output drain followed by forced
  cleanup as required. Unknown retirement is not success or successor admission.
- Current-work Stop, native session closure, complete process retirement and
  Computer power remain distinct.

These outcomes require implementation and evidence, not another policy interview.

## Owner Placement

The native runtime owner must be created outside the Connector's connection loop.
`connector.rs` returns from that loop for ordinary operational-key refresh as well
as stream loss. Recreating the owner inside the loop would end or duplicate a
harness on a routine refresh and violate D1.

The Connector-owned runtime retains native transport, process handles and observed
session/request state while a server stream is replaced. Stream publication uses
that owner's observations; it does not become the native owner. Exact catch-up
and persistence semantics belong to the delivery topic.

The process scope includes the adapter, runtime, Codex launcher/native process and
owned tool/MCP descendants. A single direct-child PID does not represent that
scope.

## Launch Environment Contract

Keep the supported Computer's existing non-root identity, read-only rootfs,
capability restrictions and original saved storage. Do not solve native startup
failure by adding root execution, host access or silently weakening seccomp.

For each new start, bind one coherent executable/artifact selection and structured
argv/environment to the actual Computer context. An existing CLI's absolute path
is interpreted inside that Computer, not on the server or Provider host. It is
not a configurable shell command string.

The implementation must explicitly account for:

- working directory/workspace roots inside the allowed saved environment;
- native persistent context, managed artifacts and temporary preparation/runtime
  data as different ownership/cleanup categories;
- writable HOME/native-state/TMP requirements rather than assuming the image's
  read-only home or `/tmp` can be written;
- the runtime required by an existing CLI wrapper, including its interpreter
  resolution;
- child environment construction without inherited Connector bootstrap/operational
  keys, server configuration or ambient backend authentication;
- native provider/configuration inputs that could otherwise select a different
  account or provider.

Directory names and equivalent helper organization are local details. Directories
under the same UID are not a security isolation boundary. Credential storage and
refresh ownership are not selected by this logical separation.

## Readiness and Input

Preparation, process presence, ACP v2 initialization, selected-authentication
application and native session setup are separate evidence boundaries.

ACP session-scoped prompting requires an active session after initialization.
Restoration must target the existing native context; inability to restore cannot
silently create fresh context. A resume response synchronizes current foreground
state and does not mean the session is idle.

The first user input must survive preparation delay/failure under REQ-2/REQ-12.
Its durable admission/dispatch settlement is a delivery-topic contract, not a
reason to add hidden retries or a future-task queue here.

Keep consuming native stdout/stderr under an owned, bounded processing contract.
ACP stdout carries protocol frames; debug/log contamination is not assistant
text. Stderr diagnostics must remain bounded and secret-safe. Raw producer wire
logging is not enabled as routine product observability.

## Normal Shutdown

On an actual lifetime-stop condition, stop admitting new native work, request
supported cooperative termination and drain output/status within finite budgets.
Escalation must address the owned process scope, not merely close a writer or
cancel an async waiter.

Coordinate inner and outer budgets. Existing Connector signal handling waits at
most 15 seconds for its current loop, and the Docker profile has a 30-second stop
budget. Neither establishes a new harness budget or actual child cleanup. The
complete Design must leave room for signal, drain, escalation and retirement
observation within the supported outer lifecycle.

Already-received partial evidence remains available. The final native output may
still be absent; graceful opportunity is not guaranteed flush or external-effect
rollback.

## Crash and Whole-Scope Retirement

### Source-backed enforcement direction, still requiring actual proof

The current supported Linux Computer runs an exact Connector executable entrypoint
inside a restricted container. Linux PID namespaces terminate their processes when
the namespace init process exits. This provides an owner-death enforcement
candidate only when the actual Connector/PID1/namespace conditions are verified.

Docker rejects nonempty PID-mode overrides. Kubernetes uses one Connector
container and rejects host PID sharing, but its current collision predicate does
not explicitly reject `shareProcessNamespace`. Do not infer the required runtime
condition merely from an intended command.

Before relying on namespace teardown, verify the supported profile and runtime
identity. An incompatible or ambiguous environment cannot be represented as
providing the D1 cleanup guarantee. A process group can support ordinary signalling
but does not independently contain `setsid`, double-fork or reparented descendants.

Kernel kill enforcement and observed retirement are also distinct. A bounded
observation failure remains uncertain; it is not permission to start a successor.
These Linux facts are not a contract for an unimplemented noncontainer backend.

### Producer-internal replacement

The inspected adapter has its own native app-server recovery. Generation-bound
prompt execution prevents blindly moving a prompt operation to a restarted native
client, but does not prove old-process physical retirement. Test that internal
replacement and the outer owner do not overlap an unretired scope.

The outer owner must not advertise successful recovery simply because the adapter
closed old pipes or can answer another request. If retirement cannot be
established, retain the failure/uncertainty boundary and do not silently use
Computer restart, fresh context or replay to hide it.

## Remaining Engineering Gates

- Exact native runtime ABI and sandbox/tool behavior on the supported non-root,
  read-only, capability-drop Computer profiles. Official Node needs runtime
  libraries not explicitly present in the current backend image.
- Safe child ownership, cancellation/drain and detached-descendant handling.
- Actual PID1/namespace/profile validation and native retirement witnesses.
- Producer internal app-server failure/replacement interacting with outer ownership.
- Input/answer/Stop racing shutdown without a late result restarting old work.
- Persistence/cleanup boundaries that leave native context and user workspace
  intact while retiring temporary execution resources.

These are verification obligations, not grounds to reopen already selected
lifetimes or invent new automatic power controls. If evidence requires a new
security relaxation or contradicts an approved outcome, report that concrete
conflict instead of silently changing the contract.

## Verification Shape

Use deterministic subprocess/protocol fixtures for normal exit, ignored SIGTERM,
wrapper-first exit, delayed stdout drain, detached and double-fork descendants,
Connector death, failed runtime-profile admission and exhausted outer budgets.
Use explicit barriers, not sleeps, to establish ordering.

Keep browser E2E to minimal representative product boundaries; process-tree and
shutdown combinations belong in lower-level owned integration tests. Actual
Docker/Kubernetes and exact native binary acceptance are separate from fixture
or static-source success.

## References

- [Connector lifecycle](../spec/domain/connector-process.md)
- [Docker boundary](../spec/domain/computer-docker.md)
- [Kubernetes boundary](../spec/domain/computer-kubernetes.md)
- [Computer execution evidence](../spec/domain/execution-evidence.md)
- [ACP v2 session setup](https://agentclientprotocol.com/protocol/v2/session-setup)
- [Linux PID namespaces](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html)
- [Producer native supervisor](https://github.com/agentclientprotocol/codex-acp/blob/34e0897/src/app-server-recovery/CodexAppServerSupervisor.ts)
- [Producer recovery](https://github.com/agentclientprotocol/codex-acp/blob/34e0897/src/app-server-recovery/AppServerRecovery.ts)
