---
title: "For snapshot-like primary content such as profiles or agent details, require resolved renderer data instead of artificial client loading states."
---

# Resolved-Data Renderers

- **Snapshot-like content** is a resolved record for this render, such as a user profile or agent details. It does not mean immutable data, SSG, build-time fetching, or a caching policy.
- Require that data in the renderer's typed props and render it directly.
- Handle initial data failure at the server/route boundary; do not add optional-data, mount guards, or loading/error branches solely to rediscover a satisfied server contract.
- Server navigation may still have a pending boundary; separate supplementary content and actual editing interactions may have their own states.

**Bad:** A required profile prop becomes optional so the renderer can return a spinner.

**Good:** `Profile({ profile }: { profile: ProfileData })` renders the resolved record.
