---
title: "chapchoo-web auth cookies are `az-token`, `az-refresh`, `az-token-expires-at`; use these exact cookie names in tests and fixtures."
---

# chapchoo-web Cookie Names

chapchoo-web middleware reads Chapchoo-specific cookie names. Tests and fixtures must use the same names or authentication silently fails.

| Cookie | Purpose |
| --- | --- |
| `az-token` | Access token |
| `az-refresh` | Refresh token |
| `az-token-expires-at` | Access token expiry timestamp |

- ALWAYS seed these names in browser contexts and fixture state.
- NEVER invent alternate names for Chapchoo auth cookies.
