//! The owner speaks in native conversation operations, not ACP wire requests.
//!
//! Enqueue and acknowledgement are separate: callers publish under the owner
//! fence and await the resulting receipt after releasing it.
use super::*;
use std::future::Future;

pub(super) struct Session(agent_client_protocol::v2::ActiveSession<Agent>);

impl Session {
    pub(super) fn from_acp(session: agent_client_protocol::v2::ActiveSession<Agent>) -> Self {
        Self(session)
    }

    pub(super) fn is_closed(&self) -> bool {
        self.0.connection().is_incoming_closed()
    }

    pub(super) fn id(&self) -> &str {
        self.0.session_id().0.as_ref()
    }

    pub(super) async fn steer(
        &self,
        text: &ConversationText,
        route: Option<&ContinuationRoute>,
    ) -> Result<Option<NativePromptAcceptance>, Error> {
        steering::steer(&self.0, text, route).await
    }

    pub(super) fn enqueue_prompt(
        &self,
        text: &ConversationText,
    ) -> impl Future<Output = Result<String, Error>> + use<> {
        let pending = self.0.send_prompt(text.as_str());
        async move {
            let receipt = pending
                .block_task()
                .await
                .map_err(|_| Error::DeliveryUncertain)?;
            Ok(receipt.message_id.to_string())
        }
    }

    pub(super) fn enqueue_model(
        &self,
        option: String,
        model: &str,
    ) -> impl Future<Output = Result<NativeModelOptions, Error>> + use<> {
        let pending = self.0.set_config_option(
            option,
            schema::SessionConfigOptionValue::Id {
                value: model.to_owned().into(),
            },
        );
        async move {
            let reply = pending.block_task().await.map_err(|_| Error::Protocol)?;
            model_selection::options(&reply.config_options).ok_or(Error::Protocol)
        }
    }

    pub(super) fn stop(&self) -> Result<(), Error> {
        self.0.cancel_active_work().map_err(|_| Error::Protocol)
    }

    pub(super) async fn answer(
        &self,
        pending: &Requests,
        response: InteractiveResponse,
    ) -> Result<(), Error> {
        requests::respond(pending, self.0.session_id(), response).await
    }

    pub(super) async fn close(&self) {
        let _ = self.stop();
        let _ = tokio::time::timeout(Duration::from_secs(2), self.0.close().block_task()).await;
    }
}
