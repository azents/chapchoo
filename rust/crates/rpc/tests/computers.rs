mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey,
};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::computers::{
    self as wire, agents_client::AgentsClient, computers_client::ComputersClient,
    provider_catalog_client::ProviderCatalogClient,
    provider_inventory_client::ProviderInventoryClient,
};
use chapchoo_rpc::{ComputerServices, memory_transport};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, ComputerRequest, ComputersService,
    PasswordWork, ProviderCapabilities, ProviderSource, ResourceEvidence, ResourceReport,
    SessionPolicy,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Code, Request, transport::Server};
type Error = Box<dyn std::error::Error + Send + Sync>;
fn auth<T>(value: T, token: &BearerSecret) -> Result<Request<T>, Error> {
    let mut r = Request::new(value);
    r.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token.expose()).parse()?,
    );
    Ok(r)
}
fn agent(value: wire::CreateAgentResult) -> Result<wire::Agent, Error> {
    match value.outcome {
        Some(wire::create_agent_result::Outcome::Agent(agent)) => Ok(agent),
        _ => Err("Expected live Agent".into()),
    }
}
async fn account(
    accounts: &AccountsService,
    admin: &BearerSecret,
    email: &str,
) -> Result<BearerSecret, Error> {
    let email = EmailAddress::parse(email)?;
    let now = AcceptanceTime::System.now()?;
    let grant = accounts
        .issue_signup_grant(&admin.digest(), &email, now)
        .await?;
    Ok(accounts
        .redeem_signup(
            &grant.proof,
            email,
            Password::parse("Aa1!fixture-user".into())?,
            AcceptanceTime::System,
            None,
        )
        .await?
        .token)
}
#[test]
fn domain_rpc_keeps_private_owner_authority_and_independent_admin_residual_inventory()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("computers.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([37; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let setup = BearerSecret::generate()?;
    tokio::runtime::Runtime::new()?.block_on(support::scope(async |fixture| {
        let store = Store::connect_sqlite(path, cipher).await?;
        let accounts = AccountsService::new(
            store.clone(),
            PasswordWork::new(1, PasswordCost::new(10)?)?,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".into(),
                setup_proof: Some(setup.digest()),
                session_policy: SessionPolicy::default(),
            },
        )?;
        let admin = accounts
            .initialize(
                &setup,
                EmailAddress::parse("admin@example.invalid")?,
                Password::parse("Aa1!fixture-admin".into())?,
                AcceptanceTime::System,
                None,
            )
            .await?
            .token;
        let user = account(&accounts, &admin, "user@example.invalid").await?;
        let foreign = account(&accounts, &admin, "foreign@example.invalid").await?;
        let foreign_team = accounts
            .list_teams(&foreign.digest(), AcceptanceTime::System.now()?)
            .await?
            .remove(0);
        let team = accounts
            .list_teams(&user.digest(), AcceptanceTime::System.now()?)
            .await?
            .remove(0);
        let authentication_service = Arc::new(chapchoo_service::AuthenticationService::new(
            store.clone(),
            Arc::new(support::authentication::Provider::default()),
            1,
        )?);
        let authentication =
            chapchoo_rpc::AuthenticationServices::new(authentication_service.clone());
        let stopped = fixture.shutdown();
        fixture.server(async move {
            let _ = stopped.await;
            authentication_service.close();
            authentication_service.finish().await
        });
        let app = Arc::new(ComputersService::new(store.clone()));
        assert!(
            app.register_control("https://control.example.invalid:50053")
                .await
                .is_err()
        );
        let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
        let key = app
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
                &chapchoo_domain::ProviderSubstrate::Kubernetes,
                &format!(
                    "{}:key",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis()
                ),
                AcceptanceTime::System,
            )
            .await?;
        let control = app.register_control("http://127.0.0.1:1").await?;
        let prepared = app
            .prepare_provider_connection(&key.provider_id, &key.key.token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&prepared).await?;
        let admission = Arc::new(AtomicBool::new(true));
        let services = ComputerServices::new(app.clone()).with_admission(admission.clone());
        let (channel, incoming) = memory_transport();
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(services.agents_server())
                .add_service(services.computers_server())
                .add_service(services.catalog_server())
                .add_service(services.inventory_server())
                .add_service(authentication.server())
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = stopped.await;
                }),
        );
        let mut agents = AgentsClient::new(channel.clone());
        let mut computers = ComputersClient::new(channel.clone());
        let mut catalog = ProviderCatalogClient::new(channel.clone());
        let mut inventory = ProviderInventoryClient::new(channel.clone());
        let choices = catalog
            .list(auth(wire::Page::default(), &user)?)
            .await?
            .into_inner();
        assert_eq!(choices.items.len(), 1);
        assert_eq!(
            inventory
                .list_providers(auth(wire::Page::default(), &user)?)
                .await
                .err()
                .ok_or("Non-admin inventory")?
                .code(),
            Code::PermissionDenied
        );
        let creation = wire::CreateAgentRequest {
            team_id: team.id.clone(),
            authentication_id: support::authentication::register_personal(
                &store,
                &user.digest(),
                AcceptanceTime::System.now()?,
            )
            .await?,
            request_id: "create".into(),
            provider_id: key.provider_id.clone(),
            name: "Private Agent name".into(),
            handle: "private-agent".into(),
            profile_image_id: app
                .agent_profile_images(&user.digest(), None, 1, AcceptanceTime::System)
                .await?[0]
                .id
                .clone(),
            palette_key: "sage".into(),
        };
        let created = agent(
            agents
                .create(auth(creation.clone(), &user)?)
                .await?
                .into_inner(),
        )?;
        let replay = agent(
            agents
                .create(auth(creation.clone(), &user)?)
                .await?
                .into_inner(),
        )?;
        assert_eq!(created.id, replay.id);
        assert_eq!(
            created
                .authentication
                .as_ref()
                .ok_or("Missing creation selection")?
                .authentication_id,
            creation.authentication_id
        );
        use chapchoo_protocol::authentication as auth_wire;
        let mut authentications =
            auth_wire::authentications_client::AuthenticationsClient::new(channel.clone());
        let personal_scope = Some(auth_wire::Scope {
            owner: Some(auth_wire::scope::Owner::Personal(())),
        });
        let registered = authentications
            .register_key(auth(
                auth_wire::RegisterKeyRequest {
                    scope: personal_scope.clone(),
                    request_id: "register-auth".into(),
                    name: "Selected key".into(),
                    key: b"fixture-key".to_vec(),
                },
                &user,
            )?)
            .await?
            .into_inner();
        let Some(auth_wire::accepted_result::Result::Record(registered)) = registered.result else {
            return Err(Error::from("Missing credential identity"));
        };
        let selection = auth_wire::SelectAgentRequest {
            request_id: "select-auth".into(),
            agent_id: created.id.clone(),
            authentication_id: registered.authentication_id.clone(),
            expected: Some(auth_wire::ExpectedSelection {
                authentication_id: creation.authentication_id.clone(),
                request_id: creation.request_id.clone(),
            }),
        };
        assert_eq!(
            authentications
                .select_agent(auth(selection.clone(), &foreign)?)
                .await
                .err()
                .ok_or("Foreign Agent management")?
                .code(),
            Code::NotFound
        );
        let selected = authentications
            .select_agent(auth(selection.clone(), &user)?)
            .await?
            .into_inner();
        assert!(matches!(
            selected.result,
            Some(auth_wire::accepted_result::Result::Selection(_))
        ));
        assert_eq!(
            authentications
                .select_agent(auth(selection, &user)?)
                .await?
                .into_inner(),
            selected
        );
        let with_selection = agents
            .get(auth(
                wire::AgentRequest {
                    agent_id: created.id.clone(),
                },
                &user,
            )?)
            .await?
            .into_inner();
        let saved = with_selection
            .authentication
            .ok_or("Missing saved reference projection")?;
        assert_eq!(saved.authentication_id, registered.authentication_id);
        assert_eq!(saved.request_id, "select-auth");
        assert_eq!(with_selection.computer, created.computer);
        let in_use = authentications
            .delete(auth(
                auth_wire::DeleteRequest {
                    scope: personal_scope,
                    request_id: "delete-selected".into(),
                    authentication_id: registered.authentication_id,
                    expected_request_id: "register-auth".into(),
                },
                &user,
            )?)
            .await?
            .into_inner();
        assert!(matches!(
            in_use.outcome,
            Some(auth_wire::delete_result::Outcome::InUse(()))
        ));
        assert!(!created.profile_image_asset_key.is_empty());
        let scoped = wire::TeamAgentRequest {
            team_id: team.id.clone(),
            handle: created.handle.clone(),
        };
        assert_eq!(
            agents
                .get_in_team(auth(scoped.clone(), &user)?)
                .await?
                .into_inner()
                .id,
            created.id
        );
        for (scope, caller) in [
            (scoped.clone(), &foreign),
            (
                wire::TeamAgentRequest {
                    team_id: foreign_team.id.clone(),
                    ..scoped
                },
                &user,
            ),
        ] {
            assert_eq!(
                agents
                    .get_in_team(auth(scope, caller)?)
                    .await
                    .err()
                    .ok_or("Scoped private access")?
                    .code(),
                Code::NotFound
            );
        }
        let edit = wire::UpdateAgentProfileRequest {
            agent_id: created.id.clone(),
            name: "Saved profile".into(),
            profile_image_id: created.profile_image_id.clone(),
            palette_key: "rose".into(),
        };
        assert_eq!(
            agents
                .update_profile(auth(edit.clone(), &foreign)?)
                .await
                .err()
                .ok_or("Foreign edit")?
                .code(),
            Code::NotFound
        );
        let saved = agents
            .update_profile(auth(edit, &user)?)
            .await?
            .into_inner();
        assert_eq!(saved.name, "Saved profile");
        assert_eq!(saved.handle, created.handle);
        assert_eq!(saved.palette_key, "rose");
        let history = wire::CreationResultRequest {
            request_id: creation.request_id.clone(),
        };
        assert!(
            agents
                .creation_result(auth(history.clone(), &foreign)?)
                .await?
                .into_inner()
                .outcome
                .is_none()
        );
        assert!(matches!(
            agents.creation_result(auth(history.clone(), &user)?).await?.into_inner().outcome,
            Some(wire::historical_creation_result::Outcome::Present(ref value))
                if value.assigned_agent_id == created.id && value.team_id == team.id
        ));
        let request = wire::AgentRequest {
            agent_id: created.id.clone(),
        };
        assert_eq!(
            agents
                .get(auth(request.clone(), &foreign)?)
                .await
                .err()
                .ok_or("Foreign private Agent")?
                .code(),
            Code::NotFound
        );
        assert_eq!(
            agents
                .get(auth(request.clone(), &admin)?)
                .await
                .err()
                .ok_or("Admin private Agent access")?
                .code(),
            Code::NotFound
        );
        assert_eq!(
            computers
                .get(auth(request.clone(), &foreign)?)
                .await
                .err()
                .ok_or("Foreign Computer")?
                .code(),
            Code::NotFound
        );
        let retry = wire::PreparationRetryRequest {
            agent_id: created.id.clone(),
            request_id: "retry".into(),
            expected_request_id: "create".into(),
        };
        assert_eq!(
            computers
                .retry_preparation(auth(retry.clone(), &foreign)?)
                .await
                .err()
                .ok_or("Foreign preparation retry")?
                .code(),
            Code::NotFound
        );
        assert!(
            computers
                .retry_preparation(auth(retry, &user)?)
                .await
                .is_err(),
            "Preparation without positive safe evidence must not be retried"
        );
        let computer = created.computer.as_ref().ok_or("Missing Computer")?;
        assert_eq!(computer.power, wire::Power::Unknown as i32);
        assert_eq!(computer.connector, wire::Connectivity::Disconnected as i32);
        assert_eq!(computer.management, wire::Connectivity::Connected as i32);
        assert!(
            computer
                .actions
                .iter()
                .any(|a| a.action == wire::Action::Start as i32 && !a.available)
        );
        let scope = ComputerRequest {
            computer_id: &computer.id,
            request_id: "create",
        };
        let source = ProviderSource {
            provider_id: &key.provider_id,
            control_id: &control.id,
            connection_id: &prepared.owner().connection_id,
            credential: &key.key.token.digest(),
        };
        let execution = app
            .reserve_execution(
                &source,
                scope,
                &chapchoo_domain::NativeScope::kubernetes("fixture".into())?,
            )
            .await?;
        for resource in &execution.resources {
            app.report_resource(
                &source,
                scope,
                ResourceReport {
                    native: resource.native(),
                    execution_id: resource.execution_id.as_deref(),
                    evidence: ResourceEvidence::OwnedPresent {
                        identity: &chapchoo_domain::NativeIdentity::KubernetesObject(
                            chapchoo_domain::KubernetesUid::parse(format!(
                                "uid-{}",
                                resource.kind().as_str()
                            ))?,
                        ),
                    },
                },
            )
            .await?;
        }
        app.report_execution_running(&source, scope, &execution.execution_id)
            .await?;
        let running = computers
            .get(auth(request.clone(), &user)?)
            .await?
            .into_inner();
        assert_eq!(running.power, wire::Power::Running as i32);
        assert!(
            running
                .actions
                .iter()
                .any(|a| a.action == wire::Action::Stop as i32 && a.available)
        );
        assert_eq!(running.connector, wire::Connectivity::Disconnected as i32);
        let invalid = wire::LifecycleRequest {
            agent_id: created.id.clone(),
            request_id: "bad".into(),
            expected_request_id: "create".into(),
            action: 999,
        };
        assert_eq!(
            computers
                .request_lifecycle(auth(invalid, &user)?)
                .await
                .err()
                .ok_or("Invalid action")?
                .code(),
            Code::InvalidArgument
        );
        let list = inventory
            .list_computers(auth(
                wire::InventoryRequest {
                    provider_id: key.provider_id.clone(),
                    page: None,
                },
                &admin,
            )?)
            .await?
            .into_inner();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].power, running.power);
        assert_eq!(list.items[0].management, running.management);
        let page = inventory
            .list_resources(auth(
                wire::ResourcePage {
                    computer_id: computer.id.clone(),
                    limit: 1,
                    after: None,
                },
                &admin,
            )?)
            .await?
            .into_inner();
        assert_eq!(page.items.len(), 1);
        let later = inventory
            .list_resources(auth(
                wire::ResourcePage {
                    computer_id: computer.id.clone(),
                    limit: 20,
                    after: page.next_after,
                },
                &admin,
            )?)
            .await?
            .into_inner();
        assert_eq!(later.items.len(), 2);
        let deletion = wire::LifecycleRequest {
            agent_id: created.id.clone(),
            request_id: "delete".into(),
            expected_request_id: "create".into(),
            action: wire::Action::Delete as i32,
        };
        let deleting = computers
            .request_lifecycle(auth(deletion, &user)?)
            .await?
            .into_inner();
        assert!(
            deleting
                .computer
                .ok_or("Deleting Computer")?
                .force_remove_available
        );
        agents
            .force_remove(auth(
                wire::ForceRemoveRequest {
                    agent_id: created.id.clone(),
                    deletion_request_id: "delete".into(),
                },
                &user,
            )?)
            .await?;
        assert_eq!(
            agents
                .get(auth(request, &user)?)
                .await
                .err()
                .ok_or("Removed Agent remains")?
                .code(),
            Code::NotFound
        );
        let residual = inventory
            .list_computers(auth(
                wire::InventoryRequest {
                    provider_id: key.provider_id.clone(),
                    page: None,
                },
                &admin,
            )?)
            .await?
            .into_inner();
        assert!(residual.items[0].residual);
        assert_eq!(residual.items[0].id, computer.id);
        assert!(matches!(
            agents
                .create(auth(creation, &user)?)
                .await?
                .into_inner()
                .outcome,
            Some(wire::create_agent_result::Outcome::Removed(_))
        ));
        assert!(matches!(
            agents.creation_result(auth(history, &user)?).await?.into_inner().outcome,
            Some(wire::historical_creation_result::Outcome::Removed(ref value))
                if value.assigned_agent_id == created.id
        ));
        assert!(
            agents
                .list(auth(
                    wire::AgentListRequest {
                        team_id: foreign_team.id,
                        page: Some(wire::Page::default()),
                    },
                    &foreign,
                )?)
                .await?
                .into_inner()
                .items
                .is_empty()
        );
        assert_eq!(
            agents
                .list(auth(
                    wire::AgentListRequest {
                        team_id: team.id,
                        page: Some(wire::Page {
                            limit: 21,
                            after: None
                        }),
                    },
                    &user
                )?)
                .await
                .err()
                .ok_or("Unbounded page")?
                .code(),
            Code::InvalidArgument
        );
        admission.store(false, Ordering::Release);
        assert_eq!(
            catalog
                .list(auth(wire::Page::default(), &user)?)
                .await
                .err()
                .ok_or("Closed admission")?
                .code(),
            Code::Unavailable
        );
        drop((agents, computers, catalog, inventory, channel));
        Ok(())
    }))
}
