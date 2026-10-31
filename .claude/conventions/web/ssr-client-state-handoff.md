---
title: "For interactive or live primary content, server-render the initial data and hydrate the same state before client-owned updates."
---

# Server Rendering, Client Continuation

- **Interactive/live content** changes through pagination, filters, real-time updates, or similar client-managed behavior.
- Fetch its initial data on the server and use it to render the initial content in server HTML.
- Initialize the client state/query with the same data and selection; hydration continues that rendered state.
- Client ownership governs subsequent interactions and updates, not deferral of the first content render.
- Client Components can participate in server rendering. Do not gate primary content behind mount, a browser-only effect, or disabled SSR merely because it has client state.

**Bad:** Deliver `initialData` but render a spinner until an effect copies it into state.

**Good:** Server-render the initial list, hydrate that list, then let the client manage later changes.
