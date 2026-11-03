//! Scoped Computer Provider process. SDK effect ownership survives stream loss.
use crate::{AppError, config::ProviderOptions};
use chapchoo_crypto::BearerSecret;
mod control;
mod docker_worker;
mod effects;
pub async fn run_docker(
    options: crate::config::DockerProviderOptions,
    owner: BearerSecret,
    stop: CancellationToken,
) -> Result<(), AppError> {
    let explicit = options
        .docker_host
        .map(|host| {
            chapchoo_docker::connection::Explicit::from_files(
                host,
                options.docker_tls,
                options.docker_tls_skip_verify,
                options.docker_ca_file.as_deref(),
                options.docker_cert_file.as_deref(),
                options.docker_key_file.as_deref(),
                options.docker_api_version,
            )
        })
        .transpose()?;
    let selected = chapchoo_docker::connection::Selection::from_process(explicit)?;
    let image_policy = match options.image_pull_policy.as_str() {
        "Always" => chapchoo_docker::ImagePolicy::Always,
        "IfNotPresent" => chapchoo_docker::ImagePolicy::IfNotPresent,
        "Never" => chapchoo_docker::ImagePolicy::Never,
        _ => return Err("Docker image pull policy is invalid".into()),
    };
    endpoint(&options.endpoint)?;
    let connector_endpoint = options
        .connector_endpoint
        .as_deref()
        .unwrap_or(&options.endpoint);
    endpoint(connector_endpoint)?;
    let adapter = chapchoo_docker::Docker::connect(
        options.provider_id.clone(),
        selected,
        chapchoo_docker::Config {
            image: options.image,
            image_policy,
            saved_mount: options.saved_mount,
            network: options.network,
            nano_cpus: options.nano_cpus,
            memory_bytes: options.memory_bytes,
        },
    )
    .await?;
    run_docker_with_adapter(
        &options.endpoint,
        connector_endpoint,
        &options.provider_id,
        &options.name,
        owner,
        Arc::new(adapter),
        stop,
    )
    .await
}
use chapchoo_kubernetes::{Kubernetes, Observation, Presence};
use chapchoo_protocol::control::{
    self as wire, provider_authentication_client::ProviderAuthenticationClient,
    provider_connection_client::ProviderConnectionClient,
};
use control::Control;
use effects::Effects;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::{
    Request, Status,
    transport::{Channel, Endpoint},
};
fn request<T>(value: T, key: &BearerSecret) -> Result<Request<T>, AppError> {
    let mut r = Request::new(value);
    r.metadata_mut()
        .insert("authorization", format!("Bearer {}", key.expose()).parse()?);
    r.set_timeout(Duration::from_secs(10));
    Ok(r)
}
fn id() -> Result<String, AppError> {
    Ok(BearerSecret::generate()?
        .digest()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
fn endpoint(value: &str) -> Result<Endpoint, AppError> {
    let parsed = url::Url::parse(value)?;
    if parsed.scheme() != "http"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("Provider requires an explicit trusted-network HTTP/2 endpoint".into());
    }
    Ok(Endpoint::from_shared(value.to_owned())?
        .connect_timeout(Duration::from_secs(5))
        .buffer_size(8)
        .concurrency_limit(8))
}
enum Adapter {
    Kubernetes(Arc<Kubernetes>),
    Docker(Arc<chapchoo_docker::Docker>),
}
impl Adapter {
    fn kind(&self) -> wire::ProviderKind {
        match self {
            Self::Kubernetes(_) => wire::ProviderKind::Kubernetes,
            Self::Docker(_) => wire::ProviderKind::Docker,
        }
    }
    fn daemon(&self) -> Option<String> {
        match self {
            Self::Kubernetes(_) => None,
            Self::Docker(adapter) => Some(adapter.daemon_id().as_str().into()),
        }
    }
    fn namespace(&self) -> &str {
        match self {
            Self::Kubernetes(adapter) => adapter.namespace(),
            Self::Docker(_) => "",
        }
    }
    async fn verify(&self) -> Result<(), AppError> {
        if let Self::Docker(adapter) = self {
            adapter.verify_daemon().await?;
        }
        Ok(())
    }
}
pub async fn run(
    options: ProviderOptions,
    owner: BearerSecret,
    stop: CancellationToken,
) -> Result<(), AppError> {
    let config = chapchoo_kubernetes::Config {
        namespace: options.namespace,
        image: options.image,
        image_pull_policy: options.image_pull_policy,
        storage_class: options.storage_class,
        storage_request: options.storage_request,
        saved_mount: options.saved_mount,
        service_account: options.service_account,
        cpu_request: options.cpu_request,
        memory_request: options.memory_request,
        cpu_limit: options.cpu_limit,
        memory_limit: options.memory_limit,
    };
    let adapter = if let Some(path) = options.kubeconfig {
        Kubernetes::from_kubeconfig(&path, options.provider_id.clone(), config).await?
    } else if options.in_cluster {
        Kubernetes::in_cluster(options.provider_id.clone(), config)?
    } else {
        return Err("Select explicit Kubernetes credentials".into());
    };
    run_with_adapter(
        &options.endpoint,
        &options.provider_id,
        &options.name,
        owner,
        Arc::new(adapter),
        stop,
    )
    .await
}
pub(crate) async fn run_with_adapter(
    endpoint_url: &str,
    provider: &str,
    name: &str,
    owner: BearerSecret,
    adapter: Arc<Kubernetes>,
    stop: CancellationToken,
) -> Result<(), AppError> {
    let channel = endpoint(endpoint_url)?.connect_lazy();
    run_connection(
        channel,
        endpoint_url,
        provider,
        name,
        owner,
        Arc::new(Adapter::Kubernetes(adapter)),
        stop,
    )
    .await
}
pub(crate) async fn run_docker_with_adapter(
    endpoint_url: &str,
    connector_endpoint: &str,
    provider: &str,
    name: &str,
    owner: BearerSecret,
    adapter: Arc<chapchoo_docker::Docker>,
    stop: CancellationToken,
) -> Result<(), AppError> {
    let result = async {
        endpoint(connector_endpoint)?;
        let channel = endpoint(endpoint_url)?.connect_lazy();
        run_connection(
            channel,
            connector_endpoint,
            provider,
            name,
            owner,
            Arc::new(Adapter::Docker(adapter.clone())),
            stop,
        )
        .await
    }
    .await;
    crate::completion::complete(result, adapter.drain().await.map_err(Into::into))
}
async fn run_connection(
    channel: Channel,
    connector_endpoint: &str,
    provider: &str,
    name: &str,
    owner: BearerSecret,
    adapter: Arc<Adapter>,
    stop: CancellationToken,
) -> Result<(), AppError> {
    adapter.verify().await?;
    let mut auth = ProviderAuthenticationClient::new(channel.clone())
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    auth.configure_system_provider(request(
        wire::ProviderConfiguration {
            provider_id: provider.into(),
            name: name.into(),
            kind: adapter.kind() as i32,
            docker_daemon_id: adapter.daemon(),
            capabilities: Some(wire::ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            }),
        },
        &owner,
    )?)
    .await?;
    let key = auth
        .rotate_key(request(
            wire::ProviderKeyRequest {
                provider_id: provider.into(),
                request_id: id()?,
                authority: wire::OwnerAuthority::System as i32,
            },
            &owner,
        )?)
        .await?
        .into_inner();
    stream_loop(
        channel,
        connector_endpoint,
        provider,
        owner,
        key,
        adapter,
        stop,
    )
    .await
}
async fn stream_loop(
    channel: Channel,
    connector_endpoint: &str,
    provider: &str,
    owner: BearerSecret,
    key: wire::ComponentKey,
    adapter: Arc<Adapter>,
    stop: CancellationToken,
) -> Result<(), AppError> {
    let mut auth = ProviderAuthenticationClient::new(channel.clone());
    let mut renewal = refresh_at(key.proactive_after.as_ref())?;
    let mut key = BearerSecret::parse(key.token)?;
    own_effects(async |effects| loop {
        if stop.is_cancelled() {
            break Ok(());
        }
        effects.reap_completed()?;
        let result = tokio::select! {result=connected(
            channel.clone(),
            provider,
            connector_endpoint,
            &key,
            adapter.clone(),
            effects,
            &stop,
        )=>result,_=tokio::time::sleep_until(renewal)=>Err(Box::new(Status::unauthenticated("Scheduled Provider key renewal")) as AppError)};
        // A consumed owned-worker failure is not a reconnectable stream error.
        // Return it through the completion boundary instead of discarding it.
        if result.as_ref().err().is_some_and(|error| {
            error.downcast_ref::<effects::EffectError>().is_some()
        }) {
            break result;
        }
        if stop.is_cancelled() {
            break Ok(());
        }
        if let Err(error) = &result
            && error.downcast_ref::<Status>().is_some_and(|s| {
                matches!(
                    s.code(),
                    tonic::Code::Unauthenticated | tonic::Code::PermissionDenied
                )
            })
        {
            // Recover only under configured owner authority, never self-rotate.
            match auth
                .rotate_key(request(
                    wire::ProviderKeyRequest {
                        provider_id: provider.into(),
                        request_id: id()?,
                        authority: wire::OwnerAuthority::System as i32,
                    },
                    &owner,
                )?)
                .await
            {
                Ok(result) => {
                    let result = result.into_inner();
                    renewal = refresh_at(result.proactive_after.as_ref())?;
                    key = BearerSecret::parse(result.token)?;
                }
                Err(error) => break Err(Box::new(error) as AppError),
            }
        }
        tokio::select! {_=stop.cancelled()=>break Ok(()),_=tokio::time::sleep(Duration::from_secs(1))=>{}}
    }).await
}

async fn own_effects(
    run: impl AsyncFnOnce(&mut Effects) -> Result<(), AppError>,
) -> Result<(), AppError> {
    let mut effects = Effects::new();
    let primary = run(&mut effects).await;
    // Every propagated loop error finishes the same retained native owner.
    crate::completion::complete(primary, effects.drain().await.map_err(Into::into))
}
fn refresh_at(value: Option<&prost_types::Timestamp>) -> Result<tokio::time::Instant, AppError> {
    let value = value.ok_or("Provider key refresh time missing")?;
    let target = i128::from(value.seconds) * 1000 + i128::from(value.nanos) / 1_000_000;
    let now = i128::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
    )?;
    Ok(tokio::time::Instant::now() + Duration::from_millis(u64::try_from((target - now).max(0))?))
}
async fn connected(
    channel: Channel,
    provider: &str,
    connector_endpoint: &str,
    key: &BearerSecret,
    adapter: Arc<Adapter>,
    effects: &mut Effects,
    stop: &CancellationToken,
) -> Result<(), AppError> {
    adapter.verify().await?;
    let auth = ProviderAuthenticationClient::new(channel.clone())
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let mut client = ProviderConnectionClient::new(channel)
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let (sender, receiver) = mpsc::channel(8);
    sender
        .send(wire::ProviderFrame {
            payload: Some(wire::provider_frame::Payload::Hello(wire::ProviderHello {
                provider_id: provider.into(),
                namespace: adapter.namespace().into(),
                connector_endpoint: connector_endpoint.into(),
                kind: adapter.kind() as i32,
                docker_daemon_id: adapter.daemon(),
            })),
        })
        .await?;
    let mut open = Request::new(ReceiverStream::new(receiver));
    open.metadata_mut()
        .insert("authorization", format!("Bearer {}", key.expose()).parse()?);
    let mut stream = tokio::time::timeout(Duration::from_secs(10), client.open(open))
        .await??
        .into_inner();
    let first = tokio::time::timeout(Duration::from_secs(10), stream.message())
        .await??
        .ok_or("Provider acceptance missing")?;
    let Some(wire::control_frame::Payload::Connected(connected)) = first.payload else {
        return Err("Expected Provider acceptance".into());
    };
    if connected.heartbeat_interval_ms == 0
        || connected.validity_ms <= connected.heartbeat_interval_ms
    {
        return Err("Provider heartbeat policy is invalid".into());
    }
    let source = wire::ProviderConnectionCheck {
        provider_id: provider.into(),
        instance_id: connected.instance_id,
        connection_id: connected.connection_id.clone(),
    };
    let mut heartbeat = tokio::time::interval(Duration::from_millis(u64::from(
        connected.heartbeat_interval_ms,
    )));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut acknowledged = tokio::time::Instant::now();
    loop {
        tokio::select! {
            _=stop.cancelled()=>return Ok(()),
            _=heartbeat.tick()=>{
                effects.reap_completed()?;
                adapter.verify().await?;
                if acknowledged.elapsed()>=Duration::from_millis(u64::from(connected.validity_ms)){return Err("Provider heartbeat acknowledgement expired".into());}
                sender.try_send(wire::ProviderFrame{payload:Some(wire::provider_frame::Payload::Heartbeat(wire::Heartbeat{connection_id:connected.connection_id.clone()}))})?;
            }
            frame=stream.message()=>{
                let frame=frame?.ok_or("Provider stream closed")?;
                match frame.payload {
                    Some(wire::control_frame::Payload::Heartbeat(value)) if value.connection_id==connected.connection_id=>acknowledged=tokio::time::Instant::now(),
                    Some(wire::control_frame::Payload::Provider(command))=>{
                        if command.remaining_ms==0||command.identity.as_ref().is_none_or(|id|id.connection_id!=connected.connection_id){continue;}
                        let control=Control{auth:auth.clone(),source:source.clone(),key:BearerSecret::parse(key.expose().to_owned())?,sender:sender.clone()};
                        if effects.active(){control.reply(&command,wire::CommandOutcome::NotStarted,false).await?;continue;}
                        // The task, not this call or stream, owns its permit. Its
                        // original identity makes late replies harmless after reconnect.
                        let adapter=adapter.clone();
                        let _waiter=effects.start(async move {
                            match adapter.as_ref() {
                                Adapter::Kubernetes(adapter)=>Worker{control,adapter:adapter.clone(),execution_terminal:false,preparation_retry_safe:false}.execute(command).await,
                                Adapter::Docker(adapter)=>docker_worker::Worker{control,adapter:adapter.clone(),preparation_retry_safe:false}.execute(command).await,
                            }
                        })?;
                    }
                    Some(wire::control_frame::Payload::Cancel(_))=>{},
                    _=>return Err("Unexpected Provider frame".into()),
                }
            }
        }
    }
}
struct Worker {
    execution_terminal: bool,
    preparation_retry_safe: bool,
    control: Control,
    adapter: Arc<Kubernetes>,
}
impl Worker {
    async fn check(&mut self, command: &wire::ProviderCommand) -> Result<(), AppError> {
        self.control.check(command).await?;
        Ok(())
    }
    async fn report(
        &mut self,
        context: &wire::ComputerContext,
        resources: Vec<wire::ResourceEvidence>,
        fact: wire::ExecutionFact,
        cleanup: bool,
    ) -> Result<(), AppError> {
        self.control.report(context, resources, fact, cleanup).await
    }
    async fn reply(
        &self,
        command: &wire::ProviderCommand,
        outcome: wire::CommandOutcome,
    ) -> Result<(), AppError> {
        self.control
            .reply(command, outcome, self.execution_terminal)
            .await
    }
    async fn execute(mut self, mut command: wire::ProviderCommand) {
        let result = self.perform(&mut command).await;
        let outcome = match result {
            Ok(()) => wire::CommandOutcome::Completed,
            Err(error) => {
                let blocked = error
                    .downcast_ref::<chapchoo_kubernetes::Error>()
                    .is_some_and(|e| {
                        matches!(
                            e,
                            chapchoo_kubernetes::Error::Ownership
                                | chapchoo_kubernetes::Error::MissingEnvironment
                                | chapchoo_kubernetes::Error::Configuration
                        )
                    });
                let safe_preparation = self.preparation_retry_safe
                    && matches!(
                        command.operation,
                        Some(wire::provider_command::Operation::Prepare(_))
                    );
                if let Some(context) = command.computer.as_ref() {
                    let _ = self
                        .report(
                            context,
                            vec![],
                            if safe_preparation {
                                wire::ExecutionFact::PreparationBlocked
                            } else if blocked {
                                wire::ExecutionFact::Blocked
                            } else {
                                wire::ExecutionFact::Uncertain
                            },
                            false,
                        )
                        .await;
                }
                if blocked || safe_preparation {
                    wire::CommandOutcome::Blocked
                } else {
                    wire::CommandOutcome::Uncertain
                }
            }
        };
        let _ = self.reply(&command, outcome).await;
    }
    async fn observed(
        &mut self,
        context: &wire::ComputerContext,
        observation: Observation,
    ) -> Result<(), AppError> {
        let fact = if observation.running {
            wire::ExecutionFact::Running
        } else {
            wire::ExecutionFact::Unspecified
        };
        self.report(context, vec![evidence(observation)], fact, false)
            .await
    }
    async fn perform(&mut self, command: &mut wire::ProviderCommand) -> Result<(), AppError> {
        self.check(command).await?;
        let context = command.computer.clone().ok_or("Computer context missing")?;
        match command
            .operation
            .clone()
            .ok_or("Provider operation missing")?
        {
            wire::provider_command::Operation::Observe(value) => {
                let mut observations = Vec::with_capacity(value.resources.len());
                let mut running = false;
                let mut retired = false;
                let mut unknown = false;
                for resource in value.resources {
                    let resource_context = wire::ComputerContext {
                        execution_id: resource
                            .execution_id
                            .clone()
                            .unwrap_or_else(|| context.execution_id.clone()),
                        ..context.clone()
                    };
                    let observed = self
                        .adapter
                        .observe(&resource_context, &resource, None)
                        .await?;
                    self.execution_terminal |= observed.terminal.is_some()
                        && resource.execution_id.as_deref() == Some(&context.execution_id);
                    running |= observed.running
                        && resource.execution_id.as_deref() == Some(&context.execution_id);
                    retired |= observed.presence == Presence::Absent
                        && resource.kind == wire::ResourceKind::Pod as i32
                        && resource.execution_id.as_deref() == Some(&context.execution_id);
                    unknown |= observed.presence == Presence::Unknown
                        && resource.kind == wire::ResourceKind::Pod as i32
                        && resource.execution_id.as_deref() == Some(&context.execution_id)
                        && resource.uid.is_some();
                    observations.push(evidence(observed));
                }
                self.report(
                    &context,
                    observations,
                    if running {
                        wire::ExecutionFact::Running
                    } else if retired {
                        wire::ExecutionFact::Retired
                    } else if unknown {
                        wire::ExecutionFact::Uncertain
                    } else {
                        wire::ExecutionFact::Unspecified
                    },
                    false,
                )
                .await?;
            }
            wire::provider_command::Operation::Prepare(mut value) => {
                if value.docker.is_some() {
                    return Err(chapchoo_kubernetes::Error::Configuration.into());
                }
                self.preparation_retry_safe = false;
                let pvc = value.saved_pvc.as_ref().ok_or("Saved PVC missing")?;
                let pvc = self
                    .adapter
                    .ensure_pvc(&context, pvc, value.initial_allocation)
                    .await
                    .inspect_err(|error| {
                        self.preparation_retry_safe = error.definitely_rejected()
                    })?;
                self.report(
                    &context,
                    vec![present(pvc.clone())],
                    wire::ExecutionFact::Unspecified,
                    false,
                )
                .await?;
                self.preparation_retry_safe = true;
                value.saved_pvc = Some(pvc.clone());
                command.operation = Some(wire::provider_command::Operation::Prepare(value.clone()));
                self.check(command).await?;
                self.preparation_retry_safe = false;
                let secret = self
                    .adapter
                    .ensure_bootstrap(
                        &context,
                        value
                            .bootstrap_secret
                            .as_ref()
                            .ok_or("Bootstrap Secret missing")?,
                        &value.connector_session_id,
                        &value.connector_initial_key,
                        &value.connector_endpoint,
                    )
                    .await
                    .inspect_err(|error| {
                        self.preparation_retry_safe = error.definitely_rejected()
                    })?;
                self.report(
                    &context,
                    vec![present(secret.clone())],
                    wire::ExecutionFact::Unspecified,
                    false,
                )
                .await?;
                self.preparation_retry_safe = true;
                value.bootstrap_secret = Some(secret.clone());
                command.operation = Some(wire::provider_command::Operation::Prepare(value.clone()));
                self.check(command).await?;
                self.preparation_retry_safe = false;
                let pod = self
                    .adapter
                    .ensure_pod(
                        &context,
                        value.pod.as_ref().ok_or("Pod missing")?,
                        &pvc,
                        &secret,
                    )
                    .await
                    .inspect_err(|error| {
                        self.preparation_retry_safe = error.definitely_rejected()
                    })?;
                self.report(
                    &context,
                    vec![present(pod.clone())],
                    wire::ExecutionFact::Unspecified,
                    false,
                )
                .await?;
                self.observed(&context, self.adapter.observe(&context, &pod, None).await?)
                    .await?;
            }
            wire::provider_command::Operation::Retire(value) => {
                if value.docker_container.is_some() {
                    return Err(chapchoo_kubernetes::Error::Configuration.into());
                }
                let pod = value.pod.as_ref().ok_or("Retirement Pod missing")?;
                if value.require_terminal {
                    let prior = self.adapter.observe(&context, pod, None).await?;
                    if prior.terminal.is_none() && prior.presence != Presence::Absent {
                        return Err("Automatic restoration requires exact terminal evidence".into());
                    }
                }
                let observed = self.adapter.retire_pod(&context, pod).await?;
                let absent = observed.presence == Presence::Absent;
                self.report(
                    &context,
                    vec![evidence(observed)],
                    if absent {
                        wire::ExecutionFact::Retired
                    } else {
                        wire::ExecutionFact::Uncertain
                    },
                    false,
                )
                .await?;
            }
            wire::provider_command::Operation::Cleanup(value) => {
                // Pod quiescence precedes Secret/PVC deletion. Unknown never
                // advances cleanup or permits destruction of saved storage.
                for resource in value
                    .resources
                    .iter()
                    .filter(|r| r.kind == wire::ResourceKind::Pod as i32)
                {
                    let is_current = resource.execution_id.as_deref()
                        == Some(&context.execution_id)
                        && !context.execution_id.is_empty();
                    let context = wire::ComputerContext {
                        execution_id: resource.execution_id.clone().ok_or("Execution missing")?,
                        ..context.clone()
                    };
                    self.check(command).await?;
                    let observed = self.adapter.retire_pod(&context, resource).await?;
                    if observed.presence != Presence::Absent {
                        return Err("Pod retirement remains uncertain".into());
                    }
                    self.report(
                        &context,
                        vec![evidence(observed)],
                        if is_current {
                            wire::ExecutionFact::Retired
                        } else {
                            wire::ExecutionFact::Unspecified
                        },
                        false,
                    )
                    .await?;
                    cleanup_absence(command, resource);
                }
                for resource in value
                    .resources
                    .into_iter()
                    .filter(|r| r.kind != wire::ResourceKind::Pod as i32)
                {
                    self.check(command).await?;
                    let context = wire::ComputerContext {
                        execution_id: resource
                            .execution_id
                            .clone()
                            .unwrap_or_else(|| context.execution_id.clone()),
                        ..context.clone()
                    };
                    let observed = self.adapter.observe(&context, &resource, None).await?;
                    if observed.presence == Presence::Present {
                        self.adapter.delete(&context, &observed.resource).await?;
                    }
                    let observed = self.adapter.observe(&context, &resource, None).await?;
                    if observed.presence != Presence::Absent {
                        return Err("Resource cleanup remains uncertain".into());
                    }
                    self.report(
                        &context,
                        vec![evidence(observed)],
                        wire::ExecutionFact::Unspecified,
                        false,
                    )
                    .await?;
                    cleanup_absence(command, &resource);
                }
                if value.confirm_complete {
                    self.report(&context, vec![], wire::ExecutionFact::Unspecified, true)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
fn cleanup_absence(command: &mut wire::ProviderCommand, resource: &wire::ManagedResource) {
    if let Some(wire::provider_command::Operation::Cleanup(value)) = command.operation.as_mut()
        && let Some(saved) = value.resources.iter_mut().find(|r| {
            r.kind == resource.kind && r.namespace == resource.namespace && r.name == resource.name
        })
    {
        saved.absence_confirmed = true;
    }
}
fn present(resource: wire::ManagedResource) -> wire::ResourceEvidence {
    wire::ResourceEvidence {
        evidence: Some(wire::resource_evidence::Evidence::Present(
            wire::OwnedPresent {
                uid: resource.uid.clone().unwrap_or_default(),
                native_identity: None,
            },
        )),
        resource: Some(resource),
    }
}
fn evidence(observation: Observation) -> wire::ResourceEvidence {
    let evidence = match observation.presence {
        Presence::Present => wire::resource_evidence::Evidence::Present(wire::OwnedPresent {
            uid: observation.resource.uid.clone().unwrap_or_default(),
            native_identity: None,
        }),
        Presence::Absent => wire::resource_evidence::Evidence::Absent(wire::DefinitivelyAbsent {
            expected_uid: observation.resource.uid.clone(),
            expected_native_identity: None,
        }),
        Presence::Unknown => wire::resource_evidence::Evidence::Unknown(wire::UnknownResource {}),
    };
    wire::ResourceEvidence {
        resource: Some(observation.resource),
        evidence: Some(evidence),
    }
}

#[cfg(test)]
mod tests;
