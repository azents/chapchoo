# Chapchoo Account/Web Acceptance

Use the product CLI and public RPC/web paths to create fixture state. Account,
grant, session and membership writes must pass the same acceptance boundaries as
ordinary users. Tests own disposable database processes/directories and their
backend/web children; stop only those owned resources.

Current web cookie policy lives in
`typescript/packages/chapchoo-web-core/src/cookies.ts`. Local HTTP uses
`chapchoo-general-session` and `chapchoo-admin-session`. HTTPS uses
`__Host-Chapchoo-General`, plus `__Host-Chapchoo-Admin` at root or
`__Secure-Chapchoo-Admin` for a non-root Admin public path. These carry the single
opaque session credential. Test login through the product and inspect only
necessary attributes; keep values out of logs, assertions, traces and screenshots.

Keep the three approved account/team journeys representative. Validate input,
permission, cookie disposition and response-order combinations at focused lower
layers. Browser network ordering requires real browser evidence rather than
only fetch-header or cookie-model assertions.

See the current account RPC/storage Specs and the active phase execution plan
for implemented behavior and remaining delivery boundaries. Backend/schema
preparation is not initial administrator setup.
