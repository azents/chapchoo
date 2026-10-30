---
title: "Keep published images environment-independent: decide all deployment settings at runtime, never compile or bake URLs, paths, endpoints or credentials into build outputs."
---

# Runtime-only Deployment Configuration

Chapchoo publishes reusable open-source images; the same image digest must run in different deployments.

- Read all deployment settings at runtime: public URL/path, backend endpoints, authentication, storage and external-service configuration.
- Do not use Docker build arguments, build-time environment variables, bundler substitution or generated constants to bake deployment settings into server or browser artifacts. Public/non-secret settings follow the same rule.
- Snapshot is a development distribution channel, not an internal-specific build. Release the same commit by promoting the same artifact/digest; channel or deployment target must not change the application output. Share build/assembly paths instead of separate snapshot/release variants.
- Pass only explicitly safe runtime values to browser code; keep private settings server-side.
- Validate promotion by running the same built artifact under different configurations without rebuilding. Toolchain, dependencies and source code are build inputs, not deployment settings.

## Bad

```dockerfile
ARG PUBLIC_URL
RUN PUBLIC_URL="$PUBLIC_URL" pnpm build
```

## Good

```sh
docker run --env CHAPCHOO_GENERAL_WEB_URL=https://example.invalid/app image@sha256:...
```
