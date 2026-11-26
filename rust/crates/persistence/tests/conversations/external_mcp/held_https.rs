//! Test-only TLS transport for an already admitted target. Production
//! resolution and certificate policy are exercised separately in the service.
use super::*;
use chapchoo_persistence::McpTestTarget;
use rustls::{ClientConfig, RootCertStore, ServerConfig, pki_types::PrivatePkcs8KeyDer};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};
use tokio_rustls::{TlsAcceptor, TlsConnector};

const HOST: &str = "mcp-fixture.invalid";
const REQUEST: &str =
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fixture"}}"#;
const RESPONSE: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"resultType":"complete","structuredContent":{"proof":"admitted"}}}"#;

pub(super) struct HeldHttps {
    listener: TcpListener,
    server: TlsAcceptor,
    client: TlsConnector,
}

impl HeldHttps {
    pub(super) async fn open() -> Result<Self, TestError> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let cert = rcgen::generate_simple_self_signed(vec![HOST.into()])?;
        let mut trust = RootCertStore::empty();
        trust.add(cert.cert.der().clone())?;
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let server = ServerConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.cert.der().clone()],
                PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der()).into(),
            )?;
        let client = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .with_root_certificates(trust)
            .with_no_client_auth();
        Ok(Self {
            listener,
            server: Arc::new(server).into(),
            client: Arc::new(client).into(),
        })
    }

    pub(super) fn endpoint(&self) -> Result<McpEndpoint, TestError> {
        Ok(McpEndpoint::parse(&format!(
            "https://{HOST}:{}/stream",
            self.listener.local_addr()?.port()
        ))?)
    }

    pub(super) async fn while_response_pending(
        self,
        admitted: McpTestTarget,
        action: impl Future<Output = Result<(), TestError>>,
    ) -> Result<(), TestError> {
        assert_eq!(admitted.endpoint.url(), self.endpoint()?.url());
        let address = self.listener.local_addr()?;
        let (observed, request_seen) = oneshot::channel();
        let (release, allow_reply) = oneshot::channel();
        let server = async {
            let (socket, _) = self.listener.accept().await?;
            let mut stream = BufReader::new(self.server.accept(socket).await?);
            let mut line = String::new();
            stream.read_line(&mut line).await?;
            assert_eq!(line, "POST /stream HTTP/1.1\r\n");
            let mut length = None;
            let mut authenticated = false;
            loop {
                line.clear();
                if stream.read_line(&mut line).await? == 0 {
                    return Err::<(), TestError>("Incomplete fixture headers".into());
                }
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.strip_prefix("Content-Length: ") {
                    length = Some(value.trim().parse::<usize>()?);
                }
                if let Some(value) = line.strip_prefix("Authorization: Bearer ") {
                    authenticated = value.trim() == NEXT_TOKEN;
                }
            }
            assert!(authenticated);
            assert_eq!(length, Some(REQUEST.len()));
            let mut bytes = vec![0; REQUEST.len()];
            stream.read_exact(&mut bytes).await?;
            assert_eq!(bytes, REQUEST.as_bytes());
            observed.send(()).map_err(|()| "No deletion waiter")?;
            allow_reply.await?;
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{RESPONSE}",
                RESPONSE.len()
            );
            stream.get_mut().write_all(reply.as_bytes()).await?;
            stream.get_mut().shutdown().await?;
            Ok(())
        };
        let request = async {
            let socket = TcpStream::connect(address).await?;
            let mut stream = self.client.connect(HOST.try_into()?, socket).await?;
            let token = admitted.bearer.ok_or("Missing admitted credential")?;
            let token = std::str::from_utf8(token.as_bytes())?;
            let bytes = zeroize::Zeroizing::new(format!(
                "POST /stream HTTP/1.1\r\nHost: {HOST}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{REQUEST}",
                REQUEST.len()
            ));
            stream.write_all(bytes.as_bytes()).await?;
            let mut reply = String::new();
            stream.read_to_string(&mut reply).await?;
            assert!(reply.starts_with("HTTP/1.1 200 OK\r\n"));
            assert_eq!(
                reply.split_once("\r\n\r\n").map(|(_, body)| body),
                Some(RESPONSE)
            );
            Ok::<_, TestError>(())
        };
        let delete = async {
            request_seen.await?;
            action.await?;
            release
                .send(())
                .map_err(|()| "Fixture closed before deletion committed")?;
            Ok::<_, TestError>(())
        };
        tokio::time::timeout(Duration::from_secs(15), async {
            tokio::try_join!(server, request, delete)
        })
        .await??;
        Ok(())
    }
}
