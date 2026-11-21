use super::*;
use futures_util::{SinkExt, StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite, duplex, split};
use tokio_util::{
    codec::{FramedRead, FramedWrite, LinesCodec},
    sync::CancellationToken,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

mod released;

#[test]
fn stop_and_matched_terminals_share_the_capture_ordering_fence() -> TestResult {
    use crate::runtime::{NativeForeground, NativeTextTarget, observations::Observations};

    let dir = tempfile::tempdir()?;
    let storage = crate::NativeStorage::open(dir.path())?;
    let capture = crate::CaptureLog::create(&storage)?;
    let root = NativeConversationId::parse("root".into())?;
    capture.bind_native(&root)?;
    let target = NativeTextTarget {
        snapshot: capture.snapshot()?,
        native_session: root,
    };
    let state = Observations::default();
    state.bind(
        target.snapshot.clone(),
        target.native_session.clone(),
        false,
    )?;
    state.enable_normal_endings()?;
    let started = |id: &str| {
        json!({"method":"turn/started","params":{"threadId":"root","turn":{"id":id,"status":"inProgress"}}}).to_string()
    };
    let completed = |thread: &str, id: &str, status: &str| {
        json!({"method":"turn/completed","params":{"threadId":thread,"turn":{"id":id,"status":status}}}).to_string()
    };
    let unrelated = completed("root", "old", "completed");
    state.record_app_server_turn(&capture, &unrelated, &target)?;
    assert!(capture.is_empty()?);
    state.record_app_server_turn(&capture, &started("exact"), &target)?;
    let running = state.current()?;
    assert_eq!(running.foreground, NativeForeground::Running);
    let work = running.work.ok_or("Missing live turn")?;
    assert_eq!(work.work_id, "exact");
    for frame in [&unrelated, &completed("child", "exact", "completed")] {
        state.record_app_server_turn(&capture, frame, &target)?;
    }
    assert_eq!(state.current()?.work, Some(work.clone()));
    state.publish_stop(&work, || Ok(()))?;
    state.record_app_server_turn(&capture, &completed("root", "exact", "completed"), &target)?;
    assert_eq!(state.current()?.foreground, NativeForeground::Idle);
    assert!(state.normal_ending()?.is_none());
    state.record_app_server_turn(&capture, &started("exact"), &target)?;
    assert!(
        state.current()?.work.is_none(),
        "Do not reopen a completed turn"
    );

    state.record_app_server_turn(&capture, &started("later"), &target)?;
    let terminal = completed("root", "later", "completed");
    state.record_app_server_turn(&capture, &terminal, &target)?;
    let ending = state.normal_ending()?.ok_or("Missing ending")?;
    assert_eq!(ending.work.work_id, "later");
    state.record_app_server_turn(&capture, &terminal, &target)?;
    assert_eq!(state.normal_ending()?, Some(ending.clone()));
    assert_eq!(ending.position.ordinal(), 4);
    Ok(())
}

#[test]
fn only_exact_root_live_events_yield_operation_evidence() -> TestResult {
    use events::{Observation, root_observation};

    let root = NativeConversationId::parse("root".into())?;
    let item = json!({
        "method": "item/started",
        "params": {"threadId": "root", "turnId": "turn",
            "item": {"type": "userMessage", "id": "item", "clientId": "operation"}}
    });
    assert_eq!(
        root_observation(&root, &item)?,
        Some(Observation::InputInserted {
            turn: "turn",
            item: "item",
            operation: "operation",
        })
    );
    for frame in [
        json!({"id": 1, "result": {"turn": {"id": "turn"}}}),
        json!({"method": "thread/status/changed", "params": {"threadId": "root", "status": {"type": "idle"}}}),
        json!({"method": "turn/completed", "params": {"threadId": "child", "turn": {"id": "child-turn", "rootTurnId": "turn", "status": "completed"}}}),
        json!({"method": "item/completed", "params": item["params"]}),
    ] {
        assert_eq!(root_observation(&root, &frame)?, None);
    }
    let mut frame = json!({"method": "turn/started", "params": {"threadId": "root", "turn": {"id": "turn", "status":"inProgress"}}});
    assert_eq!(
        root_observation(&root, &frame)?,
        Some(Observation::Started { turn: "turn" })
    );
    frame["method"] = json!("turn/completed");
    for (status, normal) in [
        ("completed", true),
        ("interrupted", false),
        ("failed", false),
    ] {
        frame["params"]["turn"]["status"] = json!(status);
        assert_eq!(
            root_observation(&root, &frame)?,
            Some(Observation::Terminal {
                turn: "turn",
                normal
            })
        );
    }
    Ok(())
}

async fn recv(reader: &mut (impl AsyncRead + Unpin)) -> TestResult<Value> {
    let frame = FramedRead::new(reader, LinesCodec::new())
        .next()
        .await
        .ok_or("Missing outbound frame")??;
    Ok(serde_json::from_str(&frame)?)
}

async fn send(writer: &mut (impl AsyncWrite + Unpin), value: Value) -> TestResult {
    FramedWrite::new(writer, LinesCodec::new())
        .send(value.to_string())
        .await?;
    Ok(())
}

#[tokio::test]
async fn deferred_requests_remain_ordered_and_cancel_by_typed_id_and_thread() -> TestResult {
    use tokio::sync::{mpsc, oneshot};

    tokio::time::timeout(Duration::from_secs(5), async {
        let (local, remote) = duplex(4096);
        let (read, write) = split(local);
        let (server_read, server_write) = split(remote);
        let mut input = FramedRead::new(server_read, LinesCodec::new());
        let mut output = FramedWrite::new(server_write, LinesCodec::new());
        let (published, mut requests) = mpsc::channel(8);
        let done = CancellationToken::new();
        let (link, driver) = wire::connect_with_requests(read, write, done.clone(), move |raw, frame| {
            let answer = if frame["method"] == "fixture/request" {
                let (tx, rx) = oneshot::channel();
                published.try_send((raw.to_owned(), Some(tx))).map_err(|_| Error::Protocol)?;
                Some(wire::ServerReply::Deferred(rx))
            } else {
                published.try_send((raw.to_owned(), None)).map_err(|_| Error::Protocol)?;
                None
            };
            Ok(answer)
        });
        let driver = tokio::spawn(driver);
        let outstanding = link.enqueue("model/list", json!({}))?;
        let call: Value = serde_json::from_str(&input.next().await.ok_or("missing client request")??)?;
        assert_eq!(call["id"], 1);
        let number = r#"{"method": "fixture/request", "id":1,"params":{"threadId":"root"}}"#;
        let string = r#"{"method":"fixture/request","id":"1","params":{"threadId":"root"}}"#;
        output.send(number).await?;
        output.send(string).await?;
        let (original, number_answer) = requests.recv().await.ok_or("numeric request")?;
        assert_eq!(original, number);
        let mut number_answer = number_answer.ok_or("numeric answer")?;
        let (_, string_answer) = requests.recv().await.ok_or("string request")?;
        let string_answer = string_answer.ok_or("string answer")?;
        output.send(json!({"id":1,"result":{"data":[]}}).to_string()).await?;
        assert_eq!(outstanding.receive().await?, json!({"data":[]}));
        output.send(json!({"method":"serverRequest/resolved","params":{"threadId":"foreign","requestId":1}}).to_string()).await?;
        let _ = requests.recv().await.ok_or("foreign resolution")?;
        assert!(!number_answer.is_closed());
        output.send(json!({"method":"serverRequest/resolved","params":{"threadId":"root","requestId":1}}).to_string()).await?;
        let _ = requests.recv().await.ok_or("root resolution")?;
        number_answer.closed().await;
        assert!(number_answer.send(Ok(json!({"action":"accept"}))).is_err());
        assert!(!string_answer.is_closed());
        string_answer.send(Ok(json!({"action":"decline"}))).map_err(|_| "answer closed")?;
        let response: Value = serde_json::from_str(&input.next().await.ok_or("missing response")??)?;
        assert_eq!(response, json!({"id":"1","result":{"action":"decline"}}));
        done.cancel();
        driver.await??;
        Ok(())
    })
    .await?
}

#[tokio::test]
async fn exact_resume_and_standard_are_sent_on_the_same_direct_connection() -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), async {
        let (client_side, server_side) = duplex(2048);
        let (read, write) = split(client_side);
        let (read_server, write_server) = split(server_side);
        let mut input = FramedRead::new(read_server, LinesCodec::new());
        let mut output = write_server;
        let done = CancellationToken::new();
        let exact = NativeConversationId::parse("existing".into())?;
        let insertions = Arc::new(events::Insertions::new(exact.clone()));
        let observed = insertions.clone();
        let (link, driver) = wire::connect(read, write, done.clone(), move |frame| {
            if frame["method"] == "fixture/request" {
                // Even the exact same numeric ID is a different request.
                Ok(Some(json!({"id": frame["id"], "result": {"seen": true}})))
            } else {
                observed.observe(&frame)?;
                Ok(None)
            }
        });
        let driver = tokio::spawn(driver);
        let server = async {
            let mut methods = vec![];
            while let Some(frame) = input.next().await {
                let value: Value = serde_json::from_str(&frame?)?;
                let method = value["method"].as_str().ok_or("Missing method")?;
                methods.push(method.to_owned());
                let result = match method {
                    "initialize" => json!({}),
                    "initialized" => continue,
                    "account/login/start" => {
                        assert_eq!(
                            value["params"],
                            json!({"type": "apiKey", "apiKey": "fixture"})
                        );
                        json!({"type": "apiKey"})
                    }
                    "thread/resume" => {
                        assert_eq!(
                            value["params"],
                            json!({
                                "threadId": "existing", "cwd": "/fixture", "excludeTurns": true,
                            })
                        );
                        json!({"thread": {"id": "existing"}})
                    }
                    "turn/start" => {
                        assert_eq!(value["params"]["threadId"], "existing");
                        let marker = value["params"]["clientUserMessageId"]
                            .as_str()
                            .ok_or("Missing insertion marker")?;
                        assert!(marker.starts_with("insertion-"));
                        assert_eq!(value["params"]["serviceTierForTurn"], "default");
                        assert!(value["params"].get("serviceTier").is_none());
                        let same_id = &value["id"];
                        send(
                            &mut output,
                            json!({"method": "fixture/request", "id": same_id}),
                        )
                        .await?;
                        let reply: Value = serde_json::from_str(
                            &input.next().await.ok_or("Missing server response")??,
                        )?;
                        assert_eq!(reply, json!({"id": same_id, "result": {"seen": true}}));
                        send(
                            &mut output,
                            json!({"id": same_id, "result": {"turn": {"id": "t"}}}),
                        )
                        .await?;
                        for (thread, item) in [("child", "wrong"), ("existing", "attributed")] {
                            send(
                                &mut output,
                                json!({"method": "item/started", "params": {
                                    "threadId": thread, "turnId": "t",
                                    "item": {"type": "userMessage", "id": item, "clientId": marker},
                                }}),
                            )
                            .await?;
                        }
                        break;
                    }
                    _ => return Err("Unexpected method".into()),
                };
                send(&mut output, json!({"id":value["id"], "result": result})).await?;
            }
            assert_eq!(
                methods,
                [
                    "initialize",
                    "initialized",
                    "account/login/start",
                    "thread/resume",
                    "turn/start"
                ]
            );
            Ok(())
        };
        let exercise = async {
            initialize(&link, Some("fixture")).await?;
            let thread = Thread::resume(&link, &exact, Path::new("/fixture")).await?;
            let input = ConversationText::parse("fixture prompt".into())?;
            let received = thread.submit(&insertions, &input, Some("default"))?.await?;
            assert_eq!(received.turn, "t");
            assert_eq!(received.item, "attributed");
            done.cancel();
            Ok::<(), Box<dyn std::error::Error>>(())
        };
        let (a, b): (TestResult, TestResult) = tokio::join!(server, exercise);
        a?;
        b?;
        // EOF can win the shutdown race once the fixture closes its halves.
        let _ = driver.await?;
        Ok(())
    })
    .await?
}

#[tokio::test]
async fn unobserved_insertion_keeps_lane_reserved_after_waiter_cancellation() -> TestResult {
    let root = NativeConversationId::parse("existing".into())?;
    let insertions = events::Insertions::new(root);
    assert!(matches!(
        insertions.publish(None, |_| Err::<(), _>(Error::NotAdmitted)),
        Err(Error::NotAdmitted)
    ));
    let (marker, receipt) = insertions.publish(Some("expected"), |marker| Ok(marker.to_owned()))?;
    drop(receipt);
    let mut event = json!({
        "method": "item/started",
        "params": {"threadId": "existing", "turnId": "later",
            "item": {"type": "userMessage", "id": "item", "clientId": marker}}
    });
    assert!(insertions.observe(&event).is_err());
    assert!(matches!(
        insertions.publish(None, |_| Ok(())),
        Err(Error::DeliveryUncertain)
    ));
    event["params"]["turnId"] = json!("expected");
    insertions.observe(&event)?;
    let (next_marker, mut receipt) = insertions.publish(None, |marker| Ok(marker.to_owned()))?;
    assert_ne!(next_marker, marker);
    insertions.observe(&event)?;
    assert!(matches!(
        receipt.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));
    Ok(())
}

#[tokio::test]
async fn steering_and_stop_send_only_the_exact_native_turn() -> TestResult {
    let (local, remote) = duplex(4096);
    let (read, write) = split(local);
    let (server_read, mut output) = split(remote);
    let mut input = FramedRead::new(server_read, LinesCodec::new());
    let done = CancellationToken::new();
    let id = NativeConversationId::parse("root".into())?;
    let insertions = Arc::new(events::Insertions::new(id.clone()));
    let observed = insertions.clone();
    let (link, driver) = wire::connect(read, write, done.clone(), move |frame| {
        observed.observe(&frame)?;
        Ok(None)
    });
    let driver = tokio::spawn(driver);
    let thread = Thread { link: &link, id };
    assert!(matches!(thread.interrupt(""), Err(Error::NotAdmitted)));
    let server = async {
        let request: Value = serde_json::from_str(&input.next().await.ok_or("Missing steer")??)?;
        assert_eq!(request["method"], "turn/steer");
        let marker = request["params"]["clientUserMessageId"]
            .as_str()
            .ok_or("Missing marker")?;
        assert_eq!(
            request["params"],
            json!({"threadId": "root", "expectedTurnId": "exact",
                   "clientUserMessageId": marker, "input": [{"type": "text", "text": "follow up"}]})
        );
        // A live insertion may precede the method response.
        send(
            &mut output,
            json!({"method": "item/started", "params": {
                "threadId": "root", "turnId": "exact",
                "item": {"type": "userMessage", "id": "inserted", "clientId": marker},
            }}),
        )
        .await?;
        send(
            &mut output,
            json!({"id": request["id"], "result": {"turnId": "exact"}}),
        )
        .await?;
        let request: Value = serde_json::from_str(&input.next().await.ok_or("Missing Stop")??)?;
        assert_eq!(request["method"], "turn/interrupt");
        assert_eq!(
            request["params"],
            json!({"threadId": "root", "turnId": "exact"})
        );
        send(&mut output, json!({"id": request["id"], "result": {}})).await
    };
    let exercise = async {
        let text = ConversationText::parse("follow up".into())?;
        let inserted = thread.steer(&insertions, "exact", &text)?.await?;
        assert_eq!(inserted.item, "inserted");
        assert_eq!(thread.interrupt("exact")?.receive().await?, json!({}));
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let (server, exercise) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(server, exercise)
    })
    .await?;
    server?;
    exercise?;
    done.cancel();
    driver.await??;
    Ok(())
}

#[tokio::test]
async fn lost_reply_is_uncertain_and_does_not_republish() -> TestResult {
    let (client_side, server_side) = duplex(2048);
    let (read, write) = split(client_side);
    let (mut read_server, write_server) = split(server_side);
    let (client, driver) = wire::connect(read, write, CancellationToken::new(), |_| Ok(None));
    let driver = tokio::spawn(driver);
    let reply = client.enqueue("turn/start", json!({}))?;
    let frame = tokio::time::timeout(Duration::from_secs(2), recv(&mut read_server)).await??;
    assert_eq!(frame["method"], "turn/start");
    drop(read_server);
    drop(write_server);
    assert_eq!(reply.receive().await, Err(CallFailure::Uncertain));
    assert!(driver.await?.is_err());
    assert!(matches!(
        client.enqueue("turn/start", json!({})),
        Err(Error::NotAdmitted)
    ));
    Ok(())
}

#[tokio::test]
async fn mcp_approval_answers_bind_to_the_exact_capture_and_never_remember() -> TestResult {
    use chapchoo_domain::{EventOccurrenceId, SnapshotId, SourcePosition};
    use tools::{Approval, Choice};

    let target = super::super::NativeTextTarget {
        snapshot: SnapshotId::parse("snapshot".into())?,
        native_session: NativeConversationId::parse("native".into())?,
    };
    let position = SourcePosition::new(1, EventOccurrenceId::parse("decision".into())?)?;
    let wrong_position = SourcePosition::new(1, EventOccurrenceId::parse("other".into())?)?;
    let frame = json!({
        "id": 7, "method": "mcpServer/elicitation/request",
        "params": {
            "threadId": "native", "turnId": "live-turn", "serverName": "chapchoo", "mode": "form",
            "requestedSchema": {"type": "object", "properties": {}},
            "_meta": {"codex_approval_kind":"mcp_tool_call", "tool_name":"inspect", "persist":"always"}
        }
    });
    for (field, value) in [
        ("threadId", json!("elsewhere")),
        ("turnId", Value::Null),
        ("serverName", json!("another")),
        ("mode", json!("url")),
        ("_meta", json!({"tool_name":"inspect"})),
        (
            "requestedSchema",
            json!({"type":"object", "properties":{"extra":{"type":"string"}}}),
        ),
    ] {
        let mut invalid = frame.clone();
        invalid["params"][field] = value;
        assert!(Approval::offer(target.clone(), position.clone(), "live-turn", invalid)?.is_none());
    }
    for (choice, expected) in [
        (Choice::ApproveOnce, json!({"action":"accept","content":{}})),
        (Choice::Decline, json!({"action":"decline","content":null})),
    ] {
        let (mut approval, reply) =
            Approval::offer(target.clone(), position.clone(), "live-turn", frame.clone())?
                .ok_or("approval expected")?;
        let wire::ServerReply::Deferred(mut reply) = reply else {
            return Err("deferred expected".into());
        };
        assert_eq!(approval.tool, "inspect");
        assert!(matches!(
            approval.answer(&target, &wrong_position, "live-turn", choice),
            Err(Error::NotAdmitted)
        ));
        assert!(matches!(
            approval.answer(&target, &position, "later-turn", choice),
            Err(Error::NotAdmitted)
        ));
        assert!(matches!(
            reply.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        approval.answer(&target, &position, "live-turn", choice)?;
        assert!(matches!(
            approval.answer(&target, &position, "live-turn", choice),
            Err(Error::NotAdmitted)
        ));
        assert_eq!(reply.await?.map_err(|_| "reply rejected")?, expected);
    }
    Ok(())
}

#[tokio::test]
async fn start_and_resume_require_the_same_launch_local_mcp_declaration() -> TestResult {
    let (local, remote) = duplex(4096);
    let (read, write) = split(local);
    let (server_read, mut output) = split(remote);
    let mut input = FramedRead::new(server_read, LinesCodec::new());
    let done = CancellationToken::new();
    let (link, driver) = wire::connect(read, write, done.clone(), |_| Ok(None));
    let driver = tokio::spawn(driver);
    let bearer = chapchoo_crypto::BearerSecret::generate()?;
    let authorization = zeroize::Zeroizing::new(format!("Bearer {}", bearer.expose()));
    let declaration = crate::NativeMcpDeclaration::http("http://127.0.0.1:12345/mcp", bearer)?;
    let server = async {
        for (method, existing) in [("thread/start", false), ("thread/resume", true)] {
            let frame = input.next().await.ok_or("Missing setup request")??;
            let request: Value = serde_json::from_str(&frame)?;
            let params = &request["params"];
            assert_eq!(request["method"], method);
            assert_eq!(params["cwd"], "/fixture");
            assert_eq!(params["threadId"].as_str(), existing.then_some("same"));
            assert_eq!(params["excludeTurns"].as_bool(), existing.then_some(true));
            assert_eq!(params["approvalPolicy"], "on-request");
            assert_eq!(params["approvalsReviewer"], "user");
            let server = &params["config"]["mcp_servers"]["chapchoo"];
            assert_eq!(server["url"], "http://127.0.0.1:12345/mcp");
            assert_eq!(server["required"], true);
            assert_eq!(server["default_tools_approval_mode"], "prompt");
            assert_eq!(server["omit_tools_from"], json!(["code_mode", "deferred"]));
            assert!(
                server["http_headers"]["Authorization"].as_str() == Some(authorization.as_str())
            );
            send(
                &mut output,
                json!({"id":request["id"],"result":{"thread":{"id":"same"}}}),
            )
            .await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let exercise = async {
        let original = Thread::open(&link, Path::new("/fixture"), None, Some(&declaration)).await?;
        let resumed = Thread::open(
            &link,
            Path::new("/fixture"),
            Some(&original.id),
            Some(&declaration),
        )
        .await?;
        assert_eq!(resumed.id, original.id);
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let (server, exercise) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(server, exercise)
    })
    .await?;
    server?;
    exercise?;
    done.cancel();
    driver.await??;
    Ok(())
}

#[tokio::test]
async fn a_different_resume_id_is_never_adopted() -> TestResult {
    let (client_side, server_side) = duplex(2048);
    let (read, write) = split(client_side);
    let (mut read_server, mut write_server) = split(server_side);
    let done = CancellationToken::new();
    let (client, driver) = wire::connect(read, write, done.clone(), |_| Ok(None));
    let driver = tokio::spawn(driver);
    let exact = NativeConversationId::parse("existing".into())?;
    let server = async {
        let frame = recv(&mut read_server).await?;
        assert_eq!(frame["params"]["threadId"], "existing");
        send(
            &mut write_server,
            json!({"id": frame["id"], "result": {"thread": {"id": "other"}}}),
        )
        .await
    };
    let (server, result) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(
            server,
            Thread::resume(&client, &exact, Path::new("/fixture"))
        )
    })
    .await?;
    server?;
    assert!(matches!(result, Err(Error::CaptureConflict)));
    done.cancel();
    driver.await??;
    Ok(())
}

#[tokio::test]
async fn native_catalog_preserves_candidate_specific_options_before_opening_a_thread() -> TestResult
{
    let (client_side, server_side) = duplex(4096);
    let (read, write) = split(client_side);
    let (read_server, mut output) = split(server_side);
    let mut input = FramedRead::new(read_server, LinesCodec::new());
    let done = CancellationToken::new();
    let (client, driver) = wire::connect(read, write, done.clone(), |_| Ok(None));
    let driver = tokio::spawn(driver);
    let server = async {
        for (id, cursor, next_cursor, efforts, tiers) in [
            (
                "one",
                None,
                Some("more"),
                vec!["low", "high"],
                vec!["priority", "ultrafast"],
            ),
            ("two", Some("more"), None, vec!["low"], vec![]),
        ] {
            let frame = input.next().await.ok_or("Missing catalog call")??;
            let request: Value = serde_json::from_str(&frame)?;
            assert_eq!(request["method"], "model/list");
            assert_eq!(request["params"]["cursor"], json!(cursor));
            let efforts: Vec<_> = efforts
                .into_iter()
                .map(|effort| json!({"reasoningEffort": effort}))
                .collect();
            let tiers: Vec<_> = tiers.into_iter().map(|id| json!({"id": id})).collect();
            send(
                &mut output,
                json!({"id": request["id"], "result": {
                    "nextCursor": next_cursor,
                    "data": [{
                        "model": id, "displayName": id,
                        "supportedReasoningEfforts": efforts, "defaultReasoningEffort": "low",
                        "serviceTiers": tiers, "defaultServiceTier": null,
                    }],
                }}),
            )
            .await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let (server, result) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(server, model_catalog(&client))
    })
    .await?;
    server?;
    let result = result?;
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "one");
    assert_eq!(result[0].efforts, ["low", "high"]);
    assert_eq!(result[0].tiers, ["priority", "ultrafast"]);
    assert_eq!(result[1].id, "two");
    assert_eq!(result[1].efforts, ["low"]);
    assert!(result[1].tiers.is_empty());
    done.cancel();
    driver.await??;
    Ok(())
}
