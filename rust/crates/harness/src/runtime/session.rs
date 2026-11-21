//! The owner speaks in native conversation operations, not ACP wire requests.
//!
//! Enqueue and acknowledgement are separate: callers publish under the owner
//! fence and await the resulting receipt after releasing it.
use super::*;
use std::future::Future;

pub(super) enum Session {
    Acp(agent_client_protocol::v2::ActiveSession<Agent>),
    AppServer(app_server::LiveThread),
}

impl Session {
    pub(super) fn from_acp(session: agent_client_protocol::v2::ActiveSession<Agent>) -> Self {
        Self::Acp(session)
    }

    pub(super) fn is_closed(&self) -> bool {
        match self {
            Self::Acp(session) => session.connection().is_incoming_closed(),
            Self::AppServer(thread) => thread.is_closed(),
        }
    }

    pub(super) fn id(&self) -> &str {
        match self {
            Self::Acp(session) => session.session_id().0.as_ref(),
            Self::AppServer(thread) => thread.id(),
        }
    }

    pub(super) async fn steer(
        &self,
        text: &ConversationText,
        route: Option<&ContinuationRoute>,
    ) -> Result<Option<NativePromptAcceptance>, Error> {
        match self {
            Self::Acp(session) => steering::steer(session, text, route).await,
            Self::AppServer(thread) => {
                let pending = match route {
                    Some(route) => route.publish(|| thread.steering(text))?,
                    None => thread.steering(text)?,
                };
                match pending {
                    Some(pending) => Ok(Some(NativePromptAcceptance::Inserted {
                        native_message_id: pending.await?,
                    })),
                    None => Ok(None),
                }
            }
        }
    }

    pub(super) fn enqueue_prompt(
        &self,
        text: &ConversationText,
        model: Option<&str>,
        tuning: &chapchoo_domain::ConversationModelTuning,
    ) -> Result<impl Future<Output = Result<String, Error>> + use<>, Error> {
        let (acp, direct) = match self {
            Self::Acp(session) => (Some(session.send_prompt(text.as_str())), None),
            Self::AppServer(thread) => (None, Some(thread.enqueue_prompt(text, model, tuning)?)),
        };
        Ok(async move {
            if let Some(pending) = direct {
                return pending.await;
            }
            let receipt = acp
                .ok_or(Error::Protocol)?
                .block_task()
                .await
                .map_err(|_| Error::DeliveryUncertain)?;
            Ok(receipt.message_id.to_string())
        })
    }

    pub(super) fn enqueue_model(
        &self,
        option: String,
        model: &str,
    ) -> Result<impl Future<Output = Result<NativeModelOptions, Error>> + use<>, Error> {
        let Self::Acp(session) = self else {
            return Err(Error::NotAdmitted);
        };
        let pending = session.set_config_option(
            option,
            schema::SessionConfigOptionValue::Id {
                value: model.to_owned().into(),
            },
        );
        Ok(async move {
            let reply = pending.block_task().await.map_err(|_| Error::Protocol)?;
            model_selection::options(&reply.config_options).ok_or(Error::Protocol)
        })
    }

    pub(super) fn stop(&self, target: &NativeWorkTarget) -> Result<(), Error> {
        match self {
            Self::Acp(session) => session.cancel_active_work().map_err(|_| Error::Protocol),
            Self::AppServer(thread) => thread.stop(&target.work_id),
        }
    }

    pub(super) async fn answer(
        &self,
        pending: &Requests,
        response: InteractiveResponse,
        exact: Option<&chapchoo_domain::SourcePosition>,
    ) -> Result<(), Error> {
        match self {
            Self::Acp(session) => {
                requests::respond_exact(pending, session.session_id(), response, exact).await
            }
            Self::AppServer(thread) => thread.respond(response, exact),
        }
    }

    pub(super) async fn close(&self) {
        match self {
            Self::Acp(session) => {
                let _ = session.cancel_active_work();
                let _ = tokio::time::timeout(Duration::from_secs(2), session.close().block_task())
                    .await;
            }
            Self::AppServer(thread) => thread.close().await,
        }
    }
}
