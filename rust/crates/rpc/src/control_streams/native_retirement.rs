//! Connection-local Docker retirement correlation. Durable phases fence loss;
//! this bounded record never reconstructs permission to repeat a native effect.
use super::*;
use chapchoo_domain::{ManagedResourceKind, ProviderKind, ResourcePresence};
use chapchoo_service::{ComputerRequest, DeliverySnapshot, NativeRetirementAdmission};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchState {
    Admitting,
    Ready,
    Forwarded,
    Completed,
    NotStarted,
    Uncertain,
}

#[derive(Clone)]
pub(super) struct NativeProgress {
    computer: wire::ComputerContext,
    admission: Option<NativeRetirementAdmission>,
    command: Option<wire::ProviderCommand>,
    state: DispatchState,
    delete: bool,
    // Bounded comparison evidence fixes all reserved native addresses, including
    // the known-never-submitted suffix, before the first forwarding await.
    inventory: [u8; 32],
    // Once an earlier command completed, a later NotStarted may retry only that
    // known-unsent suffix. It cannot release the whole request's durable fence.
    completed_prefix: bool,
}

fn denied() -> Status {
    Status::failed_precondition("Native mutation has no exact live dispatch authority")
}

fn inventory_fingerprint(snapshot: &DeliverySnapshot) -> [u8; 32] {
    let mut addresses = snapshot
        .resources
        .iter()
        .map(|resource| {
            [
                resource.kind().as_str().as_bytes(),
                resource.native().scope().kind_str().as_bytes(),
                resource.native().scope().as_str().as_bytes(),
                resource.native().name().as_bytes(),
                resource.execution_id.as_deref().unwrap_or("").as_bytes(),
            ]
            .map(|value| value.to_vec())
        })
        .collect::<Vec<_>>();
    addresses.sort();
    let values = addresses
        .iter()
        .flat_map(|address| address.iter().map(Vec::as_slice))
        .collect::<Vec<_>>();
    chapchoo_crypto::payload_fingerprint("native-retirement-reservations", &values)
}

fn native_mutation(command: &wire::ProviderCommand) -> bool {
    match command.operation.as_ref() {
        Some(wire::provider_command::Operation::Retire(value)) => value.docker_container.is_some(),
        Some(wire::provider_command::Operation::Cleanup(value)) => {
            value.resources.iter().any(|r| {
                matches!(
                    wire::ResourceKind::try_from(r.kind),
                    Ok(wire::ResourceKind::DockerContainer | wire::ResourceKind::DockerVolume)
                )
            })
        }
        _ => false,
    }
}

fn normalized(mut command: wire::ProviderCommand) -> wire::ProviderCommand {
    // Queue time reduces the budget; acknowledged absence is read again from
    // current DB authority. Neither changes the immutable call/native target.
    command.remaining_ms = 0;
    match command.operation.as_mut() {
        Some(wire::provider_command::Operation::Retire(value)) => {
            if let Some(resource) = &mut value.docker_container {
                resource.absence_confirmed = false;
            }
        }
        Some(wire::provider_command::Operation::Cleanup(value)) => {
            for resource in &mut value.resources {
                resource.absence_confirmed = false;
            }
        }
        _ => {}
    }
    command
}

fn exact(progress: &NativeProgress, command: &wire::ProviderCommand) -> bool {
    progress
        .command
        .as_ref()
        .is_some_and(|expected| normalized(expected.clone()) == normalized(command.clone()))
}

pub(super) fn before_forward(
    entry: &Connection,
    command: &wire::ProviderCommand,
) -> Result<(), Status> {
    if !native_mutation(command) {
        return Ok(());
    }
    let computer = command.computer.as_ref().ok_or_else(denied)?;
    let mut records = entry.native.lock().map_err(|_| denied())?;
    let progress = records.get_mut(&computer.computer_id).ok_or_else(denied)?;
    if progress.state != DispatchState::Ready || !exact(progress, command) {
        return Err(denied());
    }
    progress.state = DispatchState::Forwarded;
    Ok(())
}

pub(super) fn result(entry: &Connection, value: &wire::ProviderResult) -> Result<(), Status> {
    let Some(computer) = &value.computer else {
        return Ok(());
    };
    let mut records = entry.native.lock().map_err(|_| denied())?;
    let Some(progress) = records.get_mut(&computer.computer_id) else {
        return Ok(());
    };
    let Some(command) = &progress.command else {
        return Ok(());
    };
    if command.identity != value.identity
        || command.computer != value.computer
        || !matches!(
            progress.state,
            DispatchState::Forwarded | DispatchState::Uncertain
        )
    {
        return Ok(());
    }
    progress.state = match wire::CommandOutcome::try_from(value.outcome) {
        Ok(wire::CommandOutcome::Completed) => DispatchState::Completed,
        Ok(wire::CommandOutcome::NotStarted) => DispatchState::NotStarted,
        _ => DispatchState::Uncertain,
    };
    Ok(())
}

pub(super) fn forwarding_failed(
    entry: &Connection,
    command: &wire::ProviderCommand,
    failure: CallFailure,
) -> Result<(), Status> {
    if !native_mutation(command) {
        return Ok(());
    }
    let computer = command.computer.as_ref().ok_or_else(denied)?;
    let mut records = entry.native.lock().map_err(|_| denied())?;
    if let Some(progress) = records.get_mut(&computer.computer_id)
        && exact(progress, command)
        && progress.state == DispatchState::Forwarded
    {
        progress.state = match failure {
            CallFailure::Unavailable | CallFailure::Overloaded | CallFailure::Invalid => {
                DispatchState::NotStarted
            }
            // This is loss of the response waiter, not a native task outcome.
            // Preserve only this already-forwarded call's continuation authority.
            // before_forward still denies resubmission; a native Uncertain
            // result or source/connection loss independently ends continuation.
            CallFailure::Uncertain => DispatchState::Forwarded,
        };
    }
    Ok(())
}

fn fresh_absent(value: &chapchoo_service::ManagedResource, source: &ProviderSource<'_>) -> bool {
    value.presence == ResourcePresence::Absent
        && value.source_connection_id.as_deref() == Some(source.connection_id)
        && value.absence_confirmed_at.is_some()
}

impl ComponentConnectionServices {
    pub(super) async fn prune_native(
        &self,
        entry: &Connection,
        source: &ProviderSource<'_>,
    ) -> Result<(), Status> {
        let pending = entry
            .native
            .lock()
            .map_err(|_| denied())?
            .values()
            .map(|p| p.computer.clone())
            .collect::<Vec<_>>();
        for computer in pending {
            let current = self
                .inner
                .service
                .provider_delivery_snapshot(
                    source,
                    ComputerRequest {
                        computer_id: &computer.computer_id,
                        request_id: &computer.request_id,
                    },
                    None,
                )
                .await;
            let remove = match current {
                Ok(snapshot) => {
                    snapshot.computer.cleanup_completed_at.is_some()
                        || (snapshot.computer.current_execution_id.is_none()
                            && snapshot.computer.desired_state != "DELETING")
                }
                Err(error) => matches!(
                    error.reason(),
                    chapchoo_service::DomainReason::NotFound
                        | chapchoo_service::DomainReason::Conflict
                ),
            };
            if remove {
                let mut records = entry.native.lock().map_err(|_| denied())?;
                if records
                    .get(&computer.computer_id)
                    .is_some_and(|p| p.computer == computer)
                {
                    records.remove(&computer.computer_id);
                }
            }
        }
        Ok(())
    }

    /// CheckCommand uses this same exact connection registry as both forwarding
    /// paths. DB authorization alone is not permission to reconstruct Retire.
    pub(crate) fn verify_native_dequeue(
        &self,
        source: &ProviderSource<'_>,
        command: &wire::ProviderCommand,
    ) -> Result<(), Status> {
        if !native_mutation(command) {
            return Ok(());
        }
        if source.control_id != self.inner.instance_id {
            return Err(denied());
        }
        let entry = self
            .inner
            .entries
            .lock()
            .map_err(|_| denied())?
            .providers
            .get(source.provider_id)
            .cloned()
            .ok_or_else(denied)?;
        if entry.prepared.owner().connection_id != source.connection_id || entry.calls.is_closed() {
            return Err(denied());
        }
        let records = entry.native.lock().map_err(|_| denied())?;
        let context = command.computer.as_ref().ok_or_else(denied)?;
        let progress = records.get(&context.computer_id).ok_or_else(denied)?;
        if progress.state != DispatchState::Forwarded || !exact(progress, command) {
            return Err(denied());
        }
        Ok(())
    }

    pub(super) async fn reconcile_native_retirement(
        &self,
        entry: &Connection,
        computer: &wire::ComputerContext,
        mut snapshot: DeliverySnapshot,
        terminal: bool,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let source = self.inner.source(entry).ok_or_else(denied)?;
        let scope = ComputerRequest {
            computer_id: &computer.computer_id,
            request_id: &computer.request_id,
        };
        if snapshot.substrate.kind() != ProviderKind::Docker {
            return Err(denied());
        }
        {
            let mut records = entry.native.lock().map_err(|_| denied())?;
            // A new request loses the old request's suffix correlation. Its DB
            // uncertainty remains authoritative; deleting this record cannot
            // turn the new request into an admissible fresh mutation.
            if records
                .get(scope.computer_id)
                .is_some_and(|p| p.computer.request_id != scope.request_id)
            {
                records.remove(scope.computer_id);
            }
        }
        let mut progress = entry
            .native
            .lock()
            .map_err(|_| denied())?
            .get(scope.computer_id)
            .cloned();
        let deleting = snapshot.computer.desired_state == "DELETING";
        if progress.is_none() && snapshot.computer.current_phase != "UNCERTAIN" {
            if !deleting && !terminal && snapshot.computer.current_phase != "RETIRING" {
                return Err(denied());
            }
            {
                let mut records = entry.native.lock().map_err(|_| denied())?;
                if records.len() >= LOCAL_PENDING {
                    return Ok(reconciliation::outcome(wire::ForwardingOutcome::Overloaded));
                }
                // Reserve capacity before the CAS await. Cancellation loses
                // progress, not safety: Admitting never authorizes forwarding.
                records.insert(
                    scope.computer_id.into(),
                    NativeProgress {
                        computer: computer.clone(),
                        admission: None,
                        command: None,
                        state: DispatchState::Admitting,
                        delete: deleting,
                        inventory: inventory_fingerprint(&snapshot),
                        completed_prefix: false,
                    },
                );
            }
            let claim = self
                .inner
                .service
                .claim_native_retirement(
                    &source,
                    scope,
                    terminal.then_some(computer.execution_id.as_str()),
                )
                .await
                .map_err(status)?;
            let Some(admission) = claim else {
                return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
            };
            let mut records = entry.native.lock().map_err(|_| denied())?;
            let record = records.get_mut(scope.computer_id).ok_or_else(denied)?;
            record.admission = Some(admission);
            record.state = DispatchState::Completed; // No native stage submitted yet.
            progress = Some(record.clone());
        }

        // Observation is allowed even after correlation loss. It never submits
        // a Retire/Stop/Remove or proves a suffix was not previously submitted.
        let mut to_observe = if let Some(call) = progress.as_ref().and_then(|p| p.command.as_ref())
        {
            match call.operation.as_ref() {
                Some(wire::provider_command::Operation::Retire(value)) => {
                    value.docker_container.iter().cloned().collect::<Vec<_>>()
                }
                Some(wire::provider_command::Operation::Cleanup(value)) => value.resources.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        for expected in &mut to_observe {
            let decoded = crate::native_resources::decode(expected)?;
            let fresh = snapshot
                .resources
                .iter()
                .find(|resource| {
                    resource.native() == &decoded.native
                        && resource.identity() == decoded.identity.as_ref()
                        && resource.execution_id == decoded.execution_id
                })
                .ok_or_else(denied)?;
            *expected = crate::native_resources::encode(fresh);
        }
        for resource in snapshot
            .resources
            .iter()
            .filter(|r| !fresh_absent(r, &source))
        {
            let candidate = crate::native_resources::encode(resource);
            if to_observe.len() >= RESOURCE_BATCH {
                break;
            }
            if !to_observe.contains(&candidate) {
                to_observe.push(candidate);
            }
        }
        if !to_observe.is_empty() {
            let observed = self
                .dispatch_convergence(
                    entry,
                    computer,
                    wire::provider_command::Operation::Observe(wire::ObserveResources {
                        resources: to_observe,
                    }),
                    deadline,
                )
                .await?;
            if observed.outcome != wire::ForwardingOutcome::Completed as i32
                || observed
                    .result
                    .as_ref()
                    .is_none_or(|result| result.outcome != wire::CommandOutcome::Completed as i32)
            {
                return Ok(observed);
            }
        }
        snapshot = self
            .inner
            .service
            .provider_delivery_snapshot(&source, scope, None)
            .await
            .map_err(status)?;
        if let Some(current) = snapshot.computer.current_execution_id.as_deref()
            && snapshot.resources.iter().any(|r| {
                r.kind() == ManagedResourceKind::DockerContainer
                    && r.execution_id.as_deref() == Some(current)
                    && fresh_absent(r, &source)
            })
        {
            self.inner
                .service
                .retire_execution(&source, scope, current)
                .await
                .map_err(status)?;
            snapshot = self
                .inner
                .service
                .provider_delivery_snapshot(&source, scope, None)
                .await
                .map_err(status)?;
        }
        if deleting
            && snapshot.computer.current_execution_id.is_none()
            && snapshot.resources.iter().all(|r| fresh_absent(r, &source))
        {
            self.inner
                .service
                .confirm_computer_cleanup(&source, scope)
                .await
                .map_err(status)?;
            entry
                .native
                .lock()
                .map_err(|_| denied())?
                .remove(scope.computer_id);
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Completed));
        }
        if !deleting && snapshot.computer.current_execution_id.is_none() {
            entry
                .native
                .lock()
                .map_err(|_| denied())?
                .remove(scope.computer_id);
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Completed));
        }
        let progress = entry
            .native
            .lock()
            .map_err(|_| denied())?
            .get(scope.computer_id)
            .cloned();
        let Some(mut progress) = progress else {
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
        };
        if progress.inventory != inventory_fingerprint(&snapshot) {
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
        }
        if progress.state == DispatchState::NotStarted && !progress.completed_prefix {
            let admission = progress.admission.as_ref().ok_or_else(denied)?;
            self.inner
                .service
                .release_unstarted_native_retirement(&source, admission)
                .await
                .map_err(status)?;
            entry
                .native
                .lock()
                .map_err(|_| denied())?
                .remove(scope.computer_id);
            return Ok(reconciliation::outcome(
                wire::ForwardingOutcome::Unavailable,
            ));
        }
        if !matches!(
            progress.state,
            DispatchState::Completed | DispatchState::NotStarted
        ) {
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
        }
        if let Some(previous) = &progress.command {
            let targets = match previous.operation.as_ref() {
                Some(wire::provider_command::Operation::Retire(value)) => {
                    value.docker_container.iter().collect::<Vec<_>>()
                }
                Some(wire::provider_command::Operation::Cleanup(value)) => {
                    value.resources.iter().collect()
                }
                _ => return Err(denied()),
            };
            if progress.state == DispatchState::Completed {
                for expected in targets {
                    let expected = crate::native_resources::decode(expected)?;
                    if !snapshot.resources.iter().any(|r| {
                        r.native() == &expected.native
                            && r.identity() == expected.identity.as_ref()
                            && r.execution_id == expected.execution_id
                            && fresh_absent(r, &source)
                    }) {
                        return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
                    }
                }
            }
            if progress.state == DispatchState::Completed {
                progress.completed_prefix = true;
            }
        }
        let operation = if let Some(current) = snapshot.computer.current_execution_id.as_deref() {
            let resource = snapshot
                .resources
                .iter()
                .find(|r| {
                    r.kind() == ManagedResourceKind::DockerContainer
                        && r.execution_id.as_deref() == Some(current)
                })
                .ok_or_else(denied)?;
            wire::provider_command::Operation::Retire(wire::RetireExecution {
                pod: None,
                docker_container: Some(crate::native_resources::encode(resource)),
                require_terminal: terminal,
            })
        } else if progress.delete {
            // All historical execution resources must first refresh their
            // already-qualified absence. Never replay retirement of history.
            if snapshot.resources.iter().any(|r| {
                r.kind() == ManagedResourceKind::DockerContainer && !fresh_absent(r, &source)
            }) {
                return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
            }
            self.inner
                .service
                .verify_native_cleanup_suffix(&source, scope)
                .await
                .map_err(status)?;
            let volume = snapshot
                .resources
                .iter()
                .find(|r| r.kind() == ManagedResourceKind::DockerVolume)
                .ok_or_else(denied)?;
            wire::provider_command::Operation::Cleanup(wire::CleanupResources {
                resources: vec![crate::native_resources::encode(volume)],
                confirm_complete: true,
            })
        } else {
            return Ok(reconciliation::outcome(wire::ForwardingOutcome::Uncertain));
        };
        let command = wire::ProviderCommand {
            identity: Some(wire::CallIdentity {
                request_id: scope.request_id.into(),
                call_id: reconciliation::call_id()?,
                connection_id: source.connection_id.into(),
            }),
            computer: Some(wire::ComputerContext {
                computer_id: scope.computer_id.into(),
                request_id: scope.request_id.into(),
                execution_id: snapshot
                    .computer
                    .current_execution_id
                    .clone()
                    .unwrap_or_default(),
            }),
            remaining_ms: deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                .min(10_000) as u32,
            operation: Some(operation),
        };
        progress.command = Some(command.clone());
        progress.state = DispatchState::Ready;
        entry
            .native
            .lock()
            .map_err(|_| denied())?
            .insert(scope.computer_id.into(), progress);
        self.dispatch_entry(entry, command, deadline).await
    }
}
