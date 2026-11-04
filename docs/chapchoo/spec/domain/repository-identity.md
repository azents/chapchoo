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
connection and profile-image identity is a UUIDv7 issued inside its accepting
Repository operation. Existing acceptance time supplies its timestamp; entropy
failure rejects the operation. Services prepare credentials and native names,
not proposed entity IDs. An exact accepted-request recovery returns its original
assigned identities rather than issuing replacements.

The private `EntityId`/`IdentitySql` boundary stores PostgreSQL native UUID values
and SQLite 16-byte BLOB values. Private row projections serialize complete DTO IDs
as an uppercase kind prefix followed by 22 unpadded Base64URL characters. Prefixes
are `U`, `T`, `A`, `C`, `P`, `S`, `G`, `I`, `L`, `E`, `CS`, `CT`, `CN` and `PI` for
the respective kinds above. Repository ingress validates the whole canonical
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

## Connections

- [Account schema and preparation](account-storage.md) — binding, atomic backfill and read-only startup.
- [Computer schema](computer-storage.md) — references, native identities and independent cleanup receipts.
- [Agent handles and profiles](agent-profile.md) — human-facing names and saved presentation values.
- [Control directory](computer-directory.md) — Repository-issued process/connection identities still require exact captured-owner admission.

## References

- [`entity_id.rs`](../../../../rust/crates/persistence/src/entity_id.rs)
- [`identity_transition.rs`](../../../../rust/crates/persistence/src/identity_transition.rs)
- [`profile_preparation.rs`](../../../../rust/crates/persistence/src/profile_preparation.rs)
- [`migration 9`](../../../../rust/crates/persistence/migrations)
- [`profile-261104/Design`](../../design/profile-261104-agent-identity.md)
