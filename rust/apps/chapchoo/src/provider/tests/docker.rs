//! Native Control journey beneath the public Docker SDK, with owned fake bytes.
use super::*;
use crate::test_support::{self, Fixture};
use base64::Engine as _;
use chapchoo_domain::{DockerDaemonId, LifecycleAction, ManagedResourceKind, ProviderSubstrate};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Default)]
struct EngineState {
    volume: Option<Value>,
    container: Option<Value>,
    calls: Vec<&'static str>,
    generation: usize,
    copies: Vec<bool>,
    lose_start: bool,
    replace_after_volume: bool,
}
struct Engine {
    state: Arc<Mutex<EngineState>>,
    port: u16,
}
impl Engine {
    async fn new(
        fixture: &mut Fixture,
        app: Arc<ComputersService>,
        actor: chapchoo_crypto::SecretDigest,
        team: String,
    ) -> Result<Self, AppError> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let state = Arc::new(Mutex::new(EngineState::default()));
        let shared = state.clone();
        let mut stopping = fixture.shutdown();
        fixture.server(async move {
            let mut tasks = tokio::task::JoinSet::new();
            let primary = async {
            loop {
                tokio::select! {
                    _ = &mut stopping => break,
                    accepted = listener.accept() => {
                        let (stream, _) = accepted?;
                        let state = shared.clone();
                        let app = app.clone();
                        let actor = actor.clone();
                        let team = team.clone();
                        tasks.spawn(async move { exchange(stream, state, app, actor, team).await });
                    }
                }
            }
            Ok::<(), AppError>(())
            }.await;
            let cleanup = test_support::requests(&mut tasks).await;
            test_support::completed(primary, cleanup)
        });
        Ok(Self { state, port })
    }
    async fn adapter(&self) -> Result<chapchoo_docker::Docker, AppError> {
        Ok(chapchoo_docker::Docker::connect(
            chapchoo_docker::connection::Selection {
                endpoint: chapchoo_docker::connection::Endpoint::Tcp {
                    host: "127.0.0.1".into(),
                    port: self.port,
                },
                tls: None,
                api_version: None,
            },
            chapchoo_docker::Config {
                image: "public.invalid/computer:fixture".into(),
                image_policy: chapchoo_docker::ImagePolicy::Never,
                saved_mount: "/workspace".into(),
                network: None,
                nano_cpus: 1_000_000_000,
                memory_bytes: 128 * 1024 * 1024,
            },
        )
        .await?)
    }
}
async fn exchange(
    mut stream: tokio::net::TcpStream,
    state: Arc<Mutex<EngineState>>,
    app: Arc<ComputersService>,
    actor: chapchoo_crypto::SecretDigest,
    team: String,
) -> Result<(), AppError> {
    let mut header = Vec::new();
    let mut byte = [0];
    while !header.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).await?;
        header.push(byte[0]);
        if header.len() > 16384 {
            return Err("Unbounded Engine fixture headers".into());
        }
    }
    let header = std::str::from_utf8(&header)?;
    let mut lines = header.lines();
    let request = lines.next().ok_or("Missing fixture request")?;
    let mut parts = request.split_whitespace();
    let method = parts.next().ok_or("Missing fixture method")?;
    let url = url::Url::parse(&format!(
        "http://fixture{}",
        parts.next().ok_or("Missing fixture path")?
    ))?;
    let length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()?
        .unwrap_or(0);
    if length > 65536 {
        return Err("Unbounded Engine fixture body".into());
    }
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await?;
    let body: Value = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body)?
    };
    let path = url.path();
    // Inspect persisted native evidence at the actual subsequent effect boundary,
    // not after the Provider has finished its whole command.
    if path.ends_with("/containers/create") || path.ends_with("/start") {
        let agent = app
            .agents(&actor, &team, None, 20, AcceptanceTime::System)
            .await?
            .remove(0);
        let snapshot = app
            .delivery_snapshot(
                Some(&actor),
                ComputerRequest {
                    computer_id: &agent.computer.id,
                    request_id: &agent.computer.current_request_id,
                },
                None,
            )
            .await?;
        assert!(
            snapshot
                .resources
                .iter()
                .any(|r| r.kind() == ManagedResourceKind::DockerVolume && r.identity().is_some()),
            "Volume ACK must precede container creation"
        );
        if path.ends_with("/start") {
            assert!(
                snapshot
                    .resources
                    .iter()
                    .any(|r| r.kind() == ManagedResourceKind::DockerContainer
                        && r.execution_id == snapshot.computer.current_execution_id
                        && r.identity().is_some()),
                "Full container ID ACK must precede Start"
            );
        }
    }
    let (code, response) = {
        let mut state = state.lock().map_err(|_| "Engine fixture lock")?;
        if path.ends_with("/version") {
            (200, json!({"ApiVersion":"1.53","MinAPIVersion":"1.24"}))
        } else if path.ends_with("/info") {
            (
                200,
                json!({"ID":"daemon:control-fixture","OSType":"linux","Architecture":"amd64","MemoryLimit":true,"CpuCfsPeriod":true,"CpuCfsQuota":true,"SecurityOptions":["name=seccomp,profile=builtin"]}),
            )
        } else if path.contains("/images/") {
            (
                200,
                json!({"Id":format!("sha256:{}", "a".repeat(64)),"Architecture":"amd64","Os":"linux","Config":{}}),
            )
        } else if path.ends_with("/networks/bridge") {
            (
                200,
                json!({"Name":"bridge","Driver":"bridge","Internal":false}),
            )
        } else if path.ends_with("/volumes/create") {
            assert!(
                state.volume.is_none(),
                "Original volume may only be created once"
            );
            state.calls.push("volume-create");
            let volume = json!({"Name":body["Name"],"Labels":body["Labels"],"Driver":"local","Scope":"local","Options":{},"CreatedAt":"2026-11-03T01:00:00Z","Mountpoint":"/private/fixture"});
            state.volume = Some(volume.clone());
            (201, volume)
        } else if path.contains("/volumes/") {
            if method == "DELETE" {
                assert!(
                    state.container.is_none(),
                    "Volume deletion must follow container absence"
                );
                assert!(
                    url.query_pairs()
                        .all(|(k, v)| k != "force" || v == "false" || v == "0")
                );
                state.calls.push("volume-remove");
                state.volume = None;
                (204, Value::Null)
            } else {
                match &state.volume {
                    Some(value) => (200, value.clone()),
                    None => (404, json!({"message":"absent"})),
                }
            }
        } else if path.ends_with("/containers/create") {
            assert!(state.container.is_none());
            state.generation += 1;
            let id = format!("{:064x}", state.generation);
            state.calls.push("create");
            let mount = &body["HostConfig"]["Mounts"][0];
            assert_eq!(
                Some(&mount["Source"]),
                state.volume.as_ref().map(|v| &v["Name"])
            );
            state.copies.push(
                mount["VolumeOptions"]["NoCopy"]
                    .as_bool()
                    .ok_or("Missing no-copy policy")?,
            );
            let name = url
                .query_pairs()
                .find(|(key, _)| key == "name")
                .ok_or("Missing container name")?
                .1
                .into_owned();
            state.container = Some(
                json!({"Id":id,"Name":format!("/{name}"),"Config":body,"HostConfig":body["HostConfig"],"Image":format!("sha256:{}","a".repeat(64)),"Mounts":[{"Type":"volume","Name":mount["Source"],"Destination":mount["Target"],"Driver":"local","RW":true}],"State":{"Status":"created","Running":false,"Paused":false,"Restarting":false,"Dead":false,"Pid":0,"StartedAt":"0001-01-01T00:00:00Z","FinishedAt":"0001-01-01T00:00:00Z"}}),
            );
            (201, json!({"Id":id,"Warnings":[]}))
        } else if path.contains("/containers/") {
            let target = path
                .split("/containers/")
                .nth(1)
                .ok_or("Missing native target")?;
            let (id, operation) = target.split_once('/').unwrap_or((target, ""));
            let matches = state.container.as_ref().is_some_and(|c| {
                c["Id"] == id || c["Name"].as_str() == Some(format!("/{id}").as_str())
            });
            if !matches {
                (404, json!({"message":"absent"}))
            } else if operation == "json" {
                (
                    200,
                    state.container.clone().ok_or("Missing fixture container")?,
                )
            } else {
                assert_eq!(id.len(), 64, "Mutations must target the full native ID");
                let action = if method == "DELETE" {
                    "remove"
                } else {
                    operation
                };
                match action {
                    "start" => {
                        state.calls.push("start");
                        let container = state
                            .container
                            .as_mut()
                            .ok_or("Missing fixture container")?;
                        container["State"]["Status"] = json!("running");
                        container["State"]["Running"] = json!(true);
                        container["State"]["Pid"] = json!(42);
                        container["State"]["StartedAt"] = json!("2026-11-03T01:01:00Z");
                        if state.lose_start {
                            (503, json!({"message":"lost response after acceptance"}))
                        } else {
                            (204, Value::Null)
                        }
                    }
                    "update" => {
                        state.calls.push("update");
                        assert_eq!(body["RestartPolicy"]["Name"], "no");
                        assert_eq!(
                            state
                                .container
                                .as_ref()
                                .ok_or("Missing fixture container")?["State"]["Status"],
                            "exited"
                        );
                        state
                            .container
                            .as_mut()
                            .ok_or("Missing fixture container")?["HostConfig"]["RestartPolicy"] =
                            body["RestartPolicy"].clone();
                        (200, json!({"Warnings":[]}))
                    }
                    "stop" => {
                        state.calls.push("stop");
                        let container = state
                            .container
                            .as_mut()
                            .ok_or("Missing fixture container")?;
                        assert_eq!(container["HostConfig"]["RestartPolicy"]["Name"], "always");
                        container["State"]["Status"] = json!("exited");
                        container["State"]["Running"] = json!(false);
                        container["State"]["Pid"] = json!(0);
                        container["State"]["FinishedAt"] = json!("2026-11-03T01:02:00Z");
                        (204, Value::Null)
                    }
                    "remove" => {
                        assert!(url.query_pairs().all(|(k, v)| !matches!(
                            k.as_ref(),
                            "force" | "v"
                        ) || v == "false"
                            || v == "0"));
                        assert_eq!(
                            state
                                .container
                                .as_ref()
                                .ok_or("Missing fixture container")?["State"]["Running"],
                            false
                        );
                        state.calls.push("remove");
                        state.container = None;
                        (204, Value::Null)
                    }
                    _ => return Err("Unexpected native mutation".into()),
                }
            }
        } else {
            return Err("Unexpected Engine fixture route".into());
        }
    };
    if path.ends_with("/volumes/create")
        && state
            .lock()
            .map_err(|_| "Fixture lock")?
            .replace_after_volume
    {
        // Native allocation has completed, but supersede its source request
        // before the response/report ACK. No later Create or Start is allowed.
        let agent = app
            .agents(&actor, &team, None, 20, AcceptanceTime::System)
            .await?
            .remove(0);
        app.request_lifecycle(
            &actor,
            &agent.id,
            &chapchoo_service::LifecycleRequest {
                request_id: "replace-before-ack".into(),
                expected_request_id: "create".into(),
                action: LifecycleAction::Delete,
            },
            AcceptanceTime::System,
        )
        .await?;
    }
    let bytes = if code == 204 {
        Vec::new()
    } else {
        serde_json::to_vec(&response)?
    };
    stream.write_all(format!("HTTP/1.1 {code} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len()).as_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn native_control_journey_acks_allocations_and_preserves_original_volume_without_replay()
-> Result<(), AppError> {
    for (lose_start, replace_after_volume) in [(false, false), (true, false), (false, true)] {
        let directory = tempfile::tempdir()?;
        test_support::scope(async |fixture| {
            let path = directory.path().join("native-provider.sqlite");
            let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
                URL_SAFE.encode([41; 32]),
            )?)?);
            prepare_sqlite(&path, &cipher)?;
            let setup = BearerSecret::generate()?;
            let owner = cipher.issue_provider_bootstrap("provider")?;
            let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
            let auth_config = chapchoo_rpc::ProviderBootstrapConfig {
                secret_key_provider_ids: vec!["provider".into()],
                ..Default::default()
            };
            let authenticator = Arc::new(chapchoo_rpc::ProviderAuthenticator::new(
                auth_config,
                cipher.clone(),
                None,
            )?);
            let store = Store::connect_sqlite(path, cipher).await?;
            let app = Arc::new(ComputersService::new(store.clone()));
            let accounts = AccountsService::new(
                store.clone(),
                PasswordWork::new(1, PasswordCost::new(10)?)?,
                None,
                AccountsConfig {
                    general_web_url: "http://localhost:3000".into(),
                    setup_proof: Some(setup.digest()),
                    session_policy: SessionPolicy::default(),
                },
            )?;
            let user = accounts
                .initialize(
                    &setup,
                    chapchoo_domain::EmailAddress::parse("native@example.invalid")?,
                    Password::parse("Aa1!fixture-native".into())?,
                    AcceptanceTime::System,
                    None,
                )
                .await?;
            let authentication_id = test_support::authentication(
                &store,
                &user.token.digest(),
                AcceptanceTime::System.now()?,
            )
            .await?;
            let team = accounts
                .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
                .await?
                .remove(0);
            let engine =
                Engine::new(fixture, app.clone(), user.token.digest(), team.id.clone()).await?;
            engine.state.lock().map_err(|_| "Fixture lock")?.lose_start = lose_start;
            engine
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .replace_after_volume = replace_after_volume;
            let docker = Arc::new(engine.adapter().await?);
            let cleanup_docker = docker.clone();
            fixture.cleanup(async move { cleanup_docker.drain().await.map_err(Into::into) });
            let adapter = Arc::new(Adapter::Docker(docker));
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let endpoint = format!("http://{}", listener.local_addr()?);
            let peer_listener = TcpListener::bind("127.0.0.1:0").await?;
            let control = app
                .register_control(&format!("http://{}", peer_listener.local_addr()?))
                .await?;
            let connections = ComponentConnectionServices::new(
                app.clone(),
                control.id.clone(),
                Arc::new(std::sync::atomic::AtomicBool::new(true)),
            );
            let authentication = ComponentAuthenticationServices::new(
                app.clone(),
                control.id.clone(),
                connections.clone(),
            )
            .with_provider_authenticator(authenticator);
            let cleanup_app = app.clone();
            let cleanup_control = control.id.clone();
            let cleanup_connections = connections.clone();
            fixture.cleanup(async move {
                let primary = cleanup_app
                    .drain_control(&cleanup_control)
                    .await
                    .map_err(Into::into);
                let cleanup = cleanup_connections.drain().await.map_err(Into::into);
                test_support::completed(primary, cleanup)
            });
            let stopped = fixture.shutdown();
            fixture.server(
                Server::builder()
                    .add_service(authentication.provider_server())
                    .add_service(connections.provider_server())
                    .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                        let _ = stopped.await;
                    }),
            );
            let peer = chapchoo_rpc::PeerServices::new(app.clone(), connections.clone());
            let stopped_peer = fixture.shutdown();
            fixture.server(
                Server::builder()
                    .add_service(peer.server())
                    .serve_with_incoming_shutdown(TcpListenerStream::new(peer_listener), async {
                        let _ = stopped_peer.await;
                    }),
            );
            let key = app
                .exchange_bootstrap_provider(
                    &proof,
                    "Fixture",
                    chapchoo_service::ProviderCapabilities {
                        create: true,
                        start: true,
                        stop: true,
                        restart: true,
                    },
                    &ProviderSubstrate::Docker(DockerDaemonId::parse(
                        "daemon:control-fixture".into(),
                    )?),
                    &key_request_id()?,
                    AcceptanceTime::System,
                )
                .await?;
            let assigned_provider = key.provider_id;
            let key = key.key;
            let component_key = wire::ComponentKey {
                token: key.token.expose().into(),
                issued_at: None,
                expires_at: Some(prost_types::Timestamp {
                    seconds: key.expires_at / 1000,
                    nanos: 0,
                }),
                proactive_after: Some(prost_types::Timestamp {
                    seconds: key.proactive_after / 1000,
                    nanos: 0,
                }),
            };
            let component_key = wire::ProviderCredentials {
                provider_id: assigned_provider.clone(),
                key: Some(component_key),
            };
            let stop = CancellationToken::new();
            let stopping = stop.clone();
            let channel = super::super::endpoint(&endpoint)?.connect_lazy();
            let reconnect_owner =
                crate::provider_auth::ProviderAuth::fixture(owner.expose().into());
            let reconnect_key = component_key.clone();
            let reconnect_adapter = adapter.clone();
            let reconnect_endpoint = endpoint.clone();
            let provider = fixture.worker(stop.clone(), async move {
                stream_loop(
                    channel,
                    &endpoint,
                    crate::provider_auth::ProviderAuth::fixture(owner.expose().into()),
                    crate::provider_auth::configuration(
                        "provider",
                        "Fixture",
                        wire::ProviderKind::Docker,
                        Some("daemon:control-fixture".into()),
                    ),
                    component_key,
                    adapter,
                    stopping,
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if app
                        .resolve_provider_route(&assigned_provider)
                        .await?
                        .is_some()
                    {
                        return Ok::<_, AppError>(());
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await??;
            let images = app
                .agent_profile_images(&user.token.digest(), None, 1, AcceptanceTime::System)
                .await?;
            let CreationOutcome::Present(agent) = app
                .create_cloud_agent(
                    &user.token.digest(),
                    &chapchoo_service::CreateCloudAgent {
                        team_id: team.id.clone(),
                        request_id: "create".into(),
                        authentication_id: authentication_id.clone(),
                        provider_id: assigned_provider.clone(),
                        name: "Native".into(),
                        profile: chapchoo_service::CreateAgentProfile {
                            handle: "native-agent".into(),
                            image_id: images[0].id.clone(),
                            palette_key: "sage".into(),
                        },
                    },
                    AcceptanceTime::System,
                )
                .await?
            else {
                return Err("Missing native Agent".into());
            };
            let delivery = chapchoo_rpc::PeerDelivery::new(app.clone());
            connections.reconcile_once(&delivery).await?;
            let observed = app
                .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
                .await?;
            if !replace_after_volume {
                assert_eq!(
                    observed.computer.power,
                    if lose_start {
                        chapchoo_domain::ObservedPower::Unknown
                    } else {
                        chapchoo_domain::ObservedPower::Running
                    }
                );
            }
            if replace_after_volume {
                connections.reconcile_once(&delivery).await?;
                connections.reconcile_once(&delivery).await?;
                {
                    let state = engine.state.lock().map_err(|_| "Fixture lock")?;
                    assert_eq!(state.calls, ["volume-create"]);
                    assert!(state.volume.is_some() && state.container.is_none());
                }
                stop.cancel();
                tokio::time::timeout(Duration::from_secs(10), provider).await??;
            } else if lose_start {
                assert_eq!(
                    observed.computer.progress.phase,
                    chapchoo_domain::LifecyclePhase::Uncertain
                );
                // New source and repeated convergence only observe; a running native
                // process cannot settle an unknown same-process Start submission.
                stop.cancel();
                tokio::time::timeout(Duration::from_secs(10), provider).await??;
                let prior_route = app.resolve_provider_route(&assigned_provider).await?;
                let reconnect_stop = CancellationToken::new();
                let stopping = reconnect_stop.clone();
                let channel = super::super::endpoint(&reconnect_endpoint)?.connect_lazy();
                let reconnected = fixture.worker(reconnect_stop.clone(), async move {
                    stream_loop(
                        channel,
                        &reconnect_endpoint,
                        reconnect_owner,
                        crate::provider_auth::configuration(
                            "provider",
                            "Fixture",
                            wire::ProviderKind::Docker,
                            Some("daemon:control-fixture".into()),
                        ),
                        reconnect_key,
                        reconnect_adapter,
                        stopping,
                    )
                    .await
                });
                tokio::time::timeout(Duration::from_secs(10), async {
                    loop {
                        if let Some(route) = app.resolve_provider_route(&assigned_provider).await?
                            && prior_route.as_ref().is_none_or(|prior| {
                                prior.component.connection_id != route.component.connection_id
                            })
                        {
                            return Ok::<_, AppError>(());
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                })
                .await??;
                connections.reconcile_once(&delivery).await?;
                connections.reconcile_once(&delivery).await?;
                assert_eq!(
                    app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
                        .await?
                        .computer
                        .progress
                        .phase,
                    chapchoo_domain::LifecyclePhase::Uncertain
                );
                app.request_lifecycle(
                    &user.token.digest(),
                    &agent.id,
                    &chapchoo_service::LifecycleRequest {
                        request_id: "delete-uncertain".into(),
                        expected_request_id: "create".into(),
                        action: LifecycleAction::Delete,
                    },
                    AcceptanceTime::System,
                )
                .await?;
                connections.reconcile_once(&delivery).await?;
                assert_eq!(
                    engine.state.lock().map_err(|_| "Fixture lock")?.calls,
                    ["volume-create", "create", "start"]
                );
                reconnect_stop.cancel();
                tokio::time::timeout(Duration::from_secs(10), reconnected).await??;
            } else {
                for (id, previous, action, running) in [
                    ("stop", "create", LifecycleAction::Stop, false),
                    ("start", "stop", LifecycleAction::Start, true),
                    ("restart", "start", LifecycleAction::Restart, true),
                ] {
                    app.request_lifecycle(
                        &user.token.digest(),
                        &agent.id,
                        &chapchoo_service::LifecycleRequest {
                            request_id: id.into(),
                            expected_request_id: previous.into(),
                            action,
                        },
                        AcceptanceTime::System,
                    )
                    .await?;
                    connections.reconcile_once(&delivery).await?;
                    connections.reconcile_once(&delivery).await?;
                    assert_eq!(
                        app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
                            .await?
                            .computer
                            .power,
                        if running {
                            chapchoo_domain::ObservedPower::Running
                        } else {
                            chapchoo_domain::ObservedPower::Stopped
                        }
                    );
                    assert!(
                        engine
                            .state
                            .lock()
                            .map_err(|_| "Fixture lock")?
                            .volume
                            .is_some()
                    );
                }
                app.request_lifecycle(
                    &user.token.digest(),
                    &agent.id,
                    &chapchoo_service::LifecycleRequest {
                        request_id: "delete".into(),
                        expected_request_id: "restart".into(),
                        action: LifecycleAction::Delete,
                    },
                    AcceptanceTime::System,
                )
                .await?;
                connections.reconcile_once(&delivery).await?;
                connections.reconcile_once(&delivery).await?;
                assert!(
                    app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
                        .await
                        .is_err_and(|e| e.reason() == chapchoo_service::DomainReason::NotFound)
                );
                {
                    let state = engine.state.lock().map_err(|_| "Fixture lock")?;
                    assert_eq!(state.copies, [false, true, true]);
                    assert_eq!(
                        state.calls,
                        [
                            "volume-create",
                            "create",
                            "start",
                            "stop",
                            "update",
                            "remove",
                            "create",
                            "start",
                            "stop",
                            "update",
                            "remove",
                            "create",
                            "start",
                            "stop",
                            "update",
                            "remove",
                            "volume-remove"
                        ]
                    );
                    assert!(state.volume.is_none() && state.container.is_none());
                }
                stop.cancel();
                tokio::time::timeout(Duration::from_secs(10), provider).await??;
            }
            Ok(())
        })
        .await?;
    }
    Ok(())
}
