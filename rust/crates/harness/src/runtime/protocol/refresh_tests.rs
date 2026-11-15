use super::*;

async fn live_state() -> Result<(tempfile::TempDir, State, Arc<Observations>), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let capture = crate::CaptureLog::create(&storage)?;
    let native = NativeConversationId::parse("refresh-native".into())?;
    capture.bind_native(&native)?;
    let observations = Arc::new(Observations::default());
    observations.bind(capture.snapshot()?, native, true)?;
    let done = CancellationToken::new();
    let stopping = done.clone();
    let child = tokio::process::Command::new("/bin/cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let launch = tempfile::tempdir()?;
    let mut state = state(storage)?;
    state.live = Some(Live {
        child,
        protocol: tokio::spawn(async move {
            stopping.cancelled().await;
            Ok(())
        }),
        stderr: tokio::spawn(async { Ok(()) }),
        done,
        ended: CancellationToken::new(),
        session: None,
        requests: Arc::default(),
        _supply: AppliedSupply {
            lease: Arc::new(launch.path().into()),
            node: "/unused/node".into(),
            adapter: "/unused/adapter".into(),
            codex: "/unused/codex".into(),
            freshness: crate::Freshness::Current,
            versions: crate::NativeComponentVersions {
                node: "v24.0.0".into(),
                adapter: "2.6.0".into(),
                codex: "0.165.0".into(),
            },
        },
        requested_supply: HarnessSupply::Managed,
        _launch: launch,
        capture,
        observations: observations.clone(),
        _authentication: NativeAuthenticationBinding {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
            operation_id: "preparation".into(),
            execution_id: "execution".into(),
            authentication_id: "authentication".into(),
            source: [11; 24],
        },
        tools: None,
        instructions: Default::default(),
        tool_catalog: Vec::new(),
    });
    Ok((saved, state, observations))
}
fn ingest(
    state: &State,
    observations: &Observations,
    session: &str,
    update: Value,
) -> Result<(), TestError> {
    let line = json!({"method":"session/update","params":{"sessionId":session,"update":update}})
        .to_string();
    let capture = &state.live.as_ref().ok_or("Missing test owner")?.capture;
    observations.record_live(capture, &line)?;
    Ok(())
}
async fn finish(mut state: State) -> Result<(), TestError> {
    // This fixture has no namespace ownership. Reap only its exact owned child.
    state
        .live
        .as_mut()
        .ok_or("Missing test owner")?
        .child
        .kill()
        .await?;
    state.retire().await?;
    Ok(())
}

#[tokio::test]
async fn child_interactions_are_explicitly_unsupported_without_root_occurrences()
-> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let capture = crate::CaptureLog::create(&storage)?;
    capture.bind_native(&NativeConversationId::parse("native".into())?)?;
    let observations = Arc::new(Observations::default());
    observations.bind(
        capture.snapshot()?,
        NativeConversationId::parse("native".into())?,
        true,
    )?;
    let requests: Requests = Arc::default();
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let log = capture.clone();
    let observed = observations.clone();
    let (ingress, mut recorded) = tokio::sync::mpsc::unbounded_channel();
    let incoming = FramedRead::new(read, LinesCodec::new()).map(move |line| {
        let line = line.map_err(std::io::Error::other)?;
        crate::runtime::capture(&line, &|line| observed.record_live(&log, line))
            .map_err(std::io::Error::other)?;
        if line.contains("\"sessionUpdate\"") {
            let _ = ingress.send(());
        }
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
        storage.workspace(),
        NativeContext::New,
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let new = request(&mut read).await?;
    reply(&mut write, &new, json!({"sessionId":"native"})).await?;
    acknowledge_title(&mut read, &mut write).await?;
    let _session = rx.await??;
    // Stock strict-v2 rendering: terminal is independent and every first call
    // is already an update. Capture ingress, not a timing sleep, is the barrier.
    for update in [
        json!({"sessionUpdate":"state_update","state":"running"}),
        json!({"sessionUpdate":"terminal_update","terminalId":"command","command":"fixture command","cwd":"/project"}),
        json!({"sessionUpdate":"tool_call_update","toolCallId":"command","status":"in_progress","content":[{"type":"terminal","terminalId":"command"}]}),
        json!({"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}),
    ] {
        let typed =
            schema::UpdateSessionNotification::new("native", serde_json::from_value(update)?);
        write
            .write_all(
                format!(
                    "{}\n",
                    json!({"jsonrpc":"2.0","method":"session/update","params":typed})
                )
                .as_bytes(),
            )
            .await?;
        recorded.recv().await.ok_or("Missing captured ingress")?;
    }
    assert!(!observations.quiescent());
    write.write_all(format!("{}\n", json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"native","update":{"sessionUpdate":"terminal_update","terminalId":"command","exitStatus":{"exitCode":0,"signal":null}}}})).as_bytes()).await?;
    recorded
        .recv()
        .await
        .ok_or("Missing physical terminal completion")?;
    assert!(observations.quiescent());
    for (session, update) in [
        (
            "native",
            json!({"sessionUpdate":"subagent_update","sessionId":"child","capabilities":{}}),
        ),
        (
            "child",
            json!({"sessionUpdate":"state_update","state":"running"}),
        ),
        (
            "child",
            json!({"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}),
        ),
    ] {
        write.write_all(format!("{}\n", json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":session,"update":update}})).as_bytes()).await?;
    }
    for (id, method, params) in [
        (
            77,
            "session/request_permission",
            json!({"sessionId":"child","title":"Child permission","options":[{"optionId":"allow","name":"Allow","kind":"allow_once"}]}),
        ),
        (
            78,
            "elicitation/create",
            json!({"sessionId":"child","mode":"form","message":"Child form","requestedSchema":{"type":"object","properties":{},"required":[]}}),
        ),
    ] {
        write
            .write_all(
                format!(
                    "{}\n",
                    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
                )
                .as_bytes(),
            )
            .await?;
        let mut line = String::new();
        read.read_line(&mut line).await?;
        let response: Value = serde_json::from_str(&line)?;
        assert_eq!(response["id"], id);
        assert_eq!(response["error"]["code"], -32602);
        assert!(requests.lock().await.is_empty());
        assert_eq!(
            capture
                .current_boundary()?
                .ok_or("Missing source")?
                .through
                .ok_or("Missing root association")?
                .ordinal(),
            6
        );
        assert!(
            !observations.quiescent(),
            "an unsupported interaction is not work completion"
        );
    }
    done.cancel();
    connection.await??;
    Ok(())
}

#[tokio::test]
async fn refresh_gate_combines_native_calls_child_work_and_input_report_ownership()
-> Result<(), TestError> {
    let (_saved, mut state, observed) = live_state().await?;
    assert!(!state.refresh_safe());
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"state_update","state":"running"}),
    )?;
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"terminal_update","terminalId":"terminal","command":"fixture command","cwd":"/project"}),
    )?;
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"tool_call_update","toolCallId":"terminal","status":"in_progress","content":[{"type":"terminal","terminalId":"terminal"}]}),
    )?;
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}),
    )?;
    assert!(!state.refresh_safe());
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"terminal_update","terminalId":"terminal","exitStatus":{"exitCode":null,"signal":null}}),
    )?;
    assert!(!state.refresh_safe());
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"terminal_update","terminalId":"terminal","exitStatus":{"exitCode":0,"signal":null}}),
    )?;
    assert!(state.refresh_safe());
    ingest(
        &state,
        &observed,
        "refresh-native",
        json!({"sessionUpdate":"subagent_update","sessionId":"child"}),
    )?;
    let root_through = state
        .live
        .as_ref()
        .ok_or("Missing owner")?
        .capture
        .current_boundary()?
        .ok_or("Missing boundary")?
        .through;
    ingest(
        &state,
        &observed,
        "child",
        json!({"sessionUpdate":"state_update","state":"idle"}),
    )?;
    assert!(!state.refresh_safe());
    ingest(
        &state,
        &observed,
        "child",
        json!({"sessionUpdate":"state_update","state":"running"}),
    )?;
    ingest(
        &state,
        &observed,
        "child",
        json!({"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}),
    )?;
    assert_eq!(
        state
            .live
            .as_ref()
            .ok_or("Missing owner")?
            .capture
            .current_boundary()?
            .ok_or("Missing boundary")?
            .through,
        root_through,
        "child lifecycle does not rewrite or append root originals"
    );
    assert!(state.refresh_safe());
    let input = NativeInputOperation {
        operation_id: "input".into(),
        target: NativeTextTarget {
            snapshot: state
                .live
                .as_ref()
                .ok_or("Missing owner")?
                .capture
                .snapshot()?,
            native_session: NativeConversationId::parse("refresh-native".into())?,
        },
        text: ConversationText::parse("already accepted".into())?,
    };
    state.inputs.begin(&input)?;
    assert!(!state.refresh_safe());
    state
        .inputs
        .completed("input", NativePromptAcceptance::Steered)?;
    assert!(
        !state.refresh_safe(),
        "known pickup remains owned until report acknowledgement"
    );
    state
        .inputs
        .release_reported("input", &NativePromptAcceptance::Steered)?;
    assert!(state.refresh_safe());
    finish(state).await
}

#[tokio::test]
async fn cancelled_project_publication_waiter_retains_physical_completion() -> Result<(), TestError>
{
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut state = state(storage.clone())?;
    let worker = storage.clone();
    let (started, entered) = oneshot::channel();
    let (release, released) = std::sync::mpsc::channel();
    state.project_write = Some(tokio::task::spawn_blocking(move || {
        let _ = started.send(());
        released.recv().map_err(|_| Error::Storage)?;
        worker.publish_project_instructions("selected project instructions\n")
    }));
    entered.await?;
    {
        let partial = state.retire();
        tokio::pin!(partial);
        std::future::poll_fn(|cx| {
            assert!(partial.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    assert!(state.project_write.is_some());
    release.send(())?;
    assert_eq!(state.retire().await?, Retirement::NoNativeStarted);
    assert!(state.project_write.is_none());
    assert_eq!(
        std::fs::read_to_string(storage.workspace().join("AGENTS.md"))?,
        "selected project instructions\n"
    );
    Ok(())
}
