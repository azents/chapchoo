//! Real Control stream and supported SDK with an isolated synthetic HTTP service.
mod docker;
use super::*;
use crate::test_support;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey};
use chapchoo_domain::Password;
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_rpc::{ComponentAuthenticationServices, ComponentConnectionServices};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, ComputerRequest, ComputersService,
    CreationOutcome, PasswordWork, SessionPolicy,
};
use http::{Method, Request as HttpRequest, Response as HttpResponse};
use http_body_util::BodyExt;
use kube::{Client, client::Body};
use serde_json::{Value, json};
use std::{collections::BTreeMap, convert::Infallible, sync::Mutex};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

#[tokio::test]
async fn propagated_loop_error_waits_for_its_started_effect_before_returning()
-> Result<(), AppError> {
    let (release, released) = tokio::sync::oneshot::channel();
    let (completed, completion) = tokio::sync::oneshot::channel();
    let mut owned = Box::pin(own_effects(async |effects| {
        let _waiter = effects.start(async move {
            let _ = released.await;
            let _ = completed.send(7);
        })?;
        Err::<(), AppError>(std::io::Error::other("Private loop failure").into())?;
        Ok(())
    }));
    std::future::poll_fn(|context| {
        assert!(
            owned.as_mut().poll(context).is_pending(),
            "The loop failure must not drop an unfinished SDK task"
        );
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).map_err(|_| "Effect was cancelled")?;
    let failure = owned.await.err().ok_or("Missing primary failure")?;
    assert_eq!(failure.to_string(), "Private loop failure");
    assert_eq!(completion.await?, 7);
    Ok(())
}

#[tokio::test]
async fn provider_stream_creates_pod_only_after_acknowledged_original_resource_uids()
-> Result<(), AppError> {
    let dir = tempfile::tempdir()?;
    test_support::scope(async |fixture| {
    let path = dir.path().join("provider.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([39; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let setup = BearerSecret::generate()?;
    let owner = cipher.issue_provider_bootstrap("provider")?;
    let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
    let auth_config=chapchoo_rpc::ProviderBootstrapConfig { secret_key_provider_ids:vec!["provider".into()], ..Default::default() };
    let authenticator=Arc::new(chapchoo_rpc::ProviderAuthenticator::new(auth_config,cipher.clone(),None)?);
    let store = Store::connect_sqlite(path, cipher).await?;
    let app = Arc::new(ComputersService::new(store.clone()));
    let accounts = AccountsService::new(
        store,
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
            chapchoo_domain::EmailAddress::parse("provider@example.invalid")?,
            Password::parse("Aa1!fixture-provider".into())?,
            AcceptanceTime::System,
            None,
        )
        .await?;
    let team = accounts
        .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
        .await?
        .remove(0);
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
    let authentication = ComponentAuthenticationServices::new(app.clone(), control.id.clone(), connections.clone()).with_provider_authenticator(authenticator);
    let cleanup_app = app.clone();
    let cleanup_control = control.id.clone();
    let cleanup_connections = connections.clone();
    fixture.cleanup(async move {
        let primary = cleanup_app.drain_control(&cleanup_control).await.map_err(Into::into);
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
    let request_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let requests = request_count.clone();
    let objects = Arc::new(Mutex::new(BTreeMap::<String, Value>::new()));
    let substrate = objects.clone();
    let created = Arc::new(Mutex::new(Vec::<String>::new()));
    let failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let calls = created.clone();
    let violations = failures.clone();
    let database = app.clone();
    let finalizer = Arc::new(Mutex::new(None::<(String, Option<tokio::time::Instant>)>));
    let delayed_finalizer = finalizer.clone();
    let deletions = Arc::new(Mutex::new(BTreeMap::<String, usize>::new()));
    let deleted = deletions.clone();
    let current_owner = user.token.digest();
    let current_team = team.id.clone();
    let service = tower::service_fn(move |request: HttpRequest<Body>| {
        requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let objects = objects.clone();
        let calls = calls.clone();
        let violations = violations.clone();
        let database = database.clone();
        let finalizer = finalizer.clone();
        let deletions = deletions.clone();
        let actor = current_owner.clone();
        let team_id = current_team.clone();
        async move {
            let method = request.method().clone();
            let path = request.uri().path().to_owned();
            let watch = request
                .uri()
                .query()
                .is_some_and(|q| q.contains("watch=true"));
            let bytes = request
                .into_body()
                .collect()
                .await
                .map(|b| b.to_bytes())
                .unwrap_or_default();
            let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            let mut code = 200;
            let response = if method == Method::POST {
                let kind = body["kind"].as_str().unwrap_or("").to_owned();
                if kind == "Secret" || kind == "Pod" {
                    let agents = database
                        .agents(&actor, &team_id, None, 20, AcceptanceTime::System)
                        .await;
                    let valid =
                        if let Ok(agents) = agents {
                            if let Some(agent) = agents.iter().find(|a| {
                                body["metadata"]["annotations"]["chapchoo.io/computer"].as_str()
                                    == Some(&a.computer.id)
                            }) {
                                if let Ok(snapshot) = database
                                    .delivery_snapshot(
                                        Some(&actor),
                                        ComputerRequest {
                                            computer_id: &agent.computer.id,
                                            request_id: &agent.computer.current_request_id,
                                        },
                                        None,
                                    )
                                    .await
                                {
                                    snapshot.resources.iter().any(|r| {
                                        r.kind() == chapchoo_domain::ManagedResourceKind::Pvc && r.identity().is_some()
                                    }) && (kind != "Pod"
                                        || snapshot.resources.iter().any(|r| {
                                            r.kind() == chapchoo_domain::ManagedResourceKind::BootstrapSecret
                                                && r.identity().is_some()
                                        }))
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                    if !valid && let Ok(mut violations) = violations.lock() {
                        violations.push(kind.clone());
                    }
                }
                if let Ok(mut calls) = calls.lock() {
                    calls.push(kind.clone());
                }
                let mut response = body;
                response["metadata"]["uid"] = json!(format!("uid-{kind}"));
                response["metadata"]["resourceVersion"] = json!("opaque-current");
                if kind == "Pod" {
                    response["status"] = json!({"phase":"Running"});
                }
                let key = format!(
                    "{path}/{}",
                    response["metadata"]["name"].as_str().unwrap_or("")
                );
                if let Ok(mut objects) = objects.lock() {
                    objects.insert(key, response.clone());
                }
                code = 201;
                response
            } else if watch {
                let pod = objects
                    .lock()
                    .ok()
                    .and_then(|objects| objects.values().find(|v| v["kind"] == "Pod").cloned());
                if let Some(mut pod) = pod {
                    pod["status"] = json!({"phase":"Succeeded","containerStatuses":[{"name":"computer-connector","image":"fixture","imageID":"fixture-id","ready":false,"restartCount":0,"state":{"terminated":{"containerID":"containerd://fixture","exitCode":0,"reason":"Completed","finishedAt":"2026-11-02T00:00:00Z"}}}]});
                    json!({"type":"DELETED","object":pod})
                } else {
                    json!({"type":"ERROR","object":{"apiVersion":"v1","kind":"Status","code":410,"reason":"Expired"}})
                }
            } else if method == Method::DELETE {
                if let Ok(mut calls) = deletions.lock() {
                    *calls.entry(path.clone()).or_default() += 1;
                }
                let delayed = if let Ok(mut finalizer) = finalizer.lock()
                    && let Some((target, started)) = finalizer.as_mut()
                    && target == &path
                {
                    started.get_or_insert_with(tokio::time::Instant::now);
                    true
                } else {
                    false
                };
                if !delayed && let Ok(mut objects) = objects.lock() {
                    objects.remove(&path);
                }
                json!({"apiVersion":"v1","kind":"Status","status":"Success"})
            } else {
                let finalized = if let Ok(finalizer) = finalizer.lock()
                    && let Some((target, Some(started))) = finalizer.as_ref()
                {
                    target == &path && started.elapsed() >= Duration::from_secs(11)
                } else {
                    false
                };
                if finalized && let Ok(mut objects) = objects.lock() {
                    objects.remove(&path);
                }
                match objects
                    .lock()
                    .ok()
                    .and_then(|objects| objects.get(&path).cloned())
                {
                    Some(value) => value,
                    None => {
                        code = 404;
                        json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"NotFound","code":404})
                    }
                }
            };
            let mut response = HttpResponse::new(Body::from(response.to_string().into_bytes()));
            *response.status_mut() =
                http::StatusCode::from_u16(code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
            Ok::<_, Infallible>(response)
        }
    });
    let adapter = Kubernetes::new(
        Client::new(service, "fixture"),
        chapchoo_kubernetes::Config {
            namespace: "fixture".into(),
            image: "computer:fixture".into(),
            image_pull_policy: "IfNotPresent".into(),
            storage_class: None,
            storage_request: "1Gi".into(),
            saved_mount: "/saved".into(),
            service_account: "computer".into(),
            cpu_request: "100m".into(),
            memory_request: "128Mi".into(),
            cpu_limit: "1".into(),
            memory_limit: "1Gi".into(),
        },
    )?;
    let stop = CancellationToken::new();
    let provider_stop = stop.clone();
    let key = app.exchange_bootstrap_provider(&proof,"Fixture",chapchoo_service::ProviderCapabilities { create:true,start:true,stop:true,restart:true }, &chapchoo_domain::ProviderSubstrate::Kubernetes,&key_request_id()?,AcceptanceTime::System).await?;
    let assigned_provider = key.provider_id;
    let key = key.key;
    let provider_credential=key.token.digest();
    let component_key = wire::ComponentKey {
        token: key.token.expose().into(),
        issued_at: None,
        expires_at: Some(prost_types::Timestamp { seconds:key.expires_at/1000,nanos:0 }),
        proactive_after: Some(prost_types::Timestamp {
            seconds: key.proactive_after / 1000,
            nanos: 0,
        }),
    };
    let component_key = wire::ProviderCredentials {
        provider_id: assigned_provider.clone(),
        key: Some(component_key),
    };
    let channel = super::endpoint(&endpoint)?.connect_lazy();
    let provider_endpoint = endpoint.clone();
    let provider = fixture.worker(stop.clone(), async move {
        stream_loop(
            channel,
            &provider_endpoint,
            crate::provider_auth::ProviderAuth::fixture(owner.expose().into()),
            crate::provider_auth::configuration("provider","Fixture",wire::ProviderKind::Kubernetes,None),
            component_key,
            Arc::new(Adapter::Kubernetes(Arc::new(adapter))),
            provider_stop,
        )
        .await
    });
    let route = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(route) = app.resolve_provider_route(&assigned_provider).await? {
                return Ok::<_, AppError>(route);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    let images = app.agent_profile_images(&user.token.digest(), None, 1, AcceptanceTime::System).await?;
    let CreationOutcome::Present(agent) = app
        .create_cloud_agent(
            &user.token.digest(),
            &chapchoo_service::CreateCloudAgent {
                team_id: team.id.clone(),
                request_id: "create".into(),
                provider_id: assigned_provider.clone(),
                name: "Fixture".into(),
                profile: chapchoo_service::CreateAgentProfile {
                    handle: "fixture-agent".into(),
                    image_id: images[0].id.clone(),
                    palette_key: "sage".into(),
                },
            },
            AcceptanceTime::System,
        )
        .await?
    else {
        return Err("Missing Agent".into());
    };
    assert_eq!(route.component.control_id, control.id);
    let delivery = chapchoo_rpc::PeerDelivery::new(app.clone());
    // Deleting a reserved but never-admitted preparation consumes existing
    // definitive non-allocation evidence, without any create/delete submission.
    let CreationOutcome::Present(unstarted) = app
        .create_cloud_agent(
            &user.token.digest(),
            &chapchoo_service::CreateCloudAgent {
                team_id: team.id.clone(),
                request_id: "never-started".into(),
                provider_id: assigned_provider.clone(),
                name: "Unstarted fixture".into(),
                profile: chapchoo_service::CreateAgentProfile {
                    handle: "unstarted-agent".into(),
                    image_id: images[0].id.clone(),
                    palette_key: "sage".into(),
                },
            },
            AcceptanceTime::System,
        )
        .await?
    else {
        return Err("Unstarted Agent missing".into());
    };
    let unstarted_source = chapchoo_service::ProviderSource {
        provider_id: &assigned_provider,
        control_id: &route.component.control_id,
        connection_id: &route.component.connection_id,
        credential: &provider_credential,
    };
    app.reserve_execution(
        &unstarted_source,
        ComputerRequest {
            computer_id: &unstarted.computer.id,
            request_id: "never-started",
        },
        &chapchoo_domain::NativeScope::kubernetes("fixture".into())?,
    )
    .await?;
    app.request_lifecycle(
        &user.token.digest(),
        &unstarted.id,
        &chapchoo_service::LifecycleRequest {
            request_id: "delete-never-started".into(),
            expected_request_id: "never-started".into(),
            action: chapchoo_domain::LifecycleAction::Delete,
        },
        AcceptanceTime::System,
    )
    .await?;
    connections.reconcile_once(&delivery).await?;
    assert!(
        app.agent(&user.token.digest(), &unstarted.id, AcceptanceTime::System)
            .await
            .is_err_and(|error| error.reason() == chapchoo_service::DomainReason::NotFound)
    );
    let observed = app
        .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
        .await?;
    assert_eq!(
        observed.computer.power,
        chapchoo_domain::ObservedPower::Running
    );
    // Repeated convergence is observation, never another creation submission.
    connections.reconcile_once(&delivery).await?;
    assert!(failures.lock().map_err(|_| "Violations lock")?.is_empty());
    assert_eq!(
        *created.lock().map_err(|_| "Calls lock")?,
        vec!["PersistentVolumeClaim", "Secret", "Pod"]
    );
    app.request_lifecycle(
        &user.token.digest(),
        &agent.id,
        &chapchoo_service::LifecycleRequest {
            request_id: "stop".into(),
            expected_request_id: "create".into(),
            action: chapchoo_domain::LifecycleAction::Stop,
        },
        AcceptanceTime::System,
    )
    .await?;
    let connector_listener = TcpListener::bind("127.0.0.1:0").await?;
    let connector_endpoint = format!("http://{}", connector_listener.local_addr()?);
    let connector_control = app.register_control(&connector_endpoint).await?;
    let connector_connections = ComponentConnectionServices::new(
        app.clone(),
        connector_control.id.clone(),
        Arc::new(std::sync::atomic::AtomicBool::new(true)),
    );
    let connector_auth = ComponentAuthenticationServices::new(app.clone(), connector_control.id.clone(), connector_connections.clone());
    let cleanup_app = app.clone();
    let cleanup_control = connector_control.id.clone();
    let cleanup_connections = connector_connections.clone();
    fixture.cleanup(async move {
        let primary = cleanup_app.drain_control(&cleanup_control).await.map_err(Into::into);
        let cleanup = cleanup_connections.drain().await.map_err(Into::into);
        test_support::completed(primary, cleanup)
    });
    let stopped_connector_server = fixture.shutdown();
    fixture.server(
        Server::builder()
            .add_service(connector_auth.connector_server())
            .add_service(connector_connections.connector_server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(connector_listener), async {
                let _ = stopped_connector_server.await;
            }),
    );
    let snapshot = app
        .delivery_snapshot(
            Some(&user.token.digest()),
            ComputerRequest {
                computer_id: &agent.computer.id,
                request_id: "stop",
            },
            None,
        )
        .await?;
    let source = chapchoo_service::ProviderSource {
        provider_id: &assigned_provider,
        control_id: &route.component.control_id,
        connection_id: &route.component.connection_id,
        credential: &provider_credential,
    };
    let session = app
        .issue_connector_session_from_source(
            &source,
            snapshot
                .computer
                .current_execution_id
                .as_deref()
                .ok_or("Current execution")?,
            "recover-session",
            AcceptanceTime::System,
        )
        .await?;
    let connector_stop = CancellationToken::new();
    let connector_stopping = connector_stop.clone();
    let connector = fixture.worker(connector_stop.clone(), async move {
        crate::connector::run(
            &connector_endpoint,
            session.id,
            session.initial_key,
            connector_stopping,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if app
                .resolve_connector_route(&agent.computer.id)
                .await?
                .is_some()
            {
                return Ok::<_, AppError>(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    let before = request_count.load(std::sync::atomic::Ordering::SeqCst);
    // Provider management defers while a different Control owns the Connector.
    connections.reconcile_once(&delivery).await?;
    assert_eq!(
        before,
        request_count.load(std::sync::atomic::Ordering::SeqCst)
    );
    let connector_delivery = chapchoo_rpc::PeerDelivery::new(app.clone());
    connector_connections
        .reconcile_once(&connector_delivery)
        .await?;
    let stopped = app
        .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
        .await?;
    assert_eq!(
        stopped.computer.power,
        chapchoo_domain::ObservedPower::Stopped
    );
    connector_stop.cancel();
    tokio::time::timeout(Duration::from_secs(10), connector).await??;
    let primary = app.drain_control(&connector_control.id).await.map_err(Into::into);
    let cleanup = connector_connections.drain().await.map_err(Into::into);
    test_support::completed(primary, cleanup)?;
    app.request_lifecycle(
        &user.token.digest(),
        &agent.id,
        &chapchoo_service::LifecycleRequest {
            request_id: "start".into(),
            expected_request_id: "stop".into(),
            action: chapchoo_domain::LifecycleAction::Start,
        },
        AcceptanceTime::System,
    )
    .await?;
    connections.reconcile_once(&delivery).await?;
    let restarted = app
        .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
        .await?;
    assert_eq!(
        restarted.computer.power,
        chapchoo_domain::ObservedPower::Running
    );
    app.request_lifecycle(
        &user.token.digest(),
        &agent.id,
        &chapchoo_service::LifecycleRequest {
            request_id: "restart".into(),
            expected_request_id: "start".into(),
            action: chapchoo_domain::LifecycleAction::Restart,
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
        chapchoo_domain::ObservedPower::Running
    );
    assert_eq!(
        created
            .lock()
            .map_err(|_| "Calls lock")?
            .iter()
            .filter(|kind| kind.as_str() == "PersistentVolumeClaim")
            .count(),
        1
    );
    let before_restoration = app
        .delivery_snapshot(
            Some(&user.token.digest()),
            ComputerRequest {
                computer_id: &agent.computer.id,
                request_id: "restart",
            },
            None,
        )
        .await?;
    let previous = before_restoration
        .computer
        .current_execution_id
        .ok_or("Current execution before restoration")?;
    let pod = before_restoration
        .resources
        .iter()
        .find(|r| r.kind() == chapchoo_domain::ManagedResourceKind::Pod && r.execution_id.as_deref() == Some(&previous))
        .ok_or("Current Pod")?;
    let pod_path = format!("/api/v1/namespaces/{}/pods/{}", pod.native().scope().as_str(), pod.native().name());
    let mut disappeared = substrate
        .lock()
        .map_err(|_| "Substrate lock")?
        .remove(&pod_path)
        .ok_or("Synthetic current Pod")?;
    let submissions = created.lock().map_err(|_| "Calls lock")?.len();
    connections.reconcile_once(&delivery).await?;
    connections.reconcile_once(&delivery).await?;
    assert_eq!(
        created.lock().map_err(|_| "Calls lock")?.len(),
        submissions,
        "A bare 404 must not authorize recreation"
    );
    assert_eq!(
        app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
            .await?
            .computer
            .power,
        chapchoo_domain::ObservedPower::Unknown
    );
    disappeared["status"] = json!({"phase":"Succeeded","containerStatuses":[{"name":"computer-connector","image":"fixture","imageID":"fixture-id","ready":false,"restartCount":0,"state":{"terminated":{"containerID":"containerd://fixture","exitCode":0,"reason":"Completed","finishedAt":"2026-11-02T00:00:00Z"}}}]});
    substrate
        .lock()
        .map_err(|_| "Substrate lock")?
        .insert(pod_path, disappeared);
    connections.reconcile_once(&delivery).await?;
    connections.reconcile_once(&delivery).await?;
    let restored = app
        .delivery_snapshot(
            Some(&user.token.digest()),
            ComputerRequest {
                computer_id: &agent.computer.id,
                request_id: "restart",
            },
            None,
        )
        .await?;
    assert!(
        restored
            .computer
            .current_execution_id
            .as_deref()
            .is_some_and(|id| id != previous)
    );
    assert_eq!(
        app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
            .await?
            .computer
            .power,
        chapchoo_domain::ObservedPower::Running
    );
    assert_eq!(
        created
            .lock()
            .map_err(|_| "Calls lock")?
            .iter()
            .filter(|kind| kind.as_str() == "PersistentVolumeClaim")
            .count(),
        1
    );
    // Accumulate more than one cleanup frame of historical execution resources.
    for _ in 0..16 {
        let snapshot = app
            .delivery_snapshot(
                Some(&user.token.digest()),
                ComputerRequest {
                    computer_id: &agent.computer.id,
                    request_id: "restart",
                },
                None,
            )
            .await?;
        let execution = snapshot
            .computer
            .current_execution_id
            .as_deref()
            .ok_or("Current execution")?;
        let pod = snapshot
            .resources
            .iter()
            .find(|r| r.kind() == chapchoo_domain::ManagedResourceKind::Pod && r.execution_id.as_deref() == Some(execution))
            .ok_or("Current Pod")?;
        {
            let mut objects = substrate.lock().map_err(|_| "Substrate lock")?;
            let object = objects
                .get_mut(&format!(
                    "/api/v1/namespaces/{}/pods/{}",
                    pod.native().scope().as_str(), pod.native().name()
                ))
                .ok_or("Synthetic current Pod")?;
            object["status"] = json!({"phase":"Succeeded","containerStatuses":[{"name":"computer-connector","image":"fixture","imageID":"fixture-id","ready":false,"restartCount":0,"state":{"terminated":{"containerID":"containerd://fixture","exitCode":0,"reason":"Completed","finishedAt":"2026-11-02T00:00:00Z"}}}]});
        }
        connections.reconcile_once(&delivery).await?;
        connections.reconcile_once(&delivery).await?;
    }
    let accumulated = app
        .delivery_snapshot(
            Some(&user.token.digest()),
            ComputerRequest {
                computer_id: &agent.computer.id,
                request_id: "restart",
            },
            None,
        )
        .await?;
    assert!(
        accumulated.resources.len() > 32,
        "Cleanup must span bounded frames"
    );
    let delayed_secret = accumulated
        .resources
        .iter()
        .find(|r| r.kind() == chapchoo_domain::ManagedResourceKind::BootstrapSecret)
        .ok_or("Historical Secret missing")?;
    *delayed_finalizer.lock().map_err(|_| "Finalizer lock")? = Some((
        format!(
            "/api/v1/namespaces/{}/secrets/{}",
            delayed_secret.native().scope().as_str(), delayed_secret.native().name()
        ),
        None,
    ));
    app.request_lifecycle(
        &user.token.digest(),
        &agent.id,
        &chapchoo_service::LifecycleRequest {
            request_id: "delete".into(),
            expected_request_id: "restart".into(),
            action: chapchoo_domain::LifecycleAction::Delete,
        },
        AcceptanceTime::System,
    )
    .await?;
    connections.reconcile_once(&delivery).await?;
    let pending = app
        .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
        .await?;
    assert_eq!(
        pending.computer.progress.phase,
        chapchoo_domain::LifecyclePhase::Uncertain
    );
    // The first accepted batch outlives its 10s response waiter. The elapsed
    // finalizer contract is 11s; fresh convergence observes that exact batch and
    // advances only the never-submitted remainder, without another DELETE.
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            connections.reconcile_once(&delivery).await?;
            if app
                .agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
                .await
                .is_err_and(|error| error.reason() == chapchoo_service::DomainReason::NotFound)
            {
                return Ok::<(), AppError>(());
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await??;
    assert!(
        deleted
            .lock()
            .map_err(|_| "Deletion calls lock")?
            .values()
            .all(|count| *count == 1)
    );
    assert_eq!(
        app.agent(&user.token.digest(), &agent.id, AcceptanceTime::System)
            .await
            .err()
            .ok_or("Agent cleanup incomplete")?
            .reason(),
        chapchoo_service::DomainReason::NotFound
    );
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(10), provider).await??;
    Ok(())
    }).await
}
