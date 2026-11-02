# chapchoo

An independent, channel-first agent project being built alongside Azents.

**Current implementation:** General/Admin account and team applications plus the
personal Agent/Computer lifecycle foundation. Client/Admin/Control service groups,
independent Computer Provider/Computer Connector processes and the supported
Kubernetes adapter share the same domain authority. AI task execution/harness,
conversations, channels, integrations and routines are not implemented here yet.

## Development start

Prerequisites: Node.js 24, the pinned pnpm and Rust toolchains, Python 3.14 and uv.
Account/product use also requires an explicitly prepared SQLite or PostgreSQL
database and the installation's fixed key. Computer provisioning additionally
requires explicit operator-owned namespace/image/credential policy; no cluster is
created merely by starting the web app.

```sh
uvx pre-commit install
pnpm install --frozen-lockfile
# First prepare and start the backend using DEVELOPMENT.md.
CHAPCHOO_BACKEND_URL=http://127.0.0.1:50051 pnpm run dev --filter=@chapchoo/web
```

Open <http://localhost:3003>. Each web app needs its explicit matching backend
target; it does not infer a listener or another app's URL. Keep fixed keys,
first-admin proof and scoped Provider owner credentials out of tracked files,
arguments, logs and chat. Preserve the original installation key with its data.
[DEVELOPMENT.md](DEVELOPMENT.md) provides generation/setup instructions, runnable
backend/component commands and validation prerequisites.

[The Helm chart](infra/charts/chapchoo/README.md) packages reusable explicit
configuration, not private infrastructure. Source/synthetic acceptance and live
Kubernetes/storage/CNI/TLS gates are distinct; no merge or deployment is implied
by a passing local test.

## Provenance

Repository-local skills, coding conventions, Claude/Codex hooks, documentation
tooling, frontend configuration, and theme are adapted from
[azents/azents](https://github.com/azents/azents) at
`e71f614e2a8131b3db3fab7d0152c462b2447216`.
The original MIT notice is preserved in [LICENSES/azents-MIT.txt](LICENSES/azents-MIT.txt).
The skill-creator package retains its own license.

This is an independent implementation, not a copy of the Azents service or its
data. Inherited service-specific skill examples apply only when their target
actually exists in this checkout.
