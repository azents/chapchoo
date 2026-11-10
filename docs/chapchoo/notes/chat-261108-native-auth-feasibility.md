---
title: "Codex Native File Authentication Feasibility"
created: 2026-11-09
tags: [agent, codex, acp, authentication, security]
---

# Codex Native File Authentication Feasibility

This is source-backed feasibility evidence for
[chat-261108/REQ](../requirements/chat-261108-text-conversation.md) and
[chat-261108/ADR](../adr/chat-261108-text-conversation.md) D24 through D28.
It is not a successful native login, model call, runtime acceptance or Design
approval. No real credentials were read or used.

## Fixed Evaluation Profile

- ACP adapter: Codex ACP 2.5.0, source `34e0897`.
- Native: Codex 0.165.0, source
  `360319d8dee62d87e2d59fc0c21805d79519cfed`.
- Explicit isolated native home, file credential store and controlled child
  environment; no inherited workload identity or ambient credential fallback.
- ChatGPT refresh authority remains on the server. Native credential files do
  not receive a usable refresh token.

These pins are an inspected interoperability profile, not a decision to ignore
D2's compatibility constraints or allow every future release automatically.

## Initial Authentication

### API Key

The adapter's explicit API-key auth metadata reaches native
`account/login/start(type=apiKey)`. The native account processor calls
`login_with_api_key` with `config.cli_auth_credentials_store_mode`, then reloads
its auth manager. With file mode this writes `CODEX_HOME/auth.json`.

Thus the initial explicit ACP login has a supported production path. ACP itself
does not make storage ephemeral. Later file replacement is not another ACP login
and does not inherit that login's explicit cache reload.

### Server-Owned ChatGPT Tokens

The native app-server initializes `AuthManager::shared_from_config` with API-key
environment priority disabled. Absent a process-configured workload identity,
this does not install a client callback provider merely because a file contains
`auth_mode: chatgptAuthTokens`.

The persistent-store loader reads the configured authentication file and
`CodexAuth::from_auth_dot_json` supports `ChatgptAuthTokens`. File seeding therefore
has a real stock-native startup path without an adapter login extension.

The native external-token payload constructor itself uses an empty refresh-token
string. Its `tokens` schema also requires a JWT-backed `id_token` representation
and an account binding. Generate an exact supported representation from admitted
source credentials/metadata; do not copy the server bundle blindly or invent an
account identity. The native constructor derives token metadata from the access
JWT and explicit account binding; that is not JWT signature verification.
Missing or incompatible source metadata must fail eligibility rather than adopt
an unrelated cached login.

## File Replacement and Cache Adoption

`AuthManager` caches loaded authentication. File replacement does not itself
force a reload. In particular:

- API-key auth is not included in the ordinary ChatGPT unauthorized-recovery
  capability. A manually changed API key can remain cached until native behavior
  reloads it or a new process starts. D26 explicitly tolerates lack of immediate
  adoption and forbids a hidden forced re-login/restart workaround.
- A file-seeded `ChatgptAuthTokens` value supports unauthorized recovery. With no
  installed external-auth provider, its recovery sequence begins with a guarded
  file reload rather than a client callback.
- That reload checks the expected account ID. A different or missing account ID
  is rejected; replacing a file is not permission to change the live account.
- The exact inference client constructs an `UnauthorizedRecovery` for its request
  attempts and calls `recovery.next()` on a recoverable authentication error.
  After reload, the normal inference loop obtains current client/auth setup for
  its next attempt. This links the loader/recovery helper to production inference
  rather than relying only on an unused generated type.
- Proactive native-managed token refresh is limited to the `Chatgpt` variant.
  `ChatgptAuthTokens` is not an independent refresh-token owner.

This supports the file-only design conditionally. The server must publish renewed
short-lived credentials; native recovery can adopt an already-published same-account
file without Chapchoo forcing a reload. It is not a generic continuous file
watcher or an assurance that publication always beats recovery attempts. An
already-exhausted native recovery can return an authentication failure. Preserve
that outcome; do not replay the product command automatically.

## Do Not Mix the Callback Path With File Seeding

Native `account/login/start(type=chatgptAuthTokens)` installs an
`ExternalAuthBridge`. That provider retains credentials in memory and requests
`account/chatgptAuthTokens/refresh` from its client. Stock inspected ACP does not
implement that request bridge. The file-seeded path does not require that native
login operation and must not accidentally install it as a supposed refresh fix.

## Repository Security Boundaries

- D27 uses the existing saved volume/PVC, not an unverified writable `/tmp`.
  Retain non-root execution, read-only rootfs and existing mount identity.
- Apply D25's saved/applied authentication split. A file is bound to the applied
  record until the next actual harness start selects another record.
- D26 updates same-record files without harness intervention. Fenced current
  delivery must prevent a late old payload replacing a newer credential.
- Authentication traffic and secret-bearing files are excluded from the ACP
  session-event copy, product transcript, argv and routine wire diagnostics.
- D28 retains the existing trusted-network HTTP component route for execution
  credentials. Do not widen it to a public distribution endpoint or claim
  confidentiality from a bearer check. Network observers remain inside the
  explicitly selected trust boundary; exact target admission is still required.
- Existing Team leave/kick/deletion already commits terminal Computer intent and
  Connector revocation. Retain that cleanup authority; a new model-token revoke
  endpoint or immediate offline kill is not implied.

## Required Acceptance Before Runtime Claims

Use a deterministic native/adapter fixture first, then the exact restricted
Computer profile and explicitly supplied isolated live-auth prerequisites where
needed. Required cases include:

1. Initial API-key login uses the selected key and configured file store.
2. External-token file startup uses the admitted account without a usable refresh
   token, ambient account or external callback bridge.
3. Same-account replacement is adopted on the supported native recovery path;
   different-account replacement is refused.
4. Delayed file publication can fail recovery truthfully without product-command
   replay, hidden restart or forced re-login.
5. Manual API-key replacement is not falsely reported as immediate live adoption.
6. Server refresh is coordinated per record, stale results cannot overwrite
   owner replacement, and multiple authorized Agents receive current token data.
7. Process replacement/Computer-off storage retains only the accepted residue;
   initial admission does not treat leftover files as independent authority.
8. Synthetic secret markers do not appear in logs, errors, event copies or browser
   DTOs. Same-UID file permission limits remain explicit.

Source inspection establishes a credible implementation and test path, not live
OAuth refresh success, model entitlement or complete runtime compatibility.

## References

- [Native app-server startup](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/lib.rs)
- [Native account processor](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/request_processors/account_processor.rs)
- [Native auth loader and recovery](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/login/src/auth/manager.rs)
- [Native credential storage](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/login/src/auth/storage.rs)
- [Native inference client](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/core/src/client.rs)
- [Native external-auth callback provider](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/external_auth.rs)
- [Adapter authentication](https://github.com/agentclientprotocol/codex-acp/blob/34e0897/src/CodexAcpClient.ts)
- [Official credential storage](https://developers.openai.com/codex/auth/#credential-storage)
