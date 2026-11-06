---
title: "Codex Authentication Phase 1 Execution Plan"
created: 2026-11-06
tags: [harness, implementation, security]
---

# Phase Execution Plan

- Phase: 1/3 protected credential schema and primitives.
- Branch/base: `feat/codex-auth-backend` -> main294f9ff1bfc5baf5017bd38f4c9bdae0df4c6161.
- PR boundary: new protected-secret primitives and empty preserving database structures; no user-reachable registration before phase2.
- Inputs: confirmed [REQ](../requirements/harness-261106-codex-authentication.md), accepted [ADR D1/D2](../adr/harness-261106-codex-authentication.md), approved [Design rev1](../design/harness-261106-codex-authentication.md).
- Approved mechanisms: M1/M2/M4/M6/M9 foundations only; authority REQ1/2/3/4/9/10/11 and ADR-D1.
- Design delta: None.
- Deliverables: bounded zeroizing secret; authenticated envelope binding purpose/owner/object/kind; keyed payload fingerprint; compact storage identities (DTO kind issuance belongs to phase2); catalogue/attempt/operation-result/Agent selection tables with constraints; preserving migration and down protection on both engines.
- Non-goals: network provider calls, application service/RPC/UI activation, new Agent creation behavior, Computer key exchange, model calls, secret version history, root rotation.
- Removal obligations: None in existing production auth/session code. Preserve the original fixed bearer envelope and root binding; no prior external catalogue exists.
- Absence verification: existing bearer tests remain unchanged/passing; no modifications to Control/Connector/Provider/proto or runtime packaging.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Protection | /root | rust/crates/crypto/src/* | D1 | scoped secret capability and tests | crypto unit tests, Clippy |
| Schema | /root | rust/crates/persistence/migrations/{sqlite,postgres}/00000000000010_*, preparation.rs, entity_id.rs | protection contract | constrained empty storage and preparation | both-engine migration/preservation tests |
| Documentation | /root | snapshot docs, plans, owning credential/storage Spec | implementation | accurate current primitive behavior | docs checks/local links |

- Interfaces: no root change; new envelope accessors expose ciphertext/nonce only; context binds stable typed identity/scope/kind; preparation expects migration10; both engines retain existing IDs/receipts.
- Integration order: protection -> schema/identity -> preserving tests -> Specs -> whole stable diff review -> root corrections -> commit/PR.
- Independent review: `/root/harness-feature-reviewer`, root requests review of main..phase stable diff after root checks, read-only. Root owns all fixes.
- Final validation: pinned `scripts/check-rust.sh`, docs validator/unit tests, SQL constraint and populated migration tests on SQLite/PostgreSQL; all root-executed. Open PR and monitor exact-head CI/reviews.
- Scope drift: only approved foundations; no reachable incomplete registration/selection behavior, native delivery or crypto rotation.
- Context checkpoint: before this phase no feature code; at completion record SHA, actual checks, constraints, reviewer result and remaining phase2/3 work in implementation plan.
