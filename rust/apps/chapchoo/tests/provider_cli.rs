#![cfg(unix)]
//! The external Provider starts using only its native credentials and Control RPC.

use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::control::{
    self as wire,
    provider_authentication_server::{ProviderAuthentication, ProviderAuthenticationServer},
};
use std::{process::Stdio, time::Duration};
use tokio::{net::TcpListener, process::Command, sync::mpsc};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};

type Error = Box<dyn std::error::Error + Send + Sync>;
#[path = "provider_cli/docker.rs"]
mod docker;
#[path = "../test_support.rs"]
mod test_support;

struct ControlBoundary {
    configured: mpsc::Sender<wire::ProviderConfiguration>,
    owner: BearerSecret,
}

#[tonic::async_trait]
impl ProviderAuthentication for ControlBoundary {
    async fn exchange(
        &self,
        request: Request<wire::ProviderAuthenticationRequest>,
    ) -> Result<Response<wire::ComponentKey>, Status> {
        let value = request.into_inner();
        if !matches!(&value.evidence,Some(wire::provider_authentication_request::Evidence::RegistrationToken(token)) if token==self.owner.expose())
        {
            return Err(Status::unauthenticated(
                "Provider owner authentication required",
            ));
        }
        self.configured
            .send(
                value
                    .configuration
                    .ok_or_else(|| Status::invalid_argument("Provider configuration missing"))?,
            )
            .await
            .map_err(|_| Status::unavailable("Fixture receiver closed"))?;
        // A deliberate RPC denial ends the child before any native resource work.
        Err(Status::permission_denied(
            "Fixture ends after authenticated registration",
        ))
    }

    async fn check_command(
        &self,
        _: Request<wire::ProviderCommandCheck>,
    ) -> Result<Response<wire::ProviderCommandAdmission>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn check_connection(
        &self,
        _: Request<wire::ProviderConnectionCheck>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn rotate_key(
        &self,
        _: Request<wire::ProviderKeyRequest>,
    ) -> Result<Response<wire::ComponentKey>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn revoke_key(
        &self,
        _: Request<wire::ProviderKeyRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn issue_connector_session(
        &self,
        _: Request<wire::ConnectorSessionRequest>,
    ) -> Result<Response<wire::ConnectorSession>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn revoke_connector_session(
        &self,
        _: Request<wire::RevokeConnectorSessionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }

    async fn report(&self, _: Request<wire::ProviderReport>) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("Not used by this boundary fixture"))
    }
}

#[tokio::test]
async fn external_kubernetes_provider_reaches_control_without_product_db_or_root()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    test_support::scope(async |fixture| {
        let kubeconfig = dir.path().join("kubeconfig");
        std::fs::write(
            &kubeconfig,
            r#"
apiVersion: v1
kind: Config
current-context: fixture
clusters:
- name: fixture
  cluster:
    server: http://127.0.0.1:9
contexts:
- name: fixture
  context:
    cluster: fixture
    user: fixture
users:
- name: fixture
  user: {}
"#,
        )?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let owner = BearerSecret::generate()?;
        let child_token = owner.expose().to_owned();
        let credential_file = dir.path().join("provider-credential");
        std::fs::write(&credential_file, &child_token)?;
        let (configured, mut received) = mpsc::channel(2);
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(ProviderAuthenticationServer::new(ControlBoundary {
                    configured,
                    owner,
                }))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                }),
        );
        let db_canary = dir.path().join("product-db-must-not-be-created.sqlite");
        for poison_backend_config in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_chapchoo"));
            command
                .env_clear()
                .env("CHAPCHOO_PROVIDER_AUTH_METHOD", "random-token")
                .env("CHAPCHOO_PROVIDER_AUTH_FILE", &credential_file)
                .arg("computer-provider")
                .arg("--endpoint")
                .arg(&endpoint)
                .arg("--provider-id")
                .arg("external-provider")
                .arg("--name")
                .arg("External fixture")
                .arg("--namespace")
                .arg("fixture")
                .arg("--image")
                .arg("fixture.invalid/computer:current")
                .arg("--service-account")
                .arg("fixture")
                .arg("--kubeconfig")
                .arg(&kubeconfig)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true);
            if poison_backend_config {
                // Loading BackendConfig would reject conflicting DB selection and
                // the invalid root. Provider startup must not inspect any of them.
                command
                    .env("CHAPCHOO_SQLITE_PATH", &db_canary)
                    .env("CHAPCHOO_POSTGRES_URL", "invalid-backend-database")
                    .env("CHAPCHOO_SECRET_KEY", "invalid-installation-root");
            }
            let child = fixture.child(&mut command)?;
            let configuration = tokio::time::timeout(Duration::from_secs(10), received.recv())
                .await?
                .ok_or("Provider did not send its configuration over RPC")?;
            assert_eq!(configuration.provider_id, "external-provider");
            assert_eq!(configuration.name, "External fixture");
            let status =
                tokio::time::timeout(Duration::from_secs(10), child.lock().await.wait()).await??;
            assert!(
                !status.success(),
                "The explicit registration denial must end startup"
            );
            assert!(!db_canary.exists());
        }
        Ok(())
    })
    .await
}
