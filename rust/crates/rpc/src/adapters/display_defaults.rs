use chapchoo_protocol::accounts as wire;
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct DisplayDefaultsAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::display_defaults_server::DisplayDefaults for DisplayDefaultsAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "update_display_defaults"))]
    async fn update_display_defaults(
        &self,
        request: Request<wire::DisplayPatch>,
    ) -> Result<Response<wire::DisplaySettings>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::display_patch(&value)?;
        let result = self
            .context
            .service
            .update_display_defaults(
                &actor,
                value.language.as_deref(),
                value.time_zone.as_deref(),
                value.region.as_deref(),
                input::now()?,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(output::display(result)))
    }
}
