//! Real child Connector routes complete answers and exact Stop to known native rejection.
use super::*;
use chapchoo_domain::{NativeConversationId, SnapshotId, SourcePosition};
use chapchoo_persistence::{
    AgentSummary, ConnectorSource, ConversationAnswer, ConversationAnswerDelivery,
    ConversationAnswerResult, ConversationAnswerValue, ConversationDeliveryTarget,
    ConversationForeground, ConversationFormAnswer, ConversationNativeObservation,
    ConversationRequestKind, ConversationStopState, ConversationWorkTarget,
    LiveConversationRequest,
};
use chapchoo_service::{
    ConversationAnswerDraft, ConversationAnswerService, ConversationService,
    ConversationStopService,
};
use std::collections::BTreeMap;
pub(super) struct Context<'a> {
    pub store: &'a Store,
    pub actor: &'a chapchoo_crypto::SecretDigest,
    pub agent: &'a AgentSummary,
    pub processor: Arc<chapchoo_rpc::ConversationControlProcessor>,
    pub source: ConnectorSource<'a>,
    pub conversation: &'a str,
    pub snapshot: &'a SnapshotId,
    pub request_position: &'a SourcePosition,
    pub state_position: &'a SourcePosition,
}
pub(super) async fn exercise(
    fixture: &mut test_support::Fixture,
    context: Context<'_>,
) -> Result<(), Error> {
    let target = ConversationDeliveryTarget {
        snapshot: context.snapshot.clone(),
        native_session: NativeConversationId::parse("synthetic-native".into())?,
    };
    context
        .store
        .observe_conversation_request(
            &context.source,
            context.conversation,
            &target,
            &LiveConversationRequest {
                responder_token: "fixture-live-form".into(),
                occurrence: context.request_position.clone(),
                kind: ConversationRequestKind::Form,
                open: true,
            },
        )
        .await?;
    context
        .store
        .observe_conversation_native(
            &context.source,
            context.conversation,
            &ConversationNativeObservation {
                target: target.clone(),
                position: context.state_position.clone(),
                foreground: ConversationForeground::Running,
                work_id: Some("fixture-observed-work".into()),
            },
        )
        .await?;
    let answers = ConversationAnswerService::new(
        Arc::new(ConversationService::new(context.store.clone())),
        context.processor.clone(),
    )?;
    let stops = ConversationStopService::new(context.store.clone(), context.processor)?;
    let cleanup_answers = answers.clone();
    let cleanup_stops = stops.clone();
    fixture.cleanup(async move {
        let (answers, stops) = tokio::join!(cleanup_answers.finish(), cleanup_stops.finish());
        test_support::completed(answers.map_err(Into::into), stops.map_err(Into::into))
    });
    let original = ConversationAnswer::Form(ConversationFormAnswer::Accept {
        content: Some(BTreeMap::from([(
            "text".into(),
            ConversationAnswerValue::Text("한글🦀 preserved answer\n".repeat(5000)),
        )])),
    });
    let answer = answers
        .accept(
            context.actor.clone(),
            ConversationAnswerDraft {
                agent_id: context.agent.id.clone(),
                conversation_id: context.conversation.into(),
                request_id: "real-connector-answer".into(),
                target: target.clone(),
                responder_token: "fixture-live-form".into(),
                answer: original.clone(),
            },
            AcceptanceTime::System,
        )
        .await?;
    let stop = stops
        .accept(
            context.actor.clone(),
            context.agent.id.clone(),
            context.conversation.into(),
            "real-connector-stop".into(),
            ConversationWorkTarget {
                source: target,
                work_id: "fixture-observed-work".into(),
            },
            AcceptanceTime::System,
        )
        .await?;
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let answered = answers
                .current(
                    context.actor,
                    &context.agent.id,
                    context.conversation,
                    &answer.request_id,
                    AcceptanceTime::System,
                )
                .await?
                .ok_or("Missing answer receipt")?;
            let stopped = stops
                .current(
                    context.actor,
                    &context.agent.id,
                    context.conversation,
                    &stop.request_id,
                    AcceptanceTime::System,
                )
                .await?
                .ok_or("Missing Stop receipt")?;
            if answered.delivery
                == ConversationAnswerDelivery::Known(ConversationAnswerResult::Rejected)
                && stopped.state == ConversationStopState::Rejected
            {
                return Ok::<(), Error>(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    // The saved source/question is not a live native owner. The actual child
    // accepted the queue, pulled the complete answer and reported exact rejection.
    assert_eq!(
        context
            .store
            .conversation_answer(
                context.actor,
                &context.agent.id,
                context.conversation,
                &answer.request_id,
                AcceptanceTime::System.now()?
            )
            .await?
            .ok_or("Original answer lost")?
            .answer,
        original
    );
    assert!(matches!(
        context
            .store
            .check_conversation_stop(
                &context.source,
                context.conversation,
                &stop.id,
                &stop.target
            )
            .await,
        Err(chapchoo_persistence::StoreError::Conflict)
    ));
    answers.finish().await?;
    stops.finish().await?;
    Ok(())
}
