use super::*;
use chapchoo_persistence::{
    AuthenticationRefreshAdmission as Admission, AuthenticationRefreshState as RefreshState,
    CheckDeviceAdmission, DeviceAuthenticationState, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, ExchangeDeviceAdmission, ReauthenticateTarget, StartDeviceAdmission,
    StartDeviceAuthentication,
};

fn tokens(f: &Fixture, value: &str, fresh: bool) -> Result<DeviceAuthenticationTokens, TestError> {
    Ok(DeviceAuthenticationTokens {
        secret: ExternalSecret::new(value.as_bytes().to_vec())?,
        account_id: Some("execution-refresh-account".into()),
        email: None,
        plan_type: None,
        expires_at: Some(f.now + if fresh { 3_600_000 } else { 1001 }),
    })
}
async fn acquire(
    store: &Store,
    f: &Fixture,
    request: &str,
    target: Option<(&str, &str)>,
) -> Result<String, TestError> {
    let actor = f.owner.digest();
    let StartDeviceAdmission::Issue(issue) = store
        .start_external_device_authentication(
            &actor,
            &StartDeviceAuthentication {
                scope: AuthenticationScope::Personal,
                request_id: request.into(),
                name: String::new(),
                target: target.map(|(id, expected)| ReauthenticateTarget {
                    authentication_id: id.into(),
                    expected_request_id: expected.into(),
                }),
            },
            f.now,
        )
        .await?
    else {
        return Err("Missing device issuance".into());
    };
    store
        .accept_external_device_authorization(
            &actor,
            &issue,
            &DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(b"synthetic-device-only".to_vec())?,
                user_code: "SYNTH-CODE".into(),
                interval_ms: 1000,
                expires_at: f.now + 60_000,
            },
            f.now,
        )
        .await?;
    let CheckDeviceAdmission::Check(check) = store
        .check_external_device_authentication(&actor, &issue.attempt_id(), f.now + 1000)
        .await?
    else {
        return Err("Missing device check".into());
    };
    let ExchangeDeviceAdmission::Exchange(exchange) = store
        .exchange_external_device_authentication(&actor, &check.step, f.now + 1000)
        .await?
    else {
        return Err("Missing device exchange".into());
    };
    let result = store
        .complete_external_device_authentication(
            &actor,
            &exchange,
            &tokens(f, request, false)?,
            f.now + 1001,
        )
        .await?;
    match result.state {
        DeviceAuthenticationState::Registered { authentication_id } => Ok(authentication_id),
        _ => Err("Missing registered credentials".into()),
    }
}
pub(super) async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let actor = f.owner.digest();
    let key = f.connector.digest();
    let route = store
        .prepare_connector_connection(&f.session, &key, &f.control)
        .await?;
    store
        .admit_connector_connection(
            &f.session,
            &key,
            route.expected.as_ref(),
            &route.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let source = ConnectorSource {
        session_id: &f.session,
        control_id: &f.control,
        connection_id: &route.owner.connection_id,
        credential: &key,
    };
    let saved = store
        .agent(&actor, &f.agent.id, f.now)
        .await?
        .authentication
        .ok_or("Missing saved key")?;
    let oauth = acquire(store, f, "execution-oauth", None).await?;
    store
        .select_agent_authentication(
            &actor,
            &SelectAgentAuthentication {
                agent_id: f.agent.id.clone(),
                authentication_id: oauth.clone(),
                request_id: "execution-oauth-selection".into(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: saved.authentication_id.clone(),
                    request_id: saved.request_id.clone(),
                }),
            },
            f.now + 1001,
        )
        .await?;
    let conversation = store
        .current_conversation(&actor, &f.agent.id, f.now)
        .await?
        .ok_or("Missing current conversation")?;
    let input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.id.clone()),
                request_id: "execution-oauth-start".into(),
                text: ConversationText::parse("synthetic OAuth start".into())?,
                target: ConversationTarget::Snapshot(
                    conversation
                        .source
                        .snapshot()
                        .ok_or("Missing source")?
                        .clone(),
                ),
            },
            f.now,
        )
        .await?;
    let application = draft(f, &input);
    let current_route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    store
        .begin_conversation_input_preparation(&actor, &current_route, &application, f.now)
        .await?;
    let previous = store
        .execution_authentication_binding(&source, &application)
        .await?
        .ok_or("Missing prior application")?;
    let applied = store
        .begin_execution_authentication(&source, &application, Some(&previous.operation_id))
        .await?;
    assert_eq!(applied.authentication_id, oauth);
    // Account and execution consumers coalesce on one record/source attempt.
    let (one, two) = tokio::join!(
        store.begin_execution_authentication_refresh(&source, &application, f.now + 1001),
        store.begin_external_authentication_refresh(
            &actor,
            &AuthenticationScope::Personal,
            &oauth,
            f.now + 1001
        ),
    );
    let (step, original) = match (one?, two?) {
        (
            Admission::Granted { step, credentials },
            Admission::Existing {
                attempt_id,
                state: RefreshState::Possible,
            },
        )
        | (
            Admission::Existing {
                attempt_id,
                state: RefreshState::Possible,
            },
            Admission::Granted { step, credentials },
        ) => {
            assert_eq!(step.id(), attempt_id);
            (step, credentials)
        }
        other => return Err(format!("Expected one cross-consumer refresh: {other:?}").into()),
    };
    // A saved future selection does not withdraw the current applied record.
    store
        .select_agent_authentication(
            &actor,
            &SelectAgentAuthentication {
                agent_id: f.agent.id.clone(),
                authentication_id: saved.authentication_id,
                request_id: "execution-next-key".into(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: oauth.clone(),
                    request_id: "execution-oauth-selection".into(),
                }),
            },
            f.now + 1001,
        )
        .await?;
    let renewed = tokens(f, "execution-refreshed", true)?;
    let updated = store
        .complete_execution_authentication_refresh(
            &source,
            &application,
            &step,
            &renewed,
            f.now + 1002,
        )
        .await?;
    assert_ne!(updated, original.source);
    assert_eq!(
        store
            .execution_authentication_material(&source, &application, Some(&updated))
            .await?
            .secret
            .expose(),
        b"execution-refreshed"
    );
    assert!(matches!(
        store
            .complete_execution_authentication_refresh(
                &source,
                &application,
                &step,
                &renewed,
                f.now + 1003
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let Admission::Current(fresh) = store
        .begin_execution_authentication_refresh(&source, &application, f.now + 1003)
        .await?
    else {
        return Err("Fresh execution credentials refreshed again".into());
    };
    assert_eq!(fresh.source, updated);
    assert_eq!(fresh.authentication_id, oauth);
    // Reauthentication creates a new source without changing the applied record.
    acquire(
        store,
        f,
        "execution-oauth-reauth",
        Some((&oauth, "execution-oauth")),
    )
    .await?;
    let Admission::Granted { step: lost, .. } = store
        .begin_execution_authentication_refresh(&source, &application, f.now + 1004)
        .await?
    else {
        return Err("Missing new-source refresh".into());
    };
    let replacement = store
        .prepare_connector_connection(&f.session, &key, &f.control)
        .await?;
    store
        .admit_connector_connection(
            &f.session,
            &key,
            replacement.expected.as_ref(),
            &replacement.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    assert!(matches!(
        store
            .complete_execution_authentication_refresh(
                &source,
                &application,
                &lost,
                &renewed,
                f.now + 1005
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    store
        .fail_external_authentication_refresh(&lost, RefreshState::Uncertain, f.now + 1005)
        .await?;
    let current_source = ConnectorSource {
        connection_id: &replacement.owner.connection_id,
        ..source
    };
    assert!(matches!(
        store
            .begin_execution_authentication_refresh(&current_source, &application, f.now + 1006)
            .await?,
        Admission::Existing {
            state: RefreshState::Uncertain,
            ..
        }
    ));
    acquire(
        store,
        f,
        "execution-oauth-recover",
        Some((&oauth, "execution-oauth-reauth")),
    )
    .await?;
    let Admission::Granted { step: stale, .. } = store
        .begin_execution_authentication_refresh(&current_source, &application, f.now + 1007)
        .await?
    else {
        return Err("Missing recovered source grant".into());
    };
    acquire(
        store,
        f,
        "execution-oauth-replace",
        Some((&oauth, "execution-oauth-recover")),
    )
    .await?;
    assert!(matches!(
        store
            .complete_execution_authentication_refresh(
                &current_source,
                &application,
                &stale,
                &renewed,
                f.now + 1008
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .delete_external_authentication(
                &actor,
                &DeleteAuthentication {
                    scope: AuthenticationScope::Personal,
                    request_id: "execution-oauth-delete".into(),
                    authentication_id: oauth,
                    expected_request_id: "execution-oauth-replace".into(),
                },
                f.now + 1009
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert!(matches!(
        store
            .complete_execution_authentication_refresh(
                &current_source,
                &application,
                &stale,
                &renewed,
                f.now + 1010
            )
            .await,
        Err(StoreError::NotFound)
    ));
    store
        .fail_external_authentication_refresh(&stale, RefreshState::Uncertain, f.now + 1010)
        .await?;
    assert!(matches!(
        store
            .begin_execution_authentication_refresh(&current_source, &application, f.now + 1011)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .execution_authentication_binding(&current_source, &application)
            .await?,
        Some(applied)
    );
    Ok(())
}
