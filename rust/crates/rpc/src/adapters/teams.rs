use chapchoo_protocol::accounts as wire;
use chapchoo_service::TeamPatch;
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct TeamsAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::team_management_server::TeamManagement for TeamsAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "list_teams"))]
    async fn list_teams(&self, request: Request<()>) -> Result<Response<wire::Teams>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let result = self
            .context
            .service
            .list_teams(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::Teams {
            items: result.into_iter().map(output::team).collect(),
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "get_team"))]
    async fn get_team(
        &self,
        request: Request<wire::TeamRequest>,
    ) -> Result<Response<wire::Team>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
            .context
            .service
            .get_team(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::team(result)))
    }

    #[tracing::instrument(skip_all, fields(rpc = "create_team"))]
    async fn create_team(
        &self,
        request: Request<wire::CreateTeamRequest>,
    ) -> Result<Response<wire::Team>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let result = self
            .context
            .service
            .create_team(&actor, value.name, value.handle, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::team(result)))
    }

    #[tracing::instrument(skip_all, fields(rpc = "update_team"))]
    async fn update_team(
        &self,
        request: Request<wire::UpdateTeamRequest>,
    ) -> Result<Response<wire::Team>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let patch = TeamPatch { name: value.name };
        let result = self
            .context
            .service
            .update_team(&actor, &value.team_id, &patch, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::team(result)))
    }

    #[tracing::instrument(skip_all, fields(rpc = "list_members"))]
    async fn list_members(
        &self,
        request: Request<wire::TeamRequest>,
    ) -> Result<Response<wire::Members>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
            .context
            .service
            .list_members(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::Members {
            items: result.into_iter().map(output::member).collect(),
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "set_member_role"))]
    async fn set_member_role(
        &self,
        request: Request<wire::SetMemberRoleRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        let role = input::member_role(value.role)?;
        self.context
            .service
            .set_member_role(&actor, &value.team_id, &value.user_id, role, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "remove_member"))]
    async fn remove_member(
        &self,
        request: Request<wire::MemberRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        self.context
            .service
            .remove_member(&actor, &value.team_id, &value.user_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "transfer_ownership"))]
    async fn transfer_ownership(
        &self,
        request: Request<wire::MemberRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        self.context
            .service
            .transfer_ownership(&actor, &value.team_id, &value.user_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "leave_team"))]
    async fn leave_team(
        &self,
        request: Request<wire::TeamRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        self.context
            .service
            .leave_team(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "delete_team"))]
    async fn delete_team(
        &self,
        request: Request<wire::TeamRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        self.context
            .service
            .delete_team(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
