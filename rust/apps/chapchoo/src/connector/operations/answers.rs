//! Pull a complete sealed answer before the native owner rechecks its responder.
use super::*;
use chapchoo_harness::{NativeAnswerObservation, NativeAnswerOperation};
use chapchoo_service::{ConversationAnswer, EncodedNativeAnswer, encode_native_answer};
impl Operations {
    pub(in crate::connector) fn answer(
        &mut self,
        command: c::AnswerCommand,
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
    ) -> Result<bool, AppError> {
        let permit = match self.admit(leases::Kind::Answer, &command.operation_id)? {
            Some(permit) => permit,
            None => return Ok(false),
        };
        let target = command
            .target
            .as_ref()
            .ok_or("Native answer target is required")?;
        let target = NativeTextTarget {
            snapshot: SnapshotId::parse(target.snapshot_id.clone())?,
            native_session: NativeConversationId::parse(target.native_session_id.clone())?,
        };
        let key = BearerSecret::parse(key.expose().to_owned())?;
        let native = self.native.clone();
        let payload = self.payload.clone();
        let stop = self.stop.clone();
        self.tasks.spawn(async move {
            let outcome = async {
                let mut client = ConnectorCaptureClient::new(channel)
                    .max_decoding_message_size(64 * 1024)
                    .max_encoding_message_size(64 * 1024);
                let mut bytes = Vec::new();
                let mut identity: Option<(u32, String, c::SourcePosition, i32)> = None;
                let mut budget = None;
                loop {
                    if stop.is_cancelled() {
                        return Err::<_, AppError>(
                            "Answer transfer stopped before native dispatch".into(),
                        );
                    }
                    let offset = u32::try_from(bytes.len())?;
                    let chunk = client
                        .pull_answer(request(
                            c::AnswerChunkRequest {
                                source: Some(origin.clone()),
                                command: Some(command.clone()),
                                offset,
                            },
                            &key,
                        )?)
                        .await?
                        .into_inner();
                    let occurrence = chunk.occurrence.ok_or("Answer occurrence is required")?;
                    let current = (
                        chunk.total_bytes,
                        chunk.responder_token,
                        occurrence,
                        chunk.kind,
                    );
                    if chunk.offset != offset
                        || current.0 == 0
                        || current.0 > 16 * 1024 * 1024
                        || chunk.content.len() > 32768
                        || offset
                            .checked_add(u32::try_from(chunk.content.len())?)
                            .is_none_or(|end| end > current.0)
                        || identity
                            .as_ref()
                            .is_some_and(|previous| previous != &current)
                    {
                        return Err("Answer transfer identity is inconsistent".into());
                    }
                    if identity.is_none() {
                        budget = Some(payload.clone().try_acquire_many_owned(current.0)?);
                        bytes.try_reserve_exact(current.0 as usize)?;
                        identity = Some(current.clone());
                    }
                    // The final empty suffix independently revalidates the exact live source.
                    if offset == current.0 {
                        if !chunk.content.is_empty() {
                            return Err("Answer suffix is inconsistent".into());
                        }
                        break;
                    }
                    if chunk.content.is_empty() {
                        return Err("Answer transfer ended early".into());
                    }
                    bytes.extend(chunk.content);
                }
                if stop.is_cancelled() {
                    return Err("Answer stopped before native dispatch".into());
                }
                let (_, token, occurrence, kind) = identity.ok_or("Answer identity is absent")?;
                let occurrence = chapchoo_domain::SourcePosition::new(
                    occurrence.ordinal,
                    chapchoo_domain::EventOccurrenceId::parse(occurrence.occurrence_id)?,
                )?;
                let answer: ConversationAnswer = serde_json::from_slice(&bytes)?;
                drop(bytes);
                let operation = match encode_native_answer(answer)? {
                    EncodedNativeAnswer::Permission(body)
                        if kind == c::RequestKind::Permission as i32 =>
                    {
                        NativeAnswerOperation::permission(
                            command.operation_id.clone(),
                            target,
                            occurrence,
                            token,
                            &body,
                        )?
                    }
                    EncodedNativeAnswer::Form(body) if kind == c::RequestKind::Form as i32 => {
                        NativeAnswerOperation::form(
                            command.operation_id.clone(),
                            target,
                            occurrence,
                            token,
                            &body,
                        )?
                    }
                    _ => return Err("Answer kind changed during transfer".into()),
                };
                let result = native.admit_answer(operation)?.await??;
                drop(budget);
                Ok(result)
            }
            .await;
            let accepted = match outcome {
                Ok(NativeAnswerObservation::ResponderAccepted) => true,
                Ok(NativeAnswerObservation::Rejected) => false,
                // Generic errors and a retained uncertain operation never grant replay.
                _ => return None,
            };
            Some((
                Report::Answer(c::AnswerReport {
                    source: None,
                    command: Some(command),
                    responder_accepted: accepted,
                }),
                permit,
            ))
        });
        Ok(true)
    }
}
