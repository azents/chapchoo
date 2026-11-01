use chapchoo_protocol::accounts as wire;
use chapchoo_service::{AcceptanceTime, DomainReason};
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct AccountLifecycleAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::account_lifecycle_server::AccountLifecycle for AccountLifecycleAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "request_signup_email"))]
    async fn request_signup_email(
        &self,
        request: Request<wire::SignupEmailRequest>,
    ) -> Result<Response<wire::DeliveryAccepted>, Status> {
        self.context.admit()?;
        let email = input::email(&request.into_inner().email)?;
        self.context
            .service
            .request_signup_email(&email, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::DeliveryAccepted { accepted: true }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "preview_signup"))]
    async fn preview_signup(
        &self,
        request: Request<wire::ProofRequest>,
    ) -> Result<Response<wire::ProofPreview>, Status> {
        self.context.admit()?;
        let proof = input::bearer(request.into_inner().proof, DomainReason::InvalidProof)?;
        let result = self
            .context
            .service
            .preview_signup(&proof, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::preview(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "redeem_signup"))]
    async fn redeem_signup(
        &self,
        request: Request<wire::RedeemSignupRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.context.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.proof, DomainReason::InvalidProof)?;
        let email = input::email(&value.email)?;
        let password = input::password(value.password)?;
        let result = self
            .context
            .service
            .redeem_signup(
                &proof,
                email,
                password,
                AcceptanceTime::System,
                input::display(value.display)?,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(output::session(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "preview_password_reset"))]
    async fn preview_password_reset(
        &self,
        request: Request<wire::ProofRequest>,
    ) -> Result<Response<wire::ProofPreview>, Status> {
        self.context.admit()?;
        let proof = input::bearer(request.into_inner().proof, DomainReason::InvalidProof)?;
        let result = self
            .context
            .service
            .preview_password_reset(&proof, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::preview(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "redeem_password_reset"))]
    async fn redeem_password_reset(
        &self,
        request: Request<wire::RedeemPasswordResetRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.proof, DomainReason::InvalidProof)?;
        let password = input::password(value.password)?;
        self.context
            .service
            .redeem_password_reset(&proof, password, AcceptanceTime::System)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
