//! Bounded connection-local cleanup correlation, never effect replay authority.
use super::*;

type ResourceKey = (u8, String, String);
fn key(value: &wire::ManagedResource) -> ResourceKey {
    (
        match wire::ResourceKind::try_from(value.kind) {
            Ok(wire::ResourceKind::Pod) => 0,
            Ok(wire::ResourceKind::BootstrapSecret) => 1,
            _ => 2,
        },
        value.namespace.clone(),
        value.name.clone(),
    )
}
fn absent(value: &ManagedResource, source: &ProviderSource<'_>) -> bool {
    value.presence == "ABSENT"
        && value.source_connection_id.as_deref() == Some(source.connection_id)
}
fn completed(result: &wire::ProviderInvocationResult) -> bool {
    result.outcome == wire::ForwardingOutcome::Completed as i32
        && result
            .result
            .as_ref()
            .is_some_and(|r| r.outcome == wire::CommandOutcome::Completed as i32)
}
impl ComponentConnectionServices {
    pub(super) async fn prune_cleanup(
        &self,
        entry: &Connection,
        source: &ProviderSource<'_>,
    ) -> Result<(), Status> {
        let pending = entry
            .cleanup
            .lock()
            .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for progress in pending {
            let result = self
                .inner
                .service
                .provider_delivery_snapshot(
                    source,
                    ComputerRequest {
                        computer_id: &progress.computer.computer_id,
                        request_id: &progress.computer.request_id,
                    },
                    None,
                )
                .await;
            let remove = match result {
                Ok(snapshot) => snapshot.computer.cleanup_completed_at.is_some(),
                Err(error) => matches!(
                    error.reason(),
                    chapchoo_service::DomainReason::NotFound
                        | chapchoo_service::DomainReason::Conflict
                ),
            };
            if remove {
                let mut records = entry
                    .cleanup
                    .lock()
                    .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?;
                if records
                    .get(&progress.computer.computer_id)
                    .is_some_and(|current| current.computer == progress.computer)
                {
                    records.remove(&progress.computer.computer_id);
                }
            }
        }
        Ok(())
    }
    pub(super) async fn cleanup_provider(
        &self,
        entry: &Connection,
        computer: &wire::ComputerContext,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let source = self
            .inner
            .source(entry)
            .ok_or_else(|| Status::permission_denied("Provider source is required"))?;
        let scope = ComputerRequest {
            computer_id: &computer.computer_id,
            request_id: &computer.request_id,
        };
        let mut snapshot = self
            .inner
            .service
            .provider_delivery_snapshot(&source, scope, None)
            .await
            .map_err(status)?;
        let progress = {
            let mut progress = entry
                .cleanup
                .lock()
                .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?;
            if progress
                .get(&computer.computer_id)
                .is_some_and(|p| p.computer != *computer)
            {
                progress.remove(&computer.computer_id);
            }
            progress.get(&computer.computer_id).cloned()
        };
        if let Some(progress) = &progress
            && !progress.not_started
        {
            // A retired response waiter does not retire its SDK task. Observe
            // this exact accepted batch, including historical bootstrap Secrets.
            // Only committed same-source absence allows skipping its effects.
            let batch = progress
                .batch
                .iter()
                .map(|expected| {
                    snapshot
                        .resources
                        .iter()
                        .map(resource)
                        .find(|current| {
                            current.kind == expected.kind
                                && current.namespace == expected.namespace
                                && current.name == expected.name
                                && current.uid == expected.uid
                                && current.execution_id == expected.execution_id
                        })
                        .ok_or_else(|| {
                            Status::failed_precondition("Cleanup batch identity changed")
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let observed = self
                .dispatch_convergence(
                    entry,
                    computer,
                    wire::provider_command::Operation::Observe(wire::ObserveResources {
                        resources: batch,
                    }),
                    deadline,
                )
                .await?;
            if !completed(&observed) {
                return Ok(observed);
            }
            snapshot = self
                .inner
                .service
                .provider_delivery_snapshot(&source, scope, None)
                .await
                .map_err(status)?;
            if progress.batch.iter().any(|expected| {
                !snapshot
                    .resources
                    .iter()
                    .any(|r| key(&resource(r)) == key(expected) && absent(r, &source))
            }) {
                return Ok(outcome(wire::ForwardingOutcome::Uncertain));
            }
        }
        if snapshot.resources.iter().all(|r| absent(r, &source)) {
            self.inner
                .service
                .confirm_computer_cleanup(&source, scope)
                .await
                .map_err(status)?;
            entry
                .cleanup
                .lock()
                .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?
                .remove(&computer.computer_id);
            return Ok(outcome(wire::ForwardingOutcome::Completed));
        }
        if snapshot.computer.current_phase == "UNCERTAIN" && progress.is_none() {
            // Losing this transient manifest is not permission to reconstruct or
            // replay a possibly accepted DELETE. Durable absence can still finish
            // complete cleanup; otherwise unknown work remains unknown.
            return Ok(outcome(wire::ForwardingOutcome::Uncertain));
        }
        let mut resources = snapshot.resources.iter().map(resource).collect::<Vec<_>>();
        resources.sort_by_key(key);
        if let Some(progress) = &progress {
            let boundary = if progress.not_started {
                progress.batch.first()
            } else {
                progress.batch.last()
            }
            .ok_or_else(|| Status::internal("Cleanup batch is empty"))?;
            let boundary = key(boundary);
            // Completed preceding batches must retain exact current-source
            // absence. Remaining work is known never-submitted by this owner.
            let precedes = |r: &wire::ManagedResource| {
                if progress.not_started {
                    key(r) < boundary
                } else {
                    key(r) <= boundary
                }
            };
            if snapshot
                .resources
                .iter()
                .any(|r| precedes(&resource(r)) && !absent(r, &source))
            {
                return Ok(outcome(wire::ForwardingOutcome::Uncertain));
            }
            resources.retain(|r| !precedes(r));
        }
        if resources.is_empty() {
            return Ok(outcome(wire::ForwardingOutcome::Uncertain));
        }
        {
            let progress = entry
                .cleanup
                .lock()
                .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?;
            if !progress.contains_key(&computer.computer_id) && progress.len() >= LOCAL_PENDING {
                return Ok(outcome(wire::ForwardingOutcome::Overloaded));
            }
        }
        self.inner
            .service
            .report_lifecycle_issue(
                &source,
                scope,
                snapshot.computer.current_execution_id.as_deref(),
                chapchoo_domain::LifecyclePhase::Uncertain,
            )
            .await
            .map_err(status)?;
        let batches = resources.chunks(RESOURCE_BATCH);
        let count = batches.len();
        for (index, batch) in batches.enumerate() {
            // Capture the single bounded potentially accepted batch before any
            // await can retire the caller. Never store or re-enqueue old commands.
            entry
                .cleanup
                .lock()
                .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?
                .insert(
                    computer.computer_id.clone(),
                    CleanupProgress {
                        computer: computer.clone(),
                        batch: batch.to_vec(),
                        not_started: false,
                    },
                );
            let result = self
                .dispatch_convergence(
                    entry,
                    computer,
                    wire::provider_command::Operation::Cleanup(wire::CleanupResources {
                        resources: batch.to_vec(),
                        confirm_complete: index + 1 == count,
                    }),
                    deadline,
                )
                .await;
            if not_started(&result)
                && let Some(progress) = entry
                    .cleanup
                    .lock()
                    .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?
                    .get_mut(&computer.computer_id)
            {
                progress.not_started = true;
            }
            let result = result?;
            if !completed(&result) {
                return Ok(result);
            }
        }
        entry
            .cleanup
            .lock()
            .map_err(|_| Status::unavailable("Cleanup correlation unavailable"))?
            .remove(&computer.computer_id);
        Ok(outcome(wire::ForwardingOutcome::Completed))
    }
}
