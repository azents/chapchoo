---
title: "Rust Engineering Conventions and Behavior-Preserving Adoption Requirements"
created: 2026-11-04
tags: [rust, conventions, quality, refactoring]
document_role: primary
document_type: requirements
snapshot_id: rust-261104
---

# Rust Engineering Conventions and Behavior-Preserving Adoption

## Confirmed outcome

Establish researched, repository-appropriate Rust engineering conventions and
apply them consistently across the Rust codebase without changing product
behavior. The requester authorized this work on 2026-11-04 KST, after completion
of all Docker PR CI, with a separate convention setup PR followed by broad Rust
refactoring. This is maintenance work, not permission to redesign the product.

## Requirements

### REQ-1. Complete, evidence-backed coverage

Review every existing Python and TypeScript convention, recording whether its
underlying concern applies unchanged, requires a Rust-specific adaptation, is
already covered by global rules/tooling, or is not applicable. Explain exclusions
and adaptations rather than translating syntax mechanically. Review official
Rust and relevant runtime guidance and identify meaningful gaps in the existing
language conventions. Recommendations must fit the current repository and its
actual ownership, crate boundaries, error handling, asynchronous work and tests.

### REQ-2. Separate human judgment from automatic checks

Do not put any rule detectable by a linter in Rust convention documentation.
Mechanically decidable policies must be enforced automatically rather than
duplicated as prose obligations. Human guidance must support actionable semantic
judgment, explain its applicability and exceptions, and have primary-source
evidence appropriate to this repository.

### REQ-3. Reliable discoverability and scope

An agent editing an existing, deeply nested or new Rust file must reliably
discover the relevant guidance. Non-Rust edits must not receive unrelated Rust
rules. Discoverability and scope must be demonstrated by executable verification,
not declaration alone.

### REQ-4. Reviewable delivery and enforcement

The convention setup and code adoption must be independently reviewable, with
clear scope and verification evidence. Automatic policy enforcement must run in
ordinary local and CI validation and support the repository's accepted toolchain.
Readiness must reflect resolved baseline failures and successful validation of the
delivered revision, not a declaration or a different revision's result.

### REQ-5. Whole-codebase adoption without behavioral drift

Inspect all Rust crates and the application, record coverage and refactor owned
contracts to comply. Include new Docker code in the coverage inventory while
keeping its unmerged feature work distinct from convention setup. Preserve public wire,
API, database/migration, security/credential, lifecycle, cancellation and external
effect semantics unless a separately authorized requirement changes them.
Demonstrate behavior preservation through owned-contract tests without duplicating
dependency internals or using browser journeys for lower-layer combinations.
Every delivered revision must have completed review, required validation and
review-response handling.

## Constraints and completion evidence

- All Docker PR CI is already green; no Docker merge/deployment is authorized.
- As explicitly requested, deliver the independent convention setup before the
  subsequent code adoption. Merge and deployment are not authorized for either.
- Do not introduce a toolchain-version scheme or unrequested compatibility paths.
- Inspect current files and primary sources; inherited conventions may be stale.
- New Rust language scope and its existing-loader integration are necessary parts
  of the explicitly requested Rust convention setup, not a new product capability.
- Preserve active worktrees and Rust compilation caches while work continues.
- Completion report must distinguish researched rules, automated enforcement,
  inspected/refactored coverage, remaining unmerged-stack work, actual tests/CI
  and unexecuted environments. Do not stop after documentation alone.
