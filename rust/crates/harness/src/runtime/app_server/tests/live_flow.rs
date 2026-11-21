use super::*;
use crate::runtime::{
    InteractiveRequest, InteractiveResponse, NativeContext, NativeForeground,
    observations::Observations, session::Session,
};

#[tokio::test]
async fn new_session_steers_and_stops_only_its_exact_live_turn() -> TestResult {
    tokio::time::timeout(Duration::from_secs(10), async {
        let saved = tempfile::tempdir()?;
        let storage = crate::NativeStorage::open(saved.path())?;
        let capture = crate::CaptureLog::create(&storage)?;
        let observed = Arc::new(Observations::default());
        let (own, peer) = duplex(16 * 1024);
        let (read, write) = split(own);
        let done = CancellationToken::new();
        let (connection, driver) =
            Connection::connect(read, write, done.clone(), capture.clone(), observed.clone());
        let driver = tokio::spawn(driver);
        let (read, mut write) = split(peer);
        let mut input = FramedRead::new(read, LinesCodec::new());
        let barrier_finished = tokio::sync::Notify::new();
        let stop_fence_checked = tokio::sync::Notify::new();
        let server = async {
            let mut published = Vec::new();
            while let Some(line) = input.next().await {
                let message: Value = serde_json::from_str(&line?)?;
                let method = message["method"].as_str().ok_or("Missing method")?;
                let params = &message["params"];
                published.push(method.to_owned());
                let result =
                    match method {
                        "initialize" => json!({}),
                        "initialized" => continue,
                        "account/login/start" => {
                            assert_eq!(params, &json!({"type":"apiKey","apiKey":"fixture-only"}));
                            json!({"type":"apiKey"})
                        }
                        "thread/start" => {
                            assert_eq!(
                                params["cwd"],
                                storage.workspace().to_str().ok_or("Invalid cwd")?
                            );
                            json!({"thread":{"id":"new-root"},"model":"fixture"})
                        }
                        "thread/name/set" => {
                            assert_eq!(params["threadId"], "new-root");
                            json!({})
                        }
                        "model/list" | "mcpServerStatus/list" => {
                            json!({"data":[],"nextCursor":null})
                        }
                        "turn/start" | "turn/steer" => {
                            assert_eq!(params["threadId"], "new-root");
                            let (item, result) = if method == "turn/start" {
                                send(&mut write, json!({"method":"turn/started", "params":{
                                "threadId":"new-root","turn":{"id":"active","status":"inProgress"}
                            }})).await?;
                                ("first", json!({"turn":{"id":"active"}}))
                            } else {
                                assert_eq!(params["expectedTurnId"], "active");
                                assert!(params.get("model").is_none());
                                assert!(params.get("serviceTierForTurn").is_none());
                                ("steered", json!({"turnId":"active"}))
                            };
                            let item = json!({"type":"userMessage","id":item,
                            "clientId":params["clientUserMessageId"],"content":params["input"]});
                            for event in ["item/started", "item/completed"] {
                                send(
                                    &mut write,
                                    json!({"method":event,"params":{
                                        "threadId":"new-root","turnId":"active","item":item
                                    }}),
                                )
                                .await?;
                            }
                            result
                        }
                        "turn/interrupt" => {
                            assert_eq!(params, &json!({"threadId":"new-root","turnId":"active"}));
                            stop_fence_checked.notified().await;
                            send(&mut write, json!({"method":"serverRequest/resolved","params":{
                                "threadId":"new-root","requestId":"stop-grant"
                            }})).await?;
                            send(&mut write, json!({"method":"turn/completed","params":{
                            "threadId":"new-root","turn":{"id":"active","status":"interrupted"}
                        }})).await?;
                            send(&mut write, json!({"id":message["id"],"result":{}})).await?;
                            break;
                        }
                        _ => return Err("Unexpected RPC".into()),
                    };
                send(&mut write, json!({"id":message["id"],"result":result})).await?;
                if method == "turn/steer" {
                    send(&mut write, json!({
                        "id":"fixture-form","method":"mcpServer/elicitation/request",
                        "params":{
                            "threadId":"new-root","turnId":"active","serverName":"chapchoo","mode":"form",
                            "message":"Enable the fixture?",
                            "requestedSchema":{"type":"object","properties":{"enabled":{"type":"boolean"}},"required":["enabled"]}
                        }
                    })).await?;
                    let answer: Value = serde_json::from_str(&input.next().await.ok_or("Missing form reply")??)?;
                    assert_eq!(answer, json!({"id":"fixture-form","result":{"action":"accept","content":{"enabled":false}}}));
                    for (id, method, params, decision) in [
                        (17, "item/commandExecution/requestApproval", json!({
                            "threadId":"new-root","turnId":"active","itemId":"exec","startedAtMs":1,
                            "command":"printf fixture","cwd":"/fixture"
                        }), "accept"),
                        (18, "item/fileChange/requestApproval", json!({
                            "threadId":"new-root","turnId":"active","itemId":"patch","startedAtMs":2,
                            "reason":"Write the fixture"
                        }), "cancel"),
                        (22, "item/permissions/requestApproval", json!({
                            "threadId":"new-root","turnId":"active","itemId":"sandbox",
                            "startedAtMs":3,"cwd":"/fixture","reason":"Read the fixture input",
                            "permissions":{"network":{"enabled":false},"fileSystem":{"read":["/fixture/in"]}}
                        }), "grant-turn-strict"),
                        (19, "item/commandExecution/requestApproval", json!({
                            "threadId":"new-root","turnId":"old","itemId":"stale","startedAtMs":3,
                            "command":"printf stale","cwd":"/fixture"
                        }), ""),
                        (20, "item/fileChange/requestApproval", json!({
                            "threadId":"foreign","turnId":"active","itemId":"other","startedAtMs":4
                        }), ""),
                        (21, "item/fileChange/requestApproval", json!({
                            "threadId":"new-root","turnId":"active","itemId":"unknown","startedAtMs":5,
                            "reason":"Different item"
                        }), ""),
                    ] {
                        if id == 18 {
                            send(&mut write, file_change_started("patch")).await?;
                        }
                        send(&mut write, json!({"id":id,"method":method,"params":params})).await?;
                        let answer: Value = serde_json::from_str(&input.next().await.ok_or("Missing approval reply")??)?;
                        if decision.is_empty() {
                            assert_eq!(answer["id"], id);
                            assert_eq!(answer["error"]["code"], -32601);
                        } else if id == 22 {
                            assert_eq!(answer, json!({"id":22,"result":{
                                "permissions":{"network":{"enabled":false},"fileSystem":{"read":["/fixture/in"]}},
                                "scope":"turn","strictAutoReview":true
                            }}));
                        } else {
                            assert_eq!(answer, json!({"id":id,"result":{"decision":decision}}));
                        }
                    }
                    send(&mut write, json!({"id":"native-question","method":"item/tool/requestUserInput","params":{
                        "threadId":"new-root","turnId":"active","itemId":"question","isBlocking":true,
                        "questions":[{"id":"format","header":"Fixture","question":"Which format?",
                            "isOther":true,"options":[{"label":"Text","description":"Plain text"}]}]
                    }})).await?;
                    let reply: Value = serde_json::from_str(&input.next().await.ok_or("Missing native tool question reply")??)?;
                    assert_eq!(reply, json!({"id":"native-question","result":{"answers":{
                        "format":{"answers":["None of the above","user_note: Markdown"]}
                    }}}));
                    send(&mut write, file_change_started("resolved")).await?;
                    send(&mut write, json!({"id":"resolved-first","method":"item/fileChange/requestApproval",
                        "params":{"threadId":"new-root","turnId":"active","itemId":"resolved","startedAtMs":5}
                    })).await?;
                    send(&mut write, json!({"method":"serverRequest/resolved",
                        "params":{"threadId":"foreign","requestId":"resolved-first"}
                    })).await?;
                    send(&mut write, json!({"id":"barrier","method":"mcpServer/elicitation/request",
                        "params":{"threadId":"new-root","turnId":"active","serverName":"chapchoo","mode":"form",
                                  "message":"Barrier 1","requestedSchema":{"type":"object","properties":{"enabled":{"type":"boolean"}}}}
                    })).await?;
                    let barrier: Value = serde_json::from_str(&input.next().await.ok_or("Missing first barrier")??)?;
                    assert_eq!(barrier["id"], "barrier");
                    send(&mut write, json!({"method":"serverRequest/resolved",
                        "params":{"threadId":"new-root","requestId":"resolved-first"}
                    })).await?;
                    send(&mut write, json!({"id":"barrier","method":"mcpServer/elicitation/request",
                        "params":{"threadId":"new-root","turnId":"active","serverName":"chapchoo","mode":"form",
                                  "message":"Barrier 2","requestedSchema":{"type":"object","properties":{"enabled":{"type":"boolean"}}}}
                    })).await?;
                    let barrier: Value = serde_json::from_str(&input.next().await.ok_or("Missing second barrier")??)?;
                    assert_eq!(barrier["id"], "barrier");
                    send(&mut write, json!({"id":"stop-grant","method":"item/permissions/requestApproval","params":{
                        "threadId":"new-root","turnId":"active","itemId":"sandbox",
                        "startedAtMs":5,"cwd":"/fixture","permissions":{"network":{"enabled":true}}
                    }})).await?;
                    barrier_finished.notify_one();
                }
            }
            assert_eq!(
                published,
                [
                    "initialize",
                    "initialized",
                    "account/login/start",
                    "thread/start",
                    "thread/name/set",
                    "model/list",
                    "mcpServerStatus/list",
                    "turn/start",
                    "turn/steer",
                    "turn/interrupt",
                ]
            );
            Ok(())
        };
        let execute = async {
            let mcp = crate::NativeMcpDeclaration::http(
                "http://127.0.0.1:4321/mcp",
                chapchoo_crypto::BearerSecret::generate()?,
            )?;
            let session = Session::AppServer(
                connection
                    .open(
                        &storage.workspace(),
                        &NativeContext::New,
                        &mcp,
                        Some("fixture-only"),
                        true,
                    )
                    .await?,
            );
            assert_eq!(
                capture
                    .native_session()?
                    .ok_or("No capture binding")?
                    .as_str(),
                "new-root"
            );
            observed.enable_normal_endings()?;
            let text = ConversationText::parse("Fixture input".into())?;
            let Session::AppServer(thread) = &session else { return Err("Not App Server".into()) };
            assert!(session.steer(&text, None).await?.is_none());
            assert_eq!(session.enqueue_prompt(&text, None, &Default::default())?.await?, "first");
            assert!(matches!(session.steer(&text, None).await?,
                Some(crate::runtime::NativePromptAcceptance::Inserted { native_message_id })
                    if native_message_id == "steered"));
            let (id, occurrence) = loop {
                if let Some(InteractiveRequest::Form { id, occurrence, request }) = thread.pending()?.pop() {
                    assert_eq!(request.message, "Enable the fixture?");
                    break (id, occurrence);
                }
                observed.wait_change().await;
            };
            let response = |enabled: Value| -> TestResult<InteractiveResponse> {
                Ok(InteractiveResponse::Form { id: id.clone(), response: serde_json::from_value(
                    json!({"action":"accept","content":{"enabled":enabled}})
                )? })
            };
            assert!(matches!(thread.respond(response(json!("false"))?, Some(&occurrence)), Err(Error::NotAdmitted)));
            thread.respond(response(json!(false))?, Some(&occurrence))?;
            assert!(matches!(thread.respond(response(json!(false))?, Some(&occurrence)), Err(Error::NotAdmitted)));
            let permission = |id: String, option: &str| -> TestResult<InteractiveResponse> {
                Ok(InteractiveResponse::Permission { id, response: serde_json::from_value(
                    json!({"outcome":{"outcome":"selected","optionId":option}})
                )? })
            };
            for (title, option) in [
                ("Allow command execution?", "approve-once"),
                ("Allow file changes?", "cancel"),
                ("Grant sandbox permissions?", "grant-turn-strict"),
            ] {
                let (approval, position) = loop {
                    if let Some(InteractiveRequest::Permission { id, occurrence, request }) = thread.pending()?.pop() {
                        assert_eq!(request.title, title);
                        if option == "cancel" {
                            assert!(request.description.as_deref().is_some_and(|text| text.contains("/fixture/note")));
                            assert_eq!(request.options.len(), 3);
                        } else {
                            assert_eq!(request.options.len(), 4);
                        }
                        break (id, occurrence);
                    }
                    observed.wait_change().await;
                };
                assert!(matches!(thread.respond(permission(approval.clone(), option)?, Some(&occurrence)), Err(Error::NotAdmitted)));
                assert!(matches!(thread.respond(permission(approval.clone(), "approve-always")?, Some(&position)), Err(Error::NotAdmitted)));
                thread.respond(permission(approval.clone(), option)?, Some(&position))?;
                assert!(matches!(thread.respond(permission(approval, option)?, Some(&position)), Err(Error::NotAdmitted)));
            }
            let (id, at) = loop {
                if let Some(InteractiveRequest::Form { id, occurrence, request }) = thread.pending()?.pop() {
                    assert_eq!(request.message, "Codex needs your input to continue.");
                    break (id, occurrence);
                }
                observed.wait_change().await;
            };
            let answer = |choice| -> TestResult<InteractiveResponse> {
                Ok(InteractiveResponse::Form {
                    id: id.clone(),
                    response: serde_json::from_value(json!({"action":"accept","content":{
                        "format":choice,"chapchoo-note-0":" Markdown "
                    }}))?,
                })
            };
            assert!(matches!(thread.respond(answer("Not offered")?, Some(&at)), Err(Error::NotAdmitted)));
            thread.respond(answer("None of the above")?, Some(&at))?;
            assert!(matches!(thread.respond(answer("Text")?, Some(&at)), Err(Error::NotAdmitted)));
            let mut retired = None;
            for expected in ["Barrier 1", "Barrier 2"] {
                let (barrier, at, pending) = loop {
                    let pending = thread.pending()?;
                    if let Some(InteractiveRequest::Form { id, occurrence, .. }) =
                        pending.iter().find(|request| matches!(request, InteractiveRequest::Form { request, .. } if request.message == expected))
                    {
                        break (id.clone(), occurrence.clone(), pending);
                    }
                    observed.wait_change().await;
                };
                if expected == "Barrier 1" {
                    retired = pending.iter().find_map(|request| match request {
                        InteractiveRequest::Permission { id, occurrence, .. } => Some((id.clone(), occurrence.clone())),
                        _ => None,
                    });
                    assert!(retired.is_some());
                } else {
                    let (id, at) = retired.take().ok_or("Missing resolved approval")?;
                    assert!(pending.iter().all(|request| !matches!(request, InteractiveRequest::Permission { .. })));
                    assert!(matches!(thread.respond(permission(id, "approve-once")?, Some(&at)), Err(Error::NotAdmitted)));
                }
                let response = InteractiveResponse::Form { id: barrier, response: serde_json::from_value(
                    json!({"action":"accept","content":{"enabled":true}})
                )? };
                thread.respond(response, Some(&at))?;
            }
            barrier_finished.notified().await;
            let (id, at) = loop {
                if let Some(InteractiveRequest::Permission { id, occurrence, .. }) = thread.pending()?.pop() {
                    break (id, occurrence);
                }
                observed.wait_change().await;
            };
            let work = observed.current()?.work.ok_or("No live turn")?;
            let mut wrong = work.clone();
            wrong.work_id = "old".into();
            assert!(matches!(
                observed.publish_stop(&wrong, || session.stop(&wrong)),
                Err(Error::NotAdmitted)
            ));
            observed.publish_stop(&work, || session.stop(&work))?;
            assert!(thread.pending()?.is_empty());
            assert!(matches!(thread.respond(permission(id, "grant-session")?, Some(&at)), Err(Error::NotAdmitted)));
            stop_fence_checked.notify_one();
            while observed.current()?.foreground != NativeForeground::Idle {
                observed.wait_change().await;
            }
            assert!(observed.normal_ending()?.is_none());
            done.cancel();
            Ok(()) as TestResult
        };
        tokio::try_join!(server, execute)?;
        driver.await??;
        Ok(())
    })
    .await?
}

fn file_change_started(id: &str) -> Value {
    json!({"method":"item/started","params":{
        "threadId":"new-root","turnId":"active","item":{
            "id":id,"type":"fileChange","status":"inProgress",
            "changes":[{"path":"/fixture/note","kind":{"type":"add"},"diff":"fixture"}]
        }
    }})
}
