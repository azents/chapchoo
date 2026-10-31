---
title: "Use the URL as the source of truth for user-selected, reload-restorable page state; exclude transient edits and automatic/server-driven changes."
---

# Restorable Page State

- Apply to General and Admin. Store user-selected page number, applied search/filter conditions, selected detail (including detail modals), and in-page tabs in path/search parameters.
- Derive initial server selection and client selection from the URL so reload restores the same view; avoid a competing local source of truth.
- Exclude draft form/search input, in-progress list edits, server mutations, automatic refresh, and infinite-scroll cursors. Applied search conditions belong in the URL; unsubmitted input does not.
- URL ownership does not require a full-document reload for every interaction, and does not encode server records or transient UI state.

**Bad:** Keep the selected detail modal ID only in `useState`, losing it on reload.

**Good:** Read the selected detail from the URL on direct entry, reload, and client navigation.
