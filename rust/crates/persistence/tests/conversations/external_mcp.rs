use super::*;
use chapchoo_persistence::{
    AgentIntegrationChange, ConnectionStartupAdmission, ConversationDeliveryTarget,
    ConversationInputPreparation, ConversationPreparationOutcome, McpAppliedScope, McpAuthEdit,
    McpBearer, McpEndpoint, McpExecutionScope, McpPreparationScope, McpRetainedSelection,
    McpSettingsDraft,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::AsyncWriteExt;

#[path = "external_mcp/held_https.rs"]
mod held_https;

const TOKEN: &str = "synthetic-admitted-mcp-bearer";
const NEXT_TOKEN: &str = "synthetic-replaced-mcp-bearer";

#[derive(Clone, Deserialize, Serialize)]
enum FixtureDatabase {
    Sqlite(PathBuf),
    Postgres(String),
}

impl FixtureDatabase {
    async fn open(&self) -> Result<Store, TestError> {
        let key = cipher()?;
        Ok(match self {
            Self::Sqlite(path) => Store::connect_sqlite(path.clone(), key).await?,
            Self::Postgres(url) => Store::connect_postgres(url.clone(), key).await?,
        })
    }
}

#[derive(Deserialize, Serialize)]
struct DeleteCommand {
    database: FixtureDatabase,
    actor: [u8; 32],
    agent: String,
    integration: String,
    now: i64,
}

async fn commit_in_owner_process(command: DeleteCommand) -> Result<(), TestError> {
    let mut child = tokio::process::Command::new(std::env::current_exe()?)
        .args(["--exact", "external_mcp::owner_process", "--ignored"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    assert_ne!(child.id(), Some(std::process::id()));
    let mut input = child.stdin.take().ok_or("Missing fixture command pipe")?;
    let bytes = zeroize::Zeroizing::new(serde_json::to_vec(&command)?);
    tokio::time::timeout(Duration::from_secs(10), async {
        input.write_all(&bytes).await?;
        drop(input);
        child.wait().await
    })
    .await??
    .success()
    .then_some(())
    .ok_or_else(|| "Owner process did not acknowledge a committed deletion".into())
}

#[test]
#[ignore = "entry point for the disposable database test subprocess"]
fn owner_process() -> Result<(), TestError> {
    let command: DeleteCommand = serde_json::from_reader(std::io::stdin().lock())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = command.database.open().await?;
        let actor = chapchoo_crypto::SecretDigest::from_bytes(command.actor);
        let delete = AgentIntegrationChange::Delete {
            id: command.integration,
        };
        store
            .change_agent_integration(
                &actor,
                &command.agent,
                "external-mcp-delete",
                &delete,
                command.now,
            )
            .await?;
        assert!(
            store
                .agent_mcp_integrations(&actor, &command.agent, None, command.now)
                .await?
                .items
                .is_empty()
        );
        Ok(())
    })
}

fn mcp_draft(endpoint: McpEndpoint, token: &str) -> Result<McpSettingsDraft, StoreError> {
    Ok(McpSettingsDraft {
        display_name: "External MCP fixture".into(),
        endpoint,
        auth: McpAuthEdit::ReplaceBearer(McpBearer::parse(token.into())?),
    })
}

async fn exercise(database: FixtureDatabase) -> Result<(), TestError> {
    let upstream = held_https::HeldHttps::open().await?;
    let client = database.open().await?;
    let control = database.open().await?;
    let f = fixture(&client).await?;
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let route = control
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Expected component route")?;
    let startup = control
        .admit_connection_startup(&f.agent.computer.id, &route)
        .await?
        .pop()
        .ok_or("Expected current preparation")?;
    let ConnectionStartupAdmission::Dispatch { operation_id, .. } = startup else {
        return Err("Expected dispatchable preparation".into());
    };
    let conversation = client
        .current_conversation(&owner, &f.agent.id, f.now)
        .await?
        .ok_or("Expected private conversation")?;
    let scope = McpPreparationScope {
        execution: McpExecutionScope {
            agent_id: &f.agent.id,
            conversation_id: &conversation.id,
            execution_id: &f.execution,
        },
        operation_id: &operation_id,
    };
    let create = AgentIntegrationChange::CreateMcp(mcp_draft(upstream.endpoint()?, TOKEN)?);
    let saved = client
        .change_agent_integration(&owner, &f.agent.id, "external-mcp-create", &create, f.now)
        .await?;
    let selected = control
        .capture_mcp_settings_for_preparation(&source, &scope, None)
        .await?;
    assert_eq!(selected.items.len(), 1);
    let initial = selected
        .items
        .into_iter()
        .next()
        .ok_or("Missing saved settings")?;
    assert_eq!(initial.integration_id, saved.id);
    assert!(!format!("{initial:?}").contains(TOKEN));
    let initial_keys = [(saved.id.as_str(), &initial.settings_key)];
    assert_eq!(
        control
            .current_mcp_preparation_candidates(&source, &scope, &initial_keys)
            .await?,
        [true],
    );
    let wrong = McpPreparationScope {
        execution: McpExecutionScope {
            agent_id: &f.other_agent,
            ..scope.execution
        },
        operation_id: &operation_id,
    };
    let retained = |agent_id| McpRetainedSelection {
        preparation: McpPreparationScope {
            execution: McpExecutionScope {
                agent_id,
                ..scope.execution
            },
            operation_id: &operation_id,
        },
        session_id: &f.session,
        control_id: &f.control,
        connection_id: &f.connection,
    };
    assert_eq!(
        control
            .retained_mcp_selections(&[retained(&f.agent.id), retained(&f.other_agent)])
            .await?,
        [true, false],
    );
    assert!(matches!(
        control
            .capture_mcp_settings_for_preparation(&source, &wrong, None)
            .await,
        Err(StoreError::NotFound)
    ));
    let update = AgentIntegrationChange::UpdateMcp {
        id: saved.id.clone(),
        draft: mcp_draft(upstream.endpoint()?, NEXT_TOKEN)?,
    };
    client
        .change_agent_integration(&owner, &f.agent.id, "external-mcp-replace", &update, f.now)
        .await?;
    assert_eq!(
        control
            .current_mcp_preparation_candidates(&source, &scope, &initial_keys)
            .await?,
        [false],
    );
    let current = control
        .capture_mcp_settings_for_preparation(&source, &scope, None)
        .await?
        .items
        .into_iter()
        .next()
        .ok_or("Missing replacement settings")?;
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation.id,
        operation_id: &operation_id,
    };
    let target = ConversationDeliveryTarget {
        snapshot: snapshot("external-mcp-snapshot")?,
        native_session: NativeConversationId::parse("external-mcp-session".into())?,
    };
    let applied = McpAppliedScope {
        preparation: McpPreparationScope {
            execution: McpExecutionScope { ..scope.execution },
            operation_id: &operation_id,
        },
        target: &target,
    };
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(&source, &applied, &saved.id, &current.settings_key)
            .await,
        Err(StoreError::InvalidTransition),
    ));
    let instructions =
        chapchoo_domain::ProjectInstructions::new(vec![chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Synthetic external MCP fixture.".into(),
        }])?;
    control
        .seal_startup_project_instructions(&route, &draft, &instructions, br#"{"tools":[]}"#, f.now)
        .await?;
    control
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: conversation.id.clone(),
                expected_previous: None,
                snapshot: target.snapshot.clone(),
                native_session: target.native_session.clone(),
            },
        )
        .await?;
    control
        .complete_conversation_source(&source, &conversation.id, &target.snapshot, None)
        .await?;
    control
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        control
            .current_mcp_preparation_candidates(&source, &scope, &[])
            .await,
        Err(StoreError::InvalidTransition),
    ));
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(&source, &applied, &saved.id, &initial.settings_key)
            .await,
        Err(StoreError::Conflict),
    ));
    let admitted = control
        .admit_mcp_settings_for_call(&source, &applied, &saved.id, &current.settings_key)
        .await?;
    // The fixture releases the HTTPS reply only after deletion and the denied
    // new admission have both completed. The already admitted call can finish.
    upstream
        .while_response_pending(admitted, async {
            commit_in_owner_process(DeleteCommand {
                database,
                actor: *owner.as_bytes(),
                agent: f.agent.id.clone(),
                integration: saved.id.clone(),
                now: f.now,
            })
            .await?;
            assert!(matches!(
                control
                    .admit_mcp_settings_for_call(
                        &source,
                        &applied,
                        &saved.id,
                        &current.settings_key
                    )
                    .await,
                Err(StoreError::NotFound)
            ));
            Ok(())
        })
        .await?;
    // The last deletion still needs a fresh, empty selection before new work
    // can use this same Chat. It does not eagerly disturb idle owners.
    let catalogue = br#"{"tools":[]}"#;
    assert!(
        control
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation.id,
                &instructions,
                catalogue,
                f.now
            )
            .await?
            .is_none()
    );
    let receipt = client
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.id.clone()),
                request_id: "prompt-after-mcp-deletion".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse(
                    "Continue with the current tool configuration.".into(),
                )?,
            },
            f.now,
        )
        .await?;
    let claim = f.claim(&client, &conversation.id, &receipt.id).await?;
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &conversation.id,
        operation_id: &receipt.id,
        target: &target,
    };
    for (ordinal, occurrence, foreground, work_id) in [
        (
            1,
            "external-running",
            chapchoo_persistence::ConversationForeground::Running,
            Some("active-work".into()),
        ),
        (
            2,
            "external-idle",
            chapchoo_persistence::ConversationForeground::Idle,
            None,
        ),
    ] {
        let position = position(ordinal, occurrence)?;
        control
            .ingest_conversation_pair(
                &source,
                &conversation.id,
                &CapturedEventPair {
                    snapshot: target.snapshot.clone(),
                    position: position.clone(),
                    raw: b"native foreground".to_vec(),
                    normalized: b"state".to_vec(),
                },
            )
            .await?;
        let running = work_id.is_some();
        control
            .observe_conversation_native(
                &source,
                &conversation.id,
                &chapchoo_persistence::ConversationNativeObservation {
                    target: target.clone(),
                    position,
                    foreground,
                    work_id,
                    preparation: None,
                    authentication_id: None,
                    applied_supply: Some(HarnessSupply::Managed),
                },
            )
            .await?;
        if running {
            assert!(
                control
                    .admit_project_refresh(
                        &route,
                        &f.agent.id,
                        &conversation.id,
                        &instructions,
                        catalogue,
                        f.now
                    )
                    .await?
                    .is_none()
            );
        }
    }
    assert!(matches!(
        client
            .begin_conversation_input_forwarding(&owner, &route, &forwarding, f.now, &claim)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: successor,
        context,
        ..
    }) = control
        .admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation.id,
            &instructions,
            catalogue,
            f.now,
        )
        .await?
    else {
        return Err("Expected exact next-input preparation after deletion".into());
    };
    assert!(
        matches!(context, chapchoo_persistence::ConnectionStartupContext::Existing { snapshot, native_session }
        if snapshot == target.snapshot && native_session == target.native_session)
    );
    assert!(
        control
            .capture_mcp_settings_for_preparation(
                &source,
                &McpPreparationScope {
                    execution: McpExecutionScope { ..scope.execution },
                    operation_id: &successor,
                },
                None
            )
            .await?
            .items
            .is_empty()
    );
    assert!(matches!(
        client
            .begin_conversation_input_forwarding(&owner, &route, &forwarding, f.now, &claim)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let current_setting = client
        .change_agent_integration(
            &owner,
            &f.agent.id,
            "race-fixture-add",
            &AgentIntegrationChange::CreateMcp(mcp_draft(current.target.endpoint.clone(), TOKEN)?),
            f.now,
        )
        .await?;
    let serving_scope = McpAppliedScope {
        preparation: McpPreparationScope {
            execution: McpExecutionScope { ..scope.execution },
            operation_id: &successor,
        },
        target: &target,
    };
    let key = control
        .capture_mcp_settings_for_preparation(&source, &serving_scope.preparation, None)
        .await?
        .items
        .pop()
        .ok_or("Missing replacement fixture integration")?
        .settings_key;
    control
        .complete_conversation_input_preparation(
            &source,
            &ConversationInputPreparation {
                operation_id: &successor,
                ..draft
            },
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(
        control
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation.id,
                &instructions,
                catalogue,
                f.now
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        client
            .begin_conversation_input_forwarding(&owner, &route, &forwarding, f.now, &claim)
            .await?,
        chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
    ));
    let second = client
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.id.clone()),
                request_id: "second-prompt-while-observation-lags".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse(
                    "A second input while state publication lags.".into(),
                )?,
            },
            f.now,
        )
        .await?;
    assert!(
        control
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation.id,
                &instructions,
                catalogue,
                f.now
            )
            .await?
            .is_none(),
        "Possible forwarding must preserve the current scope",
    );
    control
        .complete_conversation_input_forwarding(
            &source,
            &forwarding,
            &chapchoo_persistence::ConversationInputOutcome::Inserted {
                native_message_id: "fixture-native-message".into(),
            },
        )
        .await?;
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: pending,
        ..
    }) = control
        .admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation.id,
            &instructions,
            catalogue,
            f.now,
        )
        .await?
    else {
        return Err("Expected next preparation while the previous native report lags".into());
    };
    let pending_scope = McpAppliedScope {
        preparation: McpPreparationScope {
            execution: McpExecutionScope { ..scope.execution },
            operation_id: &pending,
        },
        target: &target,
    };
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(&source, &pending_scope, &current_setting.id, &key)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    control
        .admit_mcp_settings_for_call(&source, &serving_scope, &current_setting.id, &key)
        .await?;
    assert_eq!(
        control
            .retained_mcp_selections(&[McpRetainedSelection {
                preparation: McpPreparationScope {
                    execution: McpExecutionScope { ..scope.execution },
                    operation_id: &successor,
                },
                session_id: &f.session,
                control_id: &f.control,
                connection_id: &f.connection,
            }])
            .await?,
        [true],
    );
    control
        .complete_conversation_input_preparation(
            &source,
            &ConversationInputPreparation {
                operation_id: &pending,
                ..draft
            },
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(&source, &serving_scope, &current_setting.id, &key)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    control
        .admit_mcp_settings_for_call(&source, &pending_scope, &current_setting.id, &key)
        .await?;
    // The report may have committed even if its acknowledgement was lost.
    // After replacement, the exact report is recoverable, but the old
    // Control selection is no longer retained on its original route.
    let replacement = control
        .prepare_connector_connection(&f.session, &credential, &f.control)
        .await?;
    control
        .admit_connector_connection(
            &f.session,
            &credential,
            replacement.expected.as_ref(),
            &replacement.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let next_source = ConnectorSource {
        connection_id: &replacement.owner.connection_id,
        ..source
    };
    let exact_report = ConversationInputPreparation {
        operation_id: &pending,
        ..draft
    };
    let ready = ConversationPreparationOutcome::SessionReady(target.clone());
    assert!(
        control
            .complete_conversation_input_preparation(&source, &exact_report, &ready)
            .await
            .is_err()
    );
    control
        .complete_conversation_input_preparation(&next_source, &exact_report, &ready)
        .await?;
    assert!(matches!(
        control
            .complete_conversation_input_preparation(
                &next_source,
                &exact_report,
                &ConversationPreparationOutcome::Failed(
                    chapchoo_persistence::ConversationPreparationFailure::BeforeNativeLaunch,
                )
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        control
            .admit_mcp_settings_for_call(&source, &pending_scope, &current_setting.id, &key)
            .await
            .is_err()
    );
    assert_eq!(
        control
            .retained_mcp_selections(&[McpRetainedSelection {
                preparation: pending_scope.preparation,
                session_id: &f.session,
                control_id: &f.control,
                connection_id: &f.connection,
            }])
            .await?,
        [false],
    );
    let next_route = control
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Expected replacement component route")?;
    let next_claim = f.claim(&client, &conversation.id, &second.id).await?;
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        operation_id: &second.id,
        ..forwarding
    };
    assert!(
        matches!(
            client
                .begin_conversation_input_forwarding(
                    &owner,
                    &next_route,
                    &forwarding,
                    f.now,
                    &next_claim
                )
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "An old-route preparation must not forward a new input before the replacement route applies its own selection"
    );
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: rebuilt,
        ..
    }) = control
        .admit_project_refresh(
            &next_route,
            &f.agent.id,
            &conversation.id,
            &instructions,
            catalogue,
            f.now,
        )
        .await?
    else {
        return Err("Expected a fresh selection for the same input on the new route".into());
    };
    assert_ne!(pending, rebuilt);
    // This time the application on the old route finished but its first
    // report never reached the database before replacement.
    let replacement = control
        .prepare_connector_connection(&f.session, &credential, &f.control)
        .await?;
    control
        .admit_connector_connection(
            &f.session,
            &credential,
            replacement.expected.as_ref(),
            &replacement.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let current_source = ConnectorSource {
        connection_id: &replacement.owner.connection_id,
        ..source
    };
    let current_route = control
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Expected current component route")?;
    let rebuilt_report = ConversationInputPreparation {
        operation_id: &rebuilt,
        ..draft
    };
    control
        .report_conversation_input_preparation(
            &current_source,
            &rebuilt_report,
            &ready,
            Some(next_source.connection_id),
            true,
        )
        .await?;
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(
                &current_source,
                &McpAppliedScope {
                    preparation: McpPreparationScope {
                        execution: McpExecutionScope { ..scope.execution },
                        operation_id: &rebuilt,
                    },
                    target: &target,
                },
                &current_setting.id,
                &key,
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        client
            .begin_conversation_input_forwarding(
                &owner,
                &current_route,
                &forwarding,
                f.now,
                &next_claim,
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: current,
        ..
    }) = control
        .admit_project_refresh(
            &current_route,
            &f.agent.id,
            &conversation.id,
            &instructions,
            catalogue,
            f.now,
        )
        .await?
    else {
        return Err("Expected replacement preparation after recovered report".into());
    };
    assert_ne!(rebuilt, current);
    let final_report = ConversationInputPreparation {
        operation_id: &current,
        ..draft
    };
    // Computers that do not report optional application still yield native
    // readiness. They cannot open an external call, even with exact settings.
    control
        .report_conversation_input_preparation(
            &current_source,
            &final_report,
            &ready,
            Some(current_source.connection_id),
            false,
        )
        .await?;
    let final_scope = McpAppliedScope {
        preparation: McpPreparationScope {
            execution: McpExecutionScope { ..scope.execution },
            operation_id: &current,
        },
        target: &target,
    };
    assert!(matches!(
        control
            .admit_mcp_settings_for_call(&current_source, &final_scope, &current_setting.id, &key)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        client
            .begin_conversation_input_forwarding(
                &owner,
                &current_route,
                &forwarding,
                f.now,
                &next_claim
            )
            .await?,
        chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
    ));
    Ok(())
}

#[test]
fn sqlite_external_mcp_replace_delete_and_admit_across_connections() -> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("external_mcp.sqlite");
    let key = cipher()?;
    prepare_sqlite(&path, &key)?;
    tokio::runtime::Runtime::new()?.block_on(exercise(FixtureDatabase::Sqlite(path)))
}

#[test]
fn postgres_external_mcp_replace_delete_and_admit_across_connections() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("external_mcp_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let key = cipher()?;
        prepare_postgres(&url, &key)?;
        tokio::runtime::Runtime::new()?.block_on(exercise(FixtureDatabase::Postgres(url)))
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
