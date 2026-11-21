//! Opt-in qualification against a disposable, loopback-only Codex fixture.
//! The fixture prepares the previous thread and closes its native process.
use super::*;
use crate::runtime::observations::Observations;
use crate::{
    CaptureLog, NativeStorage,
    runtime::{CaptureReplay, InteractiveRequest, InteractiveResponse, NativeContext},
};
use std::{path::PathBuf, process::Stdio};
use tokio::process::Command;

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
        let done = CancellationToken::new();
        let observations = Arc::new(Observations::default());
        let (connection, driver) = Connection::connect(
            child.stdout.take().ok_or("Missing stdout")?,
            child.stdin.take().ok_or("Missing stdin")?,
            done.clone(),
            captured.clone(),
            observations.clone(),
        );
        let driven = tokio::spawn(driver);
        let thread = connection.open(&cwd, &NativeContext::Resume {
            session: id.clone(), replay: CaptureReplay::AlreadyCaptured,
        }, &provision, None, false).await?;
        assert_eq!(thread.id(), id.as_str());
        assert!(thread.models().is_some());
        observations.enable_normal_endings()?;
        let text = ConversationText::parse("Approve the local fixture tool.".into())?;
        let inserted = tokio::time::timeout(Duration::from_secs(15), thread.enqueue_prompt(&text, None)?)
            .await.map_err(|_| "No attributed insertion")??;
        assert!(!inserted.is_empty());
        let (request_id, position) = loop {
            if let Some(InteractiveRequest::Permission { id, occurrence, request }) = thread.pending()?.pop() {
                assert_eq!(request.title, "Allow echo?");
                break (id, occurrence);
            }
            assert_ne!(observations.current()?.foreground, crate::runtime::NativeForeground::Idle,
                "Turn ended without offering approval");
            observations.wait_change().await;
        };
        let work = observations.current()?.work.ok_or("No active turn for approval")?;
        let answer = InteractiveResponse::Permission {
            id: request_id,
            response: serde_json::from_value(json!({"outcome":{"outcome":"selected","optionId":"approve-once"}}))?,
        };
        assert!(matches!(thread.respond(answer.clone(), Some(&first)), Err(Error::NotAdmitted)));
        thread.respond(answer.clone(), Some(&position))?;
        assert!(matches!(thread.respond(answer, Some(&position)), Err(Error::NotAdmitted)));
        let ending = loop {
            if let Some(ending) = observations.normal_ending()? {
                break ending;
            }
            observations.wait_change().await;
        };
        assert_eq!(ending.work, work);
        assert_eq!(ending.work.snapshot, snapshot);
        let mut pass = captured.begin(&snapshot, None)?;
        let original = pass.next_event()?.ok_or("No historical capture")?;
        assert_eq!(original.raw, prefix);
        assert_eq!(original.position, first);
        let mut contains_approval = false;
        while let Some(event) = pass.next_event()? {
            if event.position == position {
                let frame: Value = serde_json::from_str(&event.raw)?;
                assert_eq!(frame["method"], "mcpServer/elicitation/request");
                assert_eq!(frame["params"]["turnId"], work.work_id);
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
