//! Opt-in qualification against a disposable, loopback-only Codex fixture.
//! The fixture prepares the previous thread and closes its native process.
use super::*;
use crate::runtime::observations::Observations;
use crate::{
    CaptureLog, NativeStorage,
    runtime::{CaptureReplay, InteractiveRequest, InteractiveResponse, NativeContext},
};
use std::{path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};

fn fixture_process(
    binary: &Path,
    home: &Path,
    state: &Path,
    cwd: &Path,
    origin: &str,
) -> TestResult<Child> {
    Ok(Command::new(binary)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").ok_or("Missing PATH")?)
        .env("HOME", home)
        .env("CODEX_HOME", state)
        .env("OPENAI_BASE_URL", format!("{origin}/v1"))
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "127.0.0.1")
        .env("OTEL_SDK_DISABLED", "true")
        .current_dir(cwd)
        .args(["app-server", "--listen", "stdio://"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?)
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
        let bearer = chapchoo_crypto::BearerSecret::generate()?;
        // The fixture accepts this process's dynamically scoped bearer and no
        // production credential. The URL was checked before handing it over.
        let provision = crate::NativeMcpDeclaration::http(&format!("{origin}/mcp"), bearer)?;
        let mut child = fixture_process(&binary, temp.path(), &home, &cwd, &origin)?;
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
        let manual = chapchoo_domain::ConversationModelTuning {
            permission_mode: Some(chapchoo_domain::ConversationPermissionMode::AskForApproval),
            ..Default::default()
        };
        let inserted = tokio::time::timeout(Duration::from_secs(15), thread.enqueue_prompt(&text, None, &manual)?)
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
        let models = thread.models().ok_or("Missing model catalogue")?;
        let astra = models.options.iter().find(|model| model.id == "gpt-6-astra")
            .ok_or("Missing fixture model")?;
        let supported = astra.capabilities.as_ref().ok_or("Missing model capabilities")?;
        let unsupported = chapchoo_domain::ConversationModelTuning {
            effort: Some("unadvertised".into()), speed: None,
            ..Default::default()
        };
        assert!(matches!(thread.enqueue_prompt(&text, Some(&astra.id), &unsupported), Err(Error::NotAdmitted)));
        let mut preceding = ending.work.work_id.clone();
        for (label, speed) in [
            ("standard", ConversationSpeed::Standard),
            ("fast", ConversationSpeed::Fast),
            ("ultrafast", ConversationSpeed::Ultrafast),
        ] {
            if !supported.speeds.contains(&speed) {
                continue;
            }
            let tuning = chapchoo_domain::ConversationModelTuning {
                effort: Some("low".into()), speed: Some(speed),
                ..Default::default()
            };
            let text = ConversationText::parse(format!("Fixture speed {label}"))?;
            thread.enqueue_prompt(&text, Some(&astra.id), &tuning)?.await?;
            preceding = loop {
                if let Some(ending) = observations.normal_ending()?
                    && ending.work.work_id != preceding
                {
                    break ending.work.work_id;
                }
                observations.wait_change().await;
            };
        }
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
        assert!(child.wait().await?.success());
        let fresh = tempfile::tempdir()?;
        let fresh_storage = NativeStorage::open(fresh.path())?;
        let fresh_capture = CaptureLog::create(&fresh_storage)?;
        let fresh_cwd = fresh_storage.workspace();
        let fresh_state = fresh_storage.state();
        std::fs::copy(home.join("config.toml"), fresh_state.join("config.toml"))?;
        assert!(fresh_capture.is_empty()?);
        let mut child = fixture_process(&binary, fresh.path(), &fresh_state, &fresh_cwd, &origin)?;
        let observed = Arc::new(Observations::default());
        let done = CancellationToken::new();
        let (connection, driver) = Connection::connect(
            child.stdout.take().ok_or("Missing stdout")?,
            child.stdin.take().ok_or("Missing stdin")?,
            done.clone(),
            fresh_capture.clone(),
            observed.clone(),
        );
        let driver = tokio::spawn(driver);
        // Public API-key login on a private fresh state directory. The value is
        // inert and this child reaches only the fixture server.
        let thread = connection.open(&fresh_cwd, &NativeContext::New, &provision, Some("fixture-fresh-api-key"), true).await?;
        assert_ne!(thread.id(), id.as_str());
        assert_ne!(fresh_capture.snapshot()?, snapshot);
        observed.enable_normal_endings()?;
        let fresh_text = ConversationText::parse("First input after the fixture cutover.".into())?;
        thread.enqueue_prompt(&fresh_text, None, &Default::default())?.await?;
        while observed.normal_ending()?.is_none() {
            observed.wait_change().await;
        }
        done.cancel();
        driver.await??;
        assert!(child.wait().await?.success());
        Ok(())
    }).await?
}
