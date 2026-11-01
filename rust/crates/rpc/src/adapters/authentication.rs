use chapchoo_protocol::accounts as wire;
use chapchoo_service::{AcceptanceTime, DomainReason};
use tonic::{Request, Response, Status};
use zeroize::Zeroizing;

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct AuthenticationAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::authentication_server::Authentication for AuthenticationAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "methods"))]
    async fn methods(&self, _: Request<()>) -> Result<Response<wire::LoginMethods>, Status> {
        self.context.admit()?;
        let methods = self.context.service.methods();
        Ok(Response::new(wire::LoginMethods {
            password: methods.password,
            email_code: methods.email_code,
            signup_email: methods.signup_email,
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "password_login"))]
    async fn password_login(
        &self,
        request: Request<wire::PasswordLoginRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.context.admit()?;
        let value = request.into_inner();
        let email = input::email(&value.email)?;
        let result = self
            .context
            .service
            .password_login(
                &email,
                Zeroizing::new(value.password),
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(output::session(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "send_code"))]
    async fn send_code(
        &self,
        request: Request<wire::SendCodeRequest>,
    ) -> Result<Response<wire::CodeChallenge>, Status> {
        self.context.admit()?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
            .context
            .service
            .send_code(&email, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::CodeChallenge {
            binding: result.binding.expose().to_owned(),
            expires_at: Some(output::timestamp(result.expires_at)?),
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "verify_code"))]
    async fn verify_code(
        &self,
        request: Request<wire::VerifyCodeRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.context.admit()?;
        let value = request.into_inner();
        let email = input::email(&value.email)?;
        let binding = input::bearer(value.binding, DomainReason::InvalidCode)?;
        let code = input::code(value.code)?;
        let result = self
            .context
            .service
            .verify_code(&email, &binding, &code, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::session(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "renew"))]
    async fn renew(
        &self,
        request: Request<wire::RenewRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.context.admit()?;
        let token = input::bearer(request.into_inner().token, DomainReason::InvalidSession)?;
        let result = self
            .context
            .service
            .renew(&token, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::session(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "identity"))]
    async fn identity(&self, request: Request<()>) -> Result<Response<wire::User>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let result = self
            .context
            .service
            .identity(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::user(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "logout"))]
    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        self.context
            .service
            .logout(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
