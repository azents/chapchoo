---
title: "Independent Design Reference Ownership"
created: 2026-10-30
tags: [design-reference, frontend, architecture]
document_role: primary
document_type: adr
snapshot_id: reference-261030
---

# Independent Design Reference Ownership

Authority: [reference-261030/REQ](../requirements/reference-261030-design-workspace.md).
These decisions record the requester's explicitly selected structure, not new
choices inferred from the existing visual implementation.

## D1. Separate reference authoring from production presentation

For REQ-1, REQ-2 and REQ-4, use an independent Storybook workspace package.
Reference imports common UI; apps import common UI. Common UI and apps never
import reference. Reference never imports app implementations.

An app Storybook remains a product-state review surface. Making it the general
design authority would couple cross-platform exploration to product code.
Making common UI own every experiment would falsely equate design review with
production-web reuse. The independent package avoids both outcomes.

Formal shared-web changes belong in common UI. Experiments without a shared
equivalent and web representations of non-web UI belong in reference. Promotion
requires review and a decision to reuse on the web: move the implementation to
common UI and update the reference import. Non-web representations stay local
even when their designs are approved.

## D2. Scaffold without endorsing the inherited visual language

For REQ-3, the initial workspace contains ownership/onboarding examples, not a
catalog of the inherited palette or screens. Existing common UI may be shown
with an explicit provisional notice and its own provider. It is not the default
visual provider for all new design work.

Storybook's web rendering can express cross-platform design intent; it does not
claim executable native behavior. No second canvas source, token generator,
native implementation, publishing service, or new UI library is introduced.

## D3. Distinguish general representation from web specialization

The requester's subsequent clarification refines D1's move-based promotion:
for REQ-2, common UI may own a web-specific implementation while reference keeps
a more general representation of the same design concept. These have distinct
purposes and are not interchangeable duplicate implementations.

Reuse the common implementation where the design role is equivalent. On promotion,
move shared-web implementation responsibility to common UI and remove same-role
copies; retain a reference-local general expression when web-specific constraints
would otherwise distort the design reference. Document that relationship.
