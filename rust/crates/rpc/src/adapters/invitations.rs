use chapchoo_protocol::accounts as wire;
use tonic::{Request, Response, Status};

use super::HandlerContext;
use crate::{ingress as input, mapping as output, status::status};

#[derive(Debug, Clone)]
pub struct InvitationsAdapter {
    pub(super) context: HandlerContext,
}

#[tonic::async_trait]
impl wire::invitation_management_server::InvitationManagement for InvitationsAdapter {
    #[tracing::instrument(skip_all, fields(rpc = "invite_member"))]
    async fn invite_member(
        &self,
        request: Request<wire::InviteMemberRequest>,
    ) -> Result<Response<wire::Invitation>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let email = input::email(&value.email)?;
        let result = self
            .context
            .service
            .invite_member(&actor, &value.team_id, email, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::invitation(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "received_invitations"))]
    async fn received_invitations(
        &self,
        request: Request<()>,
    ) -> Result<Response<wire::Invitations>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let result = self
            .context
            .service
            .received_invitations(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::Invitations {
            items: result
                .into_iter()
                .map(output::invitation)
                .collect::<Result<_, _>>()?,
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "team_invitations"))]
    async fn team_invitations(
        &self,
        request: Request<wire::TeamRequest>,
    ) -> Result<Response<wire::Invitations>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
            .context
            .service
            .team_invitations(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::Invitations {
            items: result
                .into_iter()
                .map(output::invitation)
                .collect::<Result<_, _>>()?,
        }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "process_invitation"))]
    async fn process_invitation(
        &self,
        request: Request<wire::ProcessInvitationRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.invitation_id)?;
        let accept = input::invitation_action(value.action)?;
        self.context
            .service
            .process_invitation(&actor, &value.invitation_id, accept, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
