mod support;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey};
use chapchoo_persistence::{SessionPolicy, Store, prepare_sqlite};
use chapchoo_protocol::accounts::{
    self as wire, account_grants_client::AccountGrantsClient,
    account_lifecycle_client::AccountLifecycleClient, authentication_client::AuthenticationClient,
    display_defaults_client::DisplayDefaultsClient, installation_client::InstallationClient,
    personal_preferences_client::PersonalPreferencesClient,
    team_management_client::TeamManagementClient,
};
use chapchoo_rpc::{AccountServices, DOMAIN_REASON_METADATA};
use chapchoo_service::{AccountsConfig, AccountsService, PasswordWork};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Code, Request, Status,
    transport::{Channel, Server},
};

type TestError = Box<dyn std::error::Error + Send + Sync>;

fn authenticated<T>(body: T, token: &str) -> Result<Request<T>, TestError> {
    let mut request = Request::new(body);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    Ok(request)
}

fn assert_reason(error: &Status, code: Code, reason: &str) {
    assert_eq!(error.code(), code);
    assert_eq!(
        error
            .metadata()
            .get(DOMAIN_REASON_METADATA)
            .and_then(|value| value.to_str().ok()),
        Some(reason)
    );
}

async fn exercise(
    general: Channel,
    admin: Channel,
    proof: &BearerSecret,
    admission: Arc<AtomicBool>,
) -> Result<(), TestError> {
    let mut installation = InstallationClient::new(admin.clone());
    let mut account_grants = AccountGrantsClient::new(admin.clone());
    let mut account_lifecycle = AccountLifecycleClient::new(general.clone());
    let mut teams = TeamManagementClient::new(general.clone());
    let mut workspaces = wire::admin_workspaces_client::AdminWorkspacesClient::new(admin.clone());

    let mut auth_general = AuthenticationClient::new(general.clone());
    let mut auth_admin = AuthenticationClient::new(admin.clone());
    assert_reason(
        &workspaces
            .list_teams(())
            .await
            .err()
            .ok_or("Anonymous workspace read")?,
        Code::Unauthenticated,
        "DOMAIN_REASON_INVALID_SESSION",
    );
    assert_reason(
        &workspaces
            .list_teams(authenticated((), "invalid-session")?)
            .await
            .err()
            .ok_or("Invalid workspace credential")?,
        Code::Unauthenticated,
        "DOMAIN_REASON_INVALID_SESSION",
    );
    assert!(installation.setup_status(()).await?.into_inner().available);
    let owner = installation
        .initialize(wire::InitializeRequest {
            installation_proof: proof.expose().to_owned(),
            email: "Owner@Example.Invalid".into(),
            password: "Aa1!owner".into(),
            display: None,
        })
        .await?
        .into_inner();
    let owner_id = owner.user.as_ref().ok_or("Missing owner user")?.id.clone();
    let personal = workspaces
        .list_teams(authenticated((), &owner.token)?)
        .await?
        .into_inner();
    assert_eq!(personal.items.len(), 1);
    let created_workspace = workspaces
        .create_team(authenticated(
            wire::CreateTeamRequest {
                name: "Admin-created workspace".into(),
                handle: "admin-workspace".into(),
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    assert_eq!(created_workspace.owner_user_id, owner_id);
    assert!(
        teams
            .list_teams(authenticated((), &owner.token)?)
            .await?
            .into_inner()
            .items
            .iter()
            .any(|team| team.id == created_workspace.id)
    );
    assert_eq!(
        wire::admin_workspaces_client::AdminWorkspacesClient::new(general.clone())
            .list_teams(authenticated((), &owner.token)?)
            .await
            .err()
            .ok_or("Admin workspace service exposed on Client")?
            .code(),
        Code::Unimplemented
    );
    assert!(!installation.setup_status(()).await?.into_inner().available);
    assert!(auth_general.methods(()).await?.into_inner().password);
    assert!(!auth_admin.methods(()).await?.into_inner().email_code);
    let mut general_preferences = PersonalPreferencesClient::new(general.clone());
    let mut admin_preferences = PersonalPreferencesClient::new(admin.clone());
    let mut defaults = DisplayDefaultsClient::new(admin.clone());
    let initial = general_preferences.display_defaults(()).await?.into_inner();
    assert_eq!(
        admin_preferences.display_defaults(()).await?.into_inner(),
        initial
    );
    let changed = defaults
        .update_display_defaults(authenticated(
            wire::DisplayPatch {
                language: Some("ko".into()),
                time_zone: None,
                region: None,
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    assert_eq!(changed.language, "ko");
    assert_eq!(changed.time_zone, initial.time_zone);
    assert_eq!(
        general_preferences.display_defaults(()).await?.into_inner(),
        changed
    );
    assert_eq!(
        DisplayDefaultsClient::new(general.clone())
            .update_display_defaults(authenticated(wire::DisplayPatch::default(), &owner.token)?)
            .await
            .err()
            .ok_or("Expected absent privileged display service")?
            .code(),
        Code::Unimplemented
    );

    // Common Authentication is available on both listeners. Dedicated business surfaces
    // stay absent on the opposite listener, including setup/bootstrap methods.
    assert_eq!(
        InstallationClient::new(general.clone())
            .setup_status(())
            .await
            .err()
            .ok_or("Expected absent Installation service")?
            .code(),
        Code::Unimplemented
    );
    assert_eq!(
        TeamManagementClient::new(admin.clone())
            .list_teams(authenticated((), &owner.token)?)
            .await
            .err()
            .ok_or("Expected absent Teams service")?
            .code(),
        Code::Unimplemented
    );
    assert_eq!(
        auth_general
            .identity(authenticated((), &owner.token)?)
            .await?
            .into_inner()
            .id,
        owner_id
    );
    assert_eq!(
        auth_admin
            .identity(authenticated((), &owner.token)?)
            .await?
            .into_inner()
            .id,
        owner_id
    );
    assert_reason(
        &auth_general
            .identity(())
            .await
            .err()
            .ok_or("Expected missing bearer rejection")?,
        Code::Unauthenticated,
        "DOMAIN_REASON_INVALID_SESSION",
    );

    let grant = account_grants
        .issue_signup_grant(authenticated(
            wire::IssueGrantRequest {
                email: "member@example.invalid".into(),
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    let preview = account_lifecycle
        .preview_signup(wire::ProofRequest {
            proof: grant.proof.clone(),
        })
        .await?
        .into_inner();
    assert!(preview.usable);
    assert_eq!(preview.email_hint.as_deref(), Some("m***@example.invalid"));
    assert!(preview.expires_at.is_some());
    let member = account_lifecycle
        .redeem_signup(wire::RedeemSignupRequest {
            proof: grant.proof.clone(),
            email: "MEMBER@example.invalid".into(),
            password: "Aa1!member".into(),
            display: None,
        })
        .await?
        .into_inner();
    let member_id = member
        .user
        .as_ref()
        .ok_or("Missing member user")?
        .id
        .clone();
    let used = account_lifecycle
        .preview_signup(wire::ProofRequest { proof: grant.proof })
        .await?
        .into_inner();
    assert!(!used.usable);
    assert!(used.email_hint.is_none());
    assert!(used.expires_at.is_none());
    let mut first = AuthenticationClient::new(general.clone());
    let mut second = AuthenticationClient::new(general.clone());
    let (owner_read, member_read) = tokio::join!(
        first.identity(authenticated((), &owner.token)?),
        second.identity(authenticated((), &member.token)?)
    );
    assert_eq!(owner_read?.into_inner().id, owner_id);
    assert_eq!(member_read?.into_inner().id, member_id);
    assert_reason(
        &workspaces
            .list_teams(authenticated((), &member.token)?)
            .await
            .err()
            .ok_or("Non-admin workspace read")?,
        Code::PermissionDenied,
        "DOMAIN_REASON_PERMISSION_DENIED",
    );
    assert_reason(
        &workspaces
            .create_team(authenticated(
                wire::CreateTeamRequest {
                    name: "Forbidden".into(),
                    handle: "forbidden".into(),
                },
                &member.token,
            )?)
            .await
            .err()
            .ok_or("Non-admin workspace create")?,
        Code::PermissionDenied,
        "DOMAIN_REASON_PERMISSION_DENIED",
    );
    assert_reason(
        &account_grants
            .list_signup_grants(authenticated(
                wire::ListGrantsRequest {
                    offset: None,
                    limit: None,
                },
                &member.token,
            )?)
            .await
            .err()
            .ok_or("Expected non-admin rejection")?,
        Code::PermissionDenied,
        "DOMAIN_REASON_PERMISSION_DENIED",
    );

    let team = teams
        .create_team(authenticated(
            wire::CreateTeamRequest {
                name: "Team".into(),
                handle: "rpc-team".into(),
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    let unchanged = teams
        .update_team(authenticated(
            wire::UpdateTeamRequest {
                team_id: team.id.clone(),
                name: None,
                handle: None,
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    assert_eq!(unchanged.name, "Team");
    assert_reason(
        &teams
            .update_team(authenticated(
                wire::UpdateTeamRequest {
                    team_id: team.id.clone(),
                    name: Some(String::new()),
                    handle: None,
                },
                &owner.token,
            )?)
            .await
            .err()
            .ok_or("Expected present empty field rejection")?,
        Code::InvalidArgument,
        "DOMAIN_REASON_INVALID_INPUT",
    );
    for role in [0, -1, 9999] {
        assert_reason(
            &teams
                .set_member_role(authenticated(
                    wire::SetMemberRoleRequest {
                        team_id: team.id.clone(),
                        user_id: member_id.clone(),
                        role,
                    },
                    &owner.token,
                )?)
                .await
                .err()
                .ok_or("Expected invalid enum rejection")?,
            Code::InvalidArgument,
            "DOMAIN_REASON_INVALID_INPUT",
        );
    }
    assert_reason(
        &teams
            .leave_team(authenticated(
                wire::TeamRequest { team_id: team.id },
                &owner.token,
            )?)
            .await
            .err()
            .ok_or("Expected owner leave rejection")?,
        Code::FailedPrecondition,
        "DOMAIN_REASON_OWNER_TRANSFER_REQUIRED",
    );
    assert_reason(
        &auth_general
            .password_login(wire::PasswordLoginRequest {
                email: "member@example.invalid".into(),
                password: "wrong-password".into(),
            })
            .await
            .err()
            .ok_or("Expected credential rejection")?,
        Code::Unauthenticated,
        "DOMAIN_REASON_INVALID_CREDENTIALS",
    );

    // All sessions use one authority: a common logout on Admin revokes General.
    auth_admin.logout(authenticated((), &member.token)?).await?;
    assert_reason(
        &auth_general
            .identity(authenticated((), &member.token)?)
            .await
            .err()
            .ok_or("Expected revoked session")?,
        Code::Unauthenticated,
        "DOMAIN_REASON_INVALID_SESSION",
    );
    admission.store(false, Ordering::Release);
    assert_eq!(
        auth_general
            .methods(())
            .await
            .err()
            .ok_or("Expected admission withdrawal")?
            .code(),
        Code::Unavailable
    );
    assert_eq!(
        installation
            .setup_status(())
            .await
            .err()
            .ok_or("Expected admission withdrawal")?
            .code(),
        Code::Unavailable
    );
    Ok(())
}

#[tokio::test]
async fn two_listeners_enforce_surfaces_credentials_and_typed_ingress() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    support::scope(async |fixture| {
        let path = directory.path().join("rpc.sqlite");
        let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([2; 32]),
        )?)?);
        prepare_sqlite(&path, &cipher)?;
        let store = Store::connect_sqlite(path, cipher).await?;
        let proof = BearerSecret::generate()?;
        let application = Arc::new(AccountsService::new(
            store,
            PasswordWork::new(2, PasswordCost::new(10)?)?,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".into(),
                setup_proof: Some(proof.digest()),
                session_policy: SessionPolicy::default(),
            },
        )?);
        let admission = Arc::new(AtomicBool::new(true));
        let adapter = AccountServices::new(application).with_admission(admission.clone());
        let general_listener = TcpListener::bind("127.0.0.1:0").await?;
        let admin_listener = TcpListener::bind("127.0.0.1:0").await?;
        let general_address = general_listener.local_addr()?;
        let admin_address = admin_listener.local_addr()?;
        let general_stopped = fixture.required_shutdown("General shutdown receiver dropped");
        let admin_stopped = fixture.required_shutdown("Admin shutdown receiver dropped");
        let general_auth = adapter.authentication_server();
        let lifecycle = adapter.account_lifecycle_server();
        let teams = adapter.teams_server();
        let invitations = adapter.invitations_server();
        let general_preferences = adapter.personal_preferences_server();
        let admin_auth = adapter.authentication_server();
        let installation = adapter.installation_server();
        let grants = adapter.account_grants_server();
        let defaults = adapter.display_defaults_server();
        let admin_preferences = adapter.personal_preferences_server();
        let workspaces = adapter.admin_workspaces_server();
        fixture.server(async move {
            Server::builder()
                .add_service(general_auth)
                .add_service(general_preferences)
                .add_service(lifecycle)
                .add_service(teams)
                .add_service(invitations)
                .serve_with_incoming_shutdown(TcpListenerStream::new(general_listener), async {
                    match general_stopped.await {
                        Ok(()) | Err(_) => {}
                    }
                })
                .await
        });
        fixture.server(async move {
            Server::builder()
                .add_service(admin_auth)
                .add_service(admin_preferences)
                .add_service(installation)
                .add_service(grants)
                .add_service(defaults)
                .add_service(workspaces)
                .serve_with_incoming_shutdown(TcpListenerStream::new(admin_listener), async {
                    match admin_stopped.await {
                        Ok(()) | Err(_) => {}
                    }
                })
                .await
        });
        let general = Channel::from_shared(format!("http://{general_address}"))?
            .connect()
            .await?;
        let admin = Channel::from_shared(format!("http://{admin_address}"))?
            .connect()
            .await?;
        exercise(general, admin, &proof, admission).await
    })
    .await
}
