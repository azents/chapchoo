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
        let form_received = tokio::sync::Notify::new();
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
                    form_received.notify_one();
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
            assert!(session.steer(&text, None).await?.is_none());
            assert_eq!(session.enqueue_prompt(&text, None, &Default::default())?.await?, "first");
            assert!(matches!(session.steer(&text, None).await?,
                Some(crate::runtime::NativePromptAcceptance::Inserted { native_message_id })
                    if native_message_id == "steered"));
            let Session::AppServer(thread) = &session else { return Err("Not App Server".into()) };
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
            form_received.notified().await;
            let work = observed.current()?.work.ok_or("No live turn")?;
            let mut wrong = work.clone();
            wrong.work_id = "old".into();
            assert!(matches!(
                observed.publish_stop(&wrong, || session.stop(&wrong)),
                Err(Error::NotAdmitted)
            ));
            observed.publish_stop(&work, || session.stop(&work))?;
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
