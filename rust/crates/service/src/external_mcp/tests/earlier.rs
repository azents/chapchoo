use super::*;
use std::collections::HashMap;
use tokio::sync::Notify;

const SESSION: &str = "opaque-upstream-session";

#[derive(Default)]
struct CleanupGate {
    started: Notify,
    release: Notify,
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    version: &str,
    session: bool,
    actions: &[&str],
) -> Result<(), TestError> {
    serve_with_cleanup(listener, acceptor, version, session, actions, None).await
}

async fn serve_with_cleanup(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    version: &str,
    session: bool,
    actions: &[&str],
    cleanup: Option<&CleanupGate>,
) -> Result<(), TestError> {
    for action in actions {
        let (tcp, _) = listener.accept().await?;
        let mut stream = BufReader::new(acceptor.accept(tcp).await?);
        let mut line = String::new();
        stream.read_line(&mut line).await?;
        let delete = *action == "DELETE";
        assert_eq!(
            line,
            if delete {
                "DELETE /mcp HTTP/1.1\r\n"
            } else {
                "POST /mcp HTTP/1.1\r\n"
            },
        );
        let mut headers = HashMap::new();
        loop {
            line.clear();
            if stream.read_line(&mut line).await? == 0 {
                return Err("incomplete headers".into());
            }
            if line == "\r\n" {
                break;
            }
            let (key, value) = line.split_once(':').ok_or("invalid header")?;
            headers.insert(key.to_ascii_lowercase(), value.trim().to_owned());
        }
        let header = |key| headers.get(key).map(String::as_str);
        let bearer = format!("Bearer {TOKEN}");
        assert_eq!(header("authorization"), session.then_some(bearer.as_str()));
        let mut body = vec![0; header("content-length").unwrap_or("0").parse()?];
        stream.read_exact(&mut body).await?;
        let body: Value = if delete {
            Value::Null
        } else {
            serde_json::from_slice(&body)?
        };
        let method = match *action {
            "400" | "missing" | "modern-error" | "uncorrelated" | "401" | "hold"
            | "version-null" | "version-no-id" => "server/discover",
            "init" | "init-sse" => "initialize",
            "page-two" => "tools/list",
            "expire" | "lost" => "tools/call",
            other => other,
        };
        if !delete {
            assert_eq!(body["method"], method);
        }
        match method {
            "server/discover" => {
                assert_eq!(header("mcp-protocol-version"), Some("2026-07-28"));
                assert_eq!(header("mcp-session-id"), None);
            }
            "initialize" => {
                assert_eq!(body["params"]["protocolVersion"], "2025-11-25");
                assert_eq!(header("mcp-session-id"), None);
            }
            _ => {
                assert_eq!(header("mcp-protocol-version"), Some(version));
                assert_eq!(header("mcp-session-id"), session.then_some(SESSION));
                assert_eq!(header("mcp-method"), None);
                assert_eq!(header("mcp-name"), None);
            }
        }
        if *action == "lost" {
            stream.get_mut().shutdown().await?;
            continue;
        }
        if *action == "hold" {
            let mut byte = [0];
            let _ = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut byte)).await;
            continue;
        }
        if delete && let Some(cleanup) = cleanup {
            cleanup.started.notify_one();
            cleanup.release.notified().await;
        }
        let id = &body["id"];
        let ok = |result| json!({"jsonrpc":"2.0","id":id,"result":result}).to_string();
        let error = |code, id: &Value| {
            json!({
                "jsonrpc":"2.0","id":id,"error":{"code":code,"message":"fixture rejection"}
            })
            .to_string()
        };
        let (status, media, payload) = match *action {
            "400" => ("400 Bad Request", "application/json", error(-32000, id)),
            "missing" => ("200 OK", "application/json", error(-32601, id)),
            "modern-error" => ("400 Bad Request", "application/json", error(-32020, id)),
            "version-null" => (
                "400 Bad Request",
                "application/json",
                json!({"jsonrpc":"2.0","id":null,"error":{
                    "code":-32022,"message":"version rejection","data":{"supported":["2026-07-28"]}
                }})
                .to_string(),
            ),
            "version-no-id" => (
                "400 Bad Request",
                "application/json",
                json!({"jsonrpc":"2.0","error":{
                    "code":-32022,"message":"version rejection","data":{"supported":["2099-01-01"]}
                }})
                .to_string(),
            ),
            "uncorrelated" => (
                "400 Bad Request",
                "application/json",
                error(-32601, &json!("other")),
            ),
            "401" => ("401 Unauthorized", "application/json", error(-32601, id)),
            "expire" => ("404 Not Found", "text/plain", String::new()),
            "DELETE" => ("204 No Content", "text/plain", String::new()),
            "notifications/initialized" => ("202 Accepted", "text/plain", String::new()),
            "init" | "init-sse" => {
                let response = ok(json!({
                    "protocolVersion":version,"capabilities":{"tools":{}},
                    "serverInfo":{"name":"fixture","version":"1"}
                }));
                if *action == "init-sse" {
                    (
                        "200 OK",
                        "text/event-stream",
                        format!("event: message\ndata: {response}\n\n"),
                    )
                } else {
                    ("200 OK", "application/json", response)
                }
            }
            "tools/list" => (
                "200 OK",
                "application/json",
                ok(json!({
                    "tools":[{"name":"fixture-tool","inputSchema":{"type":"object"}}],"nextCursor":"next"
                })),
            ),
            "page-two" => {
                assert_eq!(body["params"]["cursor"], "next");
                let result =
                    ok(json!({"tools":[{"name":"other","inputSchema":{"type":"object"}}]}));
                (
                    "200 OK",
                    "text/event-stream",
                    format!("event: message\ndata: {result}\n\n"),
                )
            }
            "tools/call" => {
                assert_eq!(body["params"]["name"], "fixture-tool");
                (
                    "200 OK",
                    "application/json",
                    ok(json!({"content":[],"structuredContent":{"marker":7}})),
                )
            }
            _ => return Err("unexpected fixture action".into()),
        };
        let session = if session && method == "initialize" {
            format!("Mcp-Session-Id: {SESSION}\r\n")
        } else {
            String::new()
        };
        stream.get_mut().write_all(format!(
            "HTTP/1.1 {status}\r\nContent-Type: {media}\r\n{session}Content-Length: {}\r\nConnection: close\r\n\r\n{payload}",
            payload.len(),
        ).as_bytes()).await?;
        stream.get_mut().shutdown().await?;
    }
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err()
    );
    Ok(())
}

async fn discovery(
    version: &str,
    session: bool,
    actions: &[&str],
    expected: Result<usize, McpDiscoveryFailure>,
) -> Result<(), TestError> {
    let (listener, acceptor, client, target) =
        tls_fixture_for(session, Duration::from_secs(8)).await?;
    let slot = Arc::new(Semaphore::new(1)).try_acquire_owned()?;
    let (operation, served) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            list_tools(client, &target, slot),
            serve(listener, acceptor, version, session, actions)
        )
    })
    .await?;
    served?;
    assert_eq!(
        operation.map(|catalogue| {
            assert!(!catalogue.may_reuse_at(Instant::now()));
            catalogue.tool_count()
        }),
        expected
    );
    Ok(())
}

#[tokio::test]
async fn negotiates_all_earlier_versions_and_lists_complete_pages() -> Result<(), TestError> {
    for (version, first, init, session) in [
        ("2025-11-25", "400", "init", false),
        ("2025-06-18", "missing", "init-sse", true),
        ("2025-03-26", "400", "init", true),
    ] {
        let mut actions = vec![
            first,
            init,
            "notifications/initialized",
            "tools/list",
            "page-two",
        ];
        if session {
            actions.push("DELETE");
        }
        discovery(version, session, &actions, Ok(2)).await?;
    }
    Ok(())
}

#[tokio::test]
async fn rejects_unsupported_version_and_nonlegacy_failures_before_tools() -> Result<(), TestError>
{
    discovery(
        "2024-11-05",
        false,
        &["400", "init"],
        Err(McpDiscoveryFailure::UnsupportedVersion),
    )
    .await?;
    for (action, expected) in [
        ("401", McpDiscoveryFailure::Authentication),
        ("modern-error", McpDiscoveryFailure::InvalidResponse),
        ("uncorrelated", McpDiscoveryFailure::InvalidResponse),
        ("version-null", McpDiscoveryFailure::InvalidResponse),
        ("version-no-id", McpDiscoveryFailure::InvalidResponse),
    ] {
        discovery("2025-11-25", false, &[action], Err(expected)).await?;
    }
    Ok(())
}

#[tokio::test]
async fn call_outcome_is_not_replayed_on_session_expiry_or_reply_loss() -> Result<(), TestError> {
    for action in ["tools/call", "expire", "lost"] {
        let (listener, acceptor, client, target) = tls_fixture().await?;
        let slot = Arc::new(Semaphore::new(1)).try_acquire_owned()?;
        let actions = ["400", "init", "notifications/initialized", action, "DELETE"];
        let (result, served) = tokio::time::timeout(Duration::from_secs(3), async {
            tokio::join!(
                invocation::run(
                    client,
                    target,
                    slot,
                    rmcp::model::CallToolRequestParams::new("fixture-tool"),
                    Arc::new(serde_json::Map::new()),
                    Duration::from_secs(2)
                ),
                serve(listener, acceptor, "2025-11-25", true, &actions),
            )
        })
        .await?;
        served?;
        match (action, result) {
            ("tools/call", ExternalMcpCallOutcome::Completed { result_json }) => {
                assert_eq!(
                    serde_json::from_slice::<Value>(&result_json)?["structuredContent"]["marker"],
                    7
                );
            }
            ("expire" | "lost", ExternalMcpCallOutcome::Uncertain) => {}
            (action, result) => return Err(format!("{action}: {result:?}").into()),
        }
    }
    Ok(())
}

#[tokio::test]
async fn settled_call_returns_while_the_http_owner_retains_cleanup_capacity()
-> Result<(), TestError> {
    let (listener, acceptor, client, target) = tls_fixture().await?;
    let capacity = Arc::new(Semaphore::new(1));
    let slot = capacity.clone().try_acquire_owned()?;
    let gate = CleanupGate::default();
    let actions = [
        "400",
        "init",
        "notifications/initialized",
        "tools/call",
        "DELETE",
    ];
    let scenario = async {
        let outcome = invocation::run(
            client,
            target,
            slot,
            rmcp::model::CallToolRequestParams::new("fixture-tool"),
            Arc::new(serde_json::Map::new()),
            Duration::from_secs(2),
        )
        .await;
        assert!(matches!(outcome, ExternalMcpCallOutcome::Completed { .. }));
        gate.started.notified().await;
        assert_eq!(capacity.available_permits(), 0);
        gate.release.notify_one();
        let _released = capacity.acquire().await?;
        Ok::<_, TestError>(())
    };
    let (operation, served) = tokio::time::timeout(Duration::from_secs(4), async {
        tokio::join!(
            scenario,
            serve_with_cleanup(
                listener,
                acceptor,
                "2025-11-25",
                true,
                &actions,
                Some(&gate)
            ),
        )
    })
    .await?;
    operation?;
    served
}

#[tokio::test]
async fn timeout_does_not_probe_an_earlier_lifecycle() -> Result<(), TestError> {
    let (listener, acceptor, client, target) =
        tls_fixture_for(false, Duration::from_millis(400)).await?;
    let slot = Arc::new(Semaphore::new(1)).try_acquire_owned()?;
    let (outcome, served) = tokio::time::timeout(Duration::from_secs(4), async {
        tokio::join!(
            invocation::run(
                client,
                target,
                slot,
                rmcp::model::CallToolRequestParams::new("fixture-tool"),
                Arc::new(serde_json::Map::new()),
                Duration::from_secs(2)
            ),
            serve(listener, acceptor, "2025-11-25", false, &["hold"]),
        )
    })
    .await?;
    served?;
    assert!(matches!(
        outcome,
        ExternalMcpCallOutcome::NotExecuted(ExternalMcpRejection::Upstream(
            McpDiscoveryFailure::TimedOut
        ))
    ));
    Ok(())
}
