---
title: "Let one handling boundary own incident reporting and deliberately choose which error context may cross logs or public responses."
---

# Error disclosure ownership

- Preserve useful typed causes privately; dependency `Display`, `Debug`, request fields and source chains are not automatically safe.
- One boundary owns the incident; lower layers return evidence rather than duplicate the same failure report.
- Admit only reviewed correlation/classification fields. Instrumentation defaults do not establish disclosure authority.

## Bad

```rust
tracing::error!(?request, error = %native_source, "Request failed");
```

## Good

```rust
tracing::error!(class = failure.safe_class(), "Request failed");
// The typed failure retains its private cause for authorized handling.
```

Reference: [tracing argument/error recording](https://docs.rs/tracing/latest/tracing/attr.instrument.html).
