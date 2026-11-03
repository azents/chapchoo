---
title: "Runtime web configuration and failures"
---

# Runtime web configuration and failures

Web deployment configuration is resolved at runtime; decoding and reporting preserve safe, truthful failure boundaries.

## Configuration and failure boundaries

Typed app configuration reads the public URL, bind host and backend URL in its
configuration boundary. Each app requires an explicit backend URL; it does not
fall back to a fixed port, local backend or the other application's target. Local HTTP requires localhost/parsed loopback public
host and loopback listener; supported local start/dev commands explicitly bind
127.0.0.1. Remote public origins require HTTPS. Host/Forwarded headers do not
replace configured origin trust. Backend HTTPS uses grpc-js's CA/hostname
verification without plaintext fallback; internal plaintext requires the
operator's trusted network boundary.

Next builds have no deployment public path or backend target. Public metadata
and redirect-path consumers validate only public listener/origin/path settings;
full runtime listener/backend consumers still reject a missing explicit target.
Build metadata never requires a fake deployment endpoint to compile an artifact.
The documented production-server
configuration phase reads the runtime mount and asset prefix. A public custom
server accepts only that mount, strips it before Next dispatch, and uses the
documented webpack configuration extension with automatic chunk public paths.
Shared Pretendard/Geist Mono files and mascot art are staged from common UI
assets and served with runtime-prefixed URLs. Browser navigation and server redirects use the
safe runtime path rather than the framework's compiled basePath. Shared assets
are included through tracing/transpilation configuration.
GET `/health` under the runtime prefix validates runtime web configuration and
returns no-store process health. It does not establish backend/schema readiness
or an authenticated product journey. [Linux delivery](account-delivery.md)
describes executable-relative standalone execution and its acceptance boundary.

Canonical generated reason metadata is validated before conversion to typed
localized failures. Unknown/missing reasons retain generic transport failure.
Required decoded message presence, timestamp bounds and enum membership are
validated at the Node boundary. Public errors omit original message/stack/cause.
Node-origin infrastructure has one safe reporting boundary; known backend
failures are not redundantly logged. SSR uses a safe typed error retaining the
original source in a private diagnostic field, preventing default Next/Node
inspection from enumerating native credentials, SQL or proof material.

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Linux artifacts and image assembly](account-delivery.md) — Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.
- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.

## References

- [`typescript/apps/web/src/config`](../../../../typescript/apps/web/src/config)
- [`typescript/apps/admin-web/src/config`](../../../../typescript/apps/admin-web/src/config)
- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
