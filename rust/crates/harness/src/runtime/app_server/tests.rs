use super::*;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, duplex, split};
use tokio_util::{
    codec::{FramedRead, FramedWrite, LinesCodec},
    sync::CancellationToken,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

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
    let mut frame =
        json!({"method": "turn/started", "params": {"threadId": "root", "turn": {"id": "turn"}}});
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
async fn exact_resume_and_standard_are_sent_on_the_same_direct_connection() -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), async {
        let (client_side, server_side) = duplex(2048);
        let (read, write) = split(client_side);
        let (read_server, write_server) = split(server_side);
        let mut input = FramedRead::new(read_server, LinesCodec::new());
        let mut output = write_server;
        let done = CancellationToken::new();
        let (link, driver) = wire::connect(read, write, done.clone(), |frame| {
            if frame["method"] == "fixture/request" {
                // Even the exact same numeric ID is a different request.
                Ok(Some(json!({"id": frame["id"], "result": {"seen": true}})))
            } else {
                Err(Error::Protocol)
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
                        assert_eq!(value["params"]["clientUserMessageId"], "op");
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
            let exact = NativeConversationId::parse("existing".into())?;
            let thread = Thread::resume(&link, &exact, Path::new("/fixture")).await?;
            let input = ConversationText::parse("fixture prompt".into())?;
            let received = thread
                .enqueue_start("op", &input, Some("default"))?
                .receive()
                .await?;
            assert_eq!(received["turn"]["id"], "t");
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
