//! Opt-in qualification against a disposable, loopback-only Codex fixture.
//! The fixture prepares the previous thread and closes its native process.
use super::*;
use crate::{CaptureLog, NativeStorage, runtime::NativeTextTarget};
use std::{path::PathBuf, process::Stdio};
use tokio::{process::Command, sync::mpsc};

enum Seen {
    Approval(tools::Approval, chapchoo_domain::SourcePosition),
    Terminal(String, bool),
}

#[tokio::test]
#[ignore = "requires a caller-owned Codex 0.171.0 process fixture on loopback"]
async fn released_resume_preserves_capture_and_answers_scoped_tool_approval() -> TestResult {
    tokio::time::timeout(Duration::from_secs(65), async {
        let binary = PathBuf::from(std::env::var("CHAPCHOO_TEST_RELEASED_CODEX")?);
        let cwd = PathBuf::from(std::env::var("CHAPCHOO_TEST_RETAINED_WORKSPACE")?);
        let home = PathBuf::from(std::env::var("CHAPCHOO_TEST_RETAINED_CODEX_HOME")?);
        let origin = std::env::var("CHAPCHOO_TEST_PROBE_ORIGIN")?;
        let url = reqwest::Url::parse(&origin)?;
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("127.0.0.1"));
        assert!(url.port().is_some() && url.path() == "/" && url.as_str().trim_end_matches('/') == origin);
        assert!(binary.is_absolute() && cwd.is_absolute() && home.is_absolute());
        let id = NativeConversationId::parse(std::env::var("CHAPCHOO_TEST_RETAINED_THREAD")?)?;
        let temp = tempfile::tempdir()?;
        let storage = NativeStorage::open(temp.path())?;
        let retained = CaptureLog::create(&storage)?;
        retained.bind_native(&id)?;
        // This prefix stands for the capture that Chapchoo already retained
        // before the native process was stopped. Do not regenerate it.
        let prefix = json!({"method":"session/update","params":{"sessionId":id.as_str(),
            "update":{"sessionUpdate":"user_message_chunk","messageId":"historical","content":{"type":"text","text":"Retained prefix"}}}}).to_string();
        let first = retained.append(&prefix, None)?;
        let snapshot = retained.snapshot()?;
        drop(retained);
        let captured = CaptureLog::open(&storage)?;
        let target = NativeTextTarget { snapshot: snapshot.clone(), native_session: id.clone() };
        let bearer = chapchoo_crypto::BearerSecret::generate()?;
        // The fixture accepts this process's dynamically scoped bearer and no
        // production credential. The URL was checked before handing it over.
        let provision = crate::NativeMcpDeclaration::http(&format!("{origin}/mcp"), bearer)?;
        let mut child = Command::new(binary)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").ok_or("Missing PATH")?)
            .env("HOME", temp.path())
            .env("CODEX_HOME", &home)
            .env("OPENAI_BASE_URL", format!("{origin}/v1"))
            .env("HTTP_PROXY", "http://127.0.0.1:9")
            .env("HTTPS_PROXY", "http://127.0.0.1:9")
            .env("ALL_PROXY", "http://127.0.0.1:9")
            .env("NO_PROXY", "127.0.0.1")
            .env("OTEL_SDK_DISABLED", "true")
            .current_dir(&cwd)
            .args(["app-server", "--listen", "stdio://"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let (tx, mut rx) = mpsc::channel(16);
        let done = CancellationToken::new();
        let insertions = Arc::new(events::Insertions::new(id.clone()));
        let live_insertions = insertions.clone();
        let live_log = captured.clone();
        let live_target = target.clone();
        let mut active_turn = None::<String>;
        let (client, driver) = wire::connect_with_requests(
            child.stdout.take().ok_or("Missing stdout")?,
            child.stdin.take().ok_or("Missing stdin")?,
            done.clone(),
            move |raw, frame| {
                live_insertions.observe(&frame)?;
                match events::root_observation(&live_target.native_session, &frame)? {
                    Some(events::Observation::Started { turn }) => active_turn = Some(turn.to_owned()),
                    Some(events::Observation::Terminal { turn, normal })
                        if active_turn.as_deref() == Some(turn) =>
                    {
                        tx.try_send(Seen::Terminal(turn.to_owned(), normal)).map_err(|_| Error::Protocol)?;
                        active_turn = None;
                    }
                    _ => {}
                }
                if frame["params"]["threadId"] != live_target.native_session.as_str() {
                    return Ok(None);
                }
                match frame["method"].as_str() {
                    Some("mcpServer/elicitation/request") => {
                        let turn = active_turn.as_deref().ok_or(Error::NotAdmitted)?;
                        let position = live_log.append(raw, None)?;
                        let (approval, reply) = tools::Approval::offer(live_target.clone(), position.clone(), turn, frame)?
                            .ok_or(Error::NotAdmitted)?;
                        tx.try_send(Seen::Approval(approval, position)).map_err(|_| Error::Protocol)?;
                        Ok(Some(reply))
                    }
                    Some("item/completed" | "item/agentMessage/delta") => {
                        live_log.append(raw, None)?;
                        Ok(None)
                    }
                    _ => Ok(None),
                }
            },
        );
        let driven = tokio::spawn(driver);
        initialize(&client, None).await?;
        let thread = Thread::open(&client, &cwd, Some(&id), Some(&provision)).await?;
        assert_eq!(thread.id, id);
        let catalog = model_catalog(&client).await?;
        assert!(!catalog.is_empty());
        let status = call(&client, ProtocolStage::Session, "mcpServerStatus/list", json!({
            "threadId": id.as_str(), "serverName": "chapchoo",
        })).await?;
        assert!(status["data"].as_array().is_some_and(|servers| servers.iter().any(|server| {
            server["name"] == "chapchoo" && server["tools"].get("echo").is_some()
        })), "The scoped echo tool must be discoverable before submission");
        let text = ConversationText::parse("Approve the local fixture tool.".into())?;
        let inserted = thread.submit(&insertions, &text, Some("default"))?.await?;
        let (mut approval, position) = match rx.recv().await {
            Some(Seen::Approval(approval, position)) => (approval, position),
            Some(Seen::Terminal(_, true)) => return Err("Turn completed without requesting approval".into()),
            Some(Seen::Terminal(_, false)) => return Err("Turn failed before requesting approval".into()),
            None => { driven.await??; return Err("Driver closed before approval".into()); }
        };
        assert_eq!(approval.tool, "echo");
        assert!(matches!(approval.answer(&target, &position, "different-turn", tools::Choice::ApproveOnce), Err(Error::NotAdmitted)));
        approval.answer(&target, &position, &inserted.turn, tools::Choice::ApproveOnce)?;
        assert!(matches!(approval.answer(&target, &position, &inserted.turn, tools::Choice::ApproveOnce), Err(Error::NotAdmitted)));
        let Some(Seen::Terminal(turn, normal)) = rx.recv().await else {
            return Err("No terminal turn".into());
        };
        assert!(normal);
        assert_eq!(turn, inserted.turn);
        let mut pass = captured.begin(&snapshot, None)?;
        let original = pass.next_event()?.ok_or("No historical capture")?;
        assert_eq!(original.raw, prefix);
        assert_eq!(original.position, first);
        let mut contains_approval = false;
        while let Some(event) = pass.next_event()? {
            if event.position == position {
                let frame: Value = serde_json::from_str(&event.raw)?;
                assert_eq!(frame["method"], "mcpServer/elicitation/request");
                assert_eq!(frame["params"]["turnId"], inserted.turn);
                contains_approval = true;
            }
        }
        assert!(contains_approval);
        done.cancel();
        driven.await??;
        child.wait().await?;
        Ok(())
    }).await?
}
