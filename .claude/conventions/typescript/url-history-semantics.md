---
title: "Push URL changes that select different primary content; replace minor URL-state changes that do not warrant a new history entry."
---

# Meaningful Navigation History

- Use **push** when a user selects different primary content: pagination, a different detail view/modal target, in-page tab, or applied search/filter conditions.
- Use **replace** for other minor URL-state changes, updating the current history entry.
- Classify by the content transition, not the widget: a detail-selection modal is primary-content navigation.
- Back/Forward must restore the prior URL-selected view. Transient state excluded by [restorable page state](url-page-state.md) needs neither push nor replace.

**Bad:** Replace each selected detail, preventing Back from returning to the previous selection.

**Good:** Push detail selection; replace an incidental URL presentation adjustment.
