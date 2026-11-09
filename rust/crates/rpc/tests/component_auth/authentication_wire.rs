use super::*;
use chapchoo_domain::{ConversationTarget, ConversationText};
use chapchoo_persistence::{
    AcceptConversationInput, AuthenticationAcceptedResult, AuthenticationScope,
    CheckDeviceAdmission, ConversationInputPreparation, DeleteAuthentication,
    DeviceAuthenticationState, DeviceAuthenticationTokens, DeviceAuthorizationDraft,
    ExchangeDeviceAdmission, ExpectedAgentAuthentication, RegisterApiKey, ReplaceApiKey,
    SelectAgentAuthentication, StartDeviceAdmission, StartDeviceAuthentication,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
};
use chapchoo_service::ExternalSecret;
type Client = ConnectorCaptureClient<tonic::transport::Channel>;
fn record(result: AuthenticationAcceptedResult) -> Result<String, TestError> {
    match result {
        AuthenticationAcceptedResult::Record { authentication_id } => Ok(authentication_id),
        _ => Err("Missing record".into()),
    }
}
async fn prepare(
    context: &capture::Context<'_>,
    request: &str,
) -> Result<c::ExecutionAuthenticationReference, TestError> {
    let now = AcceptanceTime::System.now()?;
    let conversation = context
        .store
        .current_conversation(context.owner, &context.agent.id, now)
        .await?
        .ok_or("Missing conversation")?;
    let snapshot = conversation
        .source
        .snapshot()
        .ok_or("Missing captured source")?
        .clone();
    // The preceding source fixture retained an empty replacement copy.
    if matches!(
        conversation.source,
        chapchoo_domain::ConversationSource::Loading { .. }
    ) {
        let credential = BearerSecret::parse(context.key.into())?.digest();
        context
            .store
            .complete_conversation_source(
                &chapchoo_persistence::ConnectorSource {
                    session_id: &context.source.session_id,
                    control_id: &context.source.control_id,
                    connection_id: &context.source.connection_id,
                    credential: &credential,
                },
                &conversation.id,
                &snapshot,
                None,
            )
            .await?;
    }
    let input = context
        .store
        .accept_conversation_input(
            context.owner,
            &AcceptConversationInput {
                agent_id: context.agent.id.clone(),
                conversation_id: Some(conversation.id.clone()),
                request_id: request.into(),
                target: ConversationTarget::Snapshot(snapshot),
                text: ConversationText::parse("synthetic authentication start".into())?,
            },
            now,
        )
        .await?;
    Ok(c::ExecutionAuthenticationReference {
        source: Some(context.source.clone()),
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.id,
        operation_id: input.id,
    })
}
async fn seal(
    context: &capture::Context<'_>,
    target: &c::ExecutionAuthenticationReference,
) -> Result<(), TestError> {
    let route = context
        .store
        .resolve_connector_route(&context.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    context
        .store
        .begin_conversation_input_preparation(
            context.owner,
            &route,
            &ConversationInputPreparation {
                agent_id: &target.agent_id,
                conversation_id: &target.conversation_id,
                operation_id: &target.operation_id,
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    Ok(())
}
async fn acquire(
    client: &mut Client,
    target: &c::ExecutionAuthenticationReference,
    key: &str,
) -> Result<c::ExecutionCredentialDescriptor, TestError> {
    match client
        .acquire_authentication(authenticated(target.clone(), key)?)
        .await?
        .into_inner()
        .state
    {
        Some(c::execution_authentication_progress::State::Current(value)) => Ok(value),
        _ => Err("Missing current native material".into()),
    }
}
async fn read(
    client: &mut Client,
    target: &c::ExecutionAuthenticationReference,
    key: &str,
    descriptor: &c::ExecutionCredentialDescriptor,
) -> Result<Vec<u8>, TestError> {
    let mut bytes = Vec::new();
    while bytes.len() < descriptor.total_bytes as usize {
        let piece = client
            .read_authentication(authenticated(
                c::AuthenticationReadRequest {
                    target: Some(target.clone()),
                    credential_source: descriptor.credential_source.clone(),
                    offset: u32::try_from(bytes.len())?,
                },
                key,
            )?)
            .await?
            .into_inner();
        assert_eq!(piece.offset as usize, bytes.len());
        assert!(piece.content.len() <= 32 * 1024 && !piece.content.is_empty());
        assert_eq!(piece.descriptor, Some(descriptor.clone()));
        bytes.extend(piece.content);
    }
    assert_eq!(bytes.len(), descriptor.total_bytes as usize);
    client
        .check_authentication(authenticated(
            c::AuthenticationHandoffCheck {
                target: Some(target.clone()),
                credential_source: descriptor.credential_source.clone(),
            },
            key,
        )?)
        .await?;
    Ok(bytes)
}
async fn oauth(context: &capture::Context<'_>) -> Result<String, TestError> {
    let now = AcceptanceTime::System.now()?;
    let StartDeviceAdmission::Issue(issue) = context
        .store
        .start_external_device_authentication(
            context.owner,
            &StartDeviceAuthentication {
                scope: AuthenticationScope::Personal,
                request_id: "wire-oauth".into(),
                name: String::new(),
                target: None,
            },
            now,
        )
        .await?
    else {
        return Err("Missing issue".into());
    };
    context
        .store
        .accept_external_device_authorization(
            context.owner,
            &issue,
            &DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(b"fixture-device".to_vec())?,
                user_code: "SYNTH-CODE".into(),
                interval_ms: 1000,
                expires_at: now + 60000,
            },
            now,
        )
        .await?;
    let due = now + 1000;
    let CheckDeviceAdmission::Check(check) = context
        .store
        .check_external_device_authentication(context.owner, &issue.attempt_id(), due)
        .await?
    else {
        return Err("Missing check".into());
    };
    let ExchangeDeviceAdmission::Exchange(exchange) = context
        .store
        .exchange_external_device_authentication(context.owner, &check.step, due)
        .await?
    else {
        return Err("Missing exchange".into());
    };
    let result = context
        .store
        .complete_external_device_authentication(
            context.owner,
            &exchange,
            &DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"fixture-server-refresh-bundle".to_vec())?,
                account_id: Some("fixture-account".into()),
                email: None,
                plan_type: None,
                expires_at: Some(now),
            },
            due,
        )
        .await?;
    // Elapsed provider-check eligibility is a real fixture prerequisite. Do not
    // publish a future creation instant into a System-clock renewal.
    let remaining = due.saturating_sub(AcceptanceTime::System.now()?);
    if remaining > 0 {
        tokio::time::sleep(Duration::from_millis(u64::try_from(remaining)?)).await;
    }
    match result.state {
        DeviceAuthenticationState::Registered { authentication_id } => Ok(authentication_id),
        _ => Err("Missing OAuth record".into()),
    }
}
pub(super) async fn exercise(
    context: &capture::Context<'_>,
    provider: &support::authentication::Provider,
) -> Result<(), TestError> {
    let mut client = Client::connect(context.endpoint.to_owned())
        .await?
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let target = prepare(context, "wire-authentication-start").await?;
    assert!(
        client
            .get_authentication_binding(authenticated(target.clone(), context.key)?)
            .await?
            .into_inner()
            .applied
            .is_none()
    );
    let begin = c::BeginAuthenticationRequest {
        target: Some(target.clone()),
        expected_previous_operation_id: None,
    };
    assert_eq!(
        client
            .begin_authentication(authenticated(begin.clone(), context.key)?)
            .await
            .err()
            .ok_or("Unsealed setup accepted")?
            .code(),
        Code::NotFound
    );
    seal(context, &target).await?;
    let original = context
        .store
        .agent(
            context.owner,
            &context.agent.id,
            AcceptanceTime::System.now()?,
        )
        .await?
        .authentication
        .ok_or("Missing saved record")?;
    let large = vec![b'K'; 65536];
    let id = record(
        context
            .store
            .register_external_api_key(
                context.owner,
                &RegisterApiKey {
                    scope: AuthenticationScope::Personal,
                    request_id: "wire-large-key".into(),
                    name: String::new(),
                    key: ExternalSecret::new(large.clone())?,
                },
                AcceptanceTime::System.now()?,
            )
            .await?,
    )?;
    context
        .store
        .select_agent_authentication(
            context.owner,
            &SelectAgentAuthentication {
                agent_id: context.agent.id.clone(),
                authentication_id: id.clone(),
                request_id: "wire-large-selection".into(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: original.authentication_id.clone(),
                    request_id: original.request_id.clone(),
                }),
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    let applied = client
        .begin_authentication(authenticated(begin.clone(), context.key)?)
        .await?
        .into_inner();
    assert_eq!(applied.authentication_id, id);
    assert_eq!(
        client
            .begin_authentication(authenticated(begin.clone(), context.key)?)
            .await?
            .into_inner(),
        applied
    );
    let first = acquire(&mut client, &target, context.key).await?;
    assert_eq!(first.kind, c::ExecutionCredentialKind::ApiKey as i32);
    assert_eq!(first.total_bytes, 65536);
    assert_eq!(
        read(&mut client, &target, context.key, &first).await?,
        large
    );
    assert_eq!(provider.refreshes.load(Ordering::SeqCst), 0);
    assert_eq!(
        client
            .acquire_authentication(authenticated(target.clone(), context.user_key)?)
            .await
            .err()
            .ok_or("Account token read execution credentials")?
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        client
            .read_authentication(authenticated(
                c::AuthenticationReadRequest {
                    target: Some(target.clone()),
                    credential_source: vec![0; 23],
                    offset: 0
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Malformed source accepted")?
            .code(),
        Code::InvalidArgument
    );
    context
        .store
        .replace_external_api_key(
            context.owner,
            &ReplaceApiKey {
                scope: AuthenticationScope::Personal,
                authentication_id: id.clone(),
                request_id: "wire-replace-key".into(),
                expected_request_id: "wire-large-key".into(),
                key: ExternalSecret::new(vec![b'L'; 65536])?,
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    assert_eq!(
        client
            .check_authentication(authenticated(
                c::AuthenticationHandoffCheck {
                    target: Some(target.clone()),
                    credential_source: first.credential_source.clone()
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Old source handoff accepted")?
            .code(),
        Code::AlreadyExists
    );
    assert_eq!(
        client
            .read_authentication(authenticated(
                c::AuthenticationReadRequest {
                    target: Some(target.clone()),
                    credential_source: first.credential_source,
                    offset: 32768
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Mixed-source continuation accepted")?
            .code(),
        Code::AlreadyExists
    );
    context
        .store
        .select_agent_authentication(
            context.owner,
            &SelectAgentAuthentication {
                agent_id: context.agent.id.clone(),
                authentication_id: original.authentication_id,
                request_id: "wire-future-selection".into(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: id.clone(),
                    request_id: "wire-large-selection".into(),
                }),
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    let next = acquire(&mut client, &target, context.key).await?;
    assert_eq!(
        next.applied
            .as_ref()
            .ok_or("Missing applied")?
            .authentication_id,
        id
    );
    assert_eq!(
        read(&mut client, &target, context.key, &next).await?,
        vec![b'L'; 65536]
    );
    context
        .store
        .delete_external_authentication(
            context.owner,
            &DeleteAuthentication {
                scope: AuthenticationScope::Personal,
                authentication_id: id.clone(),
                request_id: "wire-delete-applied".into(),
                expected_request_id: "wire-replace-key".into(),
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    assert_eq!(
        client
            .acquire_authentication(authenticated(target.clone(), context.key)?)
            .await
            .err()
            .ok_or("Deleted record replaced with saved choice")?
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .begin_authentication(authenticated(begin, context.key)?)
            .await
            .err()
            .ok_or("Deleted binding revived")?
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .get_authentication_binding(authenticated(target.clone(), context.key)?)
            .await?
            .into_inner()
            .applied,
        Some(applied)
    );
    let oauth = oauth(context).await?;
    let saved = context
        .store
        .agent(
            context.owner,
            &context.agent.id,
            AcceptanceTime::System.now()?,
        )
        .await?
        .authentication
        .ok_or("Missing future selection")?;
    context
        .store
        .select_agent_authentication(
            context.owner,
            &SelectAgentAuthentication {
                agent_id: context.agent.id.clone(),
                authentication_id: oauth.clone(),
                request_id: "wire-oauth-selection".into(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: saved.authentication_id,
                    request_id: saved.request_id,
                }),
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    let oauth_target = prepare(context, "wire-oauth-start").await?;
    seal(context, &oauth_target).await?;
    client
        .begin_authentication(authenticated(
            c::BeginAuthenticationRequest {
                target: Some(oauth_target.clone()),
                expected_previous_operation_id: Some(target.operation_id),
            },
            context.key,
        )?)
        .await?;
    client
        .get_authentication_binding(authenticated(oauth_target.clone(), context.key)?)
        .await?;
    assert_eq!(provider.refreshes.load(Ordering::SeqCst), 0);
    let descriptor = acquire(&mut client, &oauth_target, context.key).await?;
    assert_eq!(provider.refreshes.load(Ordering::SeqCst), 1);
    assert_eq!(
        descriptor.kind,
        c::ExecutionCredentialKind::ChatgptTokens as i32
    );
    let file =
        String::from_utf8(read(&mut client, &oauth_target, context.key, &descriptor).await?)?;
    assert!(file.contains("\"refresh_token\":\"\""));
    assert!(!file.contains("fixture-server-refresh-bundle"));
    assert!(!file.contains("fixture-refreshed-server-bundle"));
    acquire(&mut client, &oauth_target, context.key).await?;
    assert_eq!(provider.refreshes.load(Ordering::SeqCst), 1);
    Ok(())
}
