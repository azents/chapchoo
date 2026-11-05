---
title: "Codex Authentication Setup Decisions"
created: 2026-11-06
document_role: primary
document_type: adr
snapshot_id: harness-261106
tags: [harness, authentication, architecture, security]
---

# Codex Authentication Setup Decisions

- Snapshot: `harness-261106`
- Requirements authority: [harness-261106/REQ](../requirements/harness-261106-codex-authentication.md), confirmed on 2026-11-06 at 06:18 KST.
- Mode: Collaborative.
- Material decision owner: requester. No remaining decisions have been delegated.
- Accepted decisions: `harness-261106/ADR-D1`, `harness-261106/ADR-D2`. Unresolved topics below are not an implementation Design.

## Current Scope

The requester excluded credential exchange with Computers on 2026-11-06 at
07:33 KST. This snapshot ends at backend registration, secret storage, management,
reauthentication and saved Agent selection. The proposed T5 delivery question is
withdrawn, not pending or accepted. Native credential input, application evidence,
Computer transport security and runtime refresh are outside this snapshot.
D1 and D2 remain valid within this boundary; their accepted text below is retained.

## Current System and Required Change

The current account, Team, Agent and Computer services provide authentication,
transactional admission, creation recovery, execution-bound Connector authority
and lifecycle cleanup. They do not implement a reusable Codex credential
catalogue, pending external login or saved Agent credential selection.

Relevant boundaries:

- [Credential primitives](../spec/domain/credential-primitives.md) and
  [`CredentialCipher`](../../../rust/crates/crypto/src/root.rs) use one
  installation root with purpose-separated derived keys. `protect_latest`
  accepts a 43-character session bearer; its recovery envelope requires exactly
  59 ciphertext bytes. It cannot store arbitrary OAuth bundles or API keys
  unchanged.
- [Account storage](../spec/domain/account-storage.md) binds a database to the
  configured root without storing that root. Existing preparation rejects a
  mismatched root. There is no existing key-rotation facility to extend by
  assumption.
- [Agent admission](../spec/domain/agent-ownership.md) and
  [`Store::create_cloud_agent`](../../../rust/crates/persistence/src/computers.rs)
  atomically record accepted creation and its canonical request fingerprint.
  The current request has no authentication selection. Existing receipts and
  Agents must survive the extension; a new required selection is not permission
  to reinterpret a prior accepted creation.
- [Connector process](../spec/domain/connector-process.md) handles only
  `ObserveConnector`. Its `ready` result is connectivity, not successful
  credential application. [Control](../spec/domain/computer-control-rpc.md)
  provides correlated calls, not a durable credential-delivery log. Existing
  trusted-network transport does not itself establish confidentiality for
  long-lived model credentials.
- Current owner-qualified Agent queries are implementation evidence, not a
  separate public/private Agent product classification. This snapshot does not
  create Agent visibility modes or reopen an aggregate-disclosure interview.

## Fixed and Derived Boundaries

These outcomes are already authorized; they are not new questions:

- User/Team ownership, reusable Agent selection, Team management/use roles and
  existing Agent/Team authorization follow REQ-1, REQ-2 and REQ-10.
- Device Auth must work before an Agent Computer exists. Successful provider
  approval and token exchange complete registration without an intermediate
  account-review screen or second confirmation. Pending, cancelled, expired
  and superseded work still needs an authoritative acceptance boundary
  (REQ-3 as amended on 2026-11-06 at 07:02 KST, REQ-9).
- API-key save has no paid validation call and is not proof of validity,
  balance, model permission or Computer application (REQ-4).
- Shared-record recovery preserves record identity and referring Agents;
  changing one Agent selection does not change another. In-use deletion is
  prohibited (REQ-6, REQ-7).
- Stored selection is not Computer delivery or execution readiness. The scope
  adds no application command, acknowledgement or partial-propagation workflow
  (REQ-8 and REQ-10 as amended on 2026-11-06 at 07:33 KST).
- Existing creation outcomes and lifecycle uncertainty must retain their
  identity and recovery meaning. Preserving old data is required; adding a
  speculative legacy authentication fallback is not (REQ-5, REQ-6, REQ-9,
  REQ-10).
- Independently hosted installations and optional operator-configured SIWC
  remain fixed constraints. Backend reauthentication does not introduce a
  runtime refresh service or credential delivery capability (REQ-11).
- Supported public integration surfaces take precedence over handwritten
  provider calls under the repository's
  [external-service integration convention](../../../.claude/conventions/global/prefer-supported-external-service-sdks.md).
  An internal-only API is not a supported substitute. Direct provider protocol
  implementation is not an admitted default merely because its source is visible.

## Material Decision Map

Topic IDs identify unfinished questions; they are not accepted decision IDs.
After an explicit choice, append the next `harness-261106/ADR-DN` with its
consequences and authority. Do not mark a recommendation as accepted.

### T1 — External-secret key authority: accepted as ADR-D1

Choose whether external credentials derive protection from the existing
installation root or require a separate operator-managed authentication root.
This changes compromise coupling, configuration, preparation and backup/restore
obligations. Both require a new purpose-specific secret-protection capability;
neither reuses the session-bearer recovery API unchanged.

### T2 — Device Auth integration boundary: accepted as ADR-D2

Establish the supported acquisition integration before deciding placement.
Device-code issuance, polling and token exchange do not inherently require a
child process or separate service. Documented Codex app-server integration uses
an executable boundary; exported Rust login functions also exist in upstream
source. Public source visibility does not by itself establish an independently
supported library integration. The earlier child-versus-worker placement brief
is suspended, not accepted. Admit a placement decision only if viable acquisition
paths actually leave a consequential placement choice.

### T3 — Pending-attempt concurrency and restart recovery: derived framing

D2 selects the existing backend/persisted-attempt pattern, not an OS login worker.
REQ-3 and REQ-9 already require retained completion identity, cancellation/expiry
fencing and no blind replay. Therefore do not ask again whether progress belongs
in the DB or a login process. Frame short database admission and final acceptance
around bounded external I/O using the existing completed-operation convention.
Distinguish pending approval, admitted exchange, known completion and uncertain
external completion; a timer does not turn an admitted one-time exchange back
into unsent work. Conditional publication alone does not prove external exchange
ran only once. Any proposed additional resumable worker, queue, lease-based replay
or retained secret history would be a new material decision, not implied here.

### T4 — Acquisition dependency delivery: determined by D2

No Codex executable, Python runtime or independent worker is required for backend
authentication acquisition. That consequence of D2 is not a further requester
choice. A future native execution dependency has its own scope and cannot be
introduced as an acquisition prerequisite. This does not declare Computer
application feasible or resolve F1.

### T5 — Computer delivery: withdrawn as out of scope

Neither routed secret payloads nor protected Connector fetch is selected.
Do not extend Control, PeerRelay, Connector, Provider or Computer TLS
configuration for this snapshot.

### F1 — Shared native input and runtime refresh: outside scope

Multiple Agents may save references to one backend credential without any
credential being sent to a Computer. Native input/refresh feasibility is not a
blocker for this bounded snapshot. It is also not declared solved or supported.
Do not include a runtime refresh mechanism, native credential files, Codex account
probe or token propagation in this Design.

## First Decision Brief — T1

**Question:** Should external-authentication secret protection use the existing
installation root or an independent operator-managed root?

### Option A — Existing installation root, separate cryptographic purpose

- Reuses the installation's existing operator key and root-binding lifecycle.
- Requires a distinct derived purpose and secret-protection interface rather
  than treating OAuth tokens/API keys as session bearers.
- Avoids an additional required root configuration and independently paired
  recovery secret.
- Compromise or loss of the installation root affects both current protected
  installation material and the new external credentials. Purpose separation
  prevents cross-purpose cryptographic reuse, not root-compromise isolation.

### Option B — Independent external-authentication root

- Introduces a separately supplied key and its own preparation/binding and
  backup/restore obligations.
- Can separate recovery-secret custody and damage from disclosure of only one
  root, if operators genuinely separate access.
- A second key supplied to the same process by the same operator does not
  protect external credentials from that process being compromised.
- Does not by itself add a secret manager, key rotation or a second runtime
  authentication mode; those would need their own authority if proposed.

**Recommendation:** A, because this feature has no approved independent key
custody boundary and the existing installation root already owns protected
credential material. The new secret capability must still have explicit purpose,
context, size and redaction contracts in the subsequent Design.

**Choice:** Option A, explicitly accepted by the requester on 2026-11-06 at
06:24 KST. See ADR-D1 below.

## D1 — Use the Existing Installation Root with a Separate External-Auth Purpose

- Reference: `harness-261106/ADR-D1`.
- Accepted: 2026-11-06 at 06:24 KST.
- Decision owner: requester.
- Authority: `harness-261106/REQ-11`; supporting constraints in REQ-2, REQ-3,
  REQ-4 and REQ-10.

### Decision

Protect external-authentication secrets using a distinct cryptographic purpose
derived from the existing installation root. Add a purpose-specific protection
capability for OAuth material and API keys; do not pass them through the
session-bearer recovery interface or widen its historical fixed representation.
Do not introduce an independently configured external-auth root.

### Rationale and Alternatives

Option A retains the existing installation key custody and recovery boundary
without adding a second mandatory operational secret. Option B, an independent
root, was not selected because no separate key custodian or process-isolation
boundary is required by the confirmed scope. Separate custody would have added
preparation and paired backup/restore obligations; merely supplying two roots to
the same process would not isolate a compromise of that process.

### Consequences and Limits

- Compromise or loss of the installation root affects both existing protected
  installation material and the new external-authentication secrets.
- Purpose separation prevents unintended cross-purpose key reuse; it does not
  reduce that root-compromise blast radius.
- The Design must specify new secret representation, context binding, bounds,
  redaction and failure behavior without changing existing session recovery.
- Current installation-root binding and recovery obligations remain applicable.
  This decision does not authorize root replacement, rotation, key versioning,
  a secret-manager integration or an additional runtime mode.
- Process placement, pending-attempt retention, Codex distribution, delivery
  confidentiality and shared refresh ownership remain unresolved. This decision
  does not establish their feasibility or approve feature implementation.

## T2 Research — Integration Before Placement

Acquisition is now accepted as backend HTTP integration in D2. The previous
child-process-versus-worker question prematurely assumed an executable integration
and is withdrawn, rather than awaiting a placement choice.

### Existing Backend Reference

Azents source at `08180c4017e4830852f7f0cbf8915f374a7f646e` provides an
existing in-process acquisition pattern:

- Its [provider client](https://github.com/azents/azents/blob/08180c4017e4830852f7f0cbf8915f374a7f646e/python/apps/azents/src/azents/services/chatgpt_oauth/client.py)
  uses HTTP for code issuance, poll-once, authorization-code exchange and refresh.
- Its [service](https://github.com/azents/azents/blob/08180c4017e4830852f7f0cbf8915f374a7f646e/python/apps/azents/src/azents/services/chatgpt_oauth/__init__.py)
  checks an owned unexpired pending session and performs provider I/O outside
  completed database operations. Browser polling drives the check; the path
  does not launch a Codex binary or a dedicated login worker.
- Its database operations conditionally consume the pending session and publish
  encrypted integration credentials atomically. This is a concrete reference
  for adapting acquisition to the Chapchoo Rust backend without an OS login
  process or a dependency on a running Azents installation.

Azents publishes during successful polling. The requester amended REQ-3 to
complete registration after successful approval and token exchange, removing the
previous additional account-review and Register action. This removes that
publication-timing difference from the reference adaptation.
Chapchoo authority, cancellation/late completion, concurrent exchange and secret
protection still need their own contracts. Existing direct-provider code is an
implementation precedent, not evidence of a supported SDK contract or automatic
approval of a direct-call exception; D2 records the actual scoped approval.

### Upstream Library and Executable Evidence

At upstream source `49f0c6022596c361fe676a5c4738fb2be34e3a93`:

- [`codex-login` exports](https://github.com/openai/codex/blob/49f0c6022596c361fe676a5c4738fb2be34e3a93/codex-rs/login/src/lib.rs)
  include `request_device_code` and `complete_device_code_login`.
- Their [implementation](https://github.com/openai/codex/blob/49f0c6022596c361fe676a5c4738fb2be34e3a93/codex-rs/login/src/device_code_auth.rs)
  performs asynchronous HTTP issuance, polling, token exchange and native
  persistence; a dedicated OS login process is not intrinsic to those operations.
- The [crate manifest](https://github.com/openai/codex/blob/49f0c6022596c361fe676a5c4738fb2be34e3a93/codex-rs/login/Cargo.toml)
  depends on numerous Codex workspace crates and inherits workspace metadata.
  Standalone SDK stability, supported third-party consumption, practical build
  compatibility and resulting dependencies have not been established.
- The [TypeScript SDK](https://github.com/openai/codex/blob/49f0c6022596c361fe676a5c4738fb2be34e3a93/sdk/typescript/README.md)
  explicitly wraps and spawns the CLI. Using that SDK is not evidence of an
  in-process authentication integration.

Distinguish documented app-server integration, exported upstream Rust code and
handwritten provider-protocol implementation. Do not select a library or permit
direct provider calls merely because source exists. Provider applicability and
pending-attempt acceptance remain evidence obligations. D1 and D2 supply the
actual accepted authority; no extra dependency is authorized by this research.

## T2 Decision Brief — Acquisition Integration

**Question:** Adapt the existing direct-HTTP acquisition reference to the Rust
backend, or integrate Codex's native implementation as a dependency?

### Option A — Typed Backend HTTP Integration

- Adapt Azents's code issuance, poll-once and authorization-code exchange into
  Chapchoo's typed provider client and completed-operation boundaries.
- No acquisition-only Codex runtime, Python process, additional service or
  per-attempt worker deployment is required.
- Chapchoo owns endpoint/request/response compatibility, bounded network calls,
  sanitization and provider-specific fixtures. Existing Azents code supplies
  evidence, not automatic compatibility or permission guarantees.
- This requires a scoped exception to the supported-SDK-first convention for
  these acquisition endpoints. No supported standalone Rust SDK contract for
  these operations has been established; upstream source exports alone do not
  settle that gap. The exception does not extend to model inference or unrelated
  provider APIs.

### Option B — Codex Native Dependency Integration

- Reuses upstream-owned login mechanics instead of implementing the provider
  exchange contract in Chapchoo.
- Documented app-server integration brings a runtime executable boundary;
  in-process Rust exports bring multiple workspace dependencies and an unproven
  standalone consumer/compatibility contract.
- A stable linked-library alternative is conditional on verifying actual build
  compatibility and an acceptable dependency boundary; it is not already proven
  feasible. It must not silently fall back to private APIs or manual HTTP.
- Selecting a native dependency would require subsequent packaging/lifecycle
  decisions; it is not permission to add a worker service automatically.

**Recommendation:** A, as the smallest acquisition boundary aligned with the
existing verified product implementation. Explicitly disclose the direct-HTTP
compatibility responsibility rather than implying that an SDK owns it.

**Choice:** Option A, explicitly accepted by the requester on 2026-11-06 at
07:15 KST. See D2 below. This does not resolve shared native Codex input/refresh
or provider applicability.

## D2 — Acquire Device Authentication in the Backend over HTTP

- Reference: `harness-261106/ADR-D2`.
- Accepted: 2026-11-06 at 07:15 KST.
- Decision owner: requester.
- Authority: `harness-261106/REQ-3`, REQ-9 and REQ-11; scoped
  requester-approved exception to the supported-SDK-first convention.

### Decision

Adapt Azents's existing device-code issuance, approval polling and
authorization-code token exchange into the Chapchoo Rust backend using a typed
HTTP integration. Keep provider I/O outside completed database operations.
Successful approval and exchange complete registration under current
authorization, without another account-review or registration-confirmation step.

Do not add a Codex binary, Python process, independently deployed worker or
running Azents service as an authentication-acquisition dependency.

### Rationale and Alternatives

The existing Azents path demonstrates in-process provider acquisition and
encrypted durable session/integration storage. A supported standalone Rust SDK
contract has not been established. Native app-server integration would require
an executable boundary; upstream Rust exports are coupled to the Codex workspace.
The selected bounded direct-provider integration avoids those acquisition-only
dependencies while making compatibility responsibility explicit.

### Consequences and Limits

- Chapchoo owns acquisition endpoint and response compatibility, typed parsing,
  bounded network operations, redaction and provider fixtures.
- Direct-call authorization is limited to code issuance, approval checking and
  authorization-code exchange. It does not authorize handwritten inference,
  a central refresh service, private SDK APIs or an external-token injection mode.
- Secret protection follows D1. Current authority, cancellation, expiry,
  supersession, concurrent requests and uncertain external outcomes still fence
  acceptance. The detailed recovery contract remains to be finalized.
- Provider applicability for independently operated installations remains an
  evidence obligation. An existing implementation is not a universal provider
  permission or successful live authentication test.
- This is technical-design authority, not implementation, merge or deployment
  approval.

## Agent-Owned Details

Subject to future accepted decisions, local names, module/file boundaries,
equivalent typed helper structures, fixture names and formatting are agent-owned
when they add no new authority, state, configuration, runtime mode or failure
behavior. Ciphertext envelope semantics, retry ownership, retained state,
configuration and source-of-truth boundaries are not hidden in this category.

## Replacement and Validation Obligations

There is no implemented external-authentication feature to remove. Current
contracts affected by the future Design include Agent creation requests and
receipts, Agent projections, credential-related RPC/generated clients,
and connected Specs. Connector commands/results, Provider processes and
Computer transport remain unchanged. Existing account, Team and lifecycle authority remains a
constraint rather than a blanket refactoring target.

The Design must map every Requirement to a mechanism and test plan, preserve
existing accepted data on both SQLite and PostgreSQL, identify exact removals,
and distinguish fixture execution from live provider authentication. No code change, provider login, paid inference,
feature implementation or deployment is authorized by this decision map.
