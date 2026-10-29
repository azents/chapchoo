---
title: "Server Account and Team Onboarding Requirements"
created: 2026-10-29
tags: [accounts, authentication, teams, administration, deployment]
document_role: primary
document_type: requirements
snapshot_id: accounts-261029
---

# Server Account and Team Onboarding Requirements

- Snapshot: `accounts-261029`
- Document reference: `accounts-261029/REQ`
- State: requester-confirmed; not yet implemented.

## Problem

Chapchoo currently has a Hello World frontend and development tooling, but no
usable account or team-management cycle. An invited user needs to join a deployed
server and start using a personal space without creating an organization first.
Users who collaborate need explicit team membership and management permissions.
The operator must be able to initialize and administer the server without making
email delivery a prerequisite for deployment.

## Primary Actor and Scenario

The primary actor is a person invited to a deployed Chapchoo server.

The person opens an email-bound signup link, creates an account, signs in, and
receives a personal space automatically. When collaboration is needed, the person
creates a team or accepts a team invitation. The person can then inspect the team
and perform only the membership-management actions permitted by their role.

This cycle ends at account and team management. It does not require an agent,
harness, external channel, or computing resource to be implemented.

## Supporting Scenarios

- An operator initializes an empty server and administers signup and account
  recovery through a dedicated Admin UI.
- An existing user signs in again or recovers a forgotten password using a
  server-administrator-issued reset link.
- A team owner or manager invites an existing or not-yet-registered user.
- A member leaves a collaborative team without losing their personal space.

## Goals

- Deliver a deployable server, a general web client, and a separate Admin UI.
- Deliver Docker images and a Helm chart, then verify the cycle on a local
  Kubernetes cluster.
- Support SQLite for routine local development and PostgreSQL configuration for
  deployment and QA in this cycle.
- Support email-based account access without requiring outbound email delivery.
- Make personal use immediate after signup and collaboration explicit.
- Keep server-administration permissions separate from team permissions.
- Establish confirmed outcomes before architecture, protocol, Rust-stack, and
  implementation-pattern decisions.

## Non-Goals for This Cycle

- Adding features or user-visible policies absent from the Azents reference
  merely as design improvements. Retain the separately requester-approved
  Chapchoo differences and the explicitly required technical outcomes.
- Introducing new account-lockout/unlock features or a new 5-failures/15-minute
  password-login policy from an unaccepted design proposal.
- Mac application packaging or implementation of Standalone/client/server app modes.
- iOS or Android applications.
- Harness providers, harness orchestration, agent execution, or computing-resource
  provisioning.
- Discord, Slack, other external channels, integrations, or routines.
- Relay service implementation.
- Implementing an external identity-provider login in this cycle.
- Replacing the installation encryption key, bulk re-encrypting stored data, or
  introducing a key-replacement session-reset command in this cycle.
- Migrating existing Azents accounts, data, or deployment.
- Adding general server-management features beyond those explicitly listed here.
- Changing unrelated operational services or deploying to environments outside
  the agreed local-cluster verification scope.

## Requirements

### REQ-1. Initialize the first server administrator

An operator can initialize a server that has no users using an installation token.

**Acceptance criteria**

- Initialization creates the first user with server-administrator authority.
- Installation-token initialization is not ordinary public signup.
- A server that already has users does not reopen first-user initialization.
- No separate kind of administrator account is required: the administrator is a
  user with an additional server-wide role.

### REQ-2. Sign up through an email-bound signup link

Users join through signup links rather than unrestricted email/password signup.

**Acceptance criteria**

- A server administrator can issue a signup link for a specified email address
  and revoke an outstanding signup link through the Admin UI.
- The link recipient can set their password and complete signup.
- The default signup-link validity is 7 days from issuance. Previewing or
  attempting to use the link does not extend it; successful signup consumes it.
  After expiry, onboarding requires a newly issued link.
- Signup remains possible when email delivery is not configured: an administrator
  can deliver the issued link out of band.
- Entering an email and password alone is not sufficient to bypass the signup-link
  requirement when email delivery is not configured.
- An unusable or revoked link does not create an account and produces an
  understandable failure state.
- Configured email delivery can deliver signup links, following the existing
  Azents signup behavior; email delivery is not universally mandatory.

### REQ-3. Email-based login with optional email functionality

Email-based login is the default. Users can authenticate with email and password
without depending on an email-delivery service.

**Acceptance criteria**

- A registered user with valid email/password credentials can sign in.
- Invalid credentials do not grant access.
- Initial administrator creation, signup, and password reset use one password
  policy: 8 through 64 printable ASCII characters and at least three of four character classes
  (uppercase letters, lowercase letters, digits, and special characters).
- Passwords remain case-sensitive. Do not silently trim whitespace, truncate an
  overlong password, or verify only a prefix of the submitted value.
- Signup, login, recovery lookup, and invitation-recipient matching use one
  email-identity rule: ignore outer whitespace and letter case across the whole
  address. Case-only variants cannot create separate accounts.
- Preserve the trimmed address for display and delivery. Do not remove dots or
  plus tags, infer provider-specific aliases, or merge different addresses.
- Login continuity follows the selected reference behavior: the default validity
  window is 180 days, eligible renewal extends it, and there is no absolute
  lifetime cap by default. An operator can configure the validity window and an
  optional absolute cap.
- A session that has expired, been logged out, or been revoked by password reset
  cannot be renewed into an authenticated session; the user must sign in again.
- When a renewal response is lost or concurrent renewal leaves a client with
  the immediately previous token, that client can obtain the same latest token
  during the accepted overlap without logging in again, provided the session
  remains valid. This does not re-execute an application action.
- A deployment without email delivery remains usable for signup-link onboarding,
  password login, and administrator-assisted password recovery.
- Email-code login is available when email delivery is configured, following the
  referenced Azents behavior; it is unavailable when delivery is not configured.
- Email login codes expire 10 minutes from issuance by default. Attempts do not
  extend that window; an expired code requires a newly requested code.
- The design permits future external-provider login without treating a provider's
  identity as the only possible user-account identity.
- External-provider application setup is not required for this cycle's basic
  deployment and login.

### REQ-4. Provide an implicit personal space

Each user receives a personal space automatically. Ordinary users do not need to
understand or create a team/workspace before using their account.

**Acceptance criteria**

- Signup does not introduce an organization-creation or workspace-selection step.
- Personal-space team-management controls are not presented as ordinary setup.
- A personal space cannot invite other members, be left by its owner, or lose its
  only owner through member management.
- Leaving a collaborative team does not remove the user's personal space.
- Unrelated server users are not treated as members of one shared public team.

### REQ-5. Create collaborative teams

Every registered user may create a collaborative team without server-administrator
approval. The creator becomes its owner.

**Acceptance criteria**

- Team creation is available to ordinary registered users.
- User-facing terminology is Team, not Workspace.
- Personal and collaborative ownership scopes remain distinguishable.
- Creating a team or owning a space does not implicitly grant access to server or
  Kubernetes computing resources. Resource execution is not implemented here.

### REQ-6. Enforce owner, manager, and member roles

Collaborative teams have owner (OWNER), manager (MANAGER), and ordinary-member
(MEMBER) roles.

**Acceptance criteria**

- Owners and managers can manage team settings and invite/remove ordinary
  members.
- Only the owner can appoint or demote managers, transfer ownership, and
  delete the team.
- Managers cannot remove other managers or the owner through the
  ordinary-member removal capability.
- Ordinary members cannot perform owner/manager management actions.
- Team ownership or management does not confer server-administrator access.
- Restrictions apply to operations themselves, not merely to visible UI controls.

### REQ-7. Invite registered and unregistered users

The signup/invitation relationship follows the existing Azents behavior, subject
to the role limits in REQ-6.

**Acceptance criteria**

- An authorized inviter can target the email of an existing or unregistered user.
- With email delivery configured, inviting an unregistered email produces an
  email-bound signup link delivered with the invitation.
- With email delivery disabled, the invitation remains pending but this invitation
  flow does not automatically issue a signup link; the recipient must first use a
  separately issued signup link.
- Successful signup alone does not automatically accept the invitation.
- After signing in, the intended recipient can see and accept or decline their
  pending invitation. Acceptance creates the membership.
- A different user cannot accept or decline an invitation they do not own.
- A processed invitation cannot be accepted or declined again as if pending.
- This does not introduce an independent join-request/approval feature.

### REQ-8. Leave or delete a collaborative team

Members and managers may leave a collaborative team. An owner must transfer
ownership to another member before leaving, or use the owner-only team deletion
operation instead.

**Acceptance criteria**

- An ordinary member or manager can leave without owner approval.
- The owner cannot leave while retaining sole ownership of that team.
- Ownership transfer makes the recipient the owner before the former owner can
  leave.
- After transfer, the former owner remains a team member with the manager role.
  Transfer does not automatically make that person leave the team or change
  server-administrator assignments.
- Only the owner can delete a collaborative team.
- Leaving or deleting a collaborative team does not delete a personal space or
  the user's server account.

### REQ-9. Dedicated, administrator-only management experience

Server administration is exposed through a separate Admin UI and a distinct Admin
client protocol, not as team management in the general web UI.

**Acceptance criteria**

- The same user identity can access the general UI and, when granted the
  server-administrator role, the Admin UI.
- An authenticated user without that role cannot obtain Admin access.
- A client authenticated to a server can use its session for both general and
  Admin operations when the user has the required role. Accessing Admin operations
  does not itself require a second login or a purpose-specific login token.
- Browser cookie boundaries may differ between web domains. This does not require
  separate general/Admin token types or automatic cross-domain web sign-in.
- Server-administrator authority is enforced by the server as well as the UI.
- The Admin experience matches the main product's look and feel: common controls,
  typography, colors, spacing and interaction states remain consistent while
  administrator-specific navigation and operations remain distinguishable.
- Signup-link administration and password-reset-link issuance are available in the
  administrator experience.
- A team manager cannot issue server account-recovery links merely because
  of their team role.
- Specific protocol transport, session format, listeners, processes, and binaries
  are not selected by this requirement.

### REQ-10. Recover a password without mandatory email delivery

A server administrator can issue a password-reset link, and the user chooses a new
password through that link, following the existing Azents recovery experience.

**Acceptance criteria**

- The link can be delivered out of band when email delivery is not configured.
- An authorized user can set a new password using a valid reset link.
- Invalid or unusable reset links do not change credentials.
- Password-reset links expire 24 hours from issuance by default. Previewing or
  attempting to redeem a link does not extend it; an expired link requires a
  new link issued by a server administrator.
- Successful reset consumes the recovery link, updates the password, and revokes
  all existing authentication sessions for that user as one atomic outcome.
- Reset completion does not automatically issue a new login session; the user
  signs in again with the new password.
- Revoked sessions no longer authenticate at either general or Admin protocol
  surfaces. The user's account, teams, and administrator role are not deleted.
- The workflow does not reveal the existing password or ask an administrator to
  choose the user's replacement password.

### REQ-11. Deployable server and web clients

The initial cycle produces a runnable server and web-based user/administrator
experiences, not only mock screens.

**Acceptance criteria**

- An operator can configure a supported deployment and perform the initialization,
  onboarding, login, recovery, and team-management cycle end to end.
- Docker images and a Helm chart are provided for the server and web experiences.
- The chart supports direct Helm deployment and Argo CD application
  synchronization. Both execute schema preparation before applying the dependent
  application rollout, stop that rollout on preparation failure, and tolerate
  repeat preparation against an already prepared installation.
- After implementation, the Helm chart is installed on the requester's local
  Kubernetes cluster and the account/team cycle is verified on that deployment.
- The exact cluster, namespace, exposure, and data-resource scope are identified
  before cluster writes. This requirement does not authorize changes to unrelated
  services or deletion of existing data.
- The cloud/server deployment can use an external PostgreSQL database.
- Local development can run the account/team server with SQLite without starting
  a separate database server.
- Deployment and QA can select PostgreSQL and verify the same required
  account/team/authentication behavior; SQLite-only tests are not evidence of
  PostgreSQL correctness.
- Both database backends are within this cycle, independently of the deferred
  Mac application work.
- The general web UI server acts as the Chapchoo client; a browser does not directly
  use the common native/server client protocol.
- Admin access uses the distinct Admin client protocol.
- Externally accessed backend connections are encrypted. Local loopback and
  explicitly trusted internal deployment networks may carry plaintext backend
  traffic; deployments must prevent direct public access to those backend ports
  and document that the internal hop is not encrypted by the public edge.
- Deployment instructions identify required settings and distinguish optional
  email delivery from mandatory prerequisites.
- The installation's encryption key remains fixed for this cycle. Restart,
  update and data restoration use the same securely retained key. A different
  key must not be accepted against the existing installation; failure does not
  erase data or reset passwords/sessions. Key replacement is unsupported.
- Chart packaging checks and an actual local-cluster deployment check are
  reported separately; successful packaging alone is not deployment verification.

### REQ-12. Recover server-administrator access through an operator CLI

If existing administrators cannot sign in, an authorized server operator can use
a CLI to grant server-administrator authority to an existing user, following the
existing Azents operator recovery approach.

**Acceptance criteria**

- An operator can identify an existing user and grant that user server-wide
  administrator authority without an existing Admin UI login.
- The promoted user can sign in using their own credentials and use the Admin UI,
  including the existing administrator-assisted password recovery workflow.
- The operation does not create a new user, reveal or replace a user's password,
  or reopen first-user initialization.
- Ordinary remote users and team managers do not obtain this capability
  merely through their application or team permissions.
- Operator recovery instructions are supplied and the recovery path is verified.
- The CLI's exact configuration, credential, and connectivity mechanisms remain
  for architecture and security design.

### REQ-13. Install one package with one central control CLI

An operator can install a unified distribution package and use one PATH-resolvable
command as the central entrypoint for backend roles, built web applications, and
operator commands.

**Acceptance criteria**

- The installed command can select backend and web components for separate or
  combined execution; exact command names are not prescribed here.
- The package supplies the built web artifacts and required web runtime so an
  operator does not need to install Node.js, pnpm, or Turbo separately or build
  the web applications on the deployment host.
- A user can access the CLI-launched web applications over HTTP on local
  loopback without first provisioning a certificate or HTTPS proxy. Externally
  exposed web deployment uses HTTPS; this local allowance does not authorize
  insecure HTTP exposure on non-loopback interfaces.
- A distribution may contain multiple files and launch multiple processes. A
  single native executable containing every runtime is not required.
- Service execution stays in the foreground for this cycle. The CLI manages
  components it starts and shuts them down on termination; persistent service
  hosting/restart is delegated to the container or OS service environment.
- An application-owned background daemon and detached-instance control features
  are not required in this cycle.
- When a selected long-running component fails to start or exits unexpectedly
  during combined execution, the CLI shuts down the other components owned by
  that invocation and exits with an error. Separate invocations are unaffected.
- Independently deployable general/Admin web applications and Docker/Helm
  deployment remain supported; unified packaging does not replace them.
- Development may continue to use pnpm/Turborepo.
- PostgreSQL access is an explicit prerequisite when that backend is selected;
  routine SQLite local development does not require an external database server.
  Application configuration remains explicit.
- This cycle provides the unified installable package for Linux x86-64 and
  ARM64. macOS package support is the next step; Windows packages are outside
  this cycle. This distribution scope does not prohibit local source development
  on other operating systems.
- Future Mac desktop application packaging remains outside this cycle,
  independently of the deferred macOS CLI distribution.

## Fixed Constraints

- The backend implementation language for this cycle is Rust. Framework and
  library choices are recorded in the snapshot ADR, not prescribed here.
- The general frontend retains the already requested Next.js/React/TypeScript
  foundation. The Admin frontend shares product UI components with the general
  frontend instead of using Refine as a separate administration framework.
- Architecture and protocol design precede Rust library selection and coding-pattern
  standardization.
- Evaluate the complete Azents convention baseline when establishing the Rust
  foundation, not only selected examples. Preserve the purpose and enforceable
  quality of relevant architecture, dependency, typing, concurrency, failure,
  persistence, configuration, observability, testing, documentation, and delivery
  rules through language-appropriate equivalents. Identify justified
  inapplicability rather than mechanically copying another language's syntax.
- Long-term adoption and community sustainability are major stack-selection
  criteria, alongside correctness and fit. Assess usage, independent adoption,
  maintenance continuity, ecosystem integration, and available support material
  against first-party evidence and established Rust project practices.
- Dependency injection, layering, strong types, async execution, linting, and
  formatting are examples of that complete review, not its scope limit.
- Authentication authority and administrative authority must be enforced at the
  server boundary, regardless of a client's UI.
- Apply the one-way storage principle in this cycle: when the user/client holds
  the original value and the server needs only to verify a submitted value,
  retain a non-recoverable verifier rather than recoverable plaintext. When
  the server must later use or return the original value, use protected
  recoverable storage. A value may need both representations only when its
  authorized operations require both verification and later retrieval.
- Requirements in this document describe outcomes. No transport, database schema,
  ORM, session format, or process/binary split is selected by these Requirements.

## Future Context, Not Current Acceptance Obligations

Architecture should be evaluated against the following agreed future direction
without implementing these features in the first cycle:

- A future common key-change/recovery system distinguishes non-recoverable
  verifiers from reversible ciphertext. When a one-way verifier cannot be
  reconstructed, abandon that verifier and require user/client password reset,
  value re-entry or re-registration as appropriate. Values that must be
  recovered are candidates for a common decrypt-and-re-encrypt mechanism.
  Hashes independent of the changed key are not automatically invalidated.
  Key-compromise response must also address exposed credentials; re-encryption
  alone does not invalidate a plaintext credential already obtained by an
  attacker. These future capabilities do not authorize key replacement now.
- A packaged Mac app can run a local server, act as UI/harness-provider client,
  and use Standalone mode by connecting to itself.
- Modes can coexist, and an app can connect to multiple servers with server
  switching.
- Standalone use has a fixed trusted local user/personal space without a login
  ceremony. A future Mac app in server mode can connect to its own server as an
  Admin client without a login ceremony; this is not remote anonymous Admin access.
- Native UI clients and web UI servers use the common client contract. Mobile UI
  clients can be added later.
- A future Mac client signs in once per server and uses that session for general
  and permitted Admin operations rather than prompting for a second Admin login.
- An agent is bound to the chosen execution environment and harness.
- Docker/Kubernetes harness orchestration and local-provider execution are future
  scenarios. Relay access is explicitly excluded from the MVP.

## Open Scope Points

- General self-service profile editing, account deletion, and broad server-wide
  account/team inspection or deletion have not been included merely because an
  Admin UI exists.

Detailed error contracts, concurrency controls, and delivery failure handling
require subsequent design grounded in verified reference behavior and accepted
Chapchoo differences. Authentication-attempt behavior is to be checked against
the reference, not treated as an invitation to add new restrictions. Default
signup/reset/code lifetimes and password-reset session revocation are confirmed
above; other proof policies are not silently selected.

## Confirmation

The requester confirmed this consolidated REQ-1 through REQ-12 document on
2026-10-29 KST before architecture and protocol decisions began.
On the same date, the requester additionally confirmed REQ-13: a unified installed
package with one PATH-registered central CLI, allowing internal files/runtimes
and preserving independent web deployment.
The requester subsequently clarified on the same date that general/Admin tokens
are unified while web cookie boundaries may vary by domain, and that a future
Mac client must not require two logins to the same server for those roles.
The requester subsequently confirmed SQLite for local development and selectable
PostgreSQL for deployment/QA in this cycle, while deferring database-library choice.
The requester also confirmed the Azents password-recovery lifecycle: successful
reset revokes all existing sessions and requires a new login.
The requester confirmed foreground central-CLI execution for this cycle, deferring
application-owned background execution/management.
The requester confirmed fail-as-one combined execution: startup/unexpected
component failure terminates that invocation's components and returns an error.
The requester clarified the team role name as manager (MANAGER) and confirmed
that the former owner remains a manager after collaborative ownership transfer.
The requester selected the reference login-continuity policy: a default 180-day
renewable validity window with no absolute lifetime cap by default.
The requester confirmed latest-token recovery during the previous-token overlap,
supported by protected server-side storage, without replaying business actions.
The requester confirmed default validity from issuance of 7 days for signup
links, 24 hours for password-reset links, and 10 minutes for email login codes,
without extending those windows on preview or attempts.
The requester confirmed whole-address case-insensitive identity matching after
outer-whitespace trimming, preserving display/delivery spelling and distinct
dot/plus-tag addresses.
The requester initially confirmed an 8-to-128-character password policy, then
revised it before implementation to 8-to-64 printable ASCII characters while
retaining at least three of the four character classes.
The requester directed remaining behavior to follow the Azents reference without
adding absent features or policies, while preserving explicitly approved
Chapchoo differences. Reference-derived details do not require another
individual policy interview. Unresolved product behavior, permissions, and
user journeys still require requester decisions; reference alignment is not
blanket delegation of product scope.
After evaluating local PostgreSQL and compatible alternatives, the requester
reconfirmed local SQLite and selectable PostgreSQL for deployment/distributed
execution. Local PostgreSQL bundling and PGlite are not current deliverables.
The requester subsequently prohibited installation-key replacement in this cycle
and deferred common key-change/re-encryption support. The requester explicitly
confirmed that the one-way storage principle applies in the current cycle,
while future unrecoverable-verifier reset/re-entry and reversible-data
re-encryption remain improvement directions.
The requester accepted a pre-deployment schema-preparation Job and explicitly
required the same chart workflow to support Argo CD as well as direct Helm use.
Confirmation establishes the outcomes and fixed constraints above, not approval
of an architecture, protocol selection, final Design, or implementation.
