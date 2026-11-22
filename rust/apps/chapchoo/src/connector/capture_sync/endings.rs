//! Report only retained live evidence. Capture supplies corroboration, never
//! creates a report. Settle a started handoff independently of the report ACK.

use super::*;
use chapchoo_domain::ConversationText;
use chapchoo_harness::{
    Error, NativeAuthenticationBinding, NativeModelDispatch, NativeNormalEnding,
};
use std::future::Future;

#[cfg(test)]
mod tests;

/// Physical owner operations, independent of the Control transport.
pub(super) trait EndingOwner {
    fn ending(&self) -> impl Future<Output = Result<Option<NativeNormalEnding>, Error>> + Send;
    fn binding(
        &self,
    ) -> impl Future<Output = Result<Option<NativeAuthenticationBinding>, Error>> + Send;
    fn decide(&self, ending: NativeNormalEnding) -> impl Future<Output = Result<(), Error>> + Send;
    fn submit(
        &self,
        ending: NativeNormalEnding,
        text: ConversationText,
        operation_id: String,
        models: NativeModelDispatch,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

impl EndingOwner for NativeHandle {
    async fn ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        self.pending_ending().await
    }
    async fn binding(&self) -> Result<Option<NativeAuthenticationBinding>, Error> {
        self.authentication_binding().await
    }
    async fn decide(&self, ending: NativeNormalEnding) -> Result<(), Error> {
        self.decide_ending(ending).await
    }
    async fn submit(
        &self,
        ending: NativeNormalEnding,
        text: ConversationText,
        operation_id: String,
        models: NativeModelDispatch,
    ) -> Result<(), Error> {
        self.continue_after_ending(ending, text, operation_id, models)
            .await
            .map(|_| ())
    }
}

pub(super) struct Delivery {
    closing: Option<Close>,
    ordering: super::super::server_tools::Ordering,
    models: super::super::models::Reports,
}
struct Close {
    ending: NativeNormalEnding,
    handoff: captured::ContinuationHandoff,
}

impl Delivery {
    pub(super) fn new(
        ordering: super::super::server_tools::Ordering,
        models: super::super::models::Reports,
    ) -> Self {
        Self {
            closing: None,
            ordering,
            models,
        }
    }

    fn client(channel: Channel) -> ConnectorCaptureClient<Channel> {
        ConnectorCaptureClient::new(channel)
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }

    pub(super) async fn advance(
        &mut self,
        native: &impl EndingOwner,
        channel: Channel,
        origin: &captured::CaptureSource,
        key: &BearerSecret,
        stop: &CancellationToken,
        route: chapchoo_harness::ContinuationRoute,
    ) -> Result<(), AppError> {
        let mut client = Self::client(channel.clone());
        if stop.is_cancelled() || !route.is_open() {
            return Ok(());
        }
        let Some(ending) = native.ending().await? else {
            return Ok(());
        };
        let Some(owner) = native.binding().await? else {
            return Ok(());
        };
        let Some(current) = client
            .current(request(
                captured::CurrentCaptureRequest {
                    source: Some(origin.clone()),
                },
                key,
            )?)
            .await?
            .into_inner()
            .conversation
        else {
            return Ok(());
        };
        let through = position(&ending.position);
        if current.status != captured::CaptureStatus::Ready as i32
            || owner.conversation_id != current.conversation_id
            || current.snapshot_id.as_deref() != Some(ending.work.snapshot.as_str())
            || current.native_session_id.as_deref() != Some(ending.work.native_session.as_str())
            || current.position.as_ref().is_none_or(|position| {
                position.ordinal < through.ordinal
                    || (position.ordinal == through.ordinal && position != &through)
            })
            || native.ending().await?.as_ref() != Some(&ending)
            || stop.is_cancelled()
            || self.ordering.input_report_pending(&owner)
        {
            return Ok(());
        }
        let target = captured::NativeTarget {
            snapshot_id: ending.work.snapshot.as_str().into(),
            native_session_id: ending.work.native_session.as_str().into(),
        };
        let occurrence = captured::NativeNormalEnding {
            position: Some(through),
            work_id: ending.work.work_id.clone(),
            launch_operation_id: owner.operation_id,
        };
        let outcome = client
            .report_native_hook(request(
                captured::NativeHookReport {
                    source: Some(origin.clone()),
                    conversation_id: current.conversation_id.clone(),
                    target: Some(target.clone()),
                    event: Some(captured::native_hook_report::Event::NormalEnding(
                        occurrence.clone(),
                    )),
                },
                key,
            )?)
            .await?
            .into_inner()
            .outcome;
        use captured::native_hook_outcome::Outcome;
        let operation_id = match outcome {
            Some(Outcome::NoAction(())) => return Ok(native.decide(ending).await?),
            Some(Outcome::ContinuationOperationId(operation_id)) => operation_id,
            None => return Err("Native hook decision is missing".into()),
        };
        let handoff = captured::ContinuationHandoff {
            source: Some(origin.clone()),
            conversation_id: current.conversation_id,
            target: Some(target),
            ending: Some(occurrence),
            operation_id: operation_id.clone(),
            guards_model_start: true,
        };
        if !route.is_open() {
            // Recover the admission only from the same live ending, without
            // starting the handoff on a withdrawn local route.
            return Ok(());
        }
        let grant = client
            .begin_continuation(request(handoff.clone(), key)?)
            .await?
            .into_inner();
        // Once Begin returns, never request another grant in order to retry an
        // effect. Retain only the exact order release across Close response loss.
        self.closing = Some(Close {
            ending: ending.clone(),
            handoff,
        });
        if let Some(context) = grant.context
            && !stop.is_cancelled()
        {
            let submitted = async {
                let text = continuation_text(context.projection)?;
                let models = self
                    .models
                    .dispatch(channel.clone(), origin.clone(), key, route)?;
                native.submit(ending, text, operation_id, models).await?;
                Ok::<_, AppError>(())
            }
            .await;
            if submitted.is_err() {
                tracing::warn!("Internal continuation did not confirm native acceptance");
            }
        }
        self.settle(native, channel, origin, key).await
    }

    pub(super) async fn settle(
        &mut self,
        native: &impl EndingOwner,
        channel: Channel,
        origin: &captured::CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        if let Some(close) = &self.closing {
            let handoff = captured::ContinuationHandoff {
                source: Some(origin.clone()),
                ..close.handoff.clone()
            };
            match Self::client(channel)
                .close_continuation(request(handoff, key)?)
                .await
            {
                Ok(_) => {}
                Err(status)
                    if matches!(
                        status.code(),
                        tonic::Code::AlreadyExists | tonic::Code::NotFound
                    ) =>
                {
                    // Old source or execution cannot contribute to this
                    // route's forwarding order.
                }
                Err(status) => return Err(status.into()),
            }
            native.decide(close.ending.clone()).await?;
            self.closing = None;
        }
        Ok(())
    }
}

fn continuation_text(projection: Vec<u8>) -> Result<ConversationText, AppError> {
    if projection.len() > 8 * 1024 {
        return Err("Continuation context is beyond the supported budget".into());
    }
    let projection = String::from_utf8(projection)?;
    let body = format!(
        "Continue active goals using the current authoritative goal context. \
        Leave goals waiting for user input on hold. \
        For Conversation Work, use the listed tasks and states; checking off \
        the tasks does not itself settle the work.\n{projection}"
    );
    super::super::system_input::render(&[("kind", "continuation")], &body)
}
