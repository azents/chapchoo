//! Typed owner answer upload, independent processing and sealed component pulls.
use super::*;
use chapchoo_domain::{NativeConversationId, SnapshotId};
use chapchoo_persistence::{
    ConversationAnswer as Answer, ConversationAnswerForwardingAdmission,
    ConversationAnswerValue as Value, ConversationDeliveryTarget, ConversationFormAnswer as Form,
    ConversationOperationForwarding,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
    conversations_client::ConversationsClient,
};
use chapchoo_service::{
    AcceptedConversationAnswer, ConversationAnswerDraft, ConversationAnswerFuture,
    ConversationAnswerProcessor, ConversationAnswerService, ConversationService,
};
use prost::Message;
use std::{collections::BTreeMap, sync::atomic::AtomicUsize};
struct Processor {
    entered: Arc<tokio::sync::Semaphore>,
    release: Arc<tokio::sync::Semaphore>,
    calls: Arc<AtomicUsize>,
}
impl ConversationAnswerProcessor for Processor {
    fn answer(&self, answer: AcceptedConversationAnswer) -> ConversationAnswerFuture {
        let entered = self.entered.clone();
        let release = self.release.clone();
        let calls = self.calls.clone();
        Box::pin(async move {
            assert!(matches!(
                answer.receipt.answer,
                Answer::Form(Form::Accept { .. })
            ));
            calls.fetch_add(1, Ordering::SeqCst);
            entered.add_permits(1);
            release
                .acquire()
                .await
                .map_err(chapchoo_service::ServiceError::infrastructure)?
                .forget();
            Ok(())
        })
    }
}
pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
) -> Result<(), TestError> {
    let mut component = ConnectorCaptureClient::connect(context.endpoint.to_owned()).await?;
    let native = c::NativeTarget {
        snapshot_id: "rpc-source".into(),
        native_session_id: "native-fixture".into(),
    };
    let position = c::SourcePosition {
        ordinal: 3,
        occurrence_id: "form-question".into(),
    };
    let raw = br#"{"method":"elicitation/create","params":{"mode":"form","sessionId":"native-fixture","message":"Original question","requestedSchema":{"type":"object","properties":{"text":{"type":"string"},"enabled":{"type":"boolean"}},"required":["text","enabled"]}}}"#;
    component
        .append(authenticated(
            c::CapturePiece {
                source: Some(context.source.clone()),
                conversation_id: conversation.into(),
                snapshot_id: "rpc-source".into(),
                position: Some(position.clone()),
                transfer_id: "form-question-transfer".into(),
                total_bytes: u32::try_from(raw.len())?,
                offset: 0,
                content: raw.to_vec(),
            },
            context.key,
        )?)
        .await?;
    let live = c::LiveRequestReport {
        source: Some(context.source.clone()),
        conversation_id: conversation.into(),
        target: Some(native.clone()),
        position: Some(position.clone()),
        kind: c::RequestKind::Form as i32,
        responder_token: "live-form".into(),
        open: true,
    };
    component
        .report_request(authenticated(live.clone(), context.key)?)
        .await?;
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse("rpc-source".into())?,
        native_session: NativeConversationId::parse("native-fixture".into())?,
    };
    let text = "한글🦀 original answer\n".repeat(5000);
    let answer = Answer::Form(Form::Accept {
        content: Some(BTreeMap::from([
            ("text".into(), Value::Text(text.clone())),
            ("enabled".into(), Value::Boolean(false)),
        ])),
    });
    let entered = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let service = ConversationAnswerService::new(
        Arc::new(ConversationService::new(context.store.clone())),
        Arc::new(Processor {
            entered: entered.clone(),
            release: release.clone(),
            calls: calls.clone(),
        }),
    )?;
    support::scope(async |fixture| {
        let cleanup = service.clone();
        let stopped = fixture.shutdown();
        let release = release.clone();
        fixture.server(async move { let _ = stopped.await; release.add_permits(2); cleanup.finish().await });
        let accepted = service.accept(context.owner.clone(), ConversationAnswerDraft {
            agent_id: context.agent.id.clone(), conversation_id: conversation.into(), request_id: "typed-answer".into(),
            target: target.clone(), responder_token: "live-form".into(), answer: answer.clone(),
        }, AcceptanceTime::System).await?;
        entered.acquire().await?.forget();
        // Browser acceptance returned while the independent processor is still gated.
        let mut owner = ConversationsClient::connect(context.endpoint.to_owned()).await?
            .max_encoding_message_size(64 * 1024).max_decoding_message_size(64 * 1024);
        let encoded = c::TypedAnswer { answer: Some(c::typed_answer::Answer::FormAccept(c::FormAcceptance {
            content: Some(c::AnswerFields { fields: vec![
                c::AnswerField { name: "text".into(), value: Some(c::AnswerValue { value: Some(c::answer_value::Value::Text(text.clone())) }) },
                c::AnswerField { name: "enabled".into(), value: Some(c::AnswerValue { value: Some(c::answer_value::Value::Boolean(false)) }) },
            ] }),
        })) }.encode_to_vec();
        for (index, bytes) in encoded.chunks(32768).enumerate() {
            let response = owner.submit_answer(authenticated(c::OwnerAnswerPiece {
                agent_id: context.agent.id.clone(), conversation_id: conversation.into(), request_id: "typed-answer".into(),
                target: Some(native.clone()), responder_token: "live-form".into(), total_bytes: u32::try_from(encoded.len())?,
                offset: u32::try_from(index * 32768)?, content: bytes.to_vec(),
            }, context.user_key)?).await?.into_inner();
            if index * 32768 + bytes.len() == encoded.len() {
                assert!(matches!(response.outcome, Some(c::owner_answer_progress::Outcome::Accepted(value)) if value.id == accepted.id));
            }
        }
        let live_read = c::OwnerLiveRequests {
            agent_id: context.agent.id.clone(), conversation_id: conversation.into(), target: Some(native.clone()),
        };
        let requests = owner.live_requests(authenticated(live_read.clone(), context.user_key)?).await?.into_inner().requests;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].occurrence, Some(position.clone()));
        assert_eq!(requests[0].responder_token, "live-form");
        assert_eq!(requests[0].answer_request_id.as_deref(), Some("typed-answer"));
        let question_record = c::OwnerFieldsRequest {
            source: Some(c::RecordSource {
                agent_id: context.agent.id.clone(), conversation_id: conversation.into(), snapshot_id: "rpc-source".into(),
            }), kind: c::RecordKind::Request as i32, native_id: "form-question".into(),
        };
        let fields = owner.fields(authenticated(question_record.clone(), context.user_key)?).await?.into_inner().fields;
        let question_field = fields.iter().find(|field| field.field == c::RecordField::Request as i32).ok_or("Missing original question")?;
        let question = owner.read_field(authenticated(c::OwnerFieldRead {
            record: Some(question_record), field: question_field.field, updated_ordinal: question_field.updated_ordinal,
            offset: 0, length: 32768,
        }, context.user_key)?).await?.into_inner();
        let question = String::from_utf8(question.content)?;
        assert!(question.contains("\"requestedSchema\""));
        assert!(question.contains("\"enabled\""));
        assert!(question.contains("Original question"));
        assert!(!question.contains("\"method\""));
        let status = c::OwnerAnswerResultRequest {
            agent_id: context.agent.id.clone(), conversation_id: conversation.into(), request_id: "typed-answer".into(),
        };
        assert_eq!(owner.answer_result(authenticated(status.clone(), context.user_key)?).await?.into_inner()
            .acceptance.ok_or("Missing answer")?.delivery, c::AnswerDelivery::Accepted as i32);
        let route = context.store.resolve_connector_route(&context.agent.computer.id).await?.ok_or("Missing route")?;
        let draft = ConversationOperationForwarding {
            agent_id: &context.agent.id, conversation_id: conversation, operation_id: &accepted.id, target: &target,
        };
        assert!(matches!(context.store.begin_conversation_answer_forwarding(context.owner, &route, &draft, AcceptanceTime::System.now()?).await?,
            ConversationAnswerForwardingAdmission::Granted(_)));
        let command = c::AnswerCommand { agent_id: context.agent.id.clone(), conversation_id: conversation.into(), operation_id: accepted.id,
            target: Some(native.clone()) };
        let mut bytes = Vec::new();
        loop {
            let chunk = component.pull_answer(authenticated(c::AnswerChunkRequest {
                source: Some(context.source.clone()), command: Some(command.clone()), offset: u32::try_from(bytes.len())?,
            }, context.key)?).await?.into_inner();
            assert_eq!(chunk.responder_token, "live-form");
            assert_eq!(chunk.occurrence, Some(position.clone()));
            assert_eq!(chunk.kind, c::RequestKind::Form as i32);
            assert!(chunk.content.len() <= 32768);
            if chunk.content.is_empty() { assert_eq!(usize::try_from(chunk.total_bytes)?, bytes.len()); break; }
            bytes.extend(chunk.content);
        }
        // Exact stored original bytes are transferred, not an RPC history envelope.
        assert!(bytes.len() > 64 * 1024);
        let report = c::AnswerReport { source: Some(context.source.clone()), command: Some(command.clone()), responder_accepted: true };
        component.report_answer(authenticated(report.clone(), context.key)?).await?;
        component.report_answer(authenticated(report, context.key)?).await?;
        assert_eq!(owner.answer_result(authenticated(status.clone(), context.user_key)?).await?.into_inner()
            .acceptance.ok_or("Missing answer result")?.delivery, c::AnswerDelivery::ResponderAccepted as i32);
        assert_eq!(component.report_answer(authenticated(c::AnswerReport { source: Some(context.source.clone()), command: Some(command),
            responder_accepted: false }, context.key)?).await.err().ok_or("Changed answer result accepted")?.code(), Code::AlreadyExists);
        assert_eq!(owner.answer_result(authenticated(status, context.key)?).await.err().ok_or("Component gained owner answer read")?.code(), Code::Unauthenticated);
        component.report_request(authenticated(c::LiveRequestReport { open: false, ..live }, context.key)?).await?;
        assert!(owner.live_requests(authenticated(live_read, context.user_key)?).await?.into_inner().requests.is_empty());
        let retained = context.store.conversation_answer(context.owner, &context.agent.id, conversation, "typed-answer",
            AcceptanceTime::System.now()?).await?.ok_or("Missing original answer")?;
        assert_eq!(retained.answer, answer);
        let read = c::OwnerAnswerRead {
            source: Some(c::RecordSource {
                agent_id: context.agent.id.clone(), conversation_id: conversation.into(), snapshot_id: "rpc-source".into(),
            }), operation_id: retained.id, offset: 0, length: 32768,
        };
        let mut complete = Vec::new();
        loop {
            let chunk = owner.read_answer(authenticated(c::OwnerAnswerRead {
                offset: u32::try_from(complete.len())?, ..read.clone()
            }, context.user_key)?).await?.into_inner();
            if chunk.content.is_empty() { assert_eq!(usize::try_from(chunk.total_bytes)?, complete.len()); break; }
            assert!(chunk.content.len() <= 32768);
            complete.extend(chunk.content);
        }
        let decoded = c::TypedAnswer::decode(complete.as_slice())?;
        let Some(c::typed_answer::Answer::FormAccept(value)) = decoded.answer else { return Err("Missing retained typed form".into()); };
        let fields = value.content.ok_or("Missing retained fields")?.fields.into_iter()
            .map(|field| (field.name, field.value.and_then(|value| value.value))).collect::<BTreeMap<_,_>>();
        assert_eq!(fields.get("text"), Some(&Some(c::answer_value::Value::Text(text.clone()))));
        assert_eq!(fields.get("enabled"), Some(&Some(c::answer_value::Value::Boolean(false))));
        let mut stale = read.clone();
        stale.source.as_mut().ok_or("Missing source")?.snapshot_id = "replaced".into();
        assert_eq!(owner.read_answer(authenticated(stale, context.user_key)?).await.err().ok_or("Changed source read retained answer")?.code(), Code::AlreadyExists);
        assert_eq!(owner.read_answer(authenticated(read, context.key)?).await.err().ok_or("Component gained retained answer read")?.code(), Code::Unauthenticated);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        Ok(())
    }).await
}
