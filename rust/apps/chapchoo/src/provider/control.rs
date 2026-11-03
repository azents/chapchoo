//! Current-source Control RPC boundary shared by native substrate workers.
use super::*;
pub(super) struct Control {
    pub auth: ProviderAuthenticationClient<Channel>,
    pub source: wire::ProviderConnectionCheck,
    pub key: BearerSecret,
    pub sender: mpsc::Sender<wire::ProviderFrame>,
}
impl Control {
    pub async fn check(
        &mut self,
        command: &wire::ProviderCommand,
    ) -> Result<wire::ProviderCommandAdmission, AppError> {
        Ok(self
            .auth
            .check_command(request(
                wire::ProviderCommandCheck {
                    source: Some(self.source.clone()),
                    command: Some(command.clone()),
                },
                &self.key,
            )?)
            .await?
            .into_inner())
    }
    pub async fn report(
        &mut self,
        context: &wire::ComputerContext,
        resources: Vec<wire::ResourceEvidence>,
        fact: wire::ExecutionFact,
        cleanup: bool,
    ) -> Result<(), AppError> {
        self.auth
            .report(request(
                wire::ProviderReport {
                    source: Some(self.source.clone()),
                    computer: Some(context.clone()),
                    resources,
                    execution: fact as i32,
                    cleanup_complete: cleanup,
                },
                &self.key,
            )?)
            .await?;
        Ok(())
    }
    pub async fn reply(
        &self,
        command: &wire::ProviderCommand,
        outcome: wire::CommandOutcome,
        terminal: bool,
    ) -> Result<(), AppError> {
        self.sender.try_send(wire::ProviderFrame {
            payload: Some(wire::provider_frame::Payload::Result(
                wire::ProviderResult {
                    identity: command.identity.clone(),
                    computer: command.computer.clone(),
                    outcome: outcome as i32,
                    resources: vec![],
                    execution: wire::ExecutionFact::Unspecified as i32,
                    execution_terminal: terminal,
                },
            )),
        })?;
        Ok(())
    }
}
