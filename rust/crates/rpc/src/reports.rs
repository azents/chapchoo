//! Shared stream/unary report admission. ACK means persisted exact-source evidence.
use crate::status::status;
use chapchoo_domain::LifecyclePhase;
use chapchoo_protocol::control as wire;
use chapchoo_service::{ComputerRequest, ComputersService, ProviderSource, ResourceReport};
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
    let native = crate::native_resources::decode(&resource)?;
    let value = evidence
        .evidence
        .ok_or_else(|| Status::invalid_argument("Resource evidence is required"))?;
    let evidence = crate::native_resources::evidence(&value, native.native.kind())?;
    if let Some(asserted) = &native.identity {
        let agrees = match &evidence {
            crate::native_resources::DecodedEvidence::Present(identity) => asserted == identity,
            crate::native_resources::DecodedEvidence::Absent(identity) => {
                Some(asserted) == identity.as_ref()
            }
            crate::native_resources::DecodedEvidence::Unknown => true,
        };
        if !agrees {
            return Err(Status::invalid_argument(
                "Native identity and evidence disagree",
            ));
        }
    }
    service
        .report_resource(
            source,
            ComputerRequest {
                computer_id: &computer.computer_id,
                request_id: &computer.request_id,
            },
            ResourceReport {
                native: &native.native,
                execution_id: native.execution_id.as_deref(),
                evidence: evidence.store(),
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
        Ok(wire::ExecutionFact::PreparationBlocked) => service
            .report_preparation_blocked(source, scope, &computer.execution_id)
            .await
            .map_err(status),
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
                || (matches!(state.current_action.as_str(), "STOP" | "RESTART")
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
