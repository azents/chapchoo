---
title: "Repository-owned entity identity"
---

# Repository-owned entity identity

Repository acceptance owns product identity issuance and storage representation.
A display handle, operation request, configured registration key and native
allocation witness are different authorities, not alternate entity IDs.

## Acceptance and representation

Every new User, Team, Agent, Computer, Provider, account session, grant,
invitation, login challenge, execution, Connector session, Control process,
connection, profile-image, authentication record/attempt, conversation and
accepted conversation-input identity is a UUIDv7 issued inside its accepting
Repository operation. Existing acceptance time supplies its timestamp; entropy
failure rejects the operation. Services prepare credentials and native names,
not proposed entity IDs. An exact accepted-request recovery returns its original
assigned identities rather than issuing replacements.

The private `EntityId`/`IdentitySql` boundary stores PostgreSQL native UUID values
and SQLite 16-byte BLOB values. Private row projections serialize complete DTO IDs
as an uppercase kind prefix followed by 22 unpadded Base64URL characters. Prefixes
are `U`, `T`, `A`, `C`, `P`, `S`, `G`, `I`, `L`, `E`, `CS`, `CT`, `CN`, `PI`,
`HA`, `HD`, `CV` and `CO` for the respective kinds above. Repository ingress validates the whole canonical
representation, expected kind, UUID version and variant. Raw UUID input,
noncanonical encoding, a different kind and legacy text IDs reject; there is no
runtime old-ID alias table.

Ordinary services, RPC and browser consumers preserve the complete opaque DTO.
They do not split prefixes, extract UUIDs or reproduce the codec. Database
keysets bind compact values with nullable cursors; browser string ordering is
not the database ordering contract. None of these IDs grants access without the
owning current authorization and scope checks.

## Deliberately separate values

- Final [Agent and Team handles](agent-profile.md) are public names with their
  own uniqueness and immutability rules.
- Request IDs identify purpose-scoped accepted operations and fingerprints, not
  entities. Existing request keys and exact historical payload hashes survive.
- Provider configuration supplies a registration key. Registration resolves it
  transactionally to one canonical Provider ID; operational calls use that
  returned ID. See [Provider authentication](provider-authentication.md).
- Native daemon IDs, Kubernetes UIDs, container IDs, original volume metadata and
  immutable native names retain their own types. Native names are independently
  generated; they are not rewritten from the DTO representation.
- Committed allocation witnesses retain exact original Computer/Provider/execution
  label values even when their corresponding product IDs change representation.
  [Native storage](computer-storage.md) and adapters use those witnesses rather
  than reconstructing ownership from current DTOs.
- Bearers, credential digests, recovery envelopes and system-owner evidence remain
  credential contracts. An encoded entity ID is never a credential.

## Explicit preserving cutover

Forward migration 9 runs only within explicit preparation under the original
installation-root binding and one database transaction. Its temporary map
inventories the closed identity/reference graph, including removed creation
receipts and historical source/execution identities without live rows. Every
ordinary reference must resolve an inventoried assignment. The map is removed
before commit and has no runtime lookup path.

Preparation preserves relational ownership, current lifecycle state, exact
accepted hashes and requests, resource names/UIDs/native identities, and original
allocation authority. It authenticates each old account, Provider registration/
operational and Connector bootstrap/operational envelope under its original
context before rewrapping the same token under its new context. Digests,
validity, overlap and revocation facts do not change. Invalid root, context,
digest, ciphertext or missing references roll back instead of resetting state.

PostgreSQL validates UUID conversions and restores the reference graph. SQLite
rebuilds the closed graph with foreign keys enabled, checks copied identities/
rows and rejects unknown incoming foreign-key dependencies or triggers attached
to rebuilt tables rather than dropping their authority silently. Failed
preparation preserves the old database and supports an explicit retry after the
cause is corrected. Downgrade refuses destructive erasure; operator recovery
requires the database and original root backup together.

Normal process startup performs read-only readiness, never this conversion.
All processes and Connectors must cut over as one compatible release; this is not
mixed-format runtime compatibility. [Preparation](account-storage.md) owns the
binding/readiness boundary. No automatic deployed maintenance or rollback is
implied by a successful code/test change.

## Operator procedure for the identity cutover

This procedure requires a separately approved maintenance window and an operator
with database, original-root and native-resource authority. It is not executed by
`serve`, a readiness probe or this document. Backend, both webs, Providers and
execution-local Connectors form one compatible release boundary.

### Establish a safe stopped boundary

1. Record the deployed source/artifact set, database engine and location,
   configured Provider registration keys and native targets. Securely inventory
   accepted request IDs, lifecycle phases, original saved-resource names/UIDs and
   allocation witnesses. Resolve ownership ambiguity before proceeding; neither
   canonical ID equality nor a configuration key authorizes adopting a resource.
2. Block new user mutations at the deployment's admission boundary. Retain only
   authorized maintenance access while settling existing operations through their
   original request and source-qualified evidence. An empty historical-result read
   is not proof that an uncertain Create stopped. Preserve its frozen request and
   payload; do not issue a replacement.
3. Use the existing [Stop/retirement contract](computer-lifecycle.md) to retire old
   execution processes while preserving their saved environments. Record confirmed
   retirement and original saved-volume/PVC identity. Agent/Delete/force-removal
   is not a maintenance shortcut. If a physical effect is uncertain or retirement
   cannot be proved, stop the cutover rather than relabel, recreate or replay it.
4. Stop background dispatch and all old backend, web, Provider and Connector
   processes using deployment-owned supervision; prevent automatic restart.
   Graceful signals withdraw [foreground admission](runtime-process.md) and join
   owned work. Provider cancellation has a 120-second budget and Connector
   cancellation 15 seconds; expiry is failure, not native-retirement evidence.
   Verify no old writer, dispatcher or execution-local process remains. There is
   no built-in global maintenance/drain switch; stopping a Provider alone does
   not establish the preceding physical boundary.

### Preserve a recoverable checkpoint and prepare

5. With writers stopped, take an engine-consistent database backup using the
   deployment's established backup tool, and securely retain the **exact original
   installation root** as the paired recovery input. SQLite requires a consistent
   database snapshot, including committed WAL state when present, not a blind
   live-file copy. PostgreSQL requires a restorable engine-consistent backup.
   Record and verify the backup's recovery procedure in an isolated approved
   environment. Saved native environments remain at their original identities;
   retain their independent backup/retention policy. A DB backup alone does not
   snapshot a volume or undo native work. Keep keys, credential-bearing config,
   backups and inventory outside source control and public logs.
6. Select the coordinated new executable and securely supply exactly one of
   `CHAPCHOO_SQLITE_PATH` or `CHAPCHOO_POSTGRES_URL`, plus `CHAPCHOO_SECRET_KEY`
   containing the original 32-byte root as padded URL-safe base64. Preserve the
   configured Provider registration keys and native targets. Check the machine's
   ordinary locale/time zone if initial display defaults/backfill are needed.
   Run **once**, with no old process admitted:

   ```sh
   chapchoo prepare
   ```

   For an isolated source development environment the equivalent is
   `cd rust && cargo run --locked -p chapchoo -- prepare`; it is not a
   production build or deployment command. Preparation owns its DB lock and
   transaction, not ingress shutdown, backups, native retirement or deployment.
7. Require successful completion. Start only matching new backend components,
   still behind the mutation boundary, with the same DB/root/configuration.
   For example, `chapchoo serve --groups client,admin,control` selects backend
   groups; retain deployment-specific listener, peer, web and TLS settings.
   Require read-only schema/key readiness before admitting matching webs and
   Providers. Reconnect under returned canonical identities; a configured
   registration key is not the canonical Provider ID.
8. Authenticate in the new web, resolve personal `my` and collaborative handles,
   inspect saved profiles/relationships and read known historical creation results.
   Check the current Provider's original saved-resource witness and verified
   environment before authorizing replacement execution. New execution uses
   matching Connector code and newly issued execution-bound inputs, never an
   old container's embedded session ID. Prove original saved-data continuity
   through the normal lifecycle before reopening normal mutations/dispatch.
   Merely healthy processes or a successful DB transaction are insufficient.

### Failure and recovery boundaries

- **Failed preparation:** keep admission closed, confirm the invocation has ended,
  inspect the cause and correct only the authorized input/integrity issue.
  Its DB transaction rolls back; explicitly retry `prepare` with the original
  root after correction. Do not remove binding/history, discard unknown dependency
  tables/triggers or reset credentials to make it pass.
- **Unknown preparation outcome:** verify process termination and database
  readiness/history before choosing retry or recovery. A disconnected shell is
  not proof of rollback.
- **Successful preparation:** no destructive down migration or old-binary
  startup is supported against the converted DB. Keep a failed new release
  isolated for diagnosis or obtain separate approval to restore the paired
  database/root checkpoint and matching old release. Account for every
  post-checkpoint accepted write before restoration; it may otherwise be lost.
- **Any recovery:** restoring the DB does not resurrect retired processes or undo
  native effects. Reconcile exact original native identities and uncertainty
  against the restored schema before resuming; never automatically repeat an
  uncertain operation. Mixed-format processes and saved-environment recreation
  are not recovery modes.

## Connections

- [Account schema and preparation](account-storage.md) — binding, atomic backfill and read-only startup.
- [Computer schema](computer-storage.md) — references, native identities and independent cleanup receipts.
- [Agent handles and profiles](agent-profile.md) — human-facing names and saved presentation values.
- [Control directory](computer-directory.md) — Repository-issued process/connection identities still require exact captured-owner admission.
- [Foreground process ownership](runtime-process.md) — readiness and owned shutdown do not perform operator maintenance or prove native retirement.

## References

- [`entity_id.rs`](../../../../rust/crates/persistence/src/entity_id.rs)
- [`identity_transition.rs`](../../../../rust/crates/persistence/src/identity_transition.rs)
- [`profile_preparation.rs`](../../../../rust/crates/persistence/src/profile_preparation.rs)
- [`operator composition`](../../../../rust/apps/chapchoo/src/main.rs)
- [`database/root configuration`](../../../../rust/apps/chapchoo/src/config.rs)
- [`migration 9`](../../../../rust/crates/persistence/migrations)
- [`profile-261104/Design`](../../design/profile-261104-agent-identity.md)
