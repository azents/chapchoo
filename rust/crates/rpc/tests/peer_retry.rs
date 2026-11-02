//! Peer retry decisions are tested independently of component stream behavior.
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey, SystemOwnerVerifier};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::control::{
    self as wire,
    peer_relay_server::{PeerRelay, PeerRelayServer},
};
use chapchoo_rpc::{PeerDelivery, memory_transport};
use chapchoo_service::{AcceptanceTime, ComputersService, ProviderCapabilities};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::oneshot, time::Instant};
use tonic::{Request, Response, Status, transport::Server};
type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone)]
struct Probe {
    calls: Arc<Mutex<Vec<wire::ProviderInvocation>>>,
    native_error: bool,
}
#[tonic::async_trait]
impl PeerRelay for Probe {
    async fn converge_provider(
        &self,
        _request: Request<wire::ProviderConvergence>,
    ) -> Result<Response<wire::ProviderInvocationResult>, Status> {
        Err(Status::unimplemented("Probe only"))
    }
    async fn invoke_provider(
        &self,
        request: Request<wire::ProviderInvocation>,
    ) -> Result<Response<wire::ProviderInvocationResult>, Status> {
        let mut calls = self
            .calls
            .lock()
            .map_err(|_| Status::internal("Probe lock failed"))?;
        calls.push(request.into_inner());
        if self.native_error {
            return Err(Status::unavailable("Injected transport-style failure"));
        }
        Ok(Response::new(wire::ProviderInvocationResult {
            outcome: if calls.len() == 1 {
                wire::ForwardingOutcome::NotOwner
            } else {
                wire::ForwardingOutcome::Completed
            } as i32,
            result: None,
        }))
    }
    async fn invoke_connector(
        &self,
        _request: Request<wire::ConnectorInvocation>,
    ) -> Result<Response<wire::ConnectorInvocationResult>, Status> {
        Err(Status::unimplemented("Probe only"))
    }
}
#[test]
fn explicit_not_owner_is_the_only_bounded_retry_and_native_error_is_uncertain() -> Result<(), Error>
{
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("retry.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([35; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let owner = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("system".into(), vec!["provider".into()], owner.digest())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let app = Arc::new(ComputersService::new(
            Store::connect_sqlite(path, cipher).await?,
            Some(verifier),
        ));
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
        for native_error in [false, true] {
            let probe = Probe {
                calls: Arc::new(Mutex::new(vec![])),
                native_error,
            };
            let calls = probe.calls.clone();
            let (channel, incoming) = memory_transport();
            let (stop, stopped) = oneshot::channel();
            let server = tokio::spawn(
                Server::builder()
                    .add_service(PeerRelayServer::new(probe))
                    .serve_with_incoming_shutdown(incoming, async {
                        let _ = stopped.await;
                    }),
            );
            let delivery = PeerDelivery::new(app.clone());
            delivery.register_local(control.id.clone(), channel)?;
            let result = delivery
                .provider(
                    wire::ProviderInvocation {
                        provider_id: "provider".into(),
                        command: Some(wire::ProviderCommand {
                            identity: Some(wire::CallIdentity {
                                request_id: "stable-request".into(),
                                call_id: "replaced".into(),
                                connection_id: String::new(),
                            }),
                            remaining_ms: 10_000,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    Instant::now() + Duration::from_secs(5),
                )
                .await?;
            {
                let calls = calls.lock().map_err(|_| "Probe lock failed")?;
                if native_error {
                    assert_eq!(result.outcome, wire::ForwardingOutcome::Uncertain as i32);
                    assert_eq!(calls.len(), 1);
                } else {
                    assert_eq!(result.outcome, wire::ForwardingOutcome::Completed as i32);
                    assert_eq!(calls.len(), 2);
                    let first = calls[0]
                        .command
                        .as_ref()
                        .and_then(|c| c.identity.as_ref())
                        .ok_or("First identity")?;
                    let second = calls[1]
                        .command
                        .as_ref()
                        .and_then(|c| c.identity.as_ref())
                        .ok_or("Second identity")?;
                    assert_eq!(first.request_id, second.request_id);
                    assert_ne!(first.call_id, second.call_id);
                    assert_eq!(second.connection_id, prepared.owner().connection_id);
                    assert!(
                        calls[1]
                            .command
                            .as_ref()
                            .ok_or("Second command")?
                            .remaining_ms
                            <= calls[0]
                                .command
                                .as_ref()
                                .ok_or("First command")?
                                .remaining_ms
                    );
                }
            }
            let _ = stop.send(());
            drop(delivery);
            server.await??;
        }
        Ok(())
    })
}
