# Rust Workspace

Read `.claude/rules/rust-conventions.md` and the bodies relevant to the edit.
The generated index is scoped to `rust/**`; global rules remain applicable.

`crates/` owns reusable domain, persistence, service, protocol and adapter
contracts. `apps/chapchoo/` composes foreground processes and operator commands.
Inspect current manifests and consumers before changing these boundaries.

The authoritative quality entrypoint is `bash scripts/check-rust.sh` from the
repository root. See `DEVELOPMENT.md` for its database/test prerequisites and
`scripts/check-rpc.sh` for generated contract and actual Node/Rust RPC checks.
