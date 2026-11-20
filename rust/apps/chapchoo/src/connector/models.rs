//! Connector-owned retained outcomes for model-governed starts.

use super::*;
use chapchoo_harness::{
    ContinuationRoute, Error, NativeModelAuthority, NativeModelDispatch, NativeModelFuture,
    NativeModelObservation, NativeModelReservation, NativeModelStart, NativeModelStartOutcome,
    NativeTextTarget,
};
use chapchoo_protocol::conversations::{
    self as wire, connector_capture_client::ConnectorCaptureClient,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub(super) struct Reports(
    Arc<Mutex<BTreeMap<String, (NativeModelStart, NativeModelStartOutcome)>>>,
);

struct Authority {
    reports: Reports,
    channel: Channel,
    origin: wire::CaptureSource,
    key: BearerSecret,
}

fn client(channel: Channel) -> ConnectorCaptureClient<Channel> {
    ConnectorCaptureClient::new(channel)
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024)
}
fn target(conversation: &str, launch: &str, native: &NativeTextTarget) -> wire::ModelTarget {
    wire::ModelTarget {
        conversation_id: conversation.into(),
        native: Some(wire::NativeTarget {
            snapshot_id: native.snapshot.as_str().into(),
            native_session_id: native.native_session.as_str().into(),
        }),
        launch_operation_id: launch.into(),
    }
}
fn start_target(start: &NativeModelStart) -> wire::ModelStartTarget {
    wire::ModelStartTarget {
        model_target: Some(target(
            &start.conversation_id,
            &start.launch_operation_id,
            &start.target,
        )),
        operation_id: start.operation_id.clone(),
    }
}
pub(super) async fn observe(
    client: &mut ConnectorCaptureClient<Channel>,
    origin: &wire::CaptureSource,
    key: &BearerSecret,
    conversation: &str,
    native: &NativeTextTarget,
    seen: NativeModelObservation,
) -> Result<(), AppError> {
    let (current, options) = seen.options.map_or((None, None), |option| {
        (
            option.current.map(|current| wire::ModelChoice {
                id: current.id,
                label: current.label,
            }),
            Some(wire::ModelOptions {
                choices: option
                    .options
                    .into_iter()
                    .map(|choice| wire::ModelChoice {
                        id: choice.id,
                        label: choice.label,
                    })
                    .collect(),
            }),
        )
    });
    client
        .report_model(request(
            wire::ModelObservationReport {
                source: Some(origin.clone()),
                target: Some(target(conversation, &seen.launch_operation_id, native)),
                order: seen.order,
                current,
                options,
                api_key_source: seen.api_key_source.map(|value| value.to_vec()),
            },
            key,
        )?)
        .await?;
    Ok(())
}

impl Reports {
    pub(super) fn dispatch(
        &self,
        channel: Channel,
        origin: wire::CaptureSource,
        key: &BearerSecret,
        route: ContinuationRoute,
    ) -> Result<NativeModelDispatch, AppError> {
        Ok(NativeModelDispatch {
            route,
            authority: Arc::new(Authority {
                reports: self.clone(),
                channel,
                origin,
                key: BearerSecret::parse(key.expose().to_owned())?,
            }),
        })
    }

    pub(super) async fn flush(
        &self,
        channel: Channel,
        origin: &wire::CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        let pending = self
            .0
            .lock()
            .map_err(|_| "Retained model results are unavailable")?
            .clone();
        let mut client = client(channel);
        for (id, (start, outcome)) in pending {
            use wire::model_start_report::Outcome;
            let report = match &outcome {
                NativeModelStartOutcome::Inserted(id) => Outcome::InsertedMessageId(id.clone()),
                NativeModelStartOutcome::NotPublished => Outcome::NotPublished(()),
                NativeModelStartOutcome::ModelUnavailableBeforeStart(model) => {
                    Outcome::ModelUnavailableBeforeStart(model.clone())
                }
            };
            let result = client
                .settle_model_start(request(
                    wire::ModelStartReport {
                        source: Some(origin.clone()),
                        target: Some(start_target(&start)),
                        outcome: Some(report),
                    },
                    key,
                )?)
                .await;
            // Control records an exact terminal non-publication tombstone even
            // if Reserve itself arrived late or its response was lost.
            if result.is_ok() {
                self.0
                    .lock()
                    .map_err(|_| "Retained model results are unavailable")?
                    .remove(&id);
            }
        }
        Ok(())
    }
}

impl NativeModelAuthority for Authority {
    fn reserve<'a>(
        &'a self,
        start: &'a NativeModelStart,
        observed: NativeModelObservation,
        capable: bool,
    ) -> NativeModelFuture<'a> {
        Box::pin(async move {
            let mut client = client(self.channel.clone());
            // A known outcome is flushed before trying another reservation;
            // transport loss can never clear a pending Control reservation.
            self.reports
                .flush(self.channel.clone(), &self.origin, &self.key)
                .await
                .map_err(|_| Error::DeliveryUncertain)?;
            observe(
                &mut client,
                &self.origin,
                &self.key,
                &start.conversation_id,
                &start.target,
                observed,
            )
            .await
            .map_err(|_| Error::NotAdmitted)?;
            let response = client
                .reserve_model_start(
                    request(
                        wire::ModelStartRequest {
                            source: Some(self.origin.clone()),
                            target: Some(start_target(start)),
                            capable,
                        },
                        &self.key,
                    )
                    .map_err(|_| Error::NotAdmitted)?,
                )
                .await
                .map_err(|_| Error::DeliveryUncertain)?
                .into_inner();
            match response.outcome {
                Some(wire::model_start_admission::Outcome::Ungoverned(())) => {
                    Ok(NativeModelReservation::Ungoverned)
                }
                Some(wire::model_start_admission::Outcome::Reserved(reserved)) => {
                    Ok(NativeModelReservation::Reserved(reserved.model_id))
                }
                None => Err(Error::Protocol),
            }
        })
    }

    fn retain(&self, start: &NativeModelStart, outcome: NativeModelStartOutcome) {
        if let Ok(mut reports) = self.reports.0.lock() {
            reports
                .entry(start.operation_id.clone())
                .or_insert_with(|| (start.clone(), outcome));
        }
    }
}
