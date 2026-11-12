use super::*;
use chapchoo_crypto::BearerSecret;

async fn provision(context: NativeContext, http: bool) -> Result<(), TestError> {
    let key = BearerSecret::generate()?;
    let declaration = crate::NativeMcpDeclaration::http(
        "http://127.0.0.1:12345/mcp",
        BearerSecret::parse(key.expose().to_owned())?,
    )?;
    assert!(!format!("{declaration:?}").contains(key.expose()));
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming =
        FramedRead::new(read, LinesCodec::new()).map(|r| r.map_err(std::io::Error::other));
    let (ready, rx) = oneshot::channel();
    let done = CancellationToken::new();
    let resumed = matches!(context, NativeContext::Resume { .. });
    let connection = tokio::spawn(connect_authenticated(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        done.clone(),
        "/workspace".into(),
        (context, None, vec![declaration.wire()]),
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    let mut response = initialization(2);
    if http {
        response["capabilities"]["session"]["mcp"] = json!({"http":{}});
    }
    reply(&mut write, &initialize, response).await?;
    if !http {
        assert!(matches!(
            rx.await?,
            Err(Error::ProtocolRejected {
                stage: crate::ProtocolStage::ServerTools,
                ..
            })
        ));
        assert!(connection.await?.is_err());
        return Ok(());
    }
    let setup = request(&mut read).await?;
    assert_eq!(
        setup.method,
        if resumed {
            "session/resume"
        } else {
            "session/new"
        }
    );
    let servers = setup
        .params
        .get("mcpServers")
        .and_then(Value::as_array)
        .ok_or("Missing native MCP declarations")?;
    assert_eq!(servers.len(), 1);
    let server: schema::McpServer = serde_json::from_value(servers[0].clone())?;
    let schema::McpServer::Http(server) = server else {
        return Err("Not HTTP".into());
    };
    assert_eq!(server.name, "chapchoo");
    assert_eq!(server.url, "http://127.0.0.1:12345/mcp");
    assert_eq!(server.headers.len(), 1);
    assert!(server.headers[0].name == "Authorization");
    assert!(server.headers[0].value == format!("Bearer {}", key.expose()));
    if resumed {
        assert_eq!(
            setup.params.get("sessionId").and_then(Value::as_str),
            Some("retained")
        );
        assert!(setup.params.get("replayFrom").is_none());
    }
    reply(
        &mut write,
        &setup,
        if resumed {
            json!({})
        } else {
            json!({"sessionId":"new-native"})
        },
    )
    .await?;
    let session = rx.await??;
    assert_eq!(
        session.session_id().to_string(),
        if resumed { "retained" } else { "new-native" }
    );
    done.cancel();
    connection.await??;
    Ok(())
}
#[tokio::test]
async fn scripted_acp_new_and_exact_resume_carry_public_http_declarations_without_a_prompt()
-> Result<(), TestError> {
    provision(NativeContext::New, true).await?;
    provision(
        NativeContext::Resume {
            session: NativeConversationId::parse("retained".into())?,
            replay: CaptureReplay::AlreadyCaptured,
        },
        true,
    )
    .await?;
    Ok(())
}
#[tokio::test]
async fn missing_http_capability_rejects_provision_instead_of_using_another_transport()
-> Result<(), TestError> {
    provision(NativeContext::New, false).await
}
