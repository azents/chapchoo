use super::*;

type TestError = Box<dyn std::error::Error + Send + Sync>;
const ENDPOINT: &str = "https://mcp.example.invalid/tools";
const OTHER_ENDPOINT: &str = "https://other.example.invalid/tools";
const TOKEN: &str = "synthetic-mcp-fixture-token";
const REPLACEMENT: &str = "replacement-mcp-fixture-token";

fn draft(endpoint: &str, auth: McpAuthEdit) -> Result<McpSettingsDraft, StoreError> {
    Ok(McpSettingsDraft {
        display_name: "Fixture MCP".into(),
        endpoint: McpEndpoint::parse(endpoint)?,
        auth,
    })
}

pub(in crate::computers) async fn management_contract(
    store: &Store,
    owner: &SecretDigest,
    stranger: &SecretDigest,
    agent: &str,
    now: i64,
) -> Result<(), TestError> {
    let create = AgentIntegrationChange::CreateMcp(draft(
        ENDPOINT,
        McpAuthEdit::ReplaceBearer(McpBearer::parse(TOKEN.into())?),
    )?);
    assert!(!format!("{create:?}").contains(TOKEN));
    assert!(!format!("{create:?}").contains(ENDPOINT));
    assert!(matches!(
        store
            .change_agent_integration(stranger, agent, "add-mcp", &create, now)
            .await,
        Err(StoreError::NotFound)
    ));
    let (first, repeated) = tokio::join!(
        store.change_agent_integration(owner, agent, "add-mcp", &create, now),
        store.change_agent_integration(owner, agent, "add-mcp", &create, now)
    );
    let saved = first?;
    assert_eq!(repeated?, saved);
    assert_eq!(saved.action, AgentIntegrationAction::Create);
    let id = saved.id.as_str();
    assert_eq!(
        store
            .agent_integration_result(owner, agent, "add-mcp", now)
            .await?
            .as_ref(),
        Some(&saved)
    );
    let conflicting_create = AgentIntegrationChange::CreateMcp(draft(
        ENDPOINT,
        McpAuthEdit::ReplaceBearer(McpBearer::parse(REPLACEMENT.into())?),
    )?);
    assert!(matches!(
        store
            .change_agent_integration(owner, agent, "add-mcp", &conflicting_create, now)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .agent_mcp_integrations(stranger, agent, None, now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .agent_integration_result(stranger, agent, "add-mcp", now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .prepare_agent_mcp_test(
                stranger,
                agent,
                Some(id),
                McpEndpoint::parse(ENDPOINT)?,
                McpAuthEdit::KeepBearer,
                now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let list = store
        .agent_mcp_integrations(owner, agent, None, now)
        .await?;
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].id, id);
    assert!(list.items[0].has_bearer);
    assert!(!format!("{list:?}").contains(TOKEN));

    let agent_key = EntityId::from_dto(EntityKind::Agent, agent)?;
    let integration_key = EntityId::from_dto(EntityKind::AgentIntegration, id)?;
    let stored = operation!(store, |connection, dialect| {
        Ok(diesel::sql_query(dialect.query(SECRET_ROW))
            .bind::<IdentitySql, _>(agent_key)
            .bind::<IdentitySql, _>(integration_key)
            .get_result::<SecretRow>(connection)
            .await?)
    })?;
    let envelope = stored.keeper(&McpEndpoint::parse(ENDPOINT)?)?;
    assert!(
        !envelope
            .ciphertext()
            .windows(TOKEN.len())
            .any(|bytes| bytes == TOKEN.as_bytes())
    );
    let different_agent = EntityId::generate(now)?.to_dto(EntityKind::Agent);
    let different_object = EntityId::generate(now)?.to_dto(EntityKind::AgentIntegration);
    for wrong_context in [
        context(&different_agent, id)?,
        context(agent, &different_object)?,
    ] {
        assert!(
            store
                .cipher
                .recover_external(&wrong_context, &envelope)
                .is_err()
        );
    }

    let acquire = |endpoint: &'static str, auth| async move {
        store
            .prepare_agent_mcp_test(
                owner,
                agent,
                Some(id),
                McpEndpoint::parse(endpoint)?,
                auth,
                now,
            )
            .await
    };
    let existing = acquire(ENDPOINT, McpAuthEdit::KeepBearer).await?;
    assert!(
        existing
            .bearer
            .is_some_and(|token| token.as_bytes() == TOKEN.as_bytes())
    );
    assert!(matches!(
        acquire(OTHER_ENDPOINT, McpAuthEdit::KeepBearer).await,
        Err(StoreError::InvalidInput)
    ));

    let update = |endpoint, auth| -> Result<_, StoreError> {
        Ok(AgentIntegrationChange::UpdateMcp {
            id: id.into(),
            draft: draft(endpoint, auth)?,
        })
    };
    let wrong_endpoint = update(OTHER_ENDPOINT, McpAuthEdit::KeepBearer)?;
    assert!(matches!(
        store
            .change_agent_integration(owner, agent, "rejected-edit", &wrong_endpoint, now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        store
            .agent_integration_result(owner, agent, "rejected-edit", now)
            .await?
            .is_none()
    );
    let keep = update(ENDPOINT, McpAuthEdit::KeepBearer)?;
    assert_eq!(
        store
            .change_agent_integration(owner, agent, "keep-mcp", &keep, now)
            .await?
            .action,
        AgentIntegrationAction::Update
    );
    let replace = update(
        OTHER_ENDPOINT,
        McpAuthEdit::ReplaceBearer(McpBearer::parse(REPLACEMENT.into())?),
    )?;
    store
        .change_agent_integration(owner, agent, "replace-mcp", &replace, now)
        .await?;
    let updated = acquire(OTHER_ENDPOINT, McpAuthEdit::KeepBearer).await?;
    assert!(
        updated
            .bearer
            .is_some_and(|token| token.as_bytes() == REPLACEMENT.as_bytes())
    );
    let remove = update(OTHER_ENDPOINT, McpAuthEdit::NoAuth)?;
    store
        .change_agent_integration(owner, agent, "remove-mcp-bearer", &remove, now)
        .await?;
    assert!(matches!(
        acquire(OTHER_ENDPOINT, McpAuthEdit::KeepBearer).await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        !store
            .agent_mcp_integrations(owner, agent, None, now)
            .await?
            .items[0]
            .has_bearer
    );

    let delete = AgentIntegrationChange::Delete { id: id.into() };
    let deleted = store
        .change_agent_integration(owner, agent, "delete-mcp", &delete, now)
        .await?;
    assert_eq!(deleted.action, AgentIntegrationAction::Delete);
    assert_eq!(
        store
            .change_agent_integration(owner, agent, "delete-mcp", &delete, now)
            .await?,
        deleted
    );
    assert!(matches!(
        acquire(OTHER_ENDPOINT, McpAuthEdit::NoAuth).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .change_agent_integration(owner, agent, "add-mcp", &create, now)
            .await?,
        saved
    );
    assert!(
        store
            .agent_mcp_integrations(owner, agent, None, now)
            .await?
            .items
            .is_empty()
    );

    let no_auth = AgentIntegrationChange::CreateMcp(draft(ENDPOINT, McpAuthEdit::NoAuth)?);
    let mut ids = Vec::new();
    for index in 0..=PAGE_SIZE {
        let receipt = store
            .change_agent_integration(owner, agent, &format!("page-mcp-{index}"), &no_auth, now)
            .await?;
        ids.push(receipt.id);
    }
    let page = store
        .agent_mcp_integrations(owner, agent, None, now)
        .await?;
    assert_eq!(page.items.len(), usize::from(PAGE_SIZE));
    let after = page.next_after.ok_or("missing cursor")?;
    assert_eq!(
        page.items.last().map(|item| item.id.as_str()),
        Some(after.as_str())
    );
    let last = store
        .agent_mcp_integrations(owner, agent, Some(&after), now)
        .await?;
    assert_eq!(last.items.len(), 1);
    assert_eq!(last.next_after, None);
    let mut actual = page
        .items
        .into_iter()
        .chain(last.items)
        .map(|item| item.id)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    ids.sort_unstable();
    assert_eq!(actual, ids);
    Ok(())
}

#[test]
fn malformed_settings_are_rejected_without_diagnostics_containing_input() {
    for endpoint in [
        "https://",
        "http://mcp.example.invalid",
        "https://user@example.invalid",
        "https://mcp.example.invalid/#secret",
    ] {
        assert!(matches!(
            McpEndpoint::parse(endpoint),
            Err(StoreError::InvalidInput)
        ));
    }
    for value in ["", "a b", "invalid\nheader", "가"] {
        assert!(matches!(
            McpBearer::parse(value.into()),
            Err(StoreError::InvalidInput)
        ));
    }
    assert!(matches!(
        McpBearer::parse("a".repeat(8193)),
        Err(StoreError::InvalidInput)
    ));
}
