use chapchoo_protocol::accounts as wire;
use chapchoo_service::{AcceptanceTime, DomainReason};
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct InstallationAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::installation_server::Installation for InstallationAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "setup_status"))]
    async fn setup_status(
        &self,
        _: Request<()>,
    ) -> Result<Response<wire::SetupStatusResult>, Status> {
        self.context.admit()?;
        let available = self.context.service.setup_status().await.map_err(status)?;
        Ok(Response::new(wire::SetupStatusResult { available }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "initialize"))]
    async fn initialize(
        &self,
        request: Request<wire::InitializeRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.context.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.installation_proof, DomainReason::InvalidProof)?;
        let email = input::email(&value.email)?;
        let password = input::password(value.password)?;
        let result = self
            .context
            .service
            .initialize(
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
}
