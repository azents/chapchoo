# Executable E2E app

This is a private Playwright application, not a shared package. Keep only
representative assembled-product journeys and genuine browser/process/network
boundaries here. Read the repository's `e2e-core-flows-only` convention.

Three serial account/team journeys cover signup/admin setup, team ownership and
reset/session/Admin isolation. Web-core owns its real-browser logout-response
integration; browser cookie arrival-order semantics are not our test subject.
Exact copy and app composition belong to the owning app; shared layout/custom
CSS belong to common UI's executable component checks, not another full product
journey. Preserve core auth, revocation and isolation coverage.

Reusable fixture/process/config/prerequisite support comes from
`@chapchoo/testenv`. This app owns the Playwright wrapper, scenarios, reporter
and runner-private artifact output. It must not import product app source.

Never print secrets, grant/proof URLs or raw child output. Preserve private
artifact cleanup, no automatic retry/replay and exact owned resource shutdown.
Do not use direct SQL writes, shared infrastructure or preexisting browser state.
See README.md for local execution and lower-layer coverage mapping.
