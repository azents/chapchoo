//! Docker-native stage orchestration. Every native mutation follows a fresh
//! exact Control check; each allocation must be ACKed before later effects.
use super::*;
use chapchoo_docker::{Bootstrap, Docker, Error, Observation, Presence};
pub(super) struct Worker {
    pub control: Control,
    pub adapter: Arc<Docker>,
    pub preparation_retry_safe: bool,
}
fn evidence(observation: Observation) -> wire::ResourceEvidence {
    let (present_identity, absent_identity) = match observation.resource.native.as_ref() {
        Some(wire::managed_resource::Native::DockerContainer(value)) => (
            value
                .container_id
                .clone()
                .map(wire::owned_present::NativeIdentity::DockerContainerId),
            value
                .container_id
                .clone()
                .map(wire::definitively_absent::ExpectedNativeIdentity::DockerContainerId),
        ),
        Some(wire::managed_resource::Native::DockerVolume(value)) => (
            value
                .identity
                .clone()
                .map(wire::owned_present::NativeIdentity::DockerVolume),
            value
                .identity
                .clone()
                .map(wire::definitively_absent::ExpectedNativeIdentity::DockerVolume),
        ),
        None => (None, None),
    };
    let fact = match observation.presence {
        Presence::Present => wire::resource_evidence::Evidence::Present(wire::OwnedPresent {
            uid: String::new(),
            native_identity: present_identity,
        }),
        Presence::Absent => wire::resource_evidence::Evidence::Absent(wire::DefinitivelyAbsent {
            expected_uid: None,
            expected_native_identity: absent_identity,
        }),
        Presence::Unknown => wire::resource_evidence::Evidence::Unknown(wire::UnknownResource {}),
    };
    wire::ResourceEvidence {
        resource: Some(observation.resource),
        evidence: Some(fact),
    }
}
fn present(resource: wire::ManagedResource) -> wire::ResourceEvidence {
    evidence(Observation {
        resource,
        presence: Presence::Present,
        running: false,
        terminal: false,
    })
}
impl Worker {
    pub async fn execute(mut self, mut command: wire::ProviderCommand) {
        let result = self.perform(&mut command).await;
        let outcome = match result {
            Ok(()) => wire::CommandOutcome::Completed,
            Err(error) => {
                if let Some(native) = error.downcast_ref::<Error>() {
                    tracing::warn!(
                        engine = ?native.engine_diagnostic(),
                        stage = command.computer.as_ref().and_then(|computer| self.adapter.container_progress_diagnostic(&computer.execution_id)),
                        native_docker = %native,
                        "Native Docker lifecycle command failed"
                    );
                } else if let Some(status) = error.downcast_ref::<tonic::Status>() {
                    tracing::warn!(code = ?status.code(), "Docker lifecycle Control RPC failed");
                }
                let blocked = error.downcast_ref::<Error>().is_some_and(|error| {
                    matches!(
                        error,
                        Error::Ownership
                            | Error::MissingEnvironment
                            | Error::Configuration
                            | Error::WorkloadPolicy
                    )
                });
                let safe = self.preparation_retry_safe
                    && matches!(
                        command.operation,
                        Some(wire::provider_command::Operation::Prepare(_))
                    );
                if let Some(context) = &command.computer {
                    let _ = self
                        .control
                        .report(
                            context,
                            vec![],
                            if safe {
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
                if safe || blocked {
                    wire::CommandOutcome::Blocked
                } else {
                    wire::CommandOutcome::Uncertain
                }
            }
        };
        let _ = self.control.reply(&command, outcome, false).await;
    }
    async fn perform(&mut self, command: &mut wire::ProviderCommand) -> Result<(), AppError> {
        let admission = self.control.check(command).await?;
        let context = command.computer.clone().ok_or("Computer context missing")?;
        match command
            .operation
            .clone()
            .ok_or("Provider operation missing")?
        {
            wire::provider_command::Operation::Observe(value) => {
                let saved = value
                    .resources
                    .iter()
                    .find(|resource| resource.kind == wire::ResourceKind::DockerVolume as i32);
                let mut resources = Vec::with_capacity(value.resources.len());
                let mut running = false;
                let mut retired = false;
                let mut unknown = false;
                for resource in &value.resources {
                    let resource_context = wire::ComputerContext {
                        execution_id: resource.execution_id.clone().unwrap_or_default(),
                        ..context.clone()
                    };
                    let observed = match wire::ResourceKind::try_from(resource.kind)? {
                        wire::ResourceKind::DockerVolume => {
                            self.adapter.observe_volume(&context, resource).await?
                        }
                        wire::ResourceKind::DockerContainer => {
                            if let Some(saved) = saved
                                .filter(|_| resource_context.execution_id == context.execution_id)
                            {
                                self.adapter
                                    .observe_container(
                                        &resource_context,
                                        resource,
                                        saved,
                                        admission.initial_allocation,
                                        admission.prior_running_completion
                                            && resource_context.execution_id
                                                == context.execution_id,
                                    )
                                    .await?
                            } else {
                                self.adapter
                                    .observe_container_absence(&resource_context, resource)
                                    .await?
                            }
                        }
                        _ => return Err(Error::Ownership.into()),
                    };
                    if resource.kind == wire::ResourceKind::DockerContainer as i32
                        && resource.execution_id.as_deref() == Some(&context.execution_id)
                    {
                        running |= observed.running;
                        retired |= observed.presence == Presence::Absent;
                        unknown |= !observed.running && observed.presence != Presence::Absent;
                    }
                    resources.push(evidence(observed));
                }
                self.control
                    .report(
                        &context,
                        resources,
                        if retired {
                            wire::ExecutionFact::Retired
                        } else if running {
                            wire::ExecutionFact::Running
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
                if value.pod.is_some()
                    || value.saved_pvc.is_some()
                    || value.bootstrap_secret.is_some()
                {
                    return Err(Error::Ownership.into());
                }
                let mut native = value.docker.clone().ok_or(Error::Ownership)?;
                let saved = native.saved_volume.as_ref().ok_or(Error::Ownership)?;
                self.preparation_retry_safe = false;
                let image = self.adapter.prepare_image().await?;
                self.control.check(command).await?;
                let prepared_volume = self
                    .adapter
                    .ensure_volume(&context, saved, value.initial_allocation)
                    .await
                    .inspect_err(|error| {
                        self.preparation_retry_safe = error.definitely_rejected()
                    })?;
                let saved = prepared_volume.resource().clone();
                self.control
                    .report(
                        &context,
                        vec![present(saved.clone())],
                        wire::ExecutionFact::Unspecified,
                        false,
                    )
                    .await?;
                native.saved_volume = Some(saved.clone());
                value.docker = Some(native.clone());
                command.operation = Some(wire::provider_command::Operation::Prepare(value.clone()));
                self.preparation_retry_safe = true;
                self.control.check(command).await?;
                self.preparation_retry_safe = false;
                let bootstrap = Bootstrap {
                    session: &value.connector_session_id,
                    initial_key: &value.connector_initial_key,
                    endpoint: &value.connector_endpoint,
                };
                let prepared = self
                    .adapter
                    .prepare_container(
                        &context,
                        native.container.as_ref().ok_or(Error::Ownership)?,
                        &prepared_volume,
                        &image,
                        &bootstrap,
                        value.initial_allocation,
                    )
                    .await
                    .inspect_err(|error| {
                        self.preparation_retry_safe = error.container_creation_retry_safe()
                    })?;
                let container = prepared.resource().clone();
                self.control
                    .report(
                        &context,
                        vec![present(container.clone())],
                        wire::ExecutionFact::Unspecified,
                        false,
                    )
                    .await?;
                native.container = Some(container);
                value.docker = Some(native);
                command.operation = Some(wire::provider_command::Operation::Prepare(value));
                self.control.check(command).await?;
                let observed = self.adapter.start_container(prepared).await?;
                let fact = if observed.running {
                    wire::ExecutionFact::Running
                } else {
                    wire::ExecutionFact::Uncertain
                };
                self.control
                    .report(&context, vec![evidence(observed)], fact, false)
                    .await?;
            }
            wire::provider_command::Operation::Retire(value) => {
                if value.pod.is_some() {
                    return Err(Error::Ownership.into());
                }
                let resource = value.docker_container.as_ref().ok_or(Error::Ownership)?;
                let saved = value.saved_volume.as_ref().ok_or(Error::Ownership)?;
                let prior = self
                    .adapter
                    .observe_container(
                        &context,
                        resource,
                        saved,
                        admission.initial_allocation,
                        admission.prior_running_completion,
                    )
                    .await?;
                if prior.presence == Presence::Absent {
                    self.control
                        .report(
                            &context,
                            vec![evidence(prior)],
                            wire::ExecutionFact::Retired,
                            false,
                        )
                        .await?;
                    return Ok(());
                }
                let retiring = self
                    .adapter
                    .begin_retirement(
                        &context,
                        resource,
                        saved,
                        admission.initial_allocation,
                        admission.prior_running_completion,
                        value.require_terminal,
                    )
                    .await?;
                self.control.check(command).await?;
                let stopped = self.adapter.stop_container(retiring).await?;
                self.control.check(command).await?;
                let updated = self.adapter.disable_restart(stopped).await?;
                self.control.check(command).await?;
                let absent = self.adapter.remove_container(updated).await?;
                self.control
                    .report(
                        &context,
                        vec![evidence(absent)],
                        wire::ExecutionFact::Retired,
                        false,
                    )
                    .await?;
            }
            wire::provider_command::Operation::Cleanup(value) => {
                if value
                    .resources
                    .iter()
                    .any(|resource| resource.kind != wire::ResourceKind::DockerVolume as i32)
                {
                    return Err(Error::Ownership.into());
                }
                for resource in &value.resources {
                    self.control.check(command).await?;
                    let observed = self.adapter.remove_volume(&context, resource).await?;
                    if observed.presence != Presence::Absent {
                        return Err(Error::Uncertain.into());
                    }
                    self.control
                        .report(
                            &context,
                            vec![evidence(observed)],
                            wire::ExecutionFact::Unspecified,
                            false,
                        )
                        .await?;
                    cleanup_absence(command, resource);
                }
                if value.confirm_complete {
                    self.control
                        .report(&context, vec![], wire::ExecutionFact::Unspecified, true)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
