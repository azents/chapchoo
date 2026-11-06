---
title: "Codex Authentication Setup Requirements"
created: 2026-11-06
implemented: 2026-11-06
document_role: primary
document_type: requirements
snapshot_id: harness-261106
tags: [harness, authentication, agent, security, frontend]
---

# Codex Authentication Setup Requirements

- Snapshot: `harness-261106`
- Document reference: `harness-261106/REQ`

## Problem

Users can create Agents with dedicated cloud Computers, but cannot register
Codex authentication, choose an authorized registered credential for an Agent,
or manage that reusable selection independently of Agent creation.

## Primary Context

### Primary Actor

An authenticated user managing an Agent and authentication they are
authorized to use.

### Primary Scenario

The user registers a ChatGPT account or OpenAI API key without needing an Agent
Computer, selects that authentication while creating a cloud Agent, and can later
identify, change or recover the Agent's authentication without losing its identity
or unintentionally changing other Agents' selections.

## Supporting Scenarios or Effects

- Register authentication from the creation/selection flow and return to that task.
- Reuse one authorized authentication record across several Agents.
- Manage Team-owned authentication within an existing Team authority boundary.
- Recover the same registered account or replace an API key while retaining the
  existing Agent selections.

## Goals

- Complete authentication setup without manual terminal or credential-file work.
- Separate record ownership, authorization to use and saved Agent selection.
- Preserve existing Agent identity, authorization, profiles and Computer controls.

## Non-Goals

- Agent conversation execution, model responses, model selection or session management.
- Personal Computer/connector onboarding or reuse of local Computer credentials.
- Persona selection, additional harnesses, user-facing cloud registration or invented
  Provider capabilities.
- A redesign of Agent/Team access permissions or additional Agent visibility modes.
- A mandatory installation-wide OpenAI application registration or mandatory SIWC.
- Paid model calls to validate an API key during registration.
- A newly required central automatic-refresh product capability.
- Sending credentials to or receiving credentials from Computers, Connector
  application commands, credential files in Computers, application acknowledgements,
  and runtime token-refresh coordination.

## Requirements

### REQ-1. Ownership and reusable selection

Authentication belongs to a User or Team, not a Provider or cloud environment.
One registered authentication can be selected by multiple authorized Agents.

**Acceptance criteria**

- Selecting a record for another Agent does not take it away from an existing Agent,
  require a new registration merely because it is in use, or duplicate its product identity.
- An Agent can select authentication authorized for its User/Team context.
- Authentication selection follows existing Agent/Team authorization rather than
  introducing a separate public/private Agent classification.
- Client filtering alone never grants permission to read, use or mutate a record.

### REQ-2. Register and manage in the correct scope

Users can register and identify authentication from its owning settings context.
The personal-space presentation does not require redundant personal/Team sections.

**Acceptance criteria**

- A first visit without records has a clear Codex connection action; subsequent visits
  present identifiable account cards and management entrypoints.
- Team settings expose only that Team's authorized records and registration scope.
- Team Owners and Managers may register, reauthenticate, replace keys and delete
  Team-owned authentication, subject to the in-use deletion restriction in REQ-7.
- All current Team members may select Team-owned authentication for Agents they
  are authorized to manage. This permits consuming the associated usage or API
  budget; it does not grant secret retrieval.
- Shared-record recovery and key replacement may affect every referring Agent;
  related Agent reads and actions follow existing Agent/Team authorization.
- Display name, authentication kind and provider-confirmed account information are
  distinguishable. Missing email/plan information is not fabricated.
- List/detail responses never include secret material.

### REQ-3. Computer-independent ChatGPT Device Auth

Users can start account authentication before a target Computer exists, open the
official device-authorization page and complete registration when provider approval
and token exchange succeed.

**Acceptance criteria**

- The flow presents an actual issued code, official verification link, pending state,
  remaining validity and cancellation. Chapchoo never asks for the OpenAI password.
- Opening the external page is not treated as approval or registration success.
- Successful provider approval and token exchange register the authentication in
  the authorized owning catalogue without an intermediate account-review screen
  or a second registration confirmation.
- Completion returns to the originating management or Agent selection task;
  in-place registration returns with the new eligible record selected and
  preserves the original non-secret input.
- A cancelled, expired or superseded attempt cannot later register itself or overwrite
  the current selection. Cancelling locally does not claim global OpenAI revocation.

### REQ-4. API-key storage has a bounded success meaning

Users can save an OpenAI API key with a display name and understand that API billing
is separate from their ChatGPT subscription.

**Acceptance criteria**

- Required inputs are labelled; key input is masked by default with a temporary display toggle.
- The official key-issuance entrypoint is available before submission.
- Saving requires nonempty valid input representation but does not incur model-call charges.
- Success is saved authentication with use unverified, not proof of validity, balance,
  model entitlement or application to a Computer.
- Secret input is discarded when cancelled and cannot be retrieved from list/detail views.

### REQ-5. Creation retains the computer-first journey

New cloud-Agent creation follows environment selection, Codex authentication, then
existing profile/name/handle/color entry.

**Acceptance criteria**

- Only registered, currently eligible cloud choices are offered; the cloud section,
  icon and selected environment card are preserved.
- Codex is the initial harness, not a fabricated multi-harness choice.
- The user selects an eligible registered authentication or registers one in place.
  Progress past authentication requires a selection.
- Existing image choices, name, immutable-handle explanation and color choices remain.
- Back/next navigation retains non-secret input, selected environment and authentication.
- Existing creation retry/recovery does not allocate duplicate Agents or silently change
  the authentication chosen for an accepted creation.

### REQ-6. Existing-Agent changes are local to that selection

Existing Agent details retain their profile and Computer controls and gain a compact
execution/authentication row with an explicit change action.

**Acceptance criteria**

- Changing authentication affects that Agent's selection only; it does not mutate the
  old shared record or another Agent's reference.
- The selection list includes authorized records already used elsewhere and disables
  unchanged selection submission where appropriate.
- Cancelling preserves the prior selection; registering in place returns with the
  new eligible record selected and preserves the original task's non-secret input.
- Pre-feature Agents are not deleted, silently assigned another person's authentication,
  or presented as authenticated merely because the feature is introduced.

### REQ-7. Recover the same shared record deliberately

Authorized managers can reauthenticate an invalid account or replace an API key while
preserving the registered record and its referring Agents.

**Acceptance criteria**

- Cancellation or failure preserves the existing record, references and known failure state.
- Successful same-account recovery updates the same stored record without
  creating a new record per Agent or changing its referring selections.
- A different ChatGPT account is not silently substituted for an existing shared identity.
- API-key replacement discloses affected connections and possible billing-project changes.
- A record still selected by an Agent is not deletable until those selections are resolved.

### REQ-8. Registration and saved selection have bounded meanings

Users can identify registered authentication and the saved Agent selection
without either being represented as credential delivery or execution readiness.

**Acceptance criteria**

- Users can distinguish no selection from a saved selection and identify the
  referenced authentication record.
- Successful registration or selection does not claim that credentials reached a
  Computer, that Codex is ready, or that models, funds or entitlements are available.
- This snapshot adds no Computer application status, application retry action or
  partial-propagation workflow.
- A cancelled or superseded selection operation cannot overwrite the current
  saved selection.

### REQ-9. Failure and uncertain-result recovery

Users retain their original task and receive a next action appropriate to the failure.

**Acceptance criteria**

- Device-code expiry offers a fresh code; provider unavailability offers retry without
  claiming the existing credential or Computer is invalid.
- Unknown registration or selection-save outcomes are not converted into confirmed
  failure and blindly resubmitted as duplicate effects.
- Model access denial, exhausted funds and transient network failures are not all
  represented as a need to log in again.
- Error messages and diagnostics do not disclose secret keys, tokens or codes beyond
  the intended active device-auth interaction.

### REQ-10. Current authority and lifecycle boundaries survive

The new setup respects current User, Team, Agent and Computer lifecycle authority.

**Acceptance criteria**

- Team membership/role loss, Team deletion and Agent deletion cannot leave newly
  authorized registration or selection work for a caller who no longer has authority.
- Agent identity, names and connection information follow existing Agent/Team
  access permissions, independently of permission to use a credential.
- Existing profile/handle contracts, Computer power/deletion semantics and safe recovery
  of uncertain creation/lifecycle results are preserved.
- Existing Computer connectivity and execution lifecycle remain independent of
  authentication registration and saved selection.

### REQ-11. Installation and secret-handling constraints

The feature supports independently hosted open-source installations without assuming
one project-operated authentication application or hosting vendor.

**Acceptance criteria**

- SIWC remains an optional operator-configured route, not a prerequisite for the
  approved basic account/API-key setup.
- Real provider credentials are protected from unauthorized catalogue, UI, log,
  exception and cross-owner access; registration has an explicit pending/confirmed boundary.
- Registering or reauthenticating a stored credential does not introduce a
  runtime refresh service or prescribe future Computer credential delivery.

## Fixed Constraints

- The approved product flow is authentication setup only, not proof of model execution.
- Public repository artifacts contain no private document provenance, private URLs,
  user research transcripts or real credential material.
- Authentication setup does not introduce separate Agent visibility modes or
  redesign existing Agent/Team access permissions.
- Existing data and accepted creation/lifecycle outcomes must survive the feature.

## Open Assumptions

- Example provider names, accounts, plans and Agent counts are illustrative, not
  guaranteed provider metadata or a new authorization rule.
- Provider availability and installation applicability remain external constraints.
  Computer consumption and runtime refresh are outside this snapshot.

## Confirmation

The requester approved the integrated product flow on 2026-11-06 KST and requested
technical design. On the same date, the requester confirmed that Team Owners and
Managers manage Team authentication and all current members may use it for Agents
they are authorized to manage. Authentication setup uses the Agent/Team access
model without introducing additional Agent visibility modes.
The requester confirmed the complete corrected repository Requirements on
2026-11-06 at 06:18 KST before ADR decisions began.
On 2026-11-06 at 07:02 KST, the requester explicitly removed the intermediate
account-review and final registration-confirmation step from Device Auth.
REQ-3 now completes registration on successful provider approval and token
exchange, retaining cancellation, expiry and existing-record recovery safeguards.
On 2026-11-06 at 07:33 KST, the requester explicitly excluded credential exchange
with Computers. The scope ends at backend authentication management and saved
Agent selection; delivery, application evidence and runtime refresh are excluded.
This confirmation authorizes technical design, not feature implementation,
merge or deployment.
