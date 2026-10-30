---
title: "Build all distribution artifacts for open-source reuse with established community standards; keep private deployment conventions and environment values outside the artifacts."
---

# Community-standard Distribution Artifacts

- Use established open-source distribution formats, tools and conventions for images, packages, Helm charts, configuration and publication.
- Make artifacts usable by community operators without Home/internal infrastructure or a project-private deployment protocol.
- Keep environment-specific domains, Secret references and orchestration wiring in deployment configuration, not public artifact variants.
- Prefer supported public interfaces and documented extension points over framework internals, generated-output patching or bespoke substitutes for standard tooling. Investigate and explain a standards conflict before introducing an exception.

## Bad

Publish an internal-only image or require a private Home deployment service to start it.

## Good

Publish an environment-independent OCI image and a reusable Helm chart; configure Home through its separate GitOps values and routes.
