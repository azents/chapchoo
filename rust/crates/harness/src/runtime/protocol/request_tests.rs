use super::*;

fn answer(id: &str, option: &str) -> InteractiveResponse {
    InteractiveResponse::Permission {
        id: id.into(),
        response: schema::RequestPermissionResponse::new(
            schema::RequestPermissionOutcome::Selected(schema::SelectedPermissionOutcome::new(
                option,
            )),
        ),
    }
}
async fn permission(
    writer: &mut tokio::io::WriteHalf<tokio::io::DuplexStream>,
    id: schema::RequestId,
) -> Result<(), TestError> {
    writer.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":id,"method":"session/request_permission","params":{
        "sessionId":"native","title":"Exact question","options":[{"optionId":"allow","name":"Allow once","kind":"allow_once"}]
    }})).as_bytes()).await?;
    Ok(())
}
#[tokio::test]
async fn captured_requests_bind_typed_ids_to_fresh_single_use_targets() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let storage = NativeStorage::open(directory.path())?;
    let source = crate::CaptureLog::create(&storage)?;
    source.bind_native(&NativeConversationId::parse("native".into())?)?;
    let requests: Requests = Arc::default();
    let observations = Arc::new(Observations::default());
    observations.bind(
        source.snapshot()?,
        NativeConversationId::parse("native".into())?,
    )?;
    let retained = source.clone();
    let observed = observations.clone();
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming = FramedRead::new(read, LinesCodec::new()).map(move |line| {
        let line = line.map_err(std::io::Error::other)?;
        capture(&line, &|line| {
            let position = retained.append(line, None)?;
            observed.captured(line, position)
        })
        .map_err(std::io::Error::other)?;
        Ok::<_, std::io::Error>(line)
    });
    let (ready, rx) = oneshot::channel();
    let done = CancellationToken::new();
    let connection = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        requests.clone(),
        observations.clone(),
        ready,
        done.clone(),
        "/workspace".into(),
        NativeContext::New,
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let new = request(&mut read).await?;
    reply(&mut write, &new, json!({"sessionId":"native"})).await?;
    let session = rx.await??;
    permission(&mut write, schema::RequestId::Number(1)).await?;
    observations.wait_change().await;
    permission(&mut write, schema::RequestId::Str("1".into())).await?;
    observations.wait_change().await;
    let targets = {
        let pending = requests.lock().await;
        assert_eq!(pending.len(), 2);
        let mut targets = pending
            .iter()
            .map(|(token, (position, _))| (position.clone(), token.clone()))
            .collect::<Vec<_>>();
        targets.sort_by_key(|(position, _)| position.ordinal());
        assert_ne!(targets[0].1, targets[1].1);
        assert_eq!(targets[0].0.ordinal(), 1);
        assert_eq!(targets[1].0.ordinal(), 2);
        targets
    };
    assert!(matches!(
        requests::respond(
            &requests,
            session.session_id(),
            answer(&targets[0].1, "not-offered")
        )
        .await,
        Err(Error::NotAdmitted)
    ));
    assert!(matches!(
        requests::respond(
            &requests,
            &schema::SessionId::new("another"),
            answer(&targets[0].1, "allow")
        )
        .await,
        Err(Error::NotAdmitted)
    ));
    requests::respond(
        &requests,
        session.session_id(),
        answer(&targets[0].1, "allow"),
    )
    .await?;
    #[derive(Deserialize)]
    struct Response {
        id: schema::RequestId,
    }
    let mut line = String::new();
    read.read_line(&mut line).await?;
    assert_eq!(
        serde_json::from_str::<Response>(&line)?.id,
        schema::RequestId::Number(1)
    );
    assert!(matches!(
        requests::respond(
            &requests,
            session.session_id(),
            answer(&targets[0].1, "allow")
        )
        .await,
        Err(Error::NotAdmitted)
    ));
    permission(&mut write, schema::RequestId::Number(1)).await?;
    observations.wait_change().await;
    let new_target = {
        let pending = requests.lock().await;
        let (token, (position, _)) = pending
            .iter()
            .find(|(_, (position, _))| position.ordinal() == 3)
            .ok_or("New occurrence")?;
        assert_ne!(token, &targets[0].1);
        assert_ne!(position.occurrence(), targets[0].0.occurrence());
        token.clone()
    };
    assert!(matches!(
        requests::respond(
            &requests,
            session.session_id(),
            answer(&targets[0].1, "allow")
        )
        .await,
        Err(Error::NotAdmitted)
    ));
    let cancelled = {
        let pending = requests.lock().await;
        let (_, pending) = pending.get(&new_target).ok_or("Pending target")?;
        match pending {
            Pending::Permission(_, r) => r.cancellation(),
            _ => return Err("Expected permission".into()),
        }
    };
    write
        .write_all(
            format!(
                "{}\n",
                json!({"jsonrpc":"2.0","method":"$/cancel_request","params":{"requestId":1}})
            )
            .as_bytes(),
        )
        .await?;
    cancelled.cancelled().await;
    assert!(matches!(
        requests::respond(
            &requests,
            session.session_id(),
            answer(&new_target, "allow")
        )
        .await,
        Err(Error::NotAdmitted)
    ));
    requests::respond(
        &requests,
        session.session_id(),
        answer(&targets[1].1, "allow"),
    )
    .await?;
    line.clear();
    read.read_line(&mut line).await?;
    assert_eq!(
        serde_json::from_str::<Response>(&line)?.id,
        schema::RequestId::Str("1".into())
    );
    let mut pass = source.begin(&source.snapshot()?, None)?;
    for (position, _) in targets {
        assert_eq!(
            pass.next_event()?.ok_or("Captured request")?.position,
            position
        );
    }
    assert_eq!(pass.next_event()?.ok_or("Third")?.position.ordinal(), 3);
    assert!(pass.next_event()?.is_none());
    write.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":55,"method":"elicitation/create","params":{
        "sessionId":"native","mode":"form","message":"Unsupported required object","requestedSchema":{
            "type":"object","properties":{"nested":{"type":"object"}},"required":["nested"]
        }
    }})).as_bytes()).await?;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Rejected {
        jsonrpc: String,
        id: schema::RequestId,
        error: Rejection,
    }
    #[derive(Deserialize)]
    struct Rejection {
        code: i32,
    }
    let mut rejected = None;
    // Closing the earlier cancelled responder may emit its own SDK error.
    // Correlate the unsupported form by its exact typed ID, not receive order.
    for _ in 0..2 {
        line.clear();
        read.read_line(&mut line).await?;
        let response: Rejected = serde_json::from_str(&line)?;
        if response.id == schema::RequestId::Number(55) {
            rejected = Some(response);
            break;
        }
        assert_eq!(response.id, schema::RequestId::Number(1));
    }
    let rejected = rejected.ok_or("Missing unsupported-form rejection")?;
    assert_eq!(rejected.jsonrpc, "2.0");
    assert_eq!(rejected.id, schema::RequestId::Number(55));
    assert_eq!(rejected.error.code, -32602);
    assert!(requests.lock().await.is_empty());
    write.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":66,"method":"elicitation/create","params":{
        "sessionId":"native","mode":"form","message":"Original question","requestedSchema":{
            "type":"object","properties":{"enabled":{"type":"boolean"},"text":{"type":"string"}},"required":["enabled","text"]
        }
    }})).as_bytes()).await?;
    observations.wait_change().await;
    let token = requests
        .lock()
        .await
        .keys()
        .next()
        .cloned()
        .ok_or("Missing form responder")?;
    let invalid = InteractiveResponse::Form {
        id: token.clone(),
        response: serde_json::from_value(
            json!({"action":"accept","content":{"enabled":"false","text":""}}),
        )?,
    };
    assert!(matches!(
        requests::respond(&requests, session.session_id(), invalid).await,
        Err(Error::NotAdmitted)
    ));
    assert!(requests.lock().await.contains_key(&token));
    let response =
        serde_json::from_value(json!({"action":"accept","content":{"enabled":false,"text":""}}))?;
    requests::respond(
        &requests,
        session.session_id(),
        InteractiveResponse::Form {
            id: token.clone(),
            response,
        },
    )
    .await?;
    line.clear();
    read.read_line(&mut line).await?;
    #[derive(Deserialize)]
    struct FormResult {
        id: schema::RequestId,
        result: schema::CreateElicitationResponse,
    }
    let response: FormResult = serde_json::from_str(&line)?;
    assert_eq!(response.id, schema::RequestId::Number(66));
    assert_eq!(
        response.result,
        serde_json::from_value(json!({"action":"accept","content":{"enabled":false,"text":""}}))?
    );
    assert!(!requests.lock().await.contains_key(&token));
    requests.lock().await.clear();
    done.cancel();
    connection.await??;
    Ok(())
}
