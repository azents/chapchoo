---
title: "Chapchoo Spec Wiki"
---
# Chapchoo Spec Wiki

This is the connected current-system Spec, not a summary beside a second Spec.
Start with a question and follow the owning page's connections. Only `spec/` is
the wiki: Requirements, ADRs, Designs, implementation and tests remain references
with their own lifecycles. Read [AGENTS.md](AGENTS.md) for ingest, query and lint.

## Follow a user or operator question

### How does an account become authorized work?

[Credential primitives](domain/credential-primitives.md) validate representation
and protect secrets. [Account services](domain/account-operations.md) prepare
credentials and submit mail outside [database transactions](domain/account-transactions.md).
[Account RPC](domain/account-rpc.md) authenticates each caller and exposes typed
outcomes. [Web sessions](domain/web-session.md) adapt that authority to requests
and cookies without inventing another credential store.

### Why can I see a workspace but not another member's Agent?

[Workspace navigation](domain/workspace-navigation.md) chooses current context;
[Agent ownership](domain/agent-ownership.md) independently checks membership and
private ownership. [Owner RPC](domain/computer-client-rpc.md) and
[Computer web views](domain/computer-web.md) project that authority. Team deletion
accepts [lifecycle intent](domain/computer-lifecycle.md), not completed deletion.

### Who may perform a Computer effect, and what proves completion?

Providers first establish [authentication and owner scope](domain/provider-authentication.md)
through explicit distributed evidence or Standalone's private internal Trust.
The [directory](domain/computer-directory.md) finds a current owner, while
[Control](domain/computer-control-rpc.md) revalidates streams and exact calls.
[Reconciliation](domain/reconciliation.md) chooses current work;
[Providers](domain/provider-process.md) perform native operations and wait for
allocation acknowledgements. [Execution evidence](domain/execution-evidence.md)
and [Docker retirement correlation](domain/native-retirement.md) preserve the
difference between accepted work, uncertainty and safe next actions. A dropped
waiter is not cancellation of a physical effect.

### What survives restart, reconnect or removal?

[Repository identity](domain/repository-identity.md) separates opaque product IDs
from native allocation witnesses; its
[operator procedure](domain/repository-identity.md#operator-procedure-for-the-identity-cutover)
defines the coordinated stopped boundary, paired backup and explicit preparation.
[Computer schema](domain/computer-storage.md) records exact native identities and
independent cleanup obligations. [Connector credentials](domain/connector-credentials.md)
belong to one execution; its [process](domain/connector-process.md) reports
connectivity, not power or harness readiness. [Docker](domain/computer-docker.md)
preserves the original volume; [Kubernetes](domain/computer-kubernetes.md)
requires exact UID and ordered terminal evidence. [Peer delivery](domain/peer-delivery.md)
re-resolves only explicit ownership rejection, never uncertain effects.

### Which behavior is presentation, and which is account state?

[General/Admin ownership](domain/account-web.md) separates app responsibilities.
[Primary rendering](domain/web-rendering.md) owns server snapshots and URL state;
[appearance](domain/appearance.md), [display settings](domain/display-settings.md)
and [mascot motion](domain/mascot-motion.md) have distinct persistence and
interaction contracts. [Design reference](domain/design-reference.md) is a design
workspace, not automatic production promotion or native execution proof.

### What does an artifact or green check actually establish?

[Foreground process ownership](domain/runtime-process.md) controls selected
components. [Linux artifact assembly](domain/account-delivery.md),
[Helm/Argo deployment](domain/deployment.md) and
[Snapshot publication](domain/snapshot-publication.md) are different boundaries.
[Acceptance evidence](domain/acceptance.md) distinguishes synthetic SDK fixtures,
real Docker, browser journeys and live cluster rollout.

## Identity, account authority and durable state

- [Repository-owned entity identity](domain/repository-identity.md) — UUIDv7 issuance, compact storage, opaque DTOs and explicit preserving cutover.
- [Agent handles and saved profiles](domain/agent-profile.md) — immutable handles, editable saved presentation, catalogue/palette choices and exact uncertain-creation recovery.
- [Credential primitives](domain/credential-primitives.md) — email/password values, bearers, encryption and hashing.
- [Backend authentication](domain/backend-authentication.md) — scoped catalogue, explicit device acquisition and saved Agent references without Computer delivery.
- [Account operations and mail](domain/account-operations.md) — password capacity, acceptance time and single-attempt delivery.
- [Account RPC](domain/account-rpc.md) — service groups, per-call credentials, wire admission and errors.
- [Account schema and preparation](domain/account-storage.md) — keys, relationships, original-root binding and readiness.
- [Private conversation storage](domain/conversation-storage.md) — saved/applied supply observations, retained input/answer acceptance and source-bound event pairs.
- [Account and Team transactions](domain/account-transactions.md) — sessions, grants, invitations, roles and context deletion.
- [Private Agent ownership](domain/agent-ownership.md) — Team scope, private access and cleanup after leave/delete.

## Computer lifecycle, resources and communication

- [Computer schema and native identities](domain/computer-storage.md) — durable relationships, preserving migrations and inventory.
- [Lifecycle admission and Provider keys](domain/computer-lifecycle.md) — current intent, safe removal, key overlap/recovery and revocation.
- [Execution reservation and evidence](domain/execution-evidence.md) — preparation fences, source-qualified reports, retirement and retention.
- [Connector credentials](domain/connector-credentials.md) — execution-bound bootstrap, operational refresh and withdrawal.
- [Control directory](domain/computer-directory.md) — exact process/connection ownership and DB-time eligibility.
- [Control streams and calls](domain/computer-control-rpc.md) — component/peer separation, bounded queues and call lifetime.
- [Native retirement correlation](domain/native-retirement.md) — exact live manifests, uncertain continuation and unsent suffixes.
- [Provider process](domain/provider-process.md) — DB-free native work, acknowledgements and retained effects.
- [Provider authentication](domain/provider-authentication.md) — bootstrap policy, SA review, protected/random credentials and internal Trust.
- [Connector process](domain/connector-process.md) — execution-local connectivity and shutdown.
- [Native preparation and lifetime](domain/native-harness.md) — strict-v2 owner, applied supply and private-scope retirement.
- [Convergence and restoration](domain/reconciliation.md) — owner precedence, observation and authorized successors.
- [Peer delivery](domain/peer-delivery.md) — generated memory/network transport, routing and retry limits.
- [Kubernetes adapter](domain/computer-kubernetes.md) — original PVC, Pod/Secret profile and ordered retirement evidence.
- [Docker adapter](domain/computer-docker.md) — selected connection, original volume, container stages and uncertainty.
- [Owner and inventory RPC](domain/computer-client-rpc.md) — private lifecycle operations and redacted administrator reads.

## Product surfaces

- [General/Admin web ownership](domain/account-web.md) — shared presentation and independent authority.
- [Workspace navigation](domain/workspace-navigation.md) — recent/personal context, selection and administration links.
- [Web request/session ownership](domain/web-session.md) — Origin/POST admission, authentication, logout and shared cookie disposition.
- [Primary rendering and URL state](domain/web-rendering.md) — SSR snapshots, initial queries and history navigation.
- [Runtime web configuration](domain/web-runtime.md) — mounts, origins, backend targets and safe failures.
- [Appearance](domain/appearance.md) — system/light/dark paint and synchronization.
- [Display settings](domain/display-settings.md) — independent language/time-zone/region and installation defaults.
- [Mascot motion](domain/mascot-motion.md) — replay, visibility and reduced-motion ownership.
- [Computer web](domain/computer-web.md) — truthful projections, bounded observation and read-only inventory.
- [Private conversation web](domain/conversation-web.md) — SSR/hydrated source-bound records, retained inputs/answers and independent current-work Stop.
- [Authentication web](domain/authentication-web.md) — scoped AI connections, in-place Device/API-key registration and saved Agent selection.
- [Design reference](domain/design-reference.md) — Storybook boundaries, promotion and verification.

## Runtime and delivery

- [Foreground ownership and health](domain/runtime-process.md) — startup, readiness and owned shutdown.
- [Linux artifacts and images](domain/account-delivery.md) — source-free package inputs and deployment-neutral assembly.
- [Helm and Argo deployment](domain/deployment.md) — explicit prerequisites, process/resource scope and hook lifecycle.
- [Snapshot publication](domain/snapshot-publication.md) — native publication, digests and downstream handoff.
- [Verification boundaries](domain/acceptance.md) — test-layer ownership and the limits of release evidence.
