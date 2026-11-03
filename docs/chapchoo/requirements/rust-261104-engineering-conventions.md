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
Use Rustfmt, compiler/Clippy lints and appropriate existing automated boundary
checks for mechanically decidable requirements. Convention bodies address
semantic decisions that tools cannot decide, with actionable applicability,
concise examples, justified exceptions and links to primary references.

### REQ-3. Reliable discoverability and scope

An agent editing Rust must discover the relevant Rust convention index through
the existing path-scoped loading mechanism. Non-Rust edits must not receive
unrelated Rust rules. Verify generated indexes, nested paths, new files and the
loading contract with executable tests; do not rely on declaration alone.

### REQ-4. Reviewable delivery and enforcement

Deliver convention content, its research/disposition evidence, scoped loading
and tooling configuration as a dedicated PR based on main. Tooling must actually
run in existing local hooks and CI and be compatible with the supported toolchain.
Resolve baseline tooling failures before presenting the setup PR as ready. Do not
merge, deploy, add a toolchain-version scheme or silently add compatibility paths.

### REQ-5. Whole-codebase adoption without behavioral drift

After the convention setup, inspect all Rust crates and the application, record
coverage and refactor owned contracts to comply in separately reviewable code
PRs. Include new Docker code in the coverage inventory while keeping its unmerged
feature stack distinct from the main-based convention PR. Preserve public wire,
API, database/migration, security/credential, lifecycle, cancellation and external
effect semantics unless a separately authorized requirement changes them.
Strengthen behavior-preserving tests where meaningful; do not duplicate dependency
internals or expand browser E2E for lower-layer combinations. Finish review,
mandatory hooks, exact-head CI and review-response handling for each delivered PR.

## Constraints and completion evidence

- All Docker PR CI is already green; no Docker merge/deployment is authorized.
- Inspect current files and primary sources; inherited conventions may be stale.
- New Rust language scope and its existing-loader integration are necessary parts
  of the explicitly requested Rust convention setup, not a new product capability.
- Preserve active worktrees and Rust compilation caches while work continues.
- Completion report must distinguish researched rules, automated enforcement,
  inspected/refactored coverage, remaining unmerged-stack work, actual tests/CI
  and unexecuted environments. Do not stop after documentation alone.
