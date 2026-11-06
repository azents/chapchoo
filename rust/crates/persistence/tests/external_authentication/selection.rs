use super::*;
use chapchoo_crypto::{BootstrapProviderPermit, SecretDigest};
use chapchoo_domain::{ConnectionStatus, LifecycleAction, ProviderKeyRequestId, ProviderSubstrate};
use chapchoo_persistence::{
    AgentSummary, ComponentKeyDraft, ComponentKeyPolicy, CreateAgentProfile, CreateCloudAgent,
    CreationOutcome, DirectoryPolicy, ExpectedAgentAuthentication, LifecycleRequest,
    ProviderCapabilities, ProviderOwner, SelectAgentAuthentication,
};

async fn register(
    store: &Store,
    actor: &SecretDigest,
    scope: &AuthenticationScope,
    request: &str,
    now: i64,
) -> Result<String, TestError> {
    record(
        store
            .register_external_api_key(
                actor,
                &RegisterApiKey {
                    scope: scope.clone(),
                    request_id: request.to_owned(),
                    name: request.to_owned(),
                    key: key("synthetic-selection-key")?,
                },
                now,
            )
            .await?,
    )
}
fn change(
    request: &str,
    agent: &str,
    authentication: &str,
    expected: Option<(&str, &str)>,
) -> SelectAgentAuthentication {
    SelectAgentAuthentication {
        request_id: request.to_owned(),
        agent_id: agent.to_owned(),
        authentication_id: authentication.to_owned(),
        expected: expected.map(|(authentication, request)| ExpectedAgentAuthentication {
            authentication_id: authentication.to_owned(),
            request_id: request.to_owned(),
        }),
    }
}
struct CreationReferences<'a> {
    image: &'a str,
    authentication: &'a str,
}
async fn create(
    store: &Store,
    actor: &SecretDigest,
    team: &str,
    provider: &str,
    references: CreationReferences<'_>,
    handle: &str,
    now: i64,
) -> Result<AgentSummary, TestError> {
    let outcome = store
        .create_cloud_agent(
            actor,
            &CreateCloudAgent {
                team_id: team.to_owned(),
                request_id: format!("create-{handle}"),
                provider_id: provider.to_owned(),
                name: handle.to_owned(),
                authentication_id: references.authentication.to_owned(),
                profile: CreateAgentProfile {
                    handle: handle.to_owned(),
                    image_id: references.image.to_owned(),
                    palette_key: "sage".to_owned(),
                },
            },
            now,
        )
        .await?;
    match outcome {
        CreationOutcome::Present(agent) => Ok(*agent),
        _ => Err("Expected a present Agent".into()),
    }
}
fn deletion(
    request: &str,
    id: &str,
    scope: &AuthenticationScope,
    expected: &str,
) -> DeleteAuthentication {
    DeleteAuthentication {
        scope: scope.clone(),
        request_id: request.to_owned(),
        authentication_id: id.to_owned(),
        expected_request_id: expected.to_owned(),
    }
}
async fn exercise_selection(store: &Store) -> Result<(), TestError> {
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
    )?;
    let sessions = seed_at(store, now).await?;
    let owner = sessions[0].digest();
    let member = sessions[1].digest();
    let stranger = sessions[2].digest();
    let member_user = store.current_identity(&member, now).await?.id;
    let personal = AuthenticationScope::Personal;
    let team = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Selections".to_owned(),
                handle: "selections".to_owned(),
            },
            now,
        )
        .await?;
    let other_team = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Other scope".to_owned(),
                handle: "other-scope".to_owned(),
            },
            now,
        )
        .await?;
    let invitation = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.id.clone(),
                recipient: EmailAddress::parse("auth1@example.invalid")?,
                signup: None,
            },
            now,
        )
        .await?;
    store
        .process_invitation(&member, &invitation.invitation.id, true, now)
        .await?;
    let shared = AuthenticationScope::Team {
        team_id: team.id.clone(),
    };
    let other_scope = AuthenticationScope::Team {
        team_id: other_team.id,
    };
    let shared_one = register(store, &owner, &shared, "shared-one", now).await?;
    let shared_two = register(store, &owner, &shared, "shared-two", now).await?;
    let personal_key = register(store, &owner, &personal, "personal-selection", now).await?;
    let member_key = register(store, &member, &personal, "member-personal", now).await?;
    let foreign_key = register(store, &stranger, &personal, "stranger-personal", now).await?;
    let other_team_key = register(store, &owner, &other_scope, "other-team-key", now).await?;

    let proof = BootstrapProviderPermit::new("selection-provider".to_owned())?.authenticated();
    let provider = store
        .rotate_provider_key(
            ProviderOwner::Bootstrap {
                proof: &proof,
                name: "Selections",
                substrate: &ProviderSubstrate::Kubernetes,
                capabilities: ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
            },
            "selection-provider",
            &ComponentKeyDraft {
                request_id: ProviderKeyRequestId::new(now, "selection-provider-key")?
                    .as_str()
                    .to_owned(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now,
        )
        .await?;
    let control = store
        .register_control("http://127.0.0.1:1", DirectoryPolicy::default())
        .await?;
    let connection = store
        .prepare_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            &control.id,
        )
        .await?;
    store
        .admit_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            connection.expected.as_ref(),
            &connection.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let image = store
        .agent_profile_images(&owner, None, 100, now)
        .await?
        .into_iter()
        .next()
        .ok_or("Missing profile image")?
        .id;
    let a = create(
        store,
        &owner,
        &team.id,
        &provider.provider_id,
        CreationReferences {
            image: &image,
            authentication: &personal_key,
        },
        "selection-a",
        now,
    )
    .await?;
    let b = create(
        store,
        &member,
        &team.id,
        &provider.provider_id,
        CreationReferences {
            image: &image,
            authentication: &member_key,
        },
        "selection-b",
        now,
    )
    .await?;
    let c = create(
        store,
        &owner,
        &team.id,
        &provider.provider_id,
        CreationReferences {
            image: &image,
            authentication: &personal_key,
        },
        "selection-c",
        now,
    )
    .await?;
    assert_eq!(
        a.authentication
            .as_ref()
            .ok_or("Missing creation selection")?
            .authentication_id,
        personal_key
    );
    assert_eq!(
        b.authentication
            .as_ref()
            .ok_or("Missing creation selection")?
            .authentication_id,
        member_key
    );
    assert_eq!(
        c.authentication
            .as_ref()
            .ok_or("Missing creation selection")?
            .authentication_id,
        personal_key
    );
    assert!(
        store
            .close_provider_connection(
                &provider.provider_id,
                &provider.key.token.digest(),
                &connection.owner
            )
            .await?
    );
    assert_eq!(
        store.agent(&owner, &a.id, now).await?.computer.management,
        ConnectionStatus::Disconnected
    );

    let first = change(
        "select-a-1",
        &a.id,
        &shared_one,
        Some((&personal_key, "create-selection-a")),
    );
    let (one, two) = tokio::join!(
        store.select_agent_authentication(&owner, &first, now),
        store.select_agent_authentication(&owner, &first, now)
    );
    let accepted = one?;
    assert_eq!(accepted, two?);
    let saved = store.agent(&owner, &a.id, now).await?;
    let reference = saved.authentication.ok_or("Missing saved reference")?;
    assert_eq!(reference.authentication_id, shared_one);
    assert_eq!(reference.request_id, "select-a-1");
    assert_eq!(
        saved.computer.current_request_id,
        a.computer.current_request_id
    );
    assert_eq!(saved.computer.management, ConnectionStatus::Disconnected);
    assert_eq!(
        format!("{reference:?}"),
        "AgentAuthenticationSelection([REDACTED])"
    );
    store
        .select_agent_authentication(
            &owner,
            &change(
                "select-c-1",
                &c.id,
                &shared_one,
                Some((&personal_key, "create-selection-c")),
            ),
            now,
        )
        .await?;
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &deletion("in-use-one", &shared_one, &shared, "shared-one"),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::InUse
    ));
    store
        .select_agent_authentication(
            &owner,
            &change(
                "select-a-2",
                &a.id,
                &shared_two,
                Some((&shared_one, "select-a-1")),
            ),
            now,
        )
        .await?;
    store
        .select_agent_authentication(
            &owner,
            &change(
                "select-a-3",
                &a.id,
                &shared_one,
                Some((&shared_two, "select-a-2")),
            ),
            now,
        )
        .await?;
    // Comparing both record and accepted request fences A -> B -> A stale intent.
    assert!(matches!(
        store
            .select_agent_authentication(
                &owner,
                &change(
                    "stale-aba",
                    &a.id,
                    &personal_key,
                    Some((&shared_one, "select-a-1"))
                ),
                now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .select_agent_authentication(&owner, &first, now)
            .await?,
        accepted
    );
    assert_eq!(
        store
            .agent(&owner, &a.id, now)
            .await?
            .authentication
            .ok_or("Missing reference")?
            .request_id,
        "select-a-3"
    );
    store
        .select_agent_authentication(
            &owner,
            &change(
                "select-a-personal",
                &a.id,
                &personal_key,
                Some((&shared_one, "select-a-3")),
            ),
            now,
        )
        .await?;
    for ineligible in [&foreign_key, &other_team_key] {
        assert!(matches!(
            store
                .select_agent_authentication(
                    &owner,
                    &change(
                        "ineligible",
                        &a.id,
                        ineligible,
                        Some((&personal_key, "select-a-personal"))
                    ),
                    now
                )
                .await,
            Err(StoreError::NotFound)
        ));
    }
    // Team ownership and credential use do not grant management of another User's Agent.
    assert!(matches!(
        store
            .select_agent_authentication(
                &owner,
                &change("owner-other-agent", &b.id, &shared_two, None),
                now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .select_agent_authentication(
                &member,
                &change("member-other-agent", &a.id, &shared_two, None),
                now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    store
        .select_agent_authentication(
            &member,
            &change(
                "select-b-team",
                &b.id,
                &shared_two,
                Some((&member_key, "create-selection-b")),
            ),
            now,
        )
        .await?;
    assert_eq!(
        store
            .agent(&member, &b.id, now)
            .await?
            .authentication
            .ok_or("Missing member selection")?
            .authentication_id,
        shared_two
    );
    store
        .rename_external_authentication(
            &owner,
            &RenameAuthentication {
                scope: shared.clone(),
                request_id: "rename-shared-one".to_owned(),
                authentication_id: shared_one.clone(),
                expected_request_id: "shared-one".to_owned(),
                name: "Renamed shared connection".to_owned(),
            },
            now,
        )
        .await?;
    let selected = store
        .agent(&owner, &c.id, now)
        .await?
        .authentication
        .ok_or("Missing C reference")?;
    assert_eq!(selected.name, "Renamed shared connection");
    assert_eq!(selected.request_id, "select-c-1");
    store
        .select_agent_authentication(
            &owner,
            &change(
                "select-c-2",
                &c.id,
                &shared_two,
                Some((&shared_one, "select-c-1")),
            ),
            now,
        )
        .await?;
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &deletion(
                    "delete-shared-one",
                    &shared_one,
                    &shared,
                    "rename-shared-one"
                ),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    // An exact completed request survives later selection changes and catalogue deletion.
    assert_eq!(
        store
            .select_agent_authentication(&owner, &first, now)
            .await?,
        accepted
    );
    assert_eq!(
        store
            .external_authentication_request_result(&owner, "select-a-1", now)
            .await?
            .ok_or("Missing selection receipt")?
            .accepted,
        accepted
    );
    store
        .replace_external_api_key(
            &owner,
            &ReplaceApiKey {
                scope: shared.clone(),
                request_id: "replace-shared-two".to_owned(),
                authentication_id: shared_two.clone(),
                expected_request_id: "shared-two".to_owned(),
                key: key("replacement-shared-key")?,
            },
            now,
        )
        .await?;
    assert_eq!(
        store
            .agent(&member, &b.id, now)
            .await?
            .authentication
            .ok_or("Lost replacement reference")?
            .authentication_id,
        shared_two
    );
    assert_eq!(
        store
            .agent(&owner, &c.id, now)
            .await?
            .authentication
            .ok_or("Lost shared reference")?
            .request_id,
        "select-c-2"
    );

    let racing = register(store, &owner, &shared, "race-register", now).await?;
    let select = change(
        "select-delete-race",
        &b.id,
        &racing,
        Some((&shared_two, "select-b-team")),
    );
    let delete = deletion("delete-select-race", &racing, &shared, "race-register");
    let (selection, deletion_result) = tokio::join!(
        store.select_agent_authentication(&member, &select, now),
        store.delete_external_authentication(&owner, &delete, now)
    );
    match (selection, deletion_result?) {
        (Ok(_), DeleteAuthenticationOutcome::InUse) => {
            store
                .select_agent_authentication(
                    &member,
                    &change(
                        "restore-member-team",
                        &b.id,
                        &shared_two,
                        Some((&racing, "select-delete-race")),
                    ),
                    now,
                )
                .await?;
            assert!(matches!(
                store
                    .delete_external_authentication(
                        &owner,
                        &deletion("delete-race-cleanup", &racing, &shared, "race-register"),
                        now
                    )
                    .await?,
                DeleteAuthenticationOutcome::Accepted(_)
            ));
        }
        (Err(StoreError::NotFound), DeleteAuthenticationOutcome::Accepted(_)) => {}
        _ => return Err("Selection/delete race violated the in-use contract".into()),
    }
    let saved = store
        .agent(&owner, &a.id, now)
        .await?
        .authentication
        .ok_or("Missing personal selection")?;
    let left = change(
        "selection-winner-left",
        &a.id,
        &shared_two,
        Some((&personal_key, &saved.request_id)),
    );
    let right = change(
        "selection-winner-right",
        &a.id,
        &shared_two,
        Some((&personal_key, &saved.request_id)),
    );
    let (left_result, right_result) = tokio::join!(
        store.select_agent_authentication(&owner, &left, now),
        store.select_agent_authentication(&owner, &right, now)
    );
    assert!(matches!(
        (left_result, right_result),
        (Ok(_), Err(StoreError::Conflict)) | (Err(StoreError::Conflict), Ok(_))
    ));
    let current = store
        .agent(&owner, &a.id, now)
        .await?
        .authentication
        .ok_or("Missing winner")?;
    store
        .select_agent_authentication(
            &owner,
            &change(
                "final-personal",
                &a.id,
                &personal_key,
                Some((&shared_two, &current.request_id)),
            ),
            now,
        )
        .await?;

    let member_selection = store
        .agent(&member, &b.id, now)
        .await?
        .authentication
        .ok_or("Missing member reference")?;
    store
        .select_agent_authentication(
            &member,
            &change(
                "select-b-personal",
                &b.id,
                &member_key,
                Some((&shared_two, &member_selection.request_id)),
            ),
            now,
        )
        .await?;
    store
        .remove_team_member(&owner, &team.id, &member_user, now)
        .await?;
    assert!(matches!(
        store
            .external_authentication_request_result(&member, "select-b-personal", now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .external_authentication(&member, &personal, &member_key, now)
            .await?
            .id,
        member_key
    );
    assert!(matches!(
        store
            .select_agent_authentication(
                &member,
                &change("after-departure", &b.id, &member_key, None),
                now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .delete_external_authentication(
                &member,
                &deletion(
                    "delete-member-key",
                    &member_key,
                    &personal,
                    "member-personal"
                ),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));

    let before = store.agent(&owner, &c.id, now).await?;
    store
        .request_lifecycle(
            &owner,
            &c.id,
            &LifecycleRequest {
                request_id: "remove-c".to_owned(),
                expected_request_id: before.computer.current_request_id,
                action: LifecycleAction::Delete,
            },
            now,
        )
        .await?;
    assert!(matches!(
        store
            .select_agent_authentication(
                &owner,
                &change(
                    "select-deleting",
                    &c.id,
                    &personal_key,
                    Some((&shared_two, "select-c-2"))
                ),
                now
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .force_remove_agent(&owner, &c.id, "remove-c", now)
        .await?;
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &deletion(
                    "delete-shared-two",
                    &shared_two,
                    &shared,
                    "replace-shared-two"
                ),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &deletion(
                    "personal-in-use",
                    &personal_key,
                    &personal,
                    "personal-selection"
                ),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::InUse
    ));
    register(store, &owner, &shared, "team-cleanup-unreferenced", now).await?;
    store.delete_team(&owner, &team.id, now).await?;
    assert!(matches!(
        store.agent(&owner, &a.id, now).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .external_authentication(&owner, &personal, &personal_key, now)
            .await?
            .id,
        personal_key
    );
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &deletion(
                    "delete-personal-after-team",
                    &personal_key,
                    &personal,
                    "personal-selection"
                ),
                now
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    Ok(())
}

#[test]
fn sqlite_agent_selection_is_owned_recoverable_and_serialized_with_deletion()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("selection.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        exercise_selection(&store).await
    })
}
#[test]
fn postgres_agent_selection_is_owned_recoverable_and_serialized_with_deletion()
-> Result<(), TestError> {
    let _fixture = POSTGRES_FIXTURE_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("selection_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            exercise_selection(&store).await
        })
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
