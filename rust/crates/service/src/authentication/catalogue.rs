use super::AuthenticationService;
use crate::ServiceError;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    AuthenticationAcceptedResult, AuthenticationRequestResult, AuthenticationScope,
    AuthenticationSummary, DeleteAuthentication, DeleteAuthenticationOutcome, RegisterApiKey,
    RenameAuthentication, ReplaceApiKey, SelectAgentAuthentication,
};

impl AuthenticationService {
    pub async fn list(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        after: Option<&str>,
        limit: u32,
        now: i64,
    ) -> Result<Vec<AuthenticationSummary>, ServiceError> {
        Ok(self
            .store
            .external_authentications(actor, scope, after, limit, now)
            .await?)
    }
    pub async fn get(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        id: &str,
        now: i64,
    ) -> Result<AuthenticationSummary, ServiceError> {
        Ok(self
            .store
            .external_authentication(actor, scope, id, now)
            .await?)
    }
    pub async fn request_result(
        &self,
        actor: &SecretDigest,
        request: &str,
        now: i64,
    ) -> Result<Option<AuthenticationRequestResult>, ServiceError> {
        Ok(self
            .store
            .external_authentication_request_result(actor, request, now)
            .await?)
    }
    pub async fn register_key(
        &self,
        actor: &SecretDigest,
        draft: &RegisterApiKey,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, ServiceError> {
        Ok(self
            .store
            .register_external_api_key(actor, draft, now)
            .await?)
    }
    pub async fn replace_key(
        &self,
        actor: &SecretDigest,
        draft: &ReplaceApiKey,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, ServiceError> {
        Ok(self
            .store
            .replace_external_api_key(actor, draft, now)
            .await?)
    }
    pub async fn rename(
        &self,
        actor: &SecretDigest,
        draft: &RenameAuthentication,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, ServiceError> {
        Ok(self
            .store
            .rename_external_authentication(actor, draft, now)
            .await?)
    }
    pub async fn delete(
        &self,
        actor: &SecretDigest,
        draft: &DeleteAuthentication,
        now: i64,
    ) -> Result<DeleteAuthenticationOutcome, ServiceError> {
        Ok(self
            .store
            .delete_external_authentication(actor, draft, now)
            .await?)
    }
    pub async fn select_agent(
        &self,
        actor: &SecretDigest,
        draft: &SelectAgentAuthentication,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, ServiceError> {
        Ok(self
            .store
            .select_agent_authentication(actor, draft, now)
            .await?)
    }
}
