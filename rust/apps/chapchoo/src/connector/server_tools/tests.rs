use super::*;
use chapchoo_protocol::conversations::CaptureSource;
use chapchoo_service::{
    ServerToolBackend, ServerToolFuture, ServerToolRequest, ServerToolScope, ServerToolValidation,
    ServerTools, ServiceError, ToolCancellation,
};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequest, CallToolRequestParams, ClientConfig, ClientRequest, ProtocolVersion},
    service::PeerRequestOptions,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_stream::wrappers::TcpListenerStream;

struct ObservedCall {
    agent_id: String,
    conversation_id: String,
    connection_id: String,
    arguments: Vec<u8>,
}

struct Backend {
    key: Mutex<chapchoo_crypto::SecretDigest>,
    calls: Mutex<Vec<ObservedCall>>,
    waiting: Semaphore,
    cancelled: Semaphore,
    stop: CancellationToken,
}
impl ServerToolBackend for Backend {
    fn authorize<'a>(&'a self, scope: &'a ServerToolScope) -> ServerToolFuture<'a, ()> {
        Box::pin(async move {
            if lock(&self.key).matches_digest(&scope.credential)
                && scope.agent_id == "agent"
                && scope.conversation_id == "conversation"
            {
                Ok(())
            } else {
                Err(ServiceError::domain(
                    chapchoo_service::DomainReason::PermissionDenied,
                ))
            }
        })
    }
    fn catalog(&self) -> Result<Vec<u8>, ServiceError> {
        Ok(br#"{"tools":[{"name":"send_message","description":"Fixture publication","inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"],"additionalProperties":false}}]}"#.to_vec())
    }
    fn validate(&self, name: &str, _: &[u8]) -> Result<(), ServerToolValidation> {
        if name == "send_message" {
            Ok(())
        } else {
            Err(ServerToolValidation::UnknownTool)
        }
    }
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        cancellation: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>> {
        Box::pin(async move {
            lock(&self.calls).push(ObservedCall {
                agent_id: request.scope.agent_id.clone(),
                conversation_id: request.scope.conversation_id.clone(),
                connection_id: request.scope.connection_id.clone(),
                arguments: request.arguments.clone(),
            });
            if request.arguments == br#"{"text":"/wait-for-cancel"}"# {
                self.waiting.add_permits(1);
                tokio::select! {
                    _ = cancellation.cancelled() => self.cancelled.add_permits(1),
                    _ = self.stop.cancelled() => (),
                }
                return Ok(
                    br#"{"content":[{"type":"text","text":"Fixture stopped"}],"isError":true}"#
                        .to_vec(),
                );
            }
            if request.arguments == br#"{"text":"/oversize-result"}"# {
                return Ok(vec![0; chapchoo_service::MAX_TOOL_PAYLOAD + 1]);
            }
            Ok(br#"{"content":[{"type":"text","text":"Message accepted."}],"structuredContent":{"message_id":"fixture-message"},"isError":false}"#.to_vec())
        })
    }
}
struct Profile;
fn http_client() -> Result<reqwest::Client, AppError> {
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_root_certificates(rustls::RootCertStore::empty())
    .with_no_client_auth();
    // Loopback-only test transport; no global provider/root or proxy changes.
    Ok(reqwest::Client::builder()
        .tls_backend_preconfigured(tls)
        .no_proxy()
        .build()?)
}
impl ClientHandler for Profile {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::default().with_protocol_version(ProtocolVersion::V_2025_06_18)
    }
}

#[tokio::test]
async fn official_sdk_http_round_trip_keeps_native_scope_across_server_key_refresh()
-> Result<(), AppError> {
    crate::test_support::scope(async |fixture| {
        let key = BearerSecret::generate()?;
        let backend = Arc::new(Backend {
            key: Mutex::new(key.digest()),
            calls: Mutex::new(Vec::new()),
            waiting: Semaphore::new(0),
            cancelled: Semaphore::new(0),
            stop: CancellationToken::new(),
        });
        let admission = Arc::new(AtomicBool::new(true));
        let adapter = chapchoo_rpc::ServerToolServices::new(
            ServerTools::new(backend.clone()),
            admission.clone(),
        );
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let stop = fixture.shutdown();
        let cleanup = adapter.clone();
        let backend_stop = backend.stop.clone();
        fixture.server(async move {
            let result = tonic::transport::Server::builder()
                .add_service(adapter.server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = stop.await;
                    admission.store(false, Ordering::Release);
                    backend_stop.cancel();
                })
                .await
                .map_err(|error| -> AppError { error.into() });
            crate::completion::complete(
                result,
                cleanup
                    .finish()
                    .await
                    .map_err(|error| -> AppError { error.into() }),
            )
        });
        let channel = tonic::transport::Endpoint::from_shared(format!("http://{address}"))?
            .buffer_size(128)
            .concurrency_limit(128)
            .connect()
            .await?;
        let current = CurrentRoute::default();
        let mut source = CaptureSource {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "connection-1".into(),
        };
        current.update(channel.clone(), source.clone(), &key)?;
        let relay = Arc::new(tokio::sync::Mutex::new(
            Relay::start(
                current.clone(),
                Recipient {
                    agent_id: "agent".into(),
                    conversation_id: "conversation".into(),
                },
            )
            .await?,
        ));
        let cleanup = relay.clone();
        fixture.cleanup(async move { cleanup.lock().await.finish().await });
        let (url, bearer) = {
            let relay = relay.lock().await;
            (relay.url().to_owned(), relay.bearer().to_owned())
        };
        let http = http_client()?;
        assert_eq!(
            http.post(&url).body("{}").send().await?.status(),
            reqwest::StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            http.post(&url)
                .bearer_auth(&bearer)
                .header("origin", "https://untrusted.example")
                .header("content-type", "application/json")
                .header("accept", "application/json, text/event-stream")
                .body("{}")
                .send()
                .await?
                .status(),
            reqwest::StatusCode::FORBIDDEN
        );
        for (name, value, expected) in [
            ("host", "untrusted.example", reqwest::StatusCode::FORBIDDEN),
            ("mcp-protocol-version", "2026-07-28", reqwest::StatusCode::BAD_REQUEST),
            ("last-event-id", "not-a-replay-cursor", reqwest::StatusCode::BAD_REQUEST),
        ] {
            assert_eq!(http.post(&url).bearer_auth(&bearer).header(name, value)
                .header("content-type","application/json").header("accept","application/json, text/event-stream")
                .body("{}").send().await?.status(), expected);
        }
        assert_eq!(http.post(&url).bearer_auth(&bearer).body(vec![b'x';65537]).send().await?.status(),
            reqwest::StatusCode::PAYLOAD_TOO_LARGE);
        let mut config = StreamableHttpClientTransportConfig::with_uri(url.clone());
        config.auth_header = Some(bearer.clone());
        let transport = StreamableHttpClientTransport::with_client(http.clone(), config);
        let client = Profile.serve(transport).await?;
        assert_eq!(
            client
                .peer_info()
                .ok_or("Missing MCP peer")?
                .protocol_version,
            ProtocolVersion::V_2025_06_18
        );
        assert_eq!(client.list_all_tools().await?.len(), 1);
        relay.lock().await.verify().await?;
        let invalid = client
            .call_tool(CallToolRequestParams::new("x".repeat(129)))
            .await
            .err()
            .ok_or("Invalid envelope was not rejected")?;
        assert!(invalid.to_string().contains("invocation was not executed"));
        assert!(!invalid.to_string().contains("may have occurred"));
        let body = "  의도적 메시지 🙂\n원문  ";
        let arguments = serde_json::from_value(serde_json::json!({"text":body}))?;
        let result = client
            .call_tool(CallToolRequestParams::new("send_message").with_arguments(arguments))
            .await?;
        assert_eq!(result.is_error, Some(false));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("message_id"))
                .and_then(serde_json::Value::as_str),
            Some("fixture-message")
        );

        let next = BearerSecret::generate()?;
        *lock(&backend.key) = next.digest();
        source.connection_id = "connection-2".into();
        current.update(channel, source, &next)?;
        client
            .call_tool(CallToolRequestParams::new("send_message").with_arguments(
                serde_json::from_value(serde_json::json!({"text":"second"}))?,
            ))
            .await?;
        {
            let calls = lock(&backend.calls);
            assert_eq!(calls.len(), 2);
            assert!(
                calls
                    .iter()
                    .all(|call| call.agent_id == "agent"
                        && call.conversation_id == "conversation")
            );
            assert_eq!(calls[0].connection_id, "connection-1");
            assert_eq!(calls[1].connection_id, "connection-2");
            let original: serde_json::Value = serde_json::from_slice(&calls[0].arguments)?;
            assert_eq!(
                original.get("text").and_then(serde_json::Value::as_str),
                Some(body)
            );
        }
        // This is the SDK's real typed cancellation notification, not dropping
        // an HTTP connection or using the handler context's generic drop token.
        let pending = client.send_request_with_option(ClientRequest::CallToolRequest(CallToolRequest::new(
            CallToolRequestParams::new("send_message").with_arguments(
                serde_json::from_value(serde_json::json!({"text":"/wait-for-cancel"}))?),
        )), PeerRequestOptions::default()).await?;
        tokio::time::timeout(std::time::Duration::from_secs(10), backend.waiting.acquire()).await??.forget();
        pending.cancel(None).await?;
        tokio::time::timeout(std::time::Duration::from_secs(10), backend.cancelled.acquire()).await??.forget();

        let isolated = Arc::new(tokio::sync::Mutex::new(Relay::start(current.clone(),
            Recipient { agent_id:"agent".into(), conversation_id:"conversation".into() }).await?));
        let cleanup = isolated.clone();
        fixture.cleanup(async move { cleanup.lock().await.finish().await });
        let (isolated_url, isolated_key) = {
            let isolated = isolated.lock().await;
            (isolated.url().to_owned(), isolated.bearer().to_owned())
        };
        let initialize = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"session-fixture","version":"0"}
        }});
        let mut sessions = Vec::new();
        for _ in 0..4 {
            let response = http.post(&isolated_url).bearer_auth(&isolated_key)
                .header("accept","application/json, text/event-stream").json(&initialize).send().await?;
            assert!(response.status().is_success());
            sessions.push(response.headers().get("mcp-session-id").ok_or("Missing session ID")?.to_str()?.to_owned());
            let _ = response.bytes().await?;
        }
        let response = http.post(&isolated_url).bearer_auth(&isolated_key)
            .header("accept","application/json, text/event-stream").json(&initialize).send().await?;
        assert!(response.status().is_server_error());
        let response = http.delete(&isolated_url).bearer_auth(&isolated_key)
            .header("mcp-session-id",&sessions[0]).header("mcp-protocol-version","2025-06-18").send().await?;
        assert!(response.status().is_success());
        let response = http.post(&isolated_url).bearer_auth(&isolated_key)
            .header("accept","application/json, text/event-stream").json(&initialize).send().await?;
        assert!(response.status().is_success());
        let _ = response.bytes().await?;

        let unsupported = client.call_tool(CallToolRequestParams::new("send_message")
            .with_arguments(serde_json::from_value(serde_json::json!({"text":"/oversize-result"}))?))
            .await.err().ok_or("Oversized effect result was represented as success")?;
        assert!(unsupported.to_string().contains("cannot be represented"));
        current.withdraw();
        assert!(
            client
                .call_tool(CallToolRequestParams::new("send_message"))
                .await
                .is_err()
        );
        assert_eq!(lock(&backend.calls).len(), 4);
        client.cancel().await?;
        relay.lock().await.close();
        assert!(relay.lock().await.verify().await.is_err());
        assert!(http.post(&url).bearer_auth(&bearer).body("{}").send().await
            .map(|response| !response.status().is_success()).unwrap_or(true));
        Ok(())
    })
    .await
}
