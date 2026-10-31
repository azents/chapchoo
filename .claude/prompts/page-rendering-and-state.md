# Page Rendering and State Prompt

Use this prompt when designing, implementing, or reviewing web pages. Apply it
to both General and Admin unless the task explicitly excludes one. Follow the
repository's scoped conventions; this prompt does not authorize unrelated
page migrations.

The language-independent rules live in `.claude/conventions/web/`, indexed by
`.claude/rules/web-conventions.md`, separately from language conventions.

## Terms

- **Primary content:** content whose required data is essential to the page's
  purpose. Without it, a page-level error or not-found outcome is preferable.
- **Supplementary content:** independent enrichment whose absence does not
  invalidate the page.
- **Snapshot-like content:** a resolved record for a particular render, such as
  a profile or agent details; not SSG, immutability, or a cache policy.
- **Interactive/live content:** content with client-managed updates, such as
  pagination or real-time status.
- **Restorable page state:** a user's selected view that should survive reload
  and be recoverable from the URL.

## Server Rendering and Client Ownership

1. Identify primary content and its required data. Fetch it server-side and
   render the actual content in server HTML. Passing serialized data alone
   does not satisfy SSR.
2. Resolve required authentication server-side before protected rendering.
   Keep credentials out of page props and browser-visible data.
3. Do not aggregate supplementary requests into the page's required-data
   boundary. Keep their loading and failures independent where they do not
   invalidate primary content.
4. Give snapshot-like renderers required, resolved data. Do not invent initial
   loading or optional-data branches inside them. Handle initial failure at
   the server/route boundary.
5. For interactive/live content, server-render the initial data, hydrate the
   same initial state, and let the client own subsequent changes. Client
   Components must not postpone primary content until mount or an effect.
6. Distinguish server-navigation pending states and genuine subsequent updates
   from an unnecessary client-only initial loading state.

## URL State and History

1. Use path/search parameters as the source of truth for user-selected,
   reload-restorable views: page number, applied searches/filters, selected
   detail or detail modal, and in-page tabs.
2. Use the same URL selection for server rendering and client continuation.
   Reload and Back/Forward must restore that view without competing local state.
3. Exclude draft input, in-progress edits, server mutations, automatic refresh,
   and infinite-scroll cursors. Distinguish unsubmitted search text from an
   applied search condition.
4. Push changes that select different primary content: page, detail target,
   tab, or applied search/filter conditions. Replace other minor URL changes.
   A modal can be primary-content navigation; widget type does not decide.
5. URL ownership does not require a full-document reload per interaction.

## Delivery Check

Explain the primary/supplementary boundary, renderer data contract, client
handoff, URL-owned selections, exceptions, and push/replace decisions.
Verify actual server-rendered content, hydration continuity, direct URL entry,
reload, and Back/Forward at the appropriate test layer. Do not treat the mere
presence of initial data or a post-mount screenshot as proof of SSR.
