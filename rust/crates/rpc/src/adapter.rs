use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use chapchoo_protocol::accounts::{
    self as wire, admin_server::Admin, auth_server::Auth, general_server::General,
};
use chapchoo_service::{AcceptanceTime, AccountsService, DomainReason, TeamPatch};
use tonic::{Request, Response, Status};
use zeroize::Zeroizing;

use crate::{ingress as input, mapping as output, status::status};

/// A clone shares application capabilities, never a caller identity.
#[derive(Debug, Clone)]
pub struct RpcService {
    service: Arc<AccountsService>,
    admission: Arc<AtomicBool>,
}

impl RpcService {
    pub fn new(service: Arc<AccountsService>) -> Self {
        Self {
            service,
            admission: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn with_admission(mut self, admission: Arc<AtomicBool>) -> Self {
        self.admission = admission;
        self
    }

    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Service is not admitting new requests"))
        }
    }

    pub fn auth_server(&self) -> wire::auth_server::AuthServer<Self> {
        wire::auth_server::AuthServer::new(self.clone()).max_decoding_message_size(64 * 1024)
    }
    pub fn general_server(&self) -> wire::general_server::GeneralServer<Self> {
        wire::general_server::GeneralServer::new(self.clone()).max_decoding_message_size(64 * 1024)
    }
    pub fn admin_server(&self) -> wire::admin_server::AdminServer<Self> {
        wire::admin_server::AdminServer::new(self.clone()).max_decoding_message_size(64 * 1024)
    }
}

#[tonic::async_trait]
impl Auth for RpcService {
    #[tracing::instrument(skip_all, fields(rpc = "methods"))]
    async fn methods(&self, _: Request<()>) -> Result<Response<wire::LoginMethods>, Status> {
        self.admit()?;
        let methods = self.service.methods();
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
        self.admit()?;
        let value = request.into_inner();
        let email = input::email(&value.email)?;
        let result = self
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
        self.admit()?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
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
        self.admit()?;
        let value = request.into_inner();
        let email = input::email(&value.email)?;
        let binding = input::bearer(value.binding, DomainReason::InvalidCode)?;
        let code = input::code(value.code)?;
        let result = self
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
        self.admit()?;
        let token = input::bearer(request.into_inner().token, DomainReason::InvalidSession)?;
        let result = self
            .service
            .renew(&token, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::session(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "identity"))]
    async fn identity(&self, request: Request<()>) -> Result<Response<wire::User>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let result = self
            .service
            .identity(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(output::user(result)?))
    }

    #[tracing::instrument(skip_all, fields(rpc = "display_defaults"))]
    async fn display_defaults(
        &self,
        _: Request<()>,
    ) -> Result<Response<wire::DisplaySettings>, Status> {
        self.admit()?;
        Ok(Response::new(output::display(
            self.service.display_defaults().await.map_err(status)?,
        )))
    }
    #[tracing::instrument(skip_all, fields(rpc = "update_display"))]
    async fn update_display(
        &self,
        request: Request<wire::DisplayPatch>,
    ) -> Result<Response<wire::User>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::display_patch(&value)?;
        let result = self
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

    #[tracing::instrument(skip_all, fields(rpc = "logout"))]
    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        self.service
            .logout(&actor, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}

#[tonic::async_trait]
impl General for RpcService {
    #[tracing::instrument(skip_all, fields(rpc = "request_signup_email"))]
    async fn request_signup_email(
        &self,
        request: Request<wire::SignupEmailRequest>,
    ) -> Result<Response<wire::DeliveryAccepted>, Status> {
        self.admit()?;
        let email = input::email(&request.into_inner().email)?;
        self.service
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
        self.admit()?;
        let proof = input::bearer(request.into_inner().proof, DomainReason::InvalidProof)?;
        let result = self
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
        self.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.proof, DomainReason::InvalidProof)?;
        let email = input::email(&value.email)?;
        let password = input::password(value.password)?;
        let result = self
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
        self.admit()?;
        let proof = input::bearer(request.into_inner().proof, DomainReason::InvalidProof)?;
        let result = self
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
        self.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.proof, DomainReason::InvalidProof)?;
        let password = input::password(value.password)?;
        self.service
            .redeem_password_reset(&proof, password, AcceptanceTime::System)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "list_teams"))]
    async fn list_teams(&self, request: Request<()>) -> Result<Response<wire::Teams>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let patch = TeamPatch {
            name: value.name,
            handle: value.handle,
        };
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        let role = input::member_role(value.role)?;
        self.service
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        self.service
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        input::id(&value.user_id)?;
        self.service
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        self.service
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        self.service
            .delete_team(&actor, &value.team_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(rpc = "invite_member"))]
    async fn invite_member(
        &self,
        request: Request<wire::InviteMemberRequest>,
    ) -> Result<Response<wire::Invitation>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let email = input::email(&value.email)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.team_id)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.invitation_id)?;
        let accept = input::invitation_action(value.action)?;
        self.service
            .process_invitation(&actor, &value.invitation_id, accept, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}

#[tonic::async_trait]
impl Admin for RpcService {
    #[tracing::instrument(skip_all, fields(rpc = "update_display_defaults"))]
    async fn update_display_defaults(
        &self,
        request: Request<wire::DisplayPatch>,
    ) -> Result<Response<wire::DisplaySettings>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::display_patch(&value)?;
        let result = self
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
    #[tracing::instrument(skip_all, fields(rpc = "setup_status"))]
    async fn setup_status(
        &self,
        _: Request<()>,
    ) -> Result<Response<wire::SetupStatusResult>, Status> {
        self.admit()?;
        let available = self.service.setup_status().await.map_err(status)?;
        Ok(Response::new(wire::SetupStatusResult { available }))
    }

    #[tracing::instrument(skip_all, fields(rpc = "initialize"))]
    async fn initialize(
        &self,
        request: Request<wire::InitializeRequest>,
    ) -> Result<Response<wire::AuthSession>, Status> {
        self.admit()?;
        let value = request.into_inner();
        let proof = input::bearer(value.installation_proof, DomainReason::InvalidProof)?;
        let email = input::email(&value.email)?;
        let password = input::password(value.password)?;
        let result = self
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

    #[tracing::instrument(skip_all, fields(rpc = "issue_signup_grant"))]
    async fn issue_signup_grant(
        &self,
        request: Request<wire::IssueGrantRequest>,
    ) -> Result<Response<wire::IssuedGrant>, Status> {
        self.admit()?;
        let actor = input::actor(&request)?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let (offset, limit) = input::pagination(request.into_inner())?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.grant_id)?;
        self.service
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let email = input::email(&request.into_inner().email)?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let (offset, limit) = input::pagination(request.into_inner())?;
        let result = self
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
        self.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        input::id(&value.grant_id)?;
        self.service
            .revoke_password_reset(&actor, &value.grant_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
