---
title: "Accounts Phase 3: Generated RPC, Services and Foreground CLI"
created: 2026-10-30
tags: [accounts, implementation, rust, protocol]
---

# Phase Execution Plan

- Phase: 3/5, generated RPC boundary, application composition and operator/foreground CLI.
- Branch/base: `feat/accounts-rpc-cli` → `feat/accounts-persistence` at `563e9a9`.
- PR boundary: runnable backend roles and operator commands over the completed
  storage, generated Rust/Node contracts and service/protocol integration tests.
- Inputs: PR #2 foundation, PR #3 completed both-engine storage, approved Design
  revision 36 with exact M1-M34 and M36-M43 authority set.
- Approved Design mechanisms: M1, M3, M5, M6, M8, M13-M15, M18-M22, M25,
  M28, M30-M32, M43; phase 1/2 mechanisms remain dependencies.
- Authority references: REQ-1 through REQ-13 as applicable; ADR-D2-D12, D14-D17,
  D20-D23, D27-D29, D33-D37. Current credential/account-storage Specs are preserved.
- Design delta: None.
- Deliverables: Protobuf messages and Auth/General/Admin services; Rust and Node
  generated clients; typed ingress/error mapping; explicit composition;
  password-work admission; optional official SES adapter; foreground role
  selection/readiness/drain; schema preparation and existing-user admin grant CLI.
- Non-goals: product web screens, bundled web packaging, images/Helm rollout,
  certificate issuance/native TLS, key replacement, streams/tasks/replay systems,
  account deletion or broad administration features.
- Interfaces: browser clients do not call gRPC directly. Common auth is registered
  on both listeners; business/admin groups stay separate. Every protected RPC
  parses a presented opaque bearer and uses repository current authority. Operator
  recovery is not registered as a remote RPC. Protobuf is the shared transport
  source, without an invented API-version negotiation scheme.
- Removal obligations: occupy only the designed reserved proto/backend paths.
  No older product protocol or user data exists. Web scaffold/theme replacement
  remains phase 4, and no temporary compatibility route is introduced.
- Absence verification: dependency graph forbids protocol-to-DB implementation
  access; generated adapters replace raw transport serialization; no web DB/root
  secret access, uncertain mutation replay, key-change command or global process kill.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Contract/generation | /root/accounts-protocol-generation; root owns proto | rust protocol crate, typescript/packages RPC client, generation scripts | approved shapes | generated compatible Rust/Node contract | generation diff check, real Node/Rust calls |
| Services | /root/accounts-team-storage | rust/crates/chapchoo-service | completed repositories, crypto | typed orchestration and bounded password work | mail/prerequisite/authorization/failure tests |
| Protocol | /root/accounts-rpc-adapter | rust/crates/chapchoo-rpc | services/contracts | typed ingress and separate service groups | real two-listener RPC, no Admin bypass |
| CLI/mail | /root | central executable and official SES adapter | services/contracts | explicit composition and foreground/operator commands | preparation/readiness/termination and SDK boundary |
| Quality/Specs | /root | Cargo/workflow/scripts/docs | integrated code | repeatable validation and reachable behavior Specs | root-owned integrated checks and exact-head CI |

- Integration order: define contract → generate → services → adapters → executable
  composition → protocol/CLI failure/lifecycle tests → Specs/CI → review → phase PR.
- Independent review: `/root/accounts-reviewer`, same read-only reviewer. Root
  requests review of the frozen integrated diff only after integrated validation.
- Final validation: Rust quality and both-engine tests; generated-source consistency;
  TypeScript client checks; actual Tonic/Node calls; per-call credential isolation;
  status/reason mapping; CLI preparation and role selection; finite cleanup;
  secret-safe failure reporting; documentation and full pre-commit.
- Scope-drift check: expose only the listed approved operations and modes; keep
  optional email and current-session behavior aligned with reference evidence.
- Context checkpoint: phase 2 is open PR #3 at `563e9a9`, reviewed independently
  with 24 unit/integration plus 2 compile-fail tests and all local gates passed.
  Exact-head phase-2 CI passed. Bootstrap PR #1 was requester-merged; root restored
  PR #2 onto main without changing its head, and the new CI passed.
  No implementation merge or real-cluster action is authorized by this plan.

## Integrated Checkpoint

- All 34 RPC methods are implemented through typed application services.
  Rust/Node contracts are generated reproducibly; production protocol code has
  no DB implementation dependency. General/Admin keep common Auth and separate
  business service registration.
- The central executable prepares schema explicitly, recovers an existing
  administrator assignment locally, runs either/both backend roles, distinguishes
  live/ready health and owns admission/drain. The SES adapter uses the official
  SDK and makes one submission attempt after commit.
- Root validation: 42 Rust unit/integration tests plus 3 compile-fail tests;
  rustfmt/Clippy/dependency boundaries; Node codec tests (5); actual Node/Rust
  calls on SQLite and PostgreSQL (71 checks each); full pre-commit and 18
  documentation-catalog tests passed. A Python formatting hook changed one
  harness file; the full hook suite then passed after formatting.
- Root found and corrected acceptance timestamps captured before expensive
  bcrypt work. The four password transitions now sample production acceptance
  time after CPU completion; fixed-clock expiry regressions cover the boundary.
  Typed repository failures and kind-scoped revocation preserve precise RPC
  reasons rather than inferring them from unrelated conflicts.
- Scope/removal check: Design delta remains None. There is no key-change command,
  remote admin-recovery method, task/replay system, raw-code fallback, product web
  screen or deployment mutation. Reserved backend/proto paths are now occupied;
  scaffold replacement and web-child ownership remain phases 4/5.
- Full independent read-only review by `/root/accounts-reviewer` found no
  reportable issues in frozen tree `43f3e3221cb248a4a2241308bbbacfa784f856aa`.
  The same reviewer independently reran all 42 Rust tests/3 compile-fail tests
  and the generated Node/actual both-engine RPC gate successfully. No correction
  or targeted re-review was required. Commit, stacked PR and exact-head CI follow.
