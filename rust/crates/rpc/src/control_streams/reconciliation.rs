//! Connection-scoped convergence reconstructs current intent, not old commands.
use super::*;
use crate::PeerDelivery;
use chapchoo_service::{ComputerRequest, ManagedComputer, ManagedResource};

fn resource(value: &ManagedResource) -> wire::ManagedResource {
    wire::ManagedResource {
        kind: match value.resource_kind.as_str() {
            "POD" => wire::ResourceKind::Pod,
            "PVC" => wire::ResourceKind::Pvc,
            _ => wire::ResourceKind::BootstrapSecret,
        } as i32,
        namespace: value.namespace.clone(),
        name: value.name.clone(),
        uid: value.resource_uid.clone(),
        execution_id: value.execution_id.clone(),
        absence_confirmed: value.absence_confirmed_at.is_some(),
    }
}
fn context(value: &ManagedComputer) -> wire::ComputerContext {
    wire::ComputerContext {
        computer_id: value.id.clone(),
        request_id: value.current_request_id.clone(),
        execution_id: value
            .current_execution_id
            .clone()
            .or_else(|| value.expected_old_execution_id.clone())
            .unwrap_or_default(),
    }
}
fn outcome(outcome: wire::ForwardingOutcome) -> wire::ProviderInvocationResult {
    wire::ProviderInvocationResult {
        outcome: outcome as i32,
        result: None,
    }
}
fn call_id() -> Result<String, Status> {
    Ok(chapchoo_crypto::BearerSecret::generate()
        .map_err(|_| Status::internal("Call identity generation failed"))?
        .digest()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
impl ComponentConnectionServices {
    /// Bound each pass to current admitted streams and one bounded Provider page.
    /// Each subsequent invocation still revalidates exact live authority.
    pub async fn reconcile_once(&self, delivery: &PeerDelivery) -> Result<(), Status> {
        self.inner.admit()?;
        let (providers, connectors) = {
            let entries = self
                .inner
                .entries
                .lock()
                .map_err(|_| Status::unavailable("Control registry unavailable"))?;
            (
                entries.providers.values().cloned().collect::<Vec<_>>(),
                entries.connectors.values().cloned().collect::<Vec<_>>(),
            )
        };
        for entry in connectors {
            if self.inner.verify(&entry).await.is_err() {
                continue;
            }
            let Target::Connector { .. } = &entry.target else {
                continue;
            };
            // The prepared target is private; current session authority comes
            // from the authenticated stream's service-owned preparation.
            let identity = self
                .inner
                .service
                .connector_identity_for_prepared(&entry.prepared)
                .await;
            let Ok(row) = identity else {
                continue;
            };
            let computer = context(&row);
            let snapshot = match self
                .inner
                .service
                .delivery_snapshot(
                    None,
                    ComputerRequest {
                        computer_id: &row.id,
                        request_id: &row.current_request_id,
                    },
                    None,
                )
                .await
            {
                Ok(snapshot) => snapshot,
                Err(_) => continue,
            };
            let authority = wire::PeerAuthority {
                context: Some(wire::peer_authority::Context::Reconciliation(
                    wire::ReconciliationAuthority {
                        computer: Some(computer.clone()),
                        owner: Some(wire::ExpectedOwner {
                            instance_id: self.inner.instance_id.clone(),
                            connection_id: entry.prepared.owner().connection_id.clone(),
                        }),
                    },
                )),
            };
            let _ = delivery
                .converge(
                    wire::ProviderConvergence {
                        provider_id: snapshot.computer.provider_id,
                        owner: None,
                        authority: Some(authority),
                        computer: Some(computer),
                        remaining_ms: 10_000,
                    },
                    Instant::now() + RESPONSE_BUDGET,
                )
                .await;
        }
        for entry in providers {
            if self.inner.verify(&entry).await.is_err() {
                continue;
            }
            let Some(source) = self.inner.source(&entry) else {
                continue;
            };
            let after = entry
                .cursor
                .lock()
                .map_err(|_| Status::unavailable("Reconciliation cursor unavailable"))?
                .clone();
            let rows = match self
                .inner
                .service
                .managed_computers(&source, after.as_deref(), 20)
                .await
            {
                Ok(rows) => rows,
                Err(_) => continue,
            };
            *entry
                .cursor
                .lock()
                .map_err(|_| Status::unavailable("Reconciliation cursor unavailable"))? =
                if rows.len() == 20 {
                    rows.last().map(|row| row.id.clone())
                } else {
                    None
                };
            for row in rows {
                if row.cleanup_completed_at.is_some() {
                    continue;
                }
                if !matches!(
                    self.inner.service.resolve_connector_route(&row.id).await,
                    Ok(None)
                ) {
                    continue;
                }
                let owner = wire::ExpectedOwner {
                    instance_id: self.inner.instance_id.clone(),
                    connection_id: entry.prepared.owner().connection_id.clone(),
                };
                let Target::Provider(id) = &entry.target else {
                    continue;
                };
                let _ = self
                    .converge_provider(id, &owner, &context(&row), Instant::now() + RESPONSE_BUDGET)
                    .await;
            }
            let _ = self
                .inner
                .service
                .collect_completed_computers(&source, 20)
                .await;
        }
        Ok(())
    }
    pub async fn converge_provider(
        &self,
        provider: &str,
        owner: &wire::ExpectedOwner,
        computer: &wire::ComputerContext,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        self.inner.admit()?;
        let entry = {
            self.inner
                .entries
                .lock()
                .map_err(|_| Status::unavailable("Control registry unavailable"))?
                .providers
                .get(provider)
                .cloned()
        };
        let Some(entry) = entry.filter(|e| {
            owner.instance_id == self.inner.instance_id
                && e.prepared.owner().connection_id == owner.connection_id
                && !e.calls.is_closed()
        }) else {
            return Ok(outcome(wire::ForwardingOutcome::NotOwner));
        };
        let Ok(_permit) = entry.convergence.try_acquire() else {
            return Ok(outcome(wire::ForwardingOutcome::Overloaded));
        };
        self.inner.verify(&entry).await?;
        let Some(policy) = &entry.policy else {
            return Err(Status::failed_precondition("Provider policy is missing"));
        };
        let source = self
            .inner
            .source(&entry)
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
        if snapshot.computer.cleanup_completed_at.is_some() {
            return Ok(outcome(wire::ForwardingOutcome::Completed));
        }
        let mut terminal = false;
        if !snapshot.resources.is_empty() {
            let current = wire::ComputerContext {
                execution_id: snapshot
                    .computer
                    .current_execution_id
                    .clone()
                    .or_else(|| snapshot.computer.expected_old_execution_id.clone())
                    .unwrap_or_default(),
                ..computer.clone()
            };
            let result = self
                .dispatch_convergence(
                    &entry,
                    &current,
                    wire::provider_command::Operation::Observe(wire::ObserveResources {
                        resources: snapshot
                            .resources
                            .iter()
                            .filter(|r| {
                                r.resource_kind == "PVC"
                                    || r.execution_id == snapshot.computer.current_execution_id
                                    || (r.resource_kind == "POD"
                                        && r.execution_id
                                            == snapshot.computer.expected_old_execution_id)
                            })
                            .take(RESOURCE_BATCH)
                            .map(resource)
                            .collect(),
                    }),
                    deadline,
                )
                .await?;
            if result.outcome != wire::ForwardingOutcome::Completed as i32
                || result
                    .result
                    .as_ref()
                    .is_none_or(|r| r.outcome != wire::CommandOutcome::Completed as i32)
            {
                return Ok(result);
            }
            terminal = result.result.as_ref().is_some_and(|r| r.execution_terminal);
            snapshot = self
                .inner
                .service
                .provider_delivery_snapshot(&source, scope, None)
                .await
                .map_err(status)?;
        }
        let state = &snapshot.computer;
        let current = wire::ComputerContext {
            execution_id: state
                .current_execution_id
                .clone()
                .or_else(|| state.expected_old_execution_id.clone())
                .unwrap_or_default(),
            ..computer.clone()
        };
        if terminal && state.desired_state == "RUNNING" && state.current_execution_id.is_some() {
            let pod = snapshot
                .resources
                .iter()
                .find(|r| r.resource_kind == "POD" && r.execution_id == state.current_execution_id)
                .ok_or_else(|| Status::failed_precondition("Terminal execution Pod missing"))?;
            self.inner
                .service
                .report_lifecycle_issue(
                    &source,
                    scope,
                    state.current_execution_id.as_deref(),
                    chapchoo_domain::LifecyclePhase::Uncertain,
                )
                .await
                .map_err(status)?;
            return self
                .dispatch_convergence(
                    &entry,
                    &current,
                    wire::provider_command::Operation::Retire(wire::RetireExecution {
                        pod: Some(resource(pod)),
                        require_terminal: true,
                    }),
                    deadline,
                )
                .await;
        }
        if state.desired_state == "DELETING" {
            if snapshot.resources.is_empty() {
                self.inner
                    .service
                    .confirm_computer_cleanup(&source, scope)
                    .await
                    .map_err(status)?;
                return Ok(outcome(wire::ForwardingOutcome::Completed));
            }
            if snapshot.resources.iter().all(|r| {
                r.presence == "ABSENT"
                    && r.source_connection_id.as_deref() == Some(source.connection_id)
            }) {
                self.inner
                    .service
                    .confirm_computer_cleanup(&source, scope)
                    .await
                    .map_err(status)?;
                return Ok(outcome(wire::ForwardingOutcome::Completed));
            }
            if state.current_phase == "UNCERTAIN" {
                return Ok(outcome(wire::ForwardingOutcome::Uncertain));
            }
            self.inner
                .service
                .report_lifecycle_issue(
                    &source,
                    scope,
                    state.current_execution_id.as_deref(),
                    chapchoo_domain::LifecyclePhase::Uncertain,
                )
                .await
                .map_err(status)?;
            let mut resources = snapshot.resources.iter().collect::<Vec<_>>();
            resources.sort_by_key(|r| match r.resource_kind.as_str() {
                "POD" => 0,
                "BOOTSTRAP_SECRET" => 1,
                _ => 2,
            });
            let batches = resources.chunks(RESOURCE_BATCH);
            let count = batches.len();
            for (index, batch) in batches.enumerate() {
                let result = self
                    .dispatch_convergence(
                        &entry,
                        &current,
                        wire::provider_command::Operation::Cleanup(wire::CleanupResources {
                            resources: batch.iter().map(|r| resource(r)).collect(),
                            confirm_complete: index + 1 == count,
                        }),
                        deadline,
                    )
                    .await?;
                if result.outcome != wire::ForwardingOutcome::Completed as i32
                    || result
                        .result
                        .as_ref()
                        .is_none_or(|r| r.outcome != wire::CommandOutcome::Completed as i32)
                {
                    return Ok(result);
                }
            }
            return Ok(outcome(wire::ForwardingOutcome::Completed));
        }
        if state.current_phase == "RETIRING" {
            let pod = snapshot
                .resources
                .iter()
                .find(|r| r.resource_kind == "POD" && r.execution_id == state.current_execution_id)
                .ok_or_else(|| Status::failed_precondition("Current execution Pod is missing"))?;
            self.inner
                .service
                .report_lifecycle_issue(
                    &source,
                    scope,
                    state.current_execution_id.as_deref(),
                    chapchoo_domain::LifecyclePhase::Uncertain,
                )
                .await
                .map_err(status)?;
            return self
                .dispatch_convergence(
                    &entry,
                    &current,
                    wire::provider_command::Operation::Retire(wire::RetireExecution {
                        pod: Some(resource(pod)),
                        require_terminal: false,
                    }),
                    deadline,
                )
                .await;
        }
        if state.desired_state != "RUNNING"
            || matches!(state.current_phase.as_str(), "UNCERTAIN" | "COMPLETED")
            || (state.current_phase == "BLOCKED" && state.current_execution_id.is_some())
        {
            return Ok(outcome(wire::ForwardingOutcome::Completed));
        }
        if snapshot.computer.prepare_admitted_at.is_some()
            && snapshot.computer.current_execution_id.is_some()
        {
            return Ok(outcome(wire::ForwardingOutcome::Uncertain));
        }
        let reservation = self
            .inner
            .service
            .reserve_execution(&source, scope, &policy.namespace)
            .await
            .map_err(status)?;
        let session = self
            .inner
            .service
            .issue_connector_session_from_source(
                &source,
                &reservation.execution_id,
                &format!("bootstrap-{}", reservation.execution_id),
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let find = |kind: &str| {
            reservation
                .resources
                .iter()
                .find(|r| {
                    r.resource_kind == kind
                        && (kind == "PVC"
                            || r.execution_id.as_deref() == Some(&reservation.execution_id))
                })
                .map(resource)
                .ok_or_else(|| Status::failed_precondition("Execution reservation incomplete"))
        };
        let command = wire::PrepareExecution {
            initial_allocation: reservation.initial_allocation,
            saved_pvc: Some(find("PVC")?),
            bootstrap_secret: Some(find("BOOTSTRAP_SECRET")?),
            pod: Some(find("POD")?),
            connector_session_id: session.id,
            connector_initial_key: session.initial_key.expose().into(),
            connector_endpoint: policy.connector_endpoint.clone(),
        };
        if !self
            .inner
            .service
            .claim_execution_dispatch(&source, scope, &reservation.execution_id)
            .await
            .map_err(status)?
        {
            return Ok(outcome(wire::ForwardingOutcome::Uncertain));
        }
        let current = wire::ComputerContext {
            execution_id: reservation.execution_id,
            ..computer.clone()
        };
        self.dispatch_convergence(
            &entry,
            &current,
            wire::provider_command::Operation::Prepare(command),
            deadline,
        )
        .await
    }
    async fn dispatch_convergence(
        &self,
        entry: &Connection,
        computer: &wire::ComputerContext,
        operation: wire::provider_command::Operation,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        self.inner.verify(entry).await?;
        let remaining_ms = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(10_000) as u32;
        if remaining_ms == 0 {
            return Ok(outcome(wire::ForwardingOutcome::Unavailable));
        }
        let command = wire::ProviderCommand {
            identity: Some(wire::CallIdentity {
                request_id: computer.request_id.clone(),
                call_id: call_id()?,
                connection_id: entry.prepared.owner().connection_id.clone(),
            }),
            computer: Some(computer.clone()),
            remaining_ms,
            operation: Some(operation),
        };
        match entry
            .calls
            .invoke(Command::Provider(Box::new(command)), deadline)
            .await
        {
            Ok(Reply::Provider(result)) => Ok(wire::ProviderInvocationResult {
                outcome: wire::ForwardingOutcome::Completed as i32,
                result: Some(*result),
            }),
            Ok(_) => Err(Status::internal("Provider reply kind mismatch")),
            Err(error) => Ok(outcome(super::outcome(error))),
        }
    }
}
