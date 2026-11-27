use super::*;
use chapchoo_persistence::McpBearer;
use rustls::{ClientConfig, RootCertStore, ServerConfig, pki_types::PrivatePkcs8KeyDer};
use serde_json::{Value, json};
use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

type TestError = Box<dyn Error + Send + Sync>;
const HOST: &str = "mcp-fixture.invalid";
const TOKEN: &str = "synthetic-mcp-fixture-bearer";

async fn tls_fixture()
-> Result<(TcpListener, TlsAcceptor, reqwest::Client, McpTestTarget), TestError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
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
    let tls = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_root_certificates(trust)
        .with_no_client_auth();
    let target = McpTestTarget {
        endpoint: McpEndpoint::parse(&format!("https://{HOST}:{}/mcp", address.port()))?,
        bearer: Some(McpBearer::parse(TOKEN.into())?),
    };
    // Unit fixture trust and address are injected here only. Production uses
    // platform TLS roots and a resolver that never yields loopback addresses.
    let client = http::OutboundMcpHttp::new()
        .builder(tls, &target)
        .map_err(|_| io::Error::other("could not build fixture HTTP client"))?
        .resolve(HOST, address)
        .build()?;
    Ok((
        listener,
        TlsAcceptor::from(Arc::new(server)),
        client,
        target,
    ))
}

async fn fixture(
    replies: Vec<(&'static str, &'static str, Value)>,
    version: &str,
    expected: Result<usize, McpDiscoveryFailure>,
) -> Result<(), TestError> {
    let (listener, acceptor, client, target) = tls_fixture().await?;
    let slot = Arc::new(Semaphore::new(1)).try_acquire_owned()?;
    let expected_methods = replies
        .iter()
        .map(|(method, _, _)| *method)
        .collect::<Vec<_>>();
    let version = version.to_owned();
    let server = async move {
        let mut methods = Vec::new();
        for (_, media, result) in replies {
            let (tcp, _) = listener.accept().await?;
            let mut stream = BufReader::new(acceptor.accept(tcp).await?);
            let mut line = String::new();
            stream.read_line(&mut line).await?;
            assert_eq!(line, "POST /mcp HTTP/1.1\r\n");
            let mut length = None;
            let mut authorized = false;
            let mut mcp_method = None;
            loop {
                line.clear();
                if stream.read_line(&mut line).await? == 0 {
                    return Err("incomplete request headers".into());
                }
                if line == "\r\n" {
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    match key.to_ascii_lowercase().as_str() {
                        "content-length" => length = Some(value.trim().parse::<usize>()?),
                        "authorization" => authorized = value.trim() == format!("Bearer {TOKEN}"),
                        "mcp-method" => mcp_method = Some(value.trim().to_owned()),
                        _ => {}
                    }
                }
            }
            assert!(authorized);
            let mut body = vec![0; length.ok_or("missing request content length")?];
            stream.read_exact(&mut body).await?;
            let body: Value = serde_json::from_slice(&body)?;
            let method = body["method"].as_str().ok_or("missing method")?;
            assert_eq!(mcp_method.as_deref(), Some(method));
            assert_eq!(
                body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
                "2026-07-28"
            );
            if methods.len() == 2 {
                assert_eq!(body["params"]["cursor"], "next");
            }
            methods.push(method.to_owned());
            let payload = if method == "server/discover" && result.is_null() {
                json!({
                    "supportedVersions": [version], "capabilities": {"tools": {}},
                    "resultType": if media == "incomplete" { "incomplete" } else { "complete" },
                    "ttlMs":0, "cacheScope":"private",
                    "_meta": {"io.modelcontextprotocol/serverInfo":{"name":"fixture","version":"1"}},
                })
            } else {
                result
            };
            let response = if media == "missing-discover" {
                json!({"jsonrpc": "2.0", "id": body["id"], "error":{"code":-32601,"message":"not found"}})
            } else {
                json!({"jsonrpc": "2.0", "id": body["id"], "result": payload})
            };
            let redirect = media == "redirect";
            let (status, media, bytes) = match media {
                "event" => (
                    "200 OK",
                    "text/event-stream",
                    format!("event: message\ndata: {response}\n\n"),
                ),
                "unauthorized" => ("401 Unauthorized", "text/plain", "redacted".into()),
                "redirect" => ("307 Temporary Redirect", "text/plain", String::new()),
                "jsonp" => ("200 OK", "application/jsonp", response.to_string()),
                "accepted" => ("202 Accepted", "application/json", response.to_string()),
                "partial" => (
                    "206 Partial Content",
                    "application/json",
                    response.to_string(),
                ),
                "mixed-case" => (
                    "200 OK",
                    "Application/JSON; charset=utf-8",
                    response.to_string(),
                ),
                _ => ("200 OK", "application/json", response.to_string()),
            };
            let extra = if redirect {
                "Location: /should-not-follow\r\n"
            } else {
                ""
            };
            stream.get_mut().write_all(format!(
                "HTTP/1.1 {status}\r\nContent-Type: {media}\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{bytes}",
                bytes.len()
            ).as_bytes()).await?;
            stream.get_mut().shutdown().await?;
        }
        assert!(
            tokio::time::timeout(Duration::from_millis(40), listener.accept())
                .await
                .is_err()
        );
        Ok::<_, TestError>(methods)
    };
    let (result, served) = tokio::time::timeout(DISCOVERY_TIMEOUT, async {
        tokio::join!(list_tools(client, &target, slot), server)
    })
    .await?;
    assert_eq!(result, expected);
    assert_eq!(served?, expected_methods);
    Ok(())
}

#[tokio::test]
async fn does_not_follow_redirects_with_the_bearer() -> Result<(), TestError> {
    fixture(
        vec![("server/discover", "redirect", Value::Null)],
        "2026-07-28",
        Err(McpDiscoveryFailure::Unavailable),
    )
    .await
}

#[tokio::test]
async fn complete_json_and_sse_catalogue_and_zero_tools() -> Result<(), TestError> {
    fixture(vec![
        ("server/discover", "json", Value::Null),
        ("tools/list", "json", json!({"tools":[{"name":"alpha","inputSchema":{"type":"object"}}], "nextCursor":"next"})),
        ("tools/list", "event", json!({"tools":[{"name":"beta","inputSchema":{"type":"object"}}]})),
    ], "2026-07-28", Ok(2)).await?;
    fixture(
        vec![
            ("server/discover", "event", Value::Null),
            ("tools/list", "json", json!({"tools":[]})),
        ],
        "2026-07-28",
        Ok(0),
    )
    .await
}

#[tokio::test]
async fn rejects_incomplete_discovery_and_unsupported_media() -> Result<(), TestError> {
    for (media, cause) in [
        ("incomplete", McpDiscoveryFailure::InvalidResponse),
        ("jsonp", McpDiscoveryFailure::InvalidResponse),
        ("accepted", McpDiscoveryFailure::Unavailable),
        ("partial", McpDiscoveryFailure::Unavailable),
        ("missing-discover", McpDiscoveryFailure::MissingDiscovery),
    ] {
        fixture(
            vec![("server/discover", media, Value::Null)],
            "2026-07-28",
            Err(cause),
        )
        .await?;
    }
    fixture(
        vec![
            ("server/discover", "mixed-case", Value::Null),
            ("tools/list", "json", json!({"tools":[]})),
        ],
        "2026-07-28",
        Ok(0),
    )
    .await
}

#[tokio::test]
async fn cancelled_waiter_retains_the_http_slot_until_the_worker_stops() -> Result<(), TestError> {
    let (listener, acceptor, client, target) = tls_fixture().await?;
    let slots = Arc::new(Semaphore::new(1));
    let permit = slots.clone().try_acquire_owned()?;
    let task = tokio::spawn(async move { list_tools(client, &target, permit).await });
    let mut stream = tokio::time::timeout(Duration::from_secs(3), async {
        let (tcp, _) = listener.accept().await?;
        let mut stream = BufReader::new(acceptor.accept(tcp).await?);
        let mut request_line = String::new();
        stream.read_line(&mut request_line).await?;
        assert_eq!(request_line, "POST /mcp HTTP/1.1\r\n");
        Ok::<_, TestError>(stream)
    })
    .await??;
    task.abort();
    assert!(task.await.is_err_and(|error| error.is_cancelled()));
    assert!(slots.try_acquire().is_err());
    stream.get_mut().shutdown().await?;
    drop(stream);
    let _next = tokio::time::timeout(Duration::from_secs(3), slots.acquire()).await??;
    Ok(())
}

#[tokio::test]
async fn strict_profile_and_rejected_or_incomplete_catalogues() -> Result<(), TestError> {
    fixture(
        vec![("server/discover", "json", Value::Null)],
        "2025-06-18",
        Err(McpDiscoveryFailure::UnsupportedVersion),
    )
    .await?;
    fixture(
        vec![("server/discover", "unauthorized", Value::Null)],
        "2026-07-28",
        Err(McpDiscoveryFailure::Authentication),
    )
    .await?;
    fixture(
        vec![
            ("server/discover", "json", Value::Null),
            (
                "tools/list",
                "json",
                json!({"tools":[], "nextCursor":"next"}),
            ),
            (
                "tools/list",
                "json",
                json!({"tools":[], "nextCursor":"next"}),
            ),
        ],
        "2026-07-28",
        Err(McpDiscoveryFailure::IncompleteCatalogue),
    )
    .await
}

#[tokio::test]
async fn actual_connector_refuses_loopback_resolutions_and_literals() -> Result<(), TestError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let http = http::OutboundMcpHttp::new();
    let literal = McpTestTarget {
        endpoint: McpEndpoint::parse(&format!("https://127.0.0.1:{port}/mcp"))?,
        bearer: None,
    };
    assert!(matches!(
        http.client(&literal),
        Err(McpDiscoveryFailure::DestinationDenied)
    ));
    let named = McpTestTarget {
        endpoint: McpEndpoint::parse(&format!("https://localhost:{port}/mcp"))?,
        bearer: None,
    };
    let client = http
        .client(&named)
        .map_err(|_| io::Error::other("could not create production profile"))?;
    assert_eq!(
        list_tools(
            client,
            &named,
            Arc::new(Semaphore::new(1)).try_acquire_owned()?
        )
        .await,
        Err(McpDiscoveryFailure::DestinationDenied)
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(30), listener.accept())
            .await
            .is_err()
    );
    Ok(())
}
