use chapchoo_protocol::accounts as wire;
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct PersonalPreferencesAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::personal_preferences_server::PersonalPreferences for PersonalPreferencesAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "display_defaults"))]
    async fn display_defaults(
        &self,
        _: Request<()>,
    ) -> Result<Response<wire::DisplaySettings>, Status> {
        self.context.admit()?;
        Ok(Response::new(output::display(
            self.context
                .service
                .display_defaults()
                .await
                .map_err(status)?,
        )))
    }

    #[tracing::instrument(skip_all, fields(rpc = "update_display"))]
    async fn update_display(
        &self,
        request: Request<wire::DisplayPatch>,
    ) -> Result<Response<wire::User>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::display_patch(&value)?;
        let result = self
            .context
            .service
            .update_display(
                &actor,
                value.language.as_deref(),
                value.time_zone.as_deref(),
                value.region.as_deref(),
                input::now()?,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(output::user(result)?))
    }
}
