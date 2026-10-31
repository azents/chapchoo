---
title: "For General and Admin pages, fetch and server-render primary content; keep supplementary data outside the page's required-data boundary."
---

# Primary Content Is Server-Rendered

- Apply to both General and Admin unless explicitly excluded.
- **Primary content** defines the page's purpose: without its required data, a page-level error or not-found outcome is more useful than the page.
- **Supplementary content** adds value independently; its absence does not invalidate the page.
- Fetch primary data on the server and render the actual content into server HTML. Serialized props or a hydrated cache alone are insufficient.
- Resolve required authentication on the server as a prerequisite for protected content; never expose credentials as page data.
- Keep supplementary loading/failure independent; do not aggregate all page data merely to make every request server-side.

**Bad:** Return only a loading shell and fetch the primary record after mount.

**Good:** Load the required record server-side, handle its route-level failure, and server-render the feature with it; load an optional sidebar independently.
