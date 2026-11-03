//! Isolated HTTP fixtures beneath public Bollard operations; no daemon or DB.
use bollard::models::{
    ContainerConfig, ContainerCreateBody, ContainerCreateResponse, ContainerInspectResponse,
    ContainerState, ContainerStateStatusEnum, ContainerUpdateBody, MountPoint,
    RestartPolicyNameEnum, SystemInfo, Volume, VolumeCreateRequest, VolumeScopeEnum,
};
use chapchoo_docker::{
    Config, Docker, Error, ImagePolicy, Presence,
    connection::{Endpoint, Selection},
};
use chapchoo_protocol::control::{self as wire, managed_resource::Native};
use http_body_util::{BodyExt, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::oneshot,
    task::{JoinHandle, JoinSet},
};

type TestError = Box<dyn std::error::Error + Send + Sync>;
#[path = "native_cases/capacity.rs"]
mod capacity;
#[path = "native_cases/containers.rs"]
mod containers;
#[path = "native_cases/images.rs"]
mod images;
#[derive(Clone, Copy)]
enum Create {
    Normal,
    AcceptedLost,
    Rejected,
}
struct State {
    info: SystemInfo,
    volume: Option<Volume>,
    create: Create,
    creates: usize,
    removes: usize,
    container: Option<ContainerInspectResponse>,
    container_actions: Vec<String>,
    reject_container: bool,
    fail_stage: Option<&'static str>,
    image_present: bool,
    image_error: bool,
    image_architecture: String,
    image_pulls: usize,
    network_internal: bool,
}
struct Fixture {
    port: u16,
    state: Arc<Mutex<State>>,
    shutdown: oneshot::Sender<()>,
    server: JoinHandle<Result<(), TestError>>,
}
fn labels() -> HashMap<String, String> {
    HashMap::from([
        ("chapchoo.io/provider".into(), "provider".into()),
        ("chapchoo.io/computer".into(), "computer".into()),
    ])
}
fn volume() -> Volume {
    Volume {
        name: "saved-original".into(),
        driver: "local".into(),
        scope: Some(VolumeScopeEnum::LOCAL),
        created_at: Some("2026-11-03T06:45:15Z".into()),
        labels: labels(),
        mountpoint: "/private/engine/volume/path".into(),
        ..Default::default()
    }
}
fn context() -> wire::ComputerContext {
    wire::ComputerContext {
        computer_id: "computer".into(),
        request_id: "request".into(),
        execution_id: "execution".into(),
    }
}
fn reserved(allocated: bool) -> wire::ManagedResource {
    wire::ManagedResource {
        kind: wire::ResourceKind::DockerVolume as i32,
        namespace: String::new(),
        name: "saved-original".into(),
        uid: None,
        execution_id: None,
        absence_confirmed: false,
        native: Some(Native::DockerVolume(wire::DockerVolumeNative {
            daemon_id: "daemon:fixture".into(),
            identity: allocated.then(|| wire::DockerVolumeIdentity {
                driver: "local".into(),
                scope: "local".into(),
                options: Default::default(),
                labels: labels(),
                created_at: Some("2026-11-03T06:45:15Z".into()),
            }),
        })),
    }
}
fn response(status: u16, bytes: Vec<u8>) -> Result<Response<Full<Bytes>>, TestError> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(bytes)))?)
}
fn message(status: u16) -> Result<Response<Full<Bytes>>, TestError> {
    response(status, br#"{"message":"fixture native outcome"}"#.to_vec())
}
async fn serve(
    request: Request<Incoming>,
    shared: Arc<Mutex<State>>,
) -> Result<Response<Full<Bytes>>, TestError> {
    let (parts, body) = request.into_parts();
    let body = body.collect().await?.to_bytes();
    let path = parts.uri.path();
    let mut state = shared.lock().map_err(|_| "Fixture lock poisoned")?;
    if path.ends_with("/version") {
        return response(
            200,
            br#"{"ApiVersion":"1.53","MinAPIVersion":"1.24"}"#.to_vec(),
        );
    }
    if path.ends_with("/info") {
        return response(200, serde_json::to_vec(&state.info)?);
    }
    if path.ends_with("/images/create") {
        state.image_pulls += 1;
        state.image_present = true;
        return if state.image_error {
            response(
                200,
                b"{\"errorDetail\":{\"message\":\"fixture pull failed\"}}\n".to_vec(),
            )
        } else {
            response(200, b"{\"status\":\"fixture image complete\"}\n".to_vec())
        };
    }
    if path.contains("/images/") && path.ends_with("/json") {
        return if state.image_present {
            response(
                200,
                serde_json::to_vec(&bollard::models::ImageInspect {
                    id: Some(format!("sha256:{}", "c".repeat(64))),
                    architecture: Some(state.image_architecture.clone()),
                    os: Some("linux".into()),
                    config: Some(Default::default()),
                    ..Default::default()
                })?,
            )
        } else {
            message(404)
        };
    }
    if path.ends_with("/networks/bridge") {
        return response(
            200,
            serde_json::to_vec(&bollard::models::NetworkInspect {
                name: Some("bridge".into()),
                driver: Some("bridge".into()),
                internal: Some(state.network_internal),
                ..Default::default()
            })?,
        );
    }
    if path.ends_with("/containers/create") && parts.method == hyper::Method::POST {
        state.container_actions.push("create".into());
        if state.reject_container {
            return message(400);
        }
        let profile: ContainerCreateBody = serde_json::from_slice(&body)?;
        let name = url::form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes())
            .find(|(key, _)| key == "name")
            .map(|(_, value)| value.into_owned())
            .ok_or("Missing native name")?;
        let mounts = profile
            .host_config
            .as_ref()
            .and_then(|host| host.mounts.as_ref())
            .ok_or("Missing native mounts")?;
        let mount = mounts.first().ok_or("Missing saved mount")?;
        state.container = Some(ContainerInspectResponse {
            id: Some("a".repeat(64)),
            name: Some(format!("/{name}")),
            image: Some(format!("sha256:{}", "c".repeat(64))),
            config: Some(ContainerConfig {
                image: profile.image,
                user: profile.user,
                entrypoint: profile.entrypoint,
                cmd: profile.cmd,
                env: profile.env,
                labels: profile.labels,
                stop_signal: profile.stop_signal,
                stop_timeout: profile.stop_timeout,
                ..Default::default()
            }),
            mounts: Some(vec![MountPoint {
                typ: Some("volume".into()),
                name: mount.source.clone(),
                destination: mount.target.clone(),
                driver: Some("local".into()),
                rw: Some(true),
                ..Default::default()
            }]),
            host_config: profile.host_config,
            state: Some(ContainerState {
                status: Some(ContainerStateStatusEnum::CREATED),
                running: Some(false),
                paused: Some(false),
                restarting: Some(false),
                dead: Some(false),
                pid: Some(0),
                started_at: Some("0001-01-01T00:00:00Z".into()),
                finished_at: Some("0001-01-01T00:00:00Z".into()),
                ..Default::default()
            }),
            ..Default::default()
        });
        return response(
            201,
            serde_json::to_vec(&ContainerCreateResponse {
                id: "a".repeat(64),
                warnings: vec![],
            })?,
        );
    }
    if path.contains("/containers/") {
        let pieces = path
            .split("/containers/")
            .last()
            .ok_or("Missing container target")?;
        let (id, operation) = pieces.split_once('/').unwrap_or((pieces, ""));
        if !matches!(id, "execution-candidate") && id != "a".repeat(64) {
            return message(404);
        }
        if operation == "json" {
            return match &state.container {
                Some(value) => response(200, serde_json::to_vec(value)?),
                None => message(404),
            };
        }
        if state.container.is_none() {
            return message(404);
        }
        let action = if parts.method == hyper::Method::DELETE {
            "remove"
        } else {
            operation
        };
        state.container_actions.push(action.into());
        let lost = state.fail_stage == Some(action);
        match action {
            "start" => {
                let value = state
                    .container
                    .as_mut()
                    .and_then(|value| value.state.as_mut())
                    .ok_or("Missing native state")?;
                value.status = Some(ContainerStateStatusEnum::RUNNING);
                value.running = Some(true);
                value.pid = Some(123);
                value.started_at = Some("2026-11-03T06:46:00Z".into());
            }
            "update" => {
                let body: ContainerUpdateBody = serde_json::from_slice(&body)?;
                if body.restart_policy.as_ref().and_then(|policy| policy.name)
                    != Some(RestartPolicyNameEnum::NO)
                {
                    return message(400);
                }
                state
                    .container
                    .as_mut()
                    .and_then(|value| value.host_config.as_mut())
                    .ok_or("Missing host config")?
                    .restart_policy = body.restart_policy;
                return if lost {
                    message(503)
                } else {
                    response(200, br#"{"Warnings":[]}"#.to_vec())
                };
            }
            "stop" => {
                let value = state
                    .container
                    .as_mut()
                    .and_then(|value| value.state.as_mut())
                    .ok_or("Missing native state")?;
                value.status = Some(ContainerStateStatusEnum::EXITED);
                value.running = Some(false);
                value.pid = Some(0);
                value.finished_at = Some("2026-11-03T06:47:00Z".into());
            }
            "remove" => {
                if url::form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes()).any(
                    |(key, value)| {
                        matches!(key.as_ref(), "force" | "v") && value != "false" && value != "0"
                    },
                ) {
                    return message(400);
                }
                state.container = None;
            }
            _ => return message(404),
        }
        return if lost {
            message(503)
        } else {
            response(204, Vec::new())
        };
    }
    if path.ends_with("/volumes/create") && parts.method == hyper::Method::POST {
        state.creates += 1;
        let requested: VolumeCreateRequest = serde_json::from_slice(&body)?;
        if requested.name.as_deref() != Some("saved-original")
            || requested.driver.as_deref() != Some("local")
            || requested.labels.as_ref() != Some(&labels())
            || requested
                .driver_opts
                .as_ref()
                .is_none_or(|values| !values.is_empty())
            || requested.cluster_volume_spec.is_some()
        {
            return message(400);
        }
        if matches!(state.create, Create::Rejected) {
            return message(400);
        }
        state.volume = Some(volume());
        if matches!(state.create, Create::AcceptedLost) {
            return message(503);
        }
        return response(201, serde_json::to_vec(&volume())?);
    }
    if path.ends_with("/volumes/saved-original") {
        if parts.method == hyper::Method::GET {
            return match &state.volume {
                Some(volume) => response(200, serde_json::to_vec(volume)?),
                None => message(404),
            };
        }
        if parts.method == hyper::Method::DELETE {
            if url::form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes())
                .any(|(key, value)| key == "force" && value != "false" && value != "0")
            {
                return message(400);
            }
            state.removes += 1;
            state.volume = None;
            return response(204, Vec::new());
        }
    }
    message(404)
}
impl Fixture {
    async fn new() -> Result<Self, TestError> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let state = Arc::new(Mutex::new(State {
            info: SystemInfo {
                id: Some("daemon:fixture".into()),
                os_type: Some("linux".into()),
                architecture: Some("aarch64".into()),
                memory_limit: Some(true),
                cpu_cfs_period: Some(true),
                cpu_cfs_quota: Some(true),
                security_options: Some(vec!["name=seccomp,profile=builtin".into()]),
                ..Default::default()
            },
            volume: None,
            create: Create::Normal,
            creates: 0,
            removes: 0,
            container: None,
            container_actions: vec![],
            reject_container: false,
            fail_stage: None,
            image_present: true,
            image_error: false,
            image_architecture: "arm64".into(),
            image_pulls: 0,
            network_internal: false,
        }));
        let shared = state.clone();
        let (shutdown, mut stop) = oneshot::channel();
        let server = tokio::spawn(async move {
            let mut tasks = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stop => break,
                    accepted = listener.accept() => {
                        let (stream, _) = accepted?;
                        let shared = shared.clone();
                        tasks.spawn(async move {
                            hyper::server::conn::http1::Builder::new().keep_alive(false)
                                .serve_connection(TokioIo::new(stream), service_fn(move |request| serve(request, shared.clone())))
                                .await.map_err(TestError::from)
                        });
                    }
                }
            }
            while let Some(result) = tasks.join_next().await {
                result??;
            }
            Ok(())
        });
        Ok(Self {
            port,
            state,
            shutdown,
            server,
        })
    }
    async fn connect(&self) -> Result<Docker, TestError> {
        self.connect_policy(ImagePolicy::Never).await
    }
    async fn connect_policy(&self, image_policy: ImagePolicy) -> Result<Docker, TestError> {
        Ok(Docker::connect(
            "provider".into(),
            Selection {
                api_version: None,
                endpoint: Endpoint::Tcp {
                    host: "127.0.0.1".into(),
                    port: self.port,
                },
                tls: None,
            },
            Config {
                image: "public.example.invalid/computer:fixture".into(),
                image_policy,
                saved_mount: "/workspace".into(),
                network: None,
                nano_cpus: 1_000_000_000,
                memory_bytes: 1024 * 1024 * 1024,
            },
        )
        .await?)
    }
    async fn finish(self, docker: Docker) -> Result<(), TestError> {
        docker.drain().await?;
        let _ = self.shutdown.send(());
        self.server.await??;
        Ok(())
    }
}

#[tokio::test]
async fn replaced_creation_identity_and_local_driver_options_remain_unverified()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    for variant in 0..3 {
        let mut value = volume();
        match variant {
            0 => value.created_at = Some("2026-11-04T06:45:15Z".into()),
            1 => {
                value.options.insert("device".into(), "/host/data".into());
            }
            _ => value.scope = Some(VolumeScopeEnum::GLOBAL),
        }
        fixture.state.lock().map_err(|_| "Fixture lock")?.volume = Some(value);
        assert!(
            docker
                .ensure_volume(&context(), &reserved(true), true)
                .await
                .is_err()
        );
    }
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!((state.creates, state.removes), (0, 0));
    }
    fixture.finish(docker).await
}

#[tokio::test]
async fn absence_proof_never_adopts_or_deletes_a_surprisingly_present_volume()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    fixture.state.lock().map_err(|_| "Fixture lock")?.volume = Some(volume());
    for allocated in [false, true] {
        let mut reservation = reserved(allocated);
        reservation.absence_confirmed = true;
        assert!(
            docker
                .observe_volume(&context(), &reservation)
                .await
                .is_err()
        );
        assert!(
            docker
                .remove_volume(&context(), &reservation)
                .await
                .is_err()
        );
    }
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!((state.creates, state.removes), (0, 0));
    }
    fixture.finish(docker).await
}

#[tokio::test]
async fn original_volume_is_never_replaced_even_during_initial_execution() -> Result<(), TestError>
{
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    assert!(matches!(
        docker
            .ensure_volume(&context(), &reserved(true), true)
            .await,
        Err(Error::MissingEnvironment)
    ));
    assert!(matches!(
        docker
            .ensure_volume(&context(), &reserved(false), false)
            .await,
        Err(Error::MissingEnvironment)
    ));
    assert_eq!(fixture.state.lock().map_err(|_| "Fixture lock")?.creates, 0);
    fixture.finish(docker).await
}
#[tokio::test]
async fn native_create_and_scoped_remove_happen_once_with_original_metadata()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let allocated = docker
        .ensure_volume(&context(), &reserved(false), true)
        .await?;
    let allocated = allocated.resource();
    let Some(Native::DockerVolume(native)) = &allocated.native else {
        return Err("Wrong native identity".into());
    };
    assert_eq!(
        native
            .identity
            .as_ref()
            .and_then(|identity| identity.created_at.as_deref()),
        Some("2026-11-03T06:45:15Z")
    );
    docker.ensure_volume(&context(), allocated, true).await?;
    let observed = docker.remove_volume(&context(), allocated).await?;
    assert_eq!(observed.presence, Presence::Absent);
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!((state.creates, state.removes), (1, 1));
    }
    fixture.finish(docker).await
}
#[tokio::test]
async fn lost_native_result_is_uncertain_and_observation_never_repeats_create()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    fixture.state.lock().map_err(|_| "Fixture lock")?.create = Create::AcceptedLost;
    let docker = fixture.connect().await?;
    let error = docker
        .ensure_volume(&context(), &reserved(false), true)
        .await
        .err()
        .ok_or("Expected lost native result")?;
    assert!(!error.definitely_rejected());
    assert_eq!(
        docker
            .observe_volume(&context(), &reserved(false))
            .await?
            .presence,
        Presence::Present
    );
    assert_eq!(fixture.state.lock().map_err(|_| "Fixture lock")?.creates, 1);
    fixture.finish(docker).await
}
#[tokio::test]
async fn only_create_rejection_is_definitive_and_unallocated_404_does_not_allow_cleanup()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    fixture.state.lock().map_err(|_| "Fixture lock")?.create = Create::Rejected;
    let docker = fixture.connect().await?;
    assert_eq!(
        docker
            .observe_volume(&context(), &reserved(false))
            .await?
            .presence,
        Presence::Unknown
    );
    assert!(matches!(
        docker.remove_volume(&context(), &reserved(false)).await,
        Err(Error::Uncertain)
    ));
    let error = docker
        .ensure_volume(&context(), &reserved(false), true)
        .await
        .err()
        .ok_or("Expected explicit rejection")?;
    assert!(error.definitely_rejected());
    let cause = std::error::Error::source(&error).ok_or("Native rejection cause was discarded")?;
    assert!(matches!(
        cause.downcast_ref::<bollard::errors::Error>(),
        Some(bollard::errors::Error::DockerResponseServerError {
            status_code: 400,
            ..
        })
    ));
    assert!(!error.to_string().contains("fixture native outcome"));
    assert!(!format!("{error:?}").contains("fixture native outcome"));
    assert_eq!(fixture.state.lock().map_err(|_| "Fixture lock")?.removes, 0);
    fixture.finish(docker).await
}
#[tokio::test]
async fn container_create_rejection_retains_native_cause_without_another_submission()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let saved = docker
        .ensure_volume(&context(), &reserved(false), true)
        .await?;
    fixture
        .state
        .lock()
        .map_err(|_| "Fixture lock")?
        .reject_container = true;
    let resource = wire::ManagedResource {
        kind: wire::ResourceKind::DockerContainer as i32,
        name: "execution-candidate".into(),
        execution_id: Some("execution".into()),
        native: Some(Native::DockerContainer(wire::DockerContainerNative {
            daemon_id: "daemon:fixture".into(),
            container_id: None,
        })),
        ..Default::default()
    };
    let image = docker.prepare_image().await?;
    let failure = docker
        .prepare_container(
            &context(),
            &resource,
            &saved,
            &image,
            &chapchoo_docker::Bootstrap {
                session: "session-original",
                initial_key: "fixture-bootstrap",
                endpoint: "http://control.invalid:50051",
            },
            true,
        )
        .await
        .err()
        .ok_or("Expected explicit container rejection")?;
    assert!(failure.container_creation_retry_safe());
    let cause =
        std::error::Error::source(&failure).ok_or("Native rejection cause was discarded")?;
    assert!(matches!(
        cause.downcast_ref::<bollard::errors::Error>(),
        Some(bollard::errors::Error::DockerResponseServerError {
            status_code: 400,
            ..
        })
    ));
    assert!(!failure.to_string().contains("fixture native outcome"));
    assert!(!format!("{failure:?}").contains("fixture native outcome"));
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!(state.container_actions, ["create"]);
        assert!(state.container.is_none());
        assert_eq!((state.creates, state.removes), (1, 0));
    }
    fixture.finish(docker).await
}
#[tokio::test]
async fn changed_daemon_or_foreign_volume_cannot_be_adopted_or_mutated() -> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    {
        let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        state.volume = Some(Volume {
            labels: HashMap::from([("foreign".into(), "owner".into())]),
            ..volume()
        });
    }
    assert!(matches!(
        docker
            .ensure_volume(&context(), &reserved(false), true)
            .await,
        Err(Error::Ownership)
    ));
    fixture.state.lock().map_err(|_| "Fixture lock")?.info.id = Some("different-daemon".into());
    assert!(matches!(
        docker
            .ensure_volume(&context(), &reserved(false), true)
            .await,
        Err(Error::DaemonIdentity)
    ));
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!((state.creates, state.removes), (0, 0));
    }
    fixture.finish(docker).await
}
