---
title: "Test the subject's own behavior and contracts, not dependency internals, library defaults, or token resolution."
---

# Test owned contracts

- Test behavior owned by the named subject. Before adding an assertion, identify which implementation owns its failure.
- Do not re-test a dependency's rendering, algorithms, defaults, serialization, or token-to-pixel conversion in its consumer, even when that consumer is shared UI.
- Test our adapters' mapping, admission, error handling, and composition only where our code adds that contract. Integration/E2E checks verify an owned boundary or product journey, not a duplicate dependency matrix.
- Remove redundant tests or move meaningful coverage to its owner. Do not replace deleted assertions with tests that merely repeat declarations.

## Bad

An Admin test asserts that a shared table's `lg` spacing resolves to `16px`.

## Good

An Admin test verifies its revoke action selects the intended grant; a transport adapter test verifies its own error mapping.
