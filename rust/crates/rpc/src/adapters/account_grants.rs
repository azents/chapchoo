use chapchoo_protocol::accounts as wire;
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct AccountGrantsAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::account_grants_server::AccountGrants for AccountGrantsAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "issue_signup_grant"))]
    async fn issue_signup_grant(
        &self,
        request: Request<wire::IssueGrantRequest>,
    ) -> Result<Response<wire::IssuedGrant>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
            .context
            .service
            .issue_signup_grant(&actor, &email, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::issued(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "list_signup_grants"))]
    async fn list_signup_grants(
        &self,
        request: Request<wire::ListGrantsRequest>,
    ) -> Result<Response<wire::Grants>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let (offset, limit) = input::pagination(request.into_inner())?;
        let result = self
            .context
            .service
            .list_signup_grants(&actor, offset, limit, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::grants(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "revoke_signup_grant"))]
    async fn revoke_signup_grant(
        &self,
        request: Request<wire::GrantRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.grant_id)?;
        self.context
            .service
            .revoke_signup_grant(&actor, &value.grant_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "issue_password_reset"))]
    async fn issue_password_reset(
        &self,
        request: Request<wire::IssueGrantRequest>,
    ) -> Result<Response<wire::IssuedGrant>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
            .context
            .service
            .issue_password_reset(&actor, &email, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::issued(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "list_password_resets"))]
    async fn list_password_resets(
        &self,
        request: Request<wire::ListGrantsRequest>,
    ) -> Result<Response<wire::Grants>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let (offset, limit) = input::pagination(request.into_inner())?;
        let result = self
            .context
            .service
            .list_password_resets(&actor, offset, limit, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::grants(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "revoke_password_reset"))]
    async fn revoke_password_reset(
        &self,
        request: Request<wire::GrantRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.grant_id)?;
        self.context
            .service
            .revoke_password_reset(&actor, &value.grant_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
