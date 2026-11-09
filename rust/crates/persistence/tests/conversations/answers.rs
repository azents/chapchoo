use super::*;
use chapchoo_persistence::{
    AcceptConversationAnswer as AnswerDraft, ConversationAnswer as Answer,
    ConversationAnswerAdmission as Admission, ConversationAnswerDelivery as Delivery,
    ConversationAnswerForwardingAdmission as ForwardingAdmission,
    ConversationAnswerResult as ResultEvidence, ConversationAnswerValue as Value,
    ConversationDeliveryTarget, ConversationFormAnswer as Form,
    ConversationOperationForwarding as Forwarding, ConversationRequestKind as Kind,
    LiveConversationRequest as Request,
};
use std::collections::BTreeMap;

pub(super) async fn exercise(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
    native: &NativeConversationId,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    let original = Answer::Form(Form::Accept {
        content: Some(BTreeMap::from([
            ("enabled".into(), Value::Boolean(false)),
            ("text".into(), Value::Text(String::new())),
            ("choices".into(), Value::Strings(Vec::new())),
            ("large".into(), Value::Integer(i64::MAX)),
        ])),
    });
    let draft = AnswerDraft {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        request_id: "form-answer",
        target: &target,
        responder_token: "live-form",
        answer: &original,
    };
    // Historical capture alone cannot create a live native response target.
    assert!(matches!(
        store
            .accept_conversation_answer(&owner, &draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let request = Request {
        responder_token: "live-form".into(),
        occurrence: position(3, "form-event")?,
        kind: Kind::Form,
        open: true,
    };
    assert!(matches!(
        store
            .observe_conversation_request(&source, conversation, &target, &request)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let question = b"original typed question".to_vec();
    let records = vec![chapchoo_persistence::ConversationRecordUpdate {
        kind: chapchoo_persistence::ConversationRecordKind::Request,
        native_id: request.occurrence.occurrence().as_str().into(),
        fields: vec![chapchoo_persistence::ConversationFieldUpdate {
            field: chapchoo_persistence::ConversationRecordField::Request,
            change: chapchoo_persistence::ConversationFieldChange::Replace(question.clone()),
        }],
    }];
    store
        .ingest_conversation_records(
            &source,
            conversation,
            &CapturedEventPair {
                snapshot: snapshot.clone(),
                position: request.occurrence.clone(),
                raw: b"original form request".to_vec(),
                normalized: serde_json::to_vec(&records)?,
            },
            &records,
        )
        .await?;
    store
        .observe_conversation_request(&source, conversation, &target, &request)
        .await?;
    let retained = store
        .conversation_answer_question(&owner, &draft, f.now)
        .await?;
    let live = store
        .conversation_live_requests(&owner, &f.agent.id, conversation, &target, f.now)
        .await?;
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].responder_token, request.responder_token);
    assert_eq!(live[0].occurrence, request.occurrence);
    assert!(live[0].answer_request_id.is_none());
    assert_eq!(retained.content, question);
    assert_eq!(retained.occurrence, request.occurrence);
    assert_eq!(retained.kind, Kind::Form);
    assert!(matches!(
        store
            .conversation_answer_question(&f.other.digest(), &draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let conflict = AnswerDraft {
        request_id: "competing-answer",
        ..draft
    };
    assert!(matches!(
        store
            .observe_conversation_request(
                &source,
                conversation,
                &target,
                &Request {
                    responder_token: "different-token-same-occurrence".into(),
                    occurrence: request.occurrence.clone(),
                    kind: Kind::Form,
                    open: true,
                }
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let wrong_kind = Answer::PermissionCancelled;
    assert!(matches!(
        store
            .accept_conversation_answer(
                &owner,
                &AnswerDraft {
                    answer: &wrong_kind,
                    ..draft
                },
                f.now
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let (one, two) = tokio::join!(
        store.accept_conversation_answer(&owner, &draft, f.now),
        store.accept_conversation_answer(&owner, &conflict, f.now)
    );
    let accepted = match (one, two) {
        (Ok(Admission::Accepted(receipt)), Err(StoreError::Conflict))
        | (Err(StoreError::Conflict), Ok(Admission::Accepted(receipt))) => receipt,
        result => return Err(format!("Expected exactly one accepted answer: {result:?}").into()),
    };
    let live = store
        .conversation_live_requests(&owner, &f.agent.id, conversation, &target, f.now)
        .await?;
    assert_eq!(
        live[0].answer_request_id.as_deref(),
        Some(accepted.request_id.as_str())
    );
    assert_eq!(accepted.answer, original);
    assert_eq!(accepted.occurrence, request.occurrence);
    assert_eq!(accepted.delivery, Delivery::Accepted);
    let forward = Forwarding {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &accepted.id,
        target: &target,
    };
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    assert!(matches!(
        store.connector_answer_chunk(&source, &forward, 0).await,
        Err(StoreError::Conflict)
    ));
    let (one, two) = tokio::join!(
        store.begin_conversation_answer_forwarding(&owner, &route, &forward, f.now),
        store.begin_conversation_answer_forwarding(&owner, &route, &forward, f.now),
    );
    assert!(matches!(
        (one?, two?),
        (
            ForwardingAdmission::Granted(_),
            ForwardingAdmission::Existing(Delivery::PossibleForwarding)
        ) | (
            ForwardingAdmission::Existing(Delivery::PossibleForwarding),
            ForwardingAdmission::Granted(_)
        )
    ));
    assert_eq!(
        store
            .conversation_answer(
                &owner,
                &f.agent.id,
                conversation,
                &accepted.request_id,
                f.now
            )
            .await?
            .ok_or("Missing answer")?
            .delivery,
        Delivery::PossibleForwarding
    );
    let chunk = store.connector_answer_chunk(&source, &forward, 0).await?;
    assert_eq!(chunk.responder_token, request.responder_token);
    assert_eq!(chunk.occurrence, request.occurrence);
    assert_eq!(chunk.kind, Kind::Form);
    assert_eq!(
        serde_json::from_slice::<Answer>(&chunk.content.bytes)?,
        original
    );
    assert_eq!(
        chunk.content.total_bytes as usize,
        chunk.content.bytes.len()
    );
    let suffix = store
        .connector_answer_chunk(&source, &forward, chunk.content.total_bytes)
        .await?;
    assert!(suffix.content.bytes.is_empty());
    assert_eq!(suffix.occurrence, request.occurrence);
    assert!(matches!(
        store
            .connector_answer_chunk(&source, &forward, chunk.content.total_bytes + 1)
            .await,
        Err(StoreError::InvalidInput)
    ));
    store
        .complete_conversation_answer_forwarding(
            &source,
            &forward,
            ResultEvidence::ResponderAccepted,
        )
        .await?;
    store
        .complete_conversation_answer_forwarding(
            &source,
            &forward,
            ResultEvidence::ResponderAccepted,
        )
        .await?;
    assert!(matches!(
        store
            .complete_conversation_answer_forwarding(&source, &forward, ResultEvidence::Rejected)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .begin_conversation_answer_forwarding(&owner, &route, &forward, f.now)
            .await?,
        ForwardingAdmission::Existing(Delivery::Known(ResultEvidence::ResponderAccepted))
    ));
    assert!(matches!(
        store.connector_answer_chunk(&source, &forward, 0).await,
        Err(StoreError::Conflict)
    ));
    let retry = AnswerDraft {
        request_id: &accepted.request_id,
        ..draft
    };
    assert!(
        matches!(store.accept_conversation_answer(&owner,&retry,f.now).await?,Admission::Existing(receipt) if receipt.id==accepted.id)
    );
    assert!(matches!(
        store
            .accept_conversation_answer(&f.other.digest(), &retry, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let changed = Answer::Form(Form::Cancel);
    assert!(matches!(
        store
            .accept_conversation_answer(
                &owner,
                &AnswerDraft {
                    answer: &changed,
                    ..retry
                },
                f.now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let closed = Request {
        open: false,
        ..request
    };
    store
        .observe_conversation_request(&source, conversation, &target, &closed)
        .await?;
    let read = store
        .conversation_answer(
            &owner,
            &f.agent.id,
            conversation,
            &accepted.request_id,
            f.now,
        )
        .await?
        .ok_or("Missing retained answer")?;
    assert!(
        store
            .conversation_live_requests(&owner, &f.agent.id, conversation, &target, f.now)
            .await?
            .is_empty()
    );
    assert_eq!(read.answer, original);
    let discovered = store
        .conversation_request_answer(
            &owner,
            &f.agent.id,
            conversation,
            snapshot,
            closed.occurrence.occurrence(),
            f.now,
        )
        .await?
        .ok_or("Missing source-bound historical answer")?;
    assert_eq!(discovered.id, read.id);
    assert_eq!(discovered.request_id, read.request_id);
    assert_eq!(
        discovered.delivery,
        Delivery::Known(ResultEvidence::ResponderAccepted)
    );
    assert!(
        store
            .conversation_request_answer(
                &owner,
                &f.agent.id,
                conversation,
                snapshot,
                &EventOccurrenceId::parse("not-a-question".into())?,
                f.now,
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .conversation_request_answer(
                &f.other.digest(),
                &f.agent.id,
                conversation,
                snapshot,
                closed.occurrence.occurrence(),
                f.now,
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .conversation_request_answer(
                &owner,
                &f.agent.id,
                conversation,
                &SnapshotId::parse("replaced-source".into())?,
                closed.occurrence.occurrence(),
                f.now,
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .conversation_retained_answer(
                &owner,
                &f.agent.id,
                conversation,
                snapshot,
                &read.id,
                f.now
            )
            .await?,
        original
    );
    assert!(matches!(
        store
            .conversation_retained_answer(
                &f.other.digest(),
                &f.agent.id,
                conversation,
                snapshot,
                &read.id,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        read.delivery,
        Delivery::Known(ResultEvidence::ResponderAccepted)
    );
    assert!(matches!(
        store
            .accept_conversation_answer(&owner, &retry, f.now)
            .await?,
        Admission::Existing(_)
    ));
    // A delayed open observation cannot resurrect a closed single-use responder.
    let closed = Request {
        responder_token: "closed-first".into(),
        occurrence: position(4, "closed-event")?,
        ..closed
    };
    store
        .ingest_conversation_pair(
            &source,
            conversation,
            &CapturedEventPair {
                snapshot: snapshot.clone(),
                position: closed.occurrence.clone(),
                raw: b"closed request".to_vec(),
                normalized: b"closed request".to_vec(),
            },
        )
        .await?;
    store
        .observe_conversation_request(&source, conversation, &target, &closed)
        .await?;
    store
        .observe_conversation_request(
            &source,
            conversation,
            &target,
            &Request {
                open: true,
                ..closed
            },
        )
        .await?;
    assert!(matches!(
        store
            .accept_conversation_answer(
                &owner,
                &AnswerDraft {
                    request_id: "closed-answer",
                    responder_token: "closed-first",
                    ..draft
                },
                f.now
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    for (index, answer) in [
        Answer::Form(Form::Decline),
        Answer::Form(Form::Cancel),
        Answer::Form(Form::Accept { content: None }),
        Answer::PermissionCancelled,
        Answer::PermissionSelected {
            option_id: "deny-once".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let token = format!("additional-{index}");
        let request = Request {
            responder_token: token.clone(),
            occurrence: position(5 + i64::try_from(index)?, &format!("request-event-{index}"))?,
            kind: if index < 3 {
                Kind::Form
            } else {
                Kind::Permission
            },
            open: true,
        };
        store
            .ingest_conversation_pair(
                &source,
                conversation,
                &CapturedEventPair {
                    snapshot: snapshot.clone(),
                    position: request.occurrence.clone(),
                    raw: b"additional request".to_vec(),
                    normalized: b"additional request".to_vec(),
                },
            )
            .await?;
        store
            .observe_conversation_request(&source, conversation, &target, &request)
            .await?;
        let draft = AnswerDraft {
            request_id: &token,
            responder_token: &token,
            answer: &answer,
            ..draft
        };
        assert!(
            matches!(store.accept_conversation_answer(&owner,&draft,f.now).await?,Admission::Accepted(receipt) if receipt.answer==answer)
        );
    }
    let invalid = Answer::Form(Form::Accept {
        content: Some(BTreeMap::from([(
            "number".into(),
            Value::Number(f64::INFINITY),
        )])),
    });
    assert!(matches!(
        store
            .accept_conversation_answer(
                &owner,
                &AnswerDraft {
                    answer: &invalid,
                    ..draft
                },
                f.now
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    Ok(())
}
pub(super) async fn replaced(
    store: &Store,
    f: &Fixture,
    conversation: &str,
) -> Result<(), TestError> {
    for request in [
        "form-answer",
        "competing-answer",
        "additional-0",
        "additional-4",
    ] {
        assert!(
            store
                .conversation_answer(&f.owner.digest(), &f.agent.id, conversation, request, f.now)
                .await?
                .is_none()
        );
    }
    Ok(())
}
