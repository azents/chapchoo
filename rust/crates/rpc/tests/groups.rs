use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, PasswordCost, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::{
    accounts::{
        authentication_client::AuthenticationClient, installation_client::InstallationClient,
        team_management_client::TeamManagementClient,
    },
    control::{
        ProviderConnectionCheck, ProviderInvocation, peer_relay_client::PeerRelayClient,
        provider_authentication_client::ProviderAuthenticationClient,
    },
};
use chapchoo_rpc::{
    AccountServices, ComponentAuthenticationServices, ComponentConnectionServices, ServiceGroup,
    ServiceGroups, memory_transport,
};
use chapchoo_service::{
    AccountsConfig, AccountsService, ComputersService, PasswordWork, SessionPolicy,
};
use std::sync::{Arc, atomic::AtomicBool};
use tonic::{Code, transport::Server};
type Error = Box<dyn std::error::Error + Send + Sync>;
#[test]
fn selectable_group_unions_deduplicate_shared_services_and_exclude_peers() -> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("groups.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([36; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let account = AccountServices::new(Arc::new(AccountsService::new(
            store.clone(),
            PasswordWork::new(1, PasswordCost::new(10)?)?,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".into(),
                setup_proof: None,
                session_policy: SessionPolicy::default(),
            },
        )?));
        let app = Arc::new(ComputersService::new(store, None));
        let computers = chapchoo_rpc::ComputerServices::new(app.clone());
        let connections = ComponentConnectionServices::new(
            app.clone(),
            "control".into(),
            Arc::new(AtomicBool::new(true)),
        );
        let auth = ComponentAuthenticationServices::new(app, "control".into(), connections.clone());
        for groups in [
            vec![ServiceGroup::Client],
            vec![ServiceGroup::Admin],
            vec![ServiceGroup::Control],
            vec![
                ServiceGroup::Client,
                ServiceGroup::Admin,
                ServiceGroup::Control,
                ServiceGroup::Client,
                ServiceGroup::Admin,
            ],
        ] {
            let groups = ServiceGroups::new(groups);
            let (channel, incoming) = memory_transport();
            let (stop, stopped) = tokio::sync::oneshot::channel();
            let server = tokio::spawn(
                Server::builder()
                    .add_routes(groups.routes(
                        Some(&account),
                        Some(&computers),
                        Some((&auth, &connections)),
                    )?)
                    .serve_with_incoming_shutdown(incoming, async {
                        let _ = stopped.await;
                    }),
            );
            let shared = AuthenticationClient::new(channel.clone()).methods(()).await;
            assert_eq!(
                shared.is_ok(),
                groups.contains(ServiceGroup::Client) || groups.contains(ServiceGroup::Admin)
            );
            let admin = InstallationClient::new(channel.clone())
                .setup_status(())
                .await;
            assert_eq!(admin.is_ok(), groups.contains(ServiceGroup::Admin));
            let client = TeamManagementClient::new(channel.clone())
                .list_teams(())
                .await
                .err()
                .ok_or("Unauthenticated team request accepted")?;
            assert_eq!(
                client.code() == Code::Unimplemented,
                !groups.contains(ServiceGroup::Client)
            );
            let workspace =
                chapchoo_protocol::accounts::admin_workspaces_client::AdminWorkspacesClient::new(
                    channel.clone(),
                )
                .list_teams(())
                .await
                .err()
                .ok_or("Anonymous workspace request accepted")?;
            assert_eq!(
                workspace.code(),
                if groups.contains(ServiceGroup::Admin) {
                    Code::Unauthenticated
                } else {
                    Code::Unimplemented
                }
            );
            let provider = ProviderAuthenticationClient::new(channel.clone())
                .check_connection(ProviderConnectionCheck::default())
                .await
                .err()
                .ok_or("Anonymous component request accepted")?;
            assert_eq!(
                provider.code() == Code::Unimplemented,
                !groups.contains(ServiceGroup::Control)
            );
            assert_eq!(
                PeerRelayClient::new(channel.clone())
                    .invoke_provider(ProviderInvocation::default())
                    .await
                    .err()
                    .ok_or("Public peer service")?
                    .code(),
                Code::Unimplemented
            );
            let _ = stop.send(());
            drop(channel);
            server.await??;
        }
        assert!(ServiceGroups::default().routes(None, None, None).is_err());
        assert!(
            ServiceGroups::new([ServiceGroup::Control])
                .routes(None, None, None)
                .is_err()
        );
        assert!(
            ServiceGroups::new([ServiceGroup::Client])
                .routes(None, None, None)
                .is_err()
        );
        Ok(())
    })
}
