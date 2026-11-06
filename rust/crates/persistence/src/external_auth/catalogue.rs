use super::*;
use crate::{Store, models::authenticate, store::operation};
use chapchoo_crypto::ExternalSecret;

pub struct RegisterApiKey {
    pub scope: AuthenticationScope,
    pub request_id: String,
    pub name: String,
    pub key: ExternalSecret,
}
pub struct ReplaceApiKey {
    pub scope: AuthenticationScope,
    pub request_id: String,
    pub authentication_id: String,
    pub expected_request_id: String,
    pub key: ExternalSecret,
}
#[derive(Debug)]
pub struct RenameAuthentication {
    pub scope: AuthenticationScope,
    pub request_id: String,
    pub authentication_id: String,
    pub expected_request_id: String,
    pub name: String,
}
#[derive(Debug)]
pub struct DeleteAuthentication {
    pub scope: AuthenticationScope,
    pub request_id: String,
    pub authentication_id: String,
    pub expected_request_id: String,
}
impl fmt::Debug for RegisterApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RegisterApiKey([REDACTED])")
    }
}
impl fmt::Debug for ReplaceApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReplaceApiKey([REDACTED])")
    }
}

fn validate_key(key: &ExternalSecret) -> Result<(), StoreError> {
    if !key.expose().iter().all(|b| b.is_ascii_graphic()) {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}

macro_rules! supersede_target {
    ($connection:expr,$dialect:expr,$id:expr,$now:expr) => {{
        diesel::sql_query($dialect.query("UPDATE external_auth_attempts SET phase='SUPERSEDED',step_id=NULL,pending_nonce=NULL,pending_ciphertext=NULL,updated_at=? WHERE target_id=? AND phase IN ('STARTING','PENDING','CHECKING','EXCHANGING','UNCERTAIN')"))
            .bind::<BigInt,_>($now).bind::<IdentitySql,_>($id).execute($connection).await?;
    }};
}

impl Store {
    /// Preflight ends before encryption. Acceptance independently rechecks scope.
    pub(super) async fn external_scope(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        manage: bool,
        now: i64,
    ) -> Result<(String, Owner), StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, scope, &user.id, manage);
            Ok((user.id, owner))
        })
    }

    pub async fn external_authentications(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        after: Option<&str>,
        limit: u32,
        now: i64,
    ) -> Result<Vec<AuthenticationSummary>, StoreError> {
        if !(1..=100).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        let after = after
            .map(|value| EntityId::from_dto(EntityKind::Authentication, value))
            .transpose()?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, scope, &user.id, false);
            let (column, id) = match (owner.user, owner.team) {
                (Some(id), None) => ("owner_user_id", id),
                (None, Some(id)) => ("owner_team_id", id),
                _ => return Err(StoreError::Integrity),
            };
            let rows = if let Some(after) = after {
                diesel::sql_query(dialect.query(&format!(
                    "{AUTH_COLUMNS} WHERE {column}=? AND id>? ORDER BY id LIMIT ?"
                )))
                .bind::<IdentitySql, _>(id)
                .bind::<IdentitySql, _>(after)
                .bind::<BigInt, _>(i64::from(limit))
                .load::<AuthenticationRow>(connection)
                .await?
            } else {
                diesel::sql_query(dialect.query(&format!(
                    "{AUTH_COLUMNS} WHERE {column}=? ORDER BY id LIMIT ?"
                )))
                .bind::<IdentitySql, _>(id)
                .bind::<BigInt, _>(i64::from(limit))
                .load::<AuthenticationRow>(connection)
                .await?
            };
            rows.into_iter().map(AuthenticationRow::summary).collect()
        })
    }

    pub async fn external_authentication(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        authentication_id: &str,
        now: i64,
    ) -> Result<AuthenticationSummary, StoreError> {
        let id = EntityId::from_dto(EntityKind::Authentication, authentication_id)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, scope, &user.id, false);
            lock_record!(connection, dialect, id, owner).summary()
        })
    }

    /// Read-only recovery never needs the original secret or a still-live catalogue row.
    pub async fn external_authentication_request_result(
        &self,
        actor: &SecretDigest,
        request: &str,
        now: i64,
    ) -> Result<Option<AuthenticationRequestResult>, StoreError> {
        request_id(request)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let Some(row) = receipt!(connection, dialect, &user.id, request) else {
                return Ok(None);
            };
            let operation = AuthenticationOperation::parse(&row.operation)?;
            let owner = admit_scope!(
                connection,
                dialect,
                &row.owner().scope()?,
                &user.id,
                operation.needs_management()
            );
            if owner != row.owner() {
                return Err(StoreError::NotFound);
            }
            Ok(Some(row.result()?))
        })
    }

    pub async fn register_external_api_key(
        &self,
        actor: &SecretDigest,
        draft: &RegisterApiKey,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, StoreError> {
        request_id(&draft.request_id)?;
        name(&draft.name)?;
        validate_key(&draft.key)?;
        let (user, owner) = self.external_scope(actor, &draft.scope, true, now).await?;
        let owner_key = owner.key()?;
        let operation = AuthenticationOperation::RegisterKey;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::OpenAiApiKey)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.name.as_bytes(),
                draft.key.expose(),
            ],
        )?;
        let id = EntityId::generate(now)?;
        let dto = id.to_dto(EntityKind::Authentication);
        let protected = self.cipher.protect_external(
            &context(&owner_key, &dto, ExternalSecretKind::OpenAiApiKey)?,
            &draft.key,
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let accepted_owner = admit_scope!(connection, dialect, &draft.scope, &user.id, true);
            if accepted_owner != owner {
                return Err(StoreError::Forbidden);
            }
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                return receipt.replay(owner, operation, &fingerprint);
            }
            diesel::sql_query(dialect.query("INSERT INTO external_authentications(id,owner_user_id,owner_team_id,kind,name,nonce,ciphertext,current_request_id,created_at,updated_at) VALUES(?,?,?,'OPENAI_API_KEY',?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(id).bind::<Nullable<IdentitySql>,_>(owner.user).bind::<Nullable<IdentitySql>,_>(owner.team)
                .bind::<Text,_>(&draft.name).bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext())
                .bind::<Text,_>(&draft.request_id).bind::<BigInt,_>(now).bind::<BigInt,_>(now).execute(connection).await?;
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                Some(id),
                None::<EntityId>,
                None::<EntityId>,
                now
            );
            Ok(AuthenticationAcceptedResult::Record {
                authentication_id: dto.clone(),
            })
        })
    }

    pub async fn replace_external_api_key(
        &self,
        actor: &SecretDigest,
        draft: &ReplaceApiKey,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, StoreError> {
        request_id(&draft.request_id)?;
        request_id(&draft.expected_request_id)?;
        validate_key(&draft.key)?;
        let id = EntityId::from_dto(EntityKind::Authentication, &draft.authentication_id)?;
        let (user, owner) = self.external_scope(actor, &draft.scope, true, now).await?;
        let owner_key = owner.key()?;
        let operation = AuthenticationOperation::ReplaceKey;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::OpenAiApiKey)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.authentication_id.as_bytes(),
                draft.expected_request_id.as_bytes(),
                draft.key.expose(),
            ],
        )?;
        let protected = self.cipher.protect_external(
            &context(
                &owner_key,
                &draft.authentication_id,
                ExternalSecretKind::OpenAiApiKey,
            )?,
            &draft.key,
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, &draft.scope, &user.id, true);
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                return receipt.replay(owner, operation, &fingerprint);
            }
            let row = lock_record!(connection, dialect, id, owner);
            if AuthenticationKind::parse(&row.kind)? != AuthenticationKind::OpenAiApiKey
                || row.current_request_id != draft.expected_request_id
            {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE external_authentications SET nonce=?,ciphertext=?,current_request_id=?,updated_at=? WHERE id=?"))
                .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext())
                .bind::<Text,_>(&draft.request_id).bind::<BigInt,_>(now).bind::<IdentitySql,_>(id).execute(connection).await?;
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                Some(id),
                None::<EntityId>,
                None::<EntityId>,
                now
            );
            Ok(AuthenticationAcceptedResult::Record {
                authentication_id: draft.authentication_id.clone(),
            })
        })
    }

    pub async fn rename_external_authentication(
        &self,
        actor: &SecretDigest,
        draft: &RenameAuthentication,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, StoreError> {
        request_id(&draft.request_id)?;
        request_id(&draft.expected_request_id)?;
        name(&draft.name)?;
        let id = EntityId::from_dto(EntityKind::Authentication, &draft.authentication_id)?;
        let (user, owner) = self.external_scope(actor, &draft.scope, true, now).await?;
        let owner_key = owner.key()?;
        let operation = AuthenticationOperation::Rename;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::OpenAiApiKey)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.authentication_id.as_bytes(),
                draft.expected_request_id.as_bytes(),
                draft.name.as_bytes(),
            ],
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, &draft.scope, &user.id, true);
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                return receipt.replay(owner, operation, &fingerprint);
            }
            let row = lock_record!(connection, dialect, id, owner);
            if row.current_request_id != draft.expected_request_id {
                return Err(StoreError::InvalidTransition);
            }
            supersede_target!(connection, dialect, id, now);
            diesel::sql_query(dialect.query("UPDATE external_authentications SET name=?,current_request_id=?,active_attempt_id=NULL,updated_at=? WHERE id=?"))
                .bind::<Text,_>(&draft.name).bind::<Text,_>(&draft.request_id).bind::<BigInt,_>(now).bind::<IdentitySql,_>(id).execute(connection).await?;
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                Some(id),
                None::<EntityId>,
                None::<EntityId>,
                now
            );
            Ok(AuthenticationAcceptedResult::Record {
                authentication_id: draft.authentication_id.clone(),
            })
        })
    }

    pub async fn delete_external_authentication(
        &self,
        actor: &SecretDigest,
        draft: &DeleteAuthentication,
        now: i64,
    ) -> Result<DeleteAuthenticationOutcome, StoreError> {
        #[derive(diesel::QueryableByName)]
        struct Count {
            #[diesel(sql_type=BigInt)]
            n: i64,
        }
        request_id(&draft.request_id)?;
        request_id(&draft.expected_request_id)?;
        let id = EntityId::from_dto(EntityKind::Authentication, &draft.authentication_id)?;
        let (user, owner) = self.external_scope(actor, &draft.scope, true, now).await?;
        let owner_key = owner.key()?;
        let operation = AuthenticationOperation::Delete;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::OpenAiApiKey)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.authentication_id.as_bytes(),
                draft.expected_request_id.as_bytes(),
            ],
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, &draft.scope, &user.id, true);
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                return Ok(DeleteAuthenticationOutcome::Accepted(receipt.replay(
                    owner,
                    operation,
                    &fingerprint,
                )?));
            }
            let row = lock_record!(connection, dialect, id, owner);
            if row.current_request_id != draft.expected_request_id {
                return Err(StoreError::InvalidTransition);
            }
            let count = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM agent_authentication_selections WHERE authentication_id=?"))
                .bind::<IdentitySql,_>(id).get_result::<Count>(connection).await?;
            if count.n != 0 {
                return Ok(DeleteAuthenticationOutcome::InUse);
            }
            supersede_target!(connection, dialect, id, now);
            diesel::sql_query(
                dialect
                    .query("UPDATE external_authentications SET active_attempt_id=NULL WHERE id=?"),
            )
            .bind::<IdentitySql, _>(id)
            .execute(connection)
            .await?;
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET target_id=NULL,expected_target_request_id=NULL WHERE target_id=?"))
                .bind::<IdentitySql,_>(id).execute(connection).await?;
            diesel::sql_query(dialect.query("DELETE FROM external_authentications WHERE id=?"))
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                Some(id),
                None::<EntityId>,
                None::<EntityId>,
                now
            );
            Ok(DeleteAuthenticationOutcome::Accepted(
                AuthenticationAcceptedResult::Deleted {
                    authentication_id: draft.authentication_id.clone(),
                },
            ))
        })
    }
}
