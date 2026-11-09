use super::*;
use chapchoo_crypto::ExternalSecret;
use std::sync::atomic::{AtomicUsize, Ordering};
struct CountingProvider(AtomicUsize);
struct CheckProvider(AtomicUsize);
impl NativeAuthenticationProvider for CheckProvider {
    fn acquire(&self) -> crate::NativeAuthenticationFuture<'_, crate::NativeAuthentication> {
        Box::pin(async { Err(Error::Authentication) })
    }
    fn check<'a>(
        &'a self,
        _: &'a NativeAuthenticationBinding,
    ) -> crate::NativeAuthenticationFuture<'a, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}
fn update(
    binding: NativeAuthenticationBinding,
    key: &str,
) -> Result<crate::NativeAuthentication, TestError> {
    Ok(crate::NativeAuthentication {
        binding,
        credentials: NativeCredentials::api_key(ExternalSecret::new(key.as_bytes().to_vec())?)?,
    })
}
impl NativeAuthenticationProvider for CountingProvider {
    fn acquire(&self) -> crate::NativeAuthenticationFuture<'_, crate::NativeAuthentication> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Err(Error::Authentication) })
    }
    fn check<'a>(
        &'a self,
        _: &'a NativeAuthenticationBinding,
    ) -> crate::NativeAuthenticationFuture<'a, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Err(Error::Authentication) })
    }
}
fn state(storage: NativeStorage) -> Result<State, Error> {
    Ok(State {
        preparer: Preparer::new(storage.clone())?,
        storage,
        live: None,
        scope: None,
        blocked: false,
        inputs: InputRegistry::default(),
        retained_capture: None,
        setups: SetupRegistry::default(),
        stops: StopRegistry::default(),
        answers: AnswerRegistry::default(),
        credential_write: None,
    })
}
#[tokio::test]
async fn already_live_setup_preserves_applied_credentials_without_calling_new_provider()
-> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let source = crate::CaptureLog::create(&storage)?;
    let id = NativeConversationId::parse("live-native".into())?;
    source.bind_native(&id)?;
    std::fs::write(storage.state().join("auth.json"), b"existing-live-auth")?;
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
    let protocol = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        done.clone(),
        storage.workspace(),
        NativeContext::New,
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let session = request(&mut read).await?;
    reply(&mut write, &session, json!({"sessionId":"live-native"})).await?;
    let session = rx.await??;
    let child = tokio::process::Command::new("/bin/cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let launch = tempfile::tempdir()?;
    let mut state = state(storage.clone())?;
    let provider = Arc::new(CountingProvider(AtomicUsize::new(0)));
    let binding = NativeAuthenticationBinding {
        agent_id: "agent".into(),
        conversation_id: "conversation".into(),
        operation_id: "old-operation".into(),
        execution_id: "execution".into(),
        authentication_id: "old-record".into(),
        source: [11; 24],
    };
    let applied_choice = HarnessSupply::Existing(chapchoo_domain::ExistingCodexPath::parse(
        "/owner/codex".into(),
    )?);
    state.live = Some(Live {
        child,
        protocol,
        stderr: tokio::spawn(async { Ok(()) }),
        done: done.clone(),
        ended: CancellationToken::new(),
        session: Some(session),
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
        requested_supply: applied_choice.clone(),
        _launch: launch,
        capture: source.clone(),
        observations: Arc::default(),
        _authentication: binding.clone(),
    });
    let result = state
        .start(
            HarnessSupply::Managed,
            NativeContext::Resume {
                session: id.clone(),
                replay: CaptureReplay::AlreadyCaptured,
            },
            source,
            provider.clone(),
        )
        .await;
    let physical = state.observe()?;
    assert_eq!(physical.authentication_id.as_deref(), Some("old-record"));
    let prepared = physical.preparation.ok_or("Missing physical preparation")?;
    assert_eq!(prepared.freshness, crate::Freshness::Current);
    assert_eq!(prepared.versions.node, "v24.0.0");
    assert_eq!(prepared.versions.adapter, "2.6.0");
    assert_eq!(prepared.versions.codex, "0.165.0");
    let observed = state
        .live
        .as_ref()
        .ok_or("Missing live owner")?
        ._authentication
        .clone();
    assert_eq!(state.observe()?.applied_supply, Some(applied_choice));
    let preserved = std::fs::read(storage.state().join("auth.json"))?;
    assert_eq!(provider.0.load(Ordering::SeqCst), 0);
    let checked = Arc::new(CheckProvider(AtomicUsize::new(0)));
    let changed = NativeAuthenticationBinding {
        source: [12; 24],
        ..binding.clone()
    };
    let publication = state
        .publish_authentication(
            binding.clone(),
            update(changed.clone(), "replacement-key")?,
            checked.clone(),
        )
        .await;
    let stale = state
        .publish_authentication(
            binding.clone(),
            update(
                NativeAuthenticationBinding {
                    source: [13; 24],
                    ..binding.clone()
                },
                "stale-key",
            )?,
            checked.clone(),
        )
        .await;
    let wrong = state
        .publish_authentication(
            changed.clone(),
            update(
                NativeAuthenticationBinding {
                    authentication_id: "saved-future-record".into(),
                    source: [13; 24],
                    ..changed.clone()
                },
                "wrong-key",
            )?,
            checked.clone(),
        )
        .await;
    let revoked = state
        .publish_authentication(
            changed.clone(),
            update(
                NativeAuthenticationBinding {
                    source: [13; 24],
                    ..changed.clone()
                },
                "revoked-key",
            )?,
            provider.clone(),
        )
        .await;
    let current = state
        .current_authentication()?
        .ok_or("Missing applied binding")?;
    let file = std::fs::read(storage.state().join("auth.json"))?;
    // No login, reload or new session request may accompany file publication.
    {
        let mut line = String::new();
        let mut next = Box::pin(read.read_line(&mut line));
        std::future::poll_fn(|cx| {
            assert!(next.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    let mut live = state.live.take().ok_or("Missing live owner")?;
    done.cancel();
    live.child.kill().await?;
    live.protocol.await??;
    live.stderr.await??;
    assert_eq!(result?, id);
    assert_eq!(observed, binding);
    assert_eq!(preserved, b"existing-live-auth");
    publication?;
    assert!(matches!(stale, Err(Error::Authentication)));
    assert!(matches!(wrong, Err(Error::Authentication)));
    assert!(matches!(revoked, Err(Error::Authentication)));
    assert_eq!(checked.0.load(Ordering::SeqCst), 1);
    assert_eq!(provider.0.load(Ordering::SeqCst), 1);
    assert_eq!(current, changed);
    assert_eq!(
        file,
        br#"{"auth_mode":"apiKey","OPENAI_API_KEY":"replacement-key"}"#
    );
    Ok(())
}
#[tokio::test]
async fn cancelled_file_waiter_retains_physical_publication_until_owner_finish()
-> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut state = state(storage.clone())?;
    let worker_storage = storage.clone();
    let (started, start) = oneshot::channel();
    let (release, released) = std::sync::mpsc::channel();
    state.credential_write = Some(tokio::task::spawn_blocking(move || {
        let _ = started.send(());
        released.recv().map_err(|_| Error::Storage)?;
        let key = NativeCredentials::api_key(
            ExternalSecret::new(b"synthetic-key".to_vec()).map_err(|_| Error::Authentication)?,
        )?;
        worker_storage.publish_credentials(&key)?;
        Ok(key)
    }));
    start.await?;
    {
        let work = state
            .credential_write
            .as_mut()
            .ok_or("Missing owned work")?;
        let mut waiter = Box::pin(work);
        std::future::poll_fn(|cx| {
            assert!(waiter.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    // A cancelled partial retirement must not take/drop the physical owner.
    {
        let mut partial = Box::pin(state.retire());
        std::future::poll_fn(|cx| {
            assert!(partial.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    assert!(state.credential_write.is_some());
    let result = async {
        let mut finish = Box::pin(state.retire());
        std::future::poll_fn(|cx| {
            assert!(finish.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        release.send(())?;
        finish.await.map_err(Into::into)
    }
    .await;
    let retirement: Retirement = result.map_err(|e: TestError| e)?;
    assert_eq!(retirement, Retirement::NoNativeStarted);
    assert!(storage.state().join("auth.json").is_file());
    assert!(state.credential_write.is_none());
    Ok(())
}

#[tokio::test]
async fn explicit_api_key_login_precedes_session_and_rejection_never_falls_back()
-> Result<(), TestError> {
    for accepted in [true, false] {
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
        let credentials =
            NativeCredentials::api_key(ExternalSecret::new(b"explicit-synthetic-key".to_vec())?)?;
        let connection = tokio::spawn(connect_authenticated(
            Lines::new(outgoing, incoming),
            Arc::default(),
            Arc::default(),
            ready,
            done.clone(),
            "/workspace".into(),
            (NativeContext::New, Some(credentials)),
        ));
        let (read, mut write) = tokio::io::split(peer);
        let mut read = BufReader::new(read);
        let initialize = request(&mut read).await?;
        reply(&mut write, &initialize, initialization(2)).await?;
        let login = request(&mut read).await?;
        assert_eq!(login.method, "auth/login");
        assert_eq!(
            login.params,
            json!({"methodId":"api-key","_meta":{"api-key":{"apiKey":"explicit-synthetic-key"}}})
        );
        if accepted {
            reply(&mut write, &login, json!({})).await?;
            let session = request(&mut read).await?;
            assert_eq!(session.method, "session/new");
            reply(&mut write, &session, json!({"sessionId":"synthetic"})).await?;
            assert_eq!(rx.await??.session_id().to_string(), "synthetic");
            done.cancel();
            connection.await??;
        } else {
            write.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":login.id,"error":{"code":-32000,"message":"synthetic-secret-rejection"}})).as_bytes()).await?;
            assert!(matches!(
                rx.await?,
                Err(Error::ProtocolRejected {
                    stage: crate::ProtocolStage::Authentication,
                    code: -32000
                })
            ));
            assert!(connection.await?.is_err());
            let mut extra = String::new();
            assert_eq!(read.read_line(&mut extra).await?, 0);
        }
    }
    Ok(())
}

#[tokio::test]
async fn external_file_startup_installs_no_native_token_login_callback() -> Result<(), TestError> {
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
    let credentials=NativeCredentials::external_token_file(ExternalSecret::new(
        br#"{"auth_mode":"chatgptAuthTokens","OPENAI_API_KEY":null,"tokens":{"id_token":"access.jwt","access_token":"access.jwt","refresh_token":"","account_id":"account"}}"#.to_vec()
    )?,"account")?;
    let connection = tokio::spawn(connect_authenticated(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        done.clone(),
        "/workspace".into(),
        (
            NativeContext::Resume {
                session: NativeConversationId::parse("retained".into())?,
                replay: CaptureReplay::AlreadyCaptured,
            },
            Some(credentials),
        ),
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let resume = request(&mut read).await?;
    assert_eq!(resume.method, "session/resume");
    reply(&mut write, &resume, json!({})).await?;
    assert_eq!(rx.await??.session_id().to_string(), "retained");
    done.cancel();
    connection.await??;
    Ok(())
}
