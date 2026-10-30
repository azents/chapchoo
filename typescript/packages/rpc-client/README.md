# Generated accounts RPC client

The shared source is `proto/chapchoo/accounts.proto`. This Node-only package
exports the generated grpc-js clients, service definitions, and message codecs
from `@chapchoo/rpc-client/accounts`. It does not store credentials or choose a
transport policy. Callers supply grpc-js channel credentials and authorization
metadata per call. Browser bundles must not import it.

Run `pnpm generate` to regenerate, or `pnpm generate:check` to compare all generated
files without modifying the checkout. Generation uses the Rust workspace's
locked vendored protoc and the package's locked ts-proto executable.

Generated output is checked into `src/generated` and compiled under strict
TypeScript settings. Formatting and handwritten lint rules apply to the
generation/test harness rather than rewriting third-party generator output.
All output uses ESM `.js` imports. Protobuf int64 values are `bigint`; timestamps
remain `{ seconds: bigint, nanos: number }` rather than lossy JavaScript Dates.
Optional fields retain presence independently of their zero value.

Generated messages are transport values, not safe diagnostic payloads. Never log
them, bearer metadata, errors containing call payloads, or explicit wire buffers.
