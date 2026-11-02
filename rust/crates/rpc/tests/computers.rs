use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey, SystemOwnerVerifier};
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
    ManagedResourceKind, PasswordWork, ProviderCapabilities, ProviderSource, ResourceEvidence,
    ResourceReport, SessionPolicy,
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
    let owner = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("system".into(), vec!["provider".into()], owner.digest())?;
    tokio::runtime::Runtime::new()?.block_on(async {
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
        let app = Arc::new(ComputersService::new(store, Some(verifier)));
        assert!(
            app.register_control("https://control.example.invalid:50053")
                .await
                .is_err()
        );
        app.configure_system_provider(
            &owner,
            "provider",
            "Fixture",
            ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            },
            AcceptanceTime::System,
        )
        .await?;
        let key = app
            .rotate_system_provider_key(&owner, "provider", "key", AcceptanceTime::System)
            .await?;
        let control = app.register_control("http://127.0.0.1:1").await?;
        let prepared = app
            .prepare_provider_connection("provider", &key.token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&prepared).await?;
        let admission = Arc::new(AtomicBool::new(true));
        let services = ComputerServices::new(app.clone()).with_admission(admission.clone());
        let (channel, incoming) = memory_transport();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(
            Server::builder()
                .add_service(services.agents_server())
                .add_service(services.computers_server())
                .add_service(services.catalog_server())
                .add_service(services.inventory_server())
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
            request_id: "create".into(),
            provider_id: "provider".into(),
            name: "Private Agent name".into(),
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
            provider_id: "provider",
            control_id: &control.id,
            connection_id: &prepared.owner().connection_id,
            credential: &key.token.digest(),
        };
        let execution = app.reserve_execution(&source, scope, "fixture").await?;
        for resource in &execution.resources {
            let kind = match resource.resource_kind.as_str() {
                "POD" => ManagedResourceKind::Pod,
                "PVC" => ManagedResourceKind::Pvc,
                _ => ManagedResourceKind::BootstrapSecret,
            };
            app.report_resource(
                &source,
                scope,
                ResourceReport {
                    kind,
                    namespace: &resource.namespace,
                    name: &resource.name,
                    execution_id: resource.execution_id.as_deref(),
                    evidence: ResourceEvidence::OwnedPresent {
                        uid: &format!("uid-{}", resource.resource_kind),
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
                    provider_id: "provider".into(),
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
                    provider_id: "provider".into(),
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
        assert!(
            agents
                .list(auth(wire::Page::default(), &foreign)?)
                .await?
                .into_inner()
                .items
                .is_empty()
        );
        assert_eq!(
            agents
                .list(auth(
                    wire::Page {
                        limit: 21,
                        after: None
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
        let _ = stop.send(());
        drop((agents, computers, catalog, inventory, channel));
        server.await??;
        Ok(())
    })
}
