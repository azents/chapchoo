use super::*;
use chapchoo_persistence::{
    ConversationMessageRead, ConversationMessageRole, ConversationMessageScope,
    ConversationMessageWindow,
};

const OLD_INPUTS:&str = "INSERT INTO conversations(id,agent_id,source_state,created_at) SELECT id,id,'NOT_STARTED',1000 FROM agents WHERE handle='chat-fixture';
INSERT INTO agent_conversations(agent_id,conversation_id) SELECT id,id FROM agents WHERE handle='chat-fixture';
INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at) SELECT id,id,'legacy-a',NULL,'  원문 **그대로** 🙂
두 줄  ',1000 FROM agents WHERE handle='chat-fixture';
INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at) SELECT computer_id,id,'legacy-b',NULL,'같은 시각의 다른 원문',1000 FROM agents WHERE handle='chat-fixture';
INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at,deleted_at) SELECT owner_user_id,id,'legacy-deleted',NULL,NULL,1000,1100 FROM agents WHERE handle='chat-fixture'";
#[derive(diesel::QueryableByName)]
struct OriginalOrder {
    #[diesel(sql_type=diesel::sql_types::Text)]
    request_id: String,
}
const ORDER: &str = "SELECT i.request_id FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id JOIN agents a ON a.id=c.agent_id WHERE a.handle='chat-fixture' AND i.submitted_text IS NOT NULL AND i.deleted_at IS NULL ORDER BY i.accepted_at,i.id";

async fn original_inputs(
    store: &Store,
    f: &Fixture,
    order: &[OriginalOrder],
) -> Result<Vec<chapchoo_persistence::ConversationInputReceipt>, TestError> {
    let conversation = store
        .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
        .await?
        .ok_or("Missing old conversation")?;
    let mut original = Vec::new();
    for input in order {
        original.push(
            store
                .conversation_input(
                    &f.owner.digest(),
                    &f.agent.id,
                    &conversation.id,
                    &input.request_id,
                    f.now,
                )
                .await?
                .ok_or("Missing original input")?,
        );
    }
    Ok(original)
}
async fn verify_transfer(
    store: &Store,
    f: &Fixture,
    original: &[chapchoo_persistence::ConversationInputReceipt],
) -> Result<(), TestError> {
    let first = original.first().ok_or("Missing transfer fixture")?;
    let owner = f.owner.digest();
    let conversation = &first.conversation_id;
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: conversation,
    };
    let transferred = store
        .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(transferred.messages.len(), 2);
    assert_eq!(transferred.through, 2);
    for (index, (message, input)) in transferred.messages.iter().zip(original).enumerate() {
        assert_eq!(message.receipt.receipt_order, i64::try_from(index)? + 1);
        assert_eq!(message.receipt.role, ConversationMessageRole::User);
        assert_eq!(
            message.receipt.source_input_id.as_deref(),
            Some(input.id.as_str())
        );
        assert_eq!(
            message.receipt.source_input_request_id.as_deref(),
            Some(input.request_id.as_str())
        );
        assert_eq!(
            message.receipt.original_accepted_at,
            Some(input.accepted_at)
        );
        assert!(message.receipt.received_at > input.accepted_at);
        let body = store
            .conversation_message_chunk(
                &owner,
                &ConversationMessageRead {
                    scope: ConversationMessageScope {
                        agent_id: &f.agent.id,
                        conversation_id: conversation,
                    },
                    message_id: &message.receipt.id,
                    offset: 0,
                    length: 32768,
                },
                f.now,
            )
            .await?;
        assert_eq!(String::from_utf8(body.bytes)?, input.text.as_str());
        let current = store
            .conversation_input(&owner, &f.agent.id, conversation, &input.request_id, f.now)
            .await?
            .ok_or("Transfer removed input")?;
        assert_eq!(current.id, input.id);
        assert_eq!(current.text, input.text);
        assert_eq!(current.accepted_at, input.accepted_at);
        assert!(
            store
                .conversation_input_delivery(&owner, &f.agent.id, conversation, &input.id, f.now)
                .await?
                .is_none()
        );
        // Receipt recovery of a still-retained old input does not append chat again.
        store
            .accept_conversation_input(
                &owner,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(conversation.clone()),
                    request_id: input.request_id.clone(),
                    target: input.target.clone(),
                    text: input.text.clone(),
                },
                f.now,
            )
            .await?;
    }
    let deleted = store
        .conversation_input_reference(&owner, &f.agent.id, conversation, "legacy-deleted", f.now)
        .await?
        .ok_or("Deleted provenance disappeared")?;
    assert!(deleted.deleted);
    assert!(
        !transferred
            .messages
            .iter()
            .any(|m| m.receipt.source_input_id.as_deref() == Some(deleted.id.as_str()))
    );
    store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.clone()),
                request_id: "new-after-prefix".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("New receipt follows transferred originals".into())?,
            },
            f.now,
        )
        .await?;
    let all = store
        .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(all.through, 3);
    assert_eq!(all.messages.len(), 3);
    assert_eq!(
        all.messages
            .iter()
            .map(|m| m.receipt.receipt_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    Ok(())
}
#[test]
fn sqlite_preserves_retained_original_input_prefix() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("chat-transfer.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher.clone()))?;
    let f = runtime.block_on(fixture(&store))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    isolate_pre_chat_migration!(&mut connection, SQLITE_MIGRATIONS);
    connection.batch_execute(OLD_INPUTS)?;
    let order = diesel::sql_query(ORDER).load::<OriginalOrder>(&mut connection)?;
    let original = runtime.block_on(original_inputs(&store, &f, &order))?;
    prepare_sqlite(&path, &cipher)?;
    runtime.block_on(verify_transfer(&store, &f, &original))?;
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}
#[test]
fn postgres_preserves_retained_original_input_prefix() -> Result<(), TestError> {
    let _fixture_lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("chat_transfer_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher.clone()))?;
        let f = runtime.block_on(fixture(&store))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        isolate_pre_chat_migration!(&mut connection, POSTGRES_MIGRATIONS);
        connection.batch_execute(OLD_INPUTS)?;
        let order = diesel::sql_query(ORDER).load::<OriginalOrder>(&mut connection)?;
        let original = runtime.block_on(original_inputs(&store, &f, &order))?;
        prepare_postgres(&url, &cipher)?;
        runtime.block_on(verify_transfer(&store, &f, &original))?;
        assert!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)
                .is_err()
        );
        Ok(())
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
