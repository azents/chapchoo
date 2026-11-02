use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};
use chapchoo_protocol::accounts as wire;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct AdminWorkspacesAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::admin_workspaces_server::AdminWorkspaces for AdminWorkspacesAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "list_admin_workspaces"))]
    async fn list_teams(&self, request: Request<()>) -> Result<Response<wire::Teams>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let teams = self
            .context
            .service
            .list_admin_workspaces(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::Teams {
            items: teams.into_iter().map(output::team).collect(),
        }))
    }
    #[tracing::instrument(skip_all, fields(rpc = "create_admin_workspace"))]
    async fn create_team(
        &self,
        request: Request<wire::CreateTeamRequest>,
    ) -> Result<Response<wire::Team>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let team = self
            .context
            .service
            .create_admin_workspace(&actor, value.name, value.handle, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::team(team)))
    }
}
