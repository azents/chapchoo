---
title: "Open links that change the sidebar menu context in a new tab/window; only the workspace switcher changes context in the current window."
---

# Sidebar Context Navigation

- Links into another sidebar menu context open a new tab/window (`target="_blank"`, `rel="noopener"`) to preserve the current working context.
- The workspace switcher is the explicit exception: selecting a Team, personal workspace, Profile or Administration changes context in the current window.
- Pages within the same sidebar context remain in the current window.

**Bad:** An account-menu Profile link replaces an active Agent workspace.

**Good:** That Profile link opens a new tab; choosing Profile in the workspace switcher replaces the current context.
