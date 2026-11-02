//! Shared stream/unary report admission. ACK means persisted exact-source evidence.
use crate::status::status;
use chapchoo_domain::LifecyclePhase;
use chapchoo_protocol::control as wire;
use chapchoo_service::{
    ComputerRequest, ComputersService, ManagedResourceKind, ProviderSource, ResourceEvidence,
    ResourceReport,
};
use tonic::Status;
pub(crate) async fn resource(
    service: &ComputersService,
    source: &ProviderSource<'_>,
    computer: &wire::ComputerContext,
    evidence: wire::ResourceEvidence,
) -> Result<(), Status> {
    let resource = evidence
        .resource
        .ok_or_else(|| Status::invalid_argument("Resource identity is required"))?;
    let kind = match wire::ResourceKind::try_from(resource.kind) {
        Ok(wire::ResourceKind::Pod) => ManagedResourceKind::Pod,
        Ok(wire::ResourceKind::Pvc) => ManagedResourceKind::Pvc,
        Ok(wire::ResourceKind::BootstrapSecret) => ManagedResourceKind::BootstrapSecret,
        _ => return Err(Status::invalid_argument("Resource kind is invalid")),
    };
    let value = evidence
        .evidence
        .ok_or_else(|| Status::invalid_argument("Resource evidence is required"))?;
    let evidence = match &value {
        wire::resource_evidence::Evidence::Present(value) => {
            ResourceEvidence::OwnedPresent { uid: &value.uid }
        }
        wire::resource_evidence::Evidence::Absent(value) => ResourceEvidence::DefinitivelyAbsent {
            expected_uid: value.expected_uid.as_deref(),
        },
        wire::resource_evidence::Evidence::Unknown(_) => ResourceEvidence::Unknown,
    };
    service
        .report_resource(
            source,
            ComputerRequest {
                computer_id: &computer.computer_id,
                request_id: &computer.request_id,
            },
            ResourceReport {
                kind,
                namespace: &resource.namespace,
                name: &resource.name,
                execution_id: resource.execution_id.as_deref(),
                evidence,
            },
        )
        .await
        .map_err(status)
}
pub(crate) async fn execution(
    service: &ComputersService,
    source: &ProviderSource<'_>,
    computer: &wire::ComputerContext,
    fact: i32,
) -> Result<(), Status> {
    let scope = ComputerRequest {
        computer_id: &computer.computer_id,
        request_id: &computer.request_id,
    };
    match wire::ExecutionFact::try_from(fact) {
        Ok(wire::ExecutionFact::Unspecified) => Ok(()),
        Ok(wire::ExecutionFact::Running) => {
            let snapshot = service
                .provider_delivery_snapshot(source, scope, Some(&computer.execution_id))
                .await
                .map_err(status)?;
            let state = snapshot.computer;
            // A running observation during accepted Stop/Restart/Delete is
            // substrate evidence, not permission to finish the new intent.
            if state.desired_state != "RUNNING"
                || state.current_phase == "RETIRING"
                || (state.current_action == "RESTART"
                    && state.expected_old_execution_id.as_deref() == Some(&computer.execution_id))
            {
                return Ok(());
            }
            service
                .report_execution_running(source, scope, &computer.execution_id)
                .await
                .map_err(status)
        }
        Ok(wire::ExecutionFact::Retired) => service
            .retire_execution(source, scope, &computer.execution_id)
            .await
            .map_err(status),
        Ok(wire::ExecutionFact::Blocked | wire::ExecutionFact::Uncertain) => service
            .report_lifecycle_issue(
                source,
                scope,
                (!computer.execution_id.is_empty()).then_some(computer.execution_id.as_str()),
                if fact == wire::ExecutionFact::Blocked as i32 {
                    LifecyclePhase::Blocked
                } else {
                    LifecyclePhase::Uncertain
                },
            )
            .await
            .map_err(status),
        Err(_) => Err(Status::invalid_argument("Execution fact is invalid")),
    }
}
