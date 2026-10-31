use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey};
use chapchoo_persistence::{SessionPolicy, Store, prepare_sqlite};
use chapchoo_protocol::accounts::{
    self as wire, admin_client::AdminClient, auth_client::AuthClient, general_client::GeneralClient,
};
use chapchoo_rpc::{DOMAIN_REASON_METADATA, RpcService};
use chapchoo_service::{AccountsConfig, AccountsService, PasswordWork};
use tokio::{net::TcpListener, sync::oneshot};
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
    let mut admin_rpc = AdminClient::new(admin.clone());
    let mut general_rpc = GeneralClient::new(general.clone());
    let mut auth_general = AuthClient::new(general.clone());
    let mut auth_admin = AuthClient::new(admin.clone());
    assert!(admin_rpc.setup_status(()).await?.into_inner().available);
    let owner = admin_rpc
        .initialize(wire::InitializeRequest {
            installation_proof: proof.expose().to_owned(),
            email: "Owner@Example.Invalid".into(),
            password: "Aa1!owner".into(),
            display: None,
        })
        .await?
        .into_inner();
    let owner_id = owner.user.as_ref().ok_or("Missing owner user")?.id.clone();
    assert!(!admin_rpc.setup_status(()).await?.into_inner().available);
    assert!(auth_general.methods(()).await?.into_inner().password);
    assert!(!auth_admin.methods(()).await?.into_inner().email_code);

    // Common Auth is available on both listeners. Dedicated business surfaces
    // stay absent on the opposite listener, including setup/bootstrap methods.
    assert_eq!(
        AdminClient::new(general.clone())
            .setup_status(())
            .await
            .err()
            .ok_or("Expected absent Admin service")?
            .code(),
        Code::Unimplemented
    );
    assert_eq!(
        GeneralClient::new(admin.clone())
            .list_teams(authenticated((), &owner.token)?)
            .await
            .err()
            .ok_or("Expected absent General service")?
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

    let grant = admin_rpc
        .issue_signup_grant(authenticated(
            wire::IssueGrantRequest {
                email: "member@example.invalid".into(),
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    let preview = general_rpc
        .preview_signup(wire::ProofRequest {
            proof: grant.proof.clone(),
        })
        .await?
        .into_inner();
    assert!(preview.usable);
    assert_eq!(preview.email_hint.as_deref(), Some("m***@example.invalid"));
    assert!(preview.expires_at.is_some());
    let member = general_rpc
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
    let used = general_rpc
        .preview_signup(wire::ProofRequest { proof: grant.proof })
        .await?
        .into_inner();
    assert!(!used.usable);
    assert!(used.email_hint.is_none());
    assert!(used.expires_at.is_none());
    let mut first = AuthClient::new(general.clone());
    let mut second = AuthClient::new(general.clone());
    let (owner_read, member_read) = tokio::join!(
        first.identity(authenticated((), &owner.token)?),
        second.identity(authenticated((), &member.token)?)
    );
    assert_eq!(owner_read?.into_inner().id, owner_id);
    assert_eq!(member_read?.into_inner().id, member_id);
    assert_reason(
        &admin_rpc
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

    let team = general_rpc
        .create_team(authenticated(
            wire::CreateTeamRequest {
                name: "Team".into(),
                handle: "rpc-team".into(),
            },
            &owner.token,
        )?)
        .await?
        .into_inner();
    let unchanged = general_rpc
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
        &general_rpc
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
            &general_rpc
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
        &general_rpc
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
        admin_rpc
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
    let adapter = RpcService::new(application).with_admission(admission.clone());
    let general_listener = TcpListener::bind("127.0.0.1:0").await?;
    let admin_listener = TcpListener::bind("127.0.0.1:0").await?;
    let general_address = general_listener.local_addr()?;
    let admin_address = admin_listener.local_addr()?;
    let (general_stop, general_stopped) = oneshot::channel();
    let (admin_stop, admin_stopped) = oneshot::channel();
    let general_auth = adapter.auth_server();
    let general_service = adapter.general_server();
    let admin_auth = adapter.auth_server();
    let admin_service = adapter.admin_server();
    let general_task = tokio::spawn(async move {
        Server::builder()
            .add_service(general_auth)
            .add_service(general_service)
            .serve_with_incoming_shutdown(TcpListenerStream::new(general_listener), async {
                match general_stopped.await {
                    Ok(()) | Err(_) => {}
                }
            })
            .await
    });
    let admin_task = tokio::spawn(async move {
        Server::builder()
            .add_service(admin_auth)
            .add_service(admin_service)
            .serve_with_incoming_shutdown(TcpListenerStream::new(admin_listener), async {
                match admin_stopped.await {
                    Ok(()) | Err(_) => {}
                }
            })
            .await
    });
    let result = async {
        let general = Channel::from_shared(format!("http://{general_address}"))?
            .connect()
            .await?;
        let admin = Channel::from_shared(format!("http://{admin_address}"))?
            .connect()
            .await?;
        exercise(general, admin, &proof, admission).await
    }
    .await;
    // Collect every owned task before propagating any test/cleanup failure.
    let general_signal = general_stop.send(());
    let admin_signal = admin_stop.send(());
    let (general_exit, admin_exit) = tokio::join!(general_task, admin_task);
    general_signal.map_err(|_| "General shutdown receiver dropped")?;
    admin_signal.map_err(|_| "Admin shutdown receiver dropped")?;
    general_exit??;
    admin_exit??;
    result
}
