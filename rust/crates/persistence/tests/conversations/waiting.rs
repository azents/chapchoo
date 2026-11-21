use super::*;
use chapchoo_persistence::{
    ConversationInputCandidate, ConversationInputPreparation, ConversationInputQueueState,
    ConversationPreparationAdmission,
};

macro_rules! verify_migration {
    ($connection:expr, $migrations:expr, $identity:expr) => {{
        let connection = &mut *$connection;
        #[derive(diesel::QueryableByName, Debug, PartialEq)]
        struct Original {
            #[diesel(sql_type=diesel::sql_types::Text)]
            id: String,
            #[diesel(sql_type=diesel::sql_types::Text)]
            request_id: String,
            #[diesel(sql_type=diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            submitted_text: Option<String>,
            #[diesel(sql_type=diesel::sql_types::BigInt)]
            accepted_at: i64,
            #[diesel(sql_type=diesel::sql_types::Text)]
            queue_state: String,
        }
        #[derive(diesel::QueryableByName)]
        struct Count {
            #[diesel(sql_type=diesel::sql_types::BigInt)]
            n: i64,
        }
        let originals = format!("SELECT {} AS id,request_id,submitted_text,accepted_at,queue_state FROM conversation_inputs ORDER BY request_id", $identity);
        let before = diesel::sql_query(&originals).load::<Original>(connection)?;
        let native_facts = "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_input_preparation";
        let facts = diesel::sql_query(native_facts).get_result::<Count>(connection)?.n;
        // Remove implicit fixture Work to isolate the older queue migration.
        connection.batch_execute("DELETE FROM conversation_goals WHERE EXISTS (SELECT 1 FROM conversation_work_items w WHERE w.conversation_id=conversation_goals.conversation_id AND w.goal_key=conversation_goals.goal_key AND w.title='')")?;
        for expected in ["00000000000032", "00000000000031", "00000000000030", "00000000000029", "00000000000028"] {
            assert_eq!(connection.revert_last_migration($migrations)?.to_string(), expected);
        }
        assert!(connection.revert_last_migration($migrations).is_err(),
            "Qualified waiting work must prevent downgrade even after session removal");
        // Retire only the synthetic authority when establishing the historical
        // baseline; original rows and native facts remain byte-for-byte intact.
        connection.batch_execute("UPDATE conversation_inputs SET readiness_waiting=FALSE,submitting_session_id=NULL,wake_predecessor=NULL,consumer_claim_id=NULL")?;
        assert_eq!(connection.revert_last_migration($migrations)?.to_string(), "00000000000027");
        assert_eq!(diesel::sql_query(&originals).load::<Original>(connection)?, before);
        connection.run_pending_migrations($migrations)?;
        assert_eq!(diesel::sql_query(&originals).load::<Original>(connection)?, before);
        assert_eq!(diesel::sql_query(native_facts).get_result::<Count>(connection)?.n, facts);
        assert_eq!(diesel::sql_query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_inputs WHERE readiness_waiting=TRUE OR submitting_session_id IS NOT NULL OR wake_predecessor IS NOT NULL OR consumer_claim_id IS NOT NULL")
            .get_result::<Count>(connection)?.n, 0, "Migration must not grant historical attempts new replay authority");
    }};
}

fn candidate(
    f: &Fixture,
    input: &chapchoo_persistence::ConversationInputReceipt,
) -> ConversationInputCandidate {
    ConversationInputCandidate {
        id: input.id.clone(),
        agent_id: f.agent.id.clone(),
        conversation_id: input.conversation_id.clone(),
    }
}
async fn accept(
    store: &Store,
    f: &Fixture,
    request: &str,
) -> Result<chapchoo_persistence::ConversationInputReceipt, TestError> {
    accept_at(store, f, request, now()?).await
}
async fn accept_at(
    store: &Store,
    f: &Fixture,
    request: &str,
    accepted_at: i64,
) -> Result<chapchoo_persistence::ConversationInputReceipt, TestError> {
    Ok(store
        .accept_conversation_input(
            &f.owner.digest(),
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: request.into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse(format!("  {request}\n한글🙂  "))?,
            },
            accepted_at,
        )
        .await?)
}

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut sql: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let first = accept(store, f, "waiting-first").await?;
    let second = accept(store, f, "waiting-second").await?;
    let one = candidate(f, &first);
    let two = candidate(f, &second);
    sql("UPDATE conversation_inputs SET input_order=NULL WHERE request_id='waiting-first'")?;
    assert!(
        store
            .claim_conversation_input(&two, DirectoryPolicy::default())
            .await?
            .is_none(),
        "A historical NULL-order PENDING predecessor must still fence new FIFO claims"
    );
    sql("UPDATE conversation_inputs SET input_order=1 WHERE request_id='waiting-first'")?;
    assert!(
        store
            .claim_conversation_input(&two, DirectoryPolicy::default())
            .await?
            .is_none()
    );
    let (left, right) = tokio::join!(
        store.claim_conversation_input(&one, DirectoryPolicy::default()),
        store.claim_conversation_input(&one, DirectoryPolicy::default()),
    );
    let claim = match (left?, right?) {
        (Some(claim), None) | (None, Some(claim)) => claim,
        _ => return Err("Exactly one server must own the FIFO head".into()),
    };
    assert_eq!(claim.receipt.text, first.text);
    let authority = chapchoo_persistence::ConversationInputAuthority {
        agent_id: f.agent.id.clone(),
        conversation_id: first.conversation_id.clone(),
        input_id: first.id.clone(),
        claim_id: claim.claim_id.clone(),
    };
    sql(
        "CREATE TEMP TABLE pending_rotation AS SELECT id,current_hash,previous_hash,previous_valid_until FROM auth_sessions; UPDATE auth_sessions SET current_hash=(SELECT current_payload_hash FROM computers WHERE current_request_id='conversation-create'),previous_hash=NULL,previous_valid_until=NULL WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='waiting-first')",
    )?;
    assert!(matches!(
        store.current_identity(&f.owner.digest(), now()?).await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(
        store.queued_conversation_input_pending(&authority).await?,
        "A still-valid submitting session survives loss of the captured credential overlap"
    );
    assert_eq!(
        store
            .queued_current_conversation(&authority)
            .await?
            .ok_or("Missing bound current conversation")?
            .id,
        first.conversation_id,
    );
    let mut foreign = authority.clone();
    foreign.agent_id = f.other_agent.clone();
    assert!(
        matches!(
            store.queued_current_conversation(&foreign).await,
            Err(StoreError::NotFound)
        ),
        "A valid claim cannot authorize another private Agent"
    );
    sql(
        "UPDATE auth_sessions SET current_hash=(SELECT current_hash FROM pending_rotation b WHERE b.id=auth_sessions.id),previous_hash=(SELECT previous_hash FROM pending_rotation b WHERE b.id=auth_sessions.id),previous_valid_until=(SELECT previous_valid_until FROM pending_rotation b WHERE b.id=auth_sessions.id); DROP TABLE pending_rotation",
    )?;
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
    };
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &f.owner.digest(),
                &route,
                &draft,
                now()?,
                &second.id,
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .release_conversation_input_claim(&first.conversation_id, &first.id, &claim.claim_id, false)
        .await?;
    let recovered = store
        .claim_conversation_input(&one, DirectoryPolicy::default())
        .await?
        .ok_or("Missing recovered FIFO claim")?;
    assert_ne!(claim.claim_id, recovered.claim_id);
    assert!(
        matches!(
            store.queued_conversation_input_pending(&authority).await,
            Err(StoreError::InvalidTransition)
        ),
        "An old observer cannot borrow its successor claim"
    );
    store
        .release_conversation_input_claim(&first.conversation_id, &first.id, &claim.claim_id, true)
        .await?;
    assert_eq!(
        store
            .renew_conversation_input_consumer(
                &first.conversation_id,
                &first.id,
                &claim.claim_id,
                DirectoryPolicy::default()
            )
            .await?,
        ConversationInputQueueState::Absent
    );
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &f.owner.digest(),
                &route,
                &draft,
                now()?,
                &claim.claim_id,
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &f.owner.digest(),
                &route,
                &draft,
                now()?,
                &recovered.claim_id,
            )
            .await?,
        ConversationPreparationAdmission::Granted { .. }
    ));
    store
        .release_conversation_input_claim(
            &first.conversation_id,
            &first.id,
            &recovered.claim_id,
            false,
        )
        .await?;
    assert_eq!(
        store
            .conversation_input_queue_state(&first.conversation_id, &first.id)
            .await?,
        ConversationInputQueueState::Error
    );
    assert!(
        store
            .claim_conversation_input(&one, DirectoryPolicy::default())
            .await?
            .is_none(),
        "Native uncertainty must never regain a claim"
    );
    let next = store
        .claim_conversation_input(&two, DirectoryPolicy::default())
        .await?
        .ok_or("Error must not block later waiting messages")?;
    store
        .release_conversation_input_claim(
            &second.conversation_id,
            &second.id,
            &next.claim_id,
            false,
        )
        .await?;

    // Only this disposable fixture writes provider observations. No Provider is contacted.
    sql(&format!(
        "UPDATE computers SET desired_state='STOPPED',current_execution_id=NULL,current_request_id='waiting-stop',current_action='STOP',current_phase='COMPLETED',observed_power='STOPPED',power_observed_at={},power_source_connection_id=(SELECT connection_id FROM provider_connections LIMIT 1),power_source_execution_id=NULL,saved_environment='VERIFIED',environment_source_connection_id=(SELECT connection_id FROM provider_connections LIMIT 1)",
        now()?
    ))?;
    let wake_input = accept(store, f, "waiting-wake").await?;
    let wake_candidate = candidate(f, &wake_input);
    assert!(
        store.conversation_input_wake(&two).await?.is_none(),
        "Old backlog cannot acquire permission from a newer Stop"
    );
    assert!(
        store
            .claim_conversation_input(&two, DirectoryPolicy::default())
            .await?
            .is_none()
    );
    let wake = store
        .conversation_input_wake(&wake_candidate)
        .await?
        .ok_or("New message must retain Cloud wake context behind the FIFO head")?;
    assert_eq!(wake.predecessor, "waiting-stop");
    let cutoff = now()?;
    sql(&format!(
        "CREATE TEMP TABLE waiting_expiry AS SELECT id,expires_at FROM auth_sessions; UPDATE auth_sessions SET expires_at={cutoff}"
    ))?;
    assert!(
        matches!(
            store.start_waiting_conversation_input(&wake).await,
            Err(StoreError::Unauthenticated)
        ),
        "Lookup-time permission cannot grant Start after current session expiry"
    );
    sql(
        "UPDATE auth_sessions SET expires_at=(SELECT expires_at FROM waiting_expiry e WHERE e.id=auth_sessions.id); DROP TABLE waiting_expiry",
    )?;
    assert!(
        store
            .conversation_input_wake(&wake_candidate)
            .await?
            .is_none(),
        "Restoring a fixture session must not revive observed withdrawn work"
    );
    let wake_input = accept(store, f, "waiting-wake-valid").await?;
    let wake_candidate = candidate(f, &wake_input);
    let wake = store
        .conversation_input_wake(&wake_candidate)
        .await?
        .ok_or("Missing fresh wake")?;
    // Replace this same session's digest beyond prior-token overlap after
    // lookup. Queued admission must derive current bound-session authority.
    sql(
        "CREATE TEMP TABLE waiting_rotation AS SELECT id,current_hash,previous_hash,previous_valid_until FROM auth_sessions; UPDATE auth_sessions SET current_hash=(SELECT current_payload_hash FROM computers WHERE current_request_id='waiting-stop' LIMIT 1),previous_hash=NULL,previous_valid_until=NULL WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='waiting-wake-valid')",
    )?;
    let started = store.start_waiting_conversation_input(&wake).await?;
    assert_eq!(started.computer.current_request_id, wake.input_id);
    // Simulate loss of queue settlement after lifecycle admission. Recovery uses
    // the same accepted request rather than another Start or latest predecessor.
    let same = store
        .conversation_input_wake(&wake_candidate)
        .await?
        .ok_or("Lost settlement must recover the same wake")?;
    assert_eq!(same.input_id, wake.input_id);
    assert_eq!(same.predecessor, wake.predecessor);
    assert_eq!(
        store
            .start_waiting_conversation_input(&same)
            .await?
            .computer
            .current_request_id,
        started.computer.current_request_id
    );
    sql(
        "UPDATE auth_sessions SET current_hash=(SELECT current_hash FROM waiting_rotation r WHERE r.id=auth_sessions.id),previous_hash=(SELECT previous_hash FROM waiting_rotation r WHERE r.id=auth_sessions.id),previous_valid_until=(SELECT previous_valid_until FROM waiting_rotation r WHERE r.id=auth_sessions.id); DROP TABLE waiting_rotation",
    )?;
    // Establish the later completed owner intent in this input-authority fixture.
    sql(
        "UPDATE computers SET desired_state='STOPPED',current_request_id='newer-explicit-stop',current_action='STOP',current_phase='COMPLETED'",
    )?;
    assert!(
        store
            .conversation_input_wake(&wake_candidate)
            .await?
            .is_none(),
        "A newer explicit Stop withdraws old wake permission"
    );
    let new_message = accept(store, f, "after-new-stop").await?;
    let final_wake = store
        .conversation_input_wake(&candidate(f, &new_message))
        .await?
        .ok_or("Missing final authority fixture")?;
    assert_eq!(
        store
            .conversation_input_wake(&candidate(f, &new_message))
            .await?
            .ok_or("A genuinely new message can wake retained backlog")?
            .predecessor,
        "newer-explicit-stop"
    );

    let lookup_input = accept(store, f, "lookup-authority-withdrawn").await?;
    let claim_input = accept(store, f, "claim-authority-withdrawn").await?;
    sql(
        "CREATE TEMP TABLE waiting_team_owners AS SELECT id,owner_user_id FROM teams WHERE id IN (SELECT team_id FROM agents WHERE handle='chat-fixture'); UPDATE teams SET owner_user_id=(SELECT m.user_id FROM memberships m WHERE m.team_id=teams.id AND m.user_id<>teams.owner_user_id LIMIT 1) WHERE id IN (SELECT id FROM waiting_team_owners); CREATE TEMP TABLE waiting_memberships AS SELECT * FROM memberships WHERE team_id IN (SELECT id FROM waiting_team_owners) AND user_id IN (SELECT owner_user_id FROM waiting_team_owners); DELETE FROM memberships WHERE team_id IN (SELECT id FROM waiting_team_owners) AND user_id IN (SELECT owner_user_id FROM waiting_team_owners)",
    )?;
    assert!(matches!(
        store.start_waiting_conversation_input(&final_wake).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .conversation_input_wake(&candidate(f, &lookup_input))
            .await?
            .is_none()
    );
    assert!(
        store
            .claim_conversation_input(&candidate(f, &claim_input), DirectoryPolicy::default())
            .await?
            .is_none()
    );
    sql(
        "INSERT INTO memberships SELECT * FROM waiting_memberships; UPDATE teams SET owner_user_id=(SELECT owner_user_id FROM waiting_team_owners o WHERE o.id=teams.id) WHERE id IN (SELECT id FROM waiting_team_owners); DROP TABLE waiting_memberships; DROP TABLE waiting_team_owners",
    )?;
    for withdrawn in [&new_message, &lookup_input, &claim_input] {
        assert_eq!(
            store
                .conversation_input_queue_state(&withdrawn.conversation_id, &withdrawn.id)
                .await?,
            ConversationInputQueueState::Error
        );
        assert!(
            store
                .conversation_input_wake(&candidate(f, withdrawn))
                .await?
                .is_none()
        );
        assert!(
            store
                .claim_conversation_input(&candidate(f, withdrawn), DirectoryPolicy::default())
                .await?
                .is_none()
        );
    }
    let deleted = accept(store, f, "deleted-before-wake-admission").await?;
    let deleted_wake = store
        .conversation_input_wake(&candidate(f, &deleted))
        .await?
        .ok_or("Missing delete-race wake")?;
    store
        .fail_conversation_input(&deleted.conversation_id, &deleted.id)
        .await?;
    store
        .delete_failed_conversation_input(
            &f.owner.digest(),
            &chapchoo_persistence::ConversationInputScope {
                agent_id: &f.agent.id,
                conversation_id: &deleted.conversation_id,
                target: &ConversationTarget::NotStarted,
            },
            &deleted.request_id,
            now()?,
        )
        .await?;
    assert!(matches!(
        store.start_waiting_conversation_input(&deleted_wake).await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .agent(&f.owner.digest(), &f.agent.id, now()?)
            .await?
            .computer
            .current_request_id,
        "newer-explicit-stop"
    );

    // Canonical kind owns this exclusion, independent of the Kubernetes fixture.
    sql(
        "INSERT INTO computers(id,owner_user_id,kind,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) SELECT i.id,c.owner_user_id,'LOCAL','STOPPED','local-stop',c.current_payload_hash,'STOP','COMPLETED',c.created_at FROM conversation_inputs i JOIN conversations v ON v.id=i.conversation_id JOIN agents a ON a.id=v.agent_id JOIN computers c ON c.id=a.computer_id WHERE i.request_id='waiting-first'; UPDATE agents SET computer_id=(SELECT id FROM conversation_inputs WHERE request_id='waiting-first'),computer_kind='LOCAL' WHERE id=(SELECT agent_id FROM conversations WHERE id=(SELECT conversation_id FROM conversation_inputs WHERE request_id='waiting-first'))",
    )?;
    let local = accept(store, f, "local-waiting").await?;
    assert!(
        store
            .conversation_input_wake(&candidate(f, &local))
            .await?
            .is_none()
    );
    sql("UPDATE computers SET desired_state='RUNNING'")?;
    let mut later = std::collections::HashSet::new();
    let scan_time = now()?;
    for index in 0..40 {
        later.insert(
            accept_at(store, f, &format!("bounded-candidate-{index}"), scan_time)
                .await?
                .id,
        );
    }
    let frontier = store
        .waiting_conversation_input_frontier()
        .await?
        .ok_or("Missing scan frontier")?;
    let page = store.waiting_conversation_inputs(None, &frontier).await?;
    assert_eq!(page.len(), 32);
    // UUIDv7 random suffixes do not order same-millisecond insertions.
    let appended = accept_at(store, f, "appended-during-scan", scan_time + 1).await?;
    let after = page.last().ok_or("Missing candidate cursor")?.id.clone();
    let rest = store
        .waiting_conversation_inputs(Some(&after), &frontier)
        .await?;
    assert!(rest.len() <= 32);
    let discovered: std::collections::HashSet<_> =
        page.into_iter().chain(rest).map(|row| row.id).collect();
    assert!(
        later.is_subset(&discovered),
        "A blocked prefix must not hide later metadata candidates"
    );
    assert!(
        !discovered.contains(&appended.id),
        "Arrivals must not extend a frozen scan"
    );
    let before_revoke = store
        .claim_conversation_input(&two, DirectoryPolicy::default())
        .await?
        .ok_or("Manual readiness can resume Local waiting")?;
    sql("UPDATE auth_sessions SET revoked_at=1")?;
    let withdrawn = chapchoo_persistence::ConversationInputAuthority {
        agent_id: f.agent.id.clone(),
        conversation_id: second.conversation_id.clone(),
        input_id: second.id.clone(),
        claim_id: before_revoke.claim_id.clone(),
    };
    assert!(matches!(
        store.queued_conversation_input_pending(&withdrawn).await,
        Err(StoreError::Unauthenticated)
    ));
    let second_draft = ConversationInputPreparation {
        operation_id: &second.id,
        ..draft
    };
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &f.owner.digest(),
                &route,
                &second_draft,
                now()?,
                &before_revoke.claim_id,
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    store
        .release_conversation_input_claim(
            &second.conversation_id,
            &second.id,
            &before_revoke.claim_id,
            false,
        )
        .await?;
    assert!(
        store
            .claim_conversation_input(&two, DirectoryPolicy::default())
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_input_queue_state(&second.conversation_id, &second.id)
            .await?,
        ConversationInputQueueState::Error
    );
    // Independently isolate expiry on another already-accepted fixture input.
    sql("UPDATE auth_sessions SET revoked_at=NULL,created_at=0,expires_at=1")?;
    assert!(
        store
            .claim_conversation_input(&candidate(f, &local), DirectoryPolicy::default())
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_input_queue_state(&local.conversation_id, &local.id)
            .await?,
        ConversationInputQueueState::Error
    );
    // Session deletion removes the binding, never the accepted original or its
    // native uncertainty; the FK cannot make logout/purge fail.
    sql("DELETE FROM auth_sessions")?;
    assert!(
        store
            .claim_conversation_input(&candidate(f, &new_message), DirectoryPolicy::default())
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_input_queue_state(&new_message.conversation_id, &new_message.id)
            .await?,
        ConversationInputQueueState::Error
    );
    Ok(())
}

#[test]
fn sqlite_waiting_fifo_claim_and_wake_authority() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("waiting.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let f = fixture(&store).await?;
        exercise(&store, &f, |query| {
            connection.batch_execute(query)?;
            Ok(())
        })
        .await
    })?;
    verify_migration!(&mut connection, SQLITE_MIGRATIONS, "hex(id)");
    Ok(())
}
#[test]
fn postgres_waiting_fifo_claim_and_wake_authority() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!("waiting_{}", std::process::id());
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            let f = fixture(&store).await?;
            exercise(&store, &f, |query| {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| -> Result<(), TestError> {
                            connection.batch_execute(query)?;
                            Ok(())
                        })
                        .join()
                        .map_err(|_| "Fixture SQL worker failed")?
                })
            })
            .await
        })?;
        verify_migration!(&mut connection, POSTGRES_MIGRATIONS, "CAST(id AS TEXT)");
        Ok(())
    })();
    support::combine(
        result,
        control
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .map_err(Into::into),
    )
    .map(|_| ())
}

#[test]
fn postgres_wake_rechecks_session_after_lifecycle_lock_wait() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control =
        <AsyncConnectionWrapper<AsyncPgConnection> as diesel::Connection>::establish(&base)?;
    let schema = format!("waiting_lock_{}", std::process::id());
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            use diesel_async::{AsyncConnection, SimpleAsyncConnection};
            let store = Store::connect_postgres(url.clone(), cipher).await?;
            let f = fixture(&store).await?;
            let mut blocker = AsyncPgConnection::establish(&url).await?;
            let mut observer = AsyncPgConnection::establish(&url).await?;
            blocker.batch_execute(&format!(
                "UPDATE computers SET desired_state='STOPPED',current_execution_id=NULL,current_request_id='lock-stop',current_action='STOP',current_phase='COMPLETED',observed_power='STOPPED',power_observed_at={},power_source_connection_id=(SELECT connection_id FROM provider_connections LIMIT 1),power_source_execution_id=NULL,saved_environment='VERIFIED',environment_source_connection_id=(SELECT connection_id FROM provider_connections LIMIT 1)", now()?
            )).await?;
            let input = accept(&store, &f, "lock-expiry-wake").await?;
            let wake = store.conversation_input_wake(&candidate(&f, &input)).await?.ok_or("Missing lock-wait wake")?;
            let pid: BackendPid = diesel_async::RunQueryDsl::get_result(
                diesel::sql_query("SELECT pg_backend_pid() AS pid"), &mut blocker,
            ).await?;
            blocker.batch_execute("BEGIN; SELECT id FROM computers FOR UPDATE").await?;
            let processing = store.clone();
            let mut task = tokio::spawn(async move { processing.start_waiting_conversation_input(&wake).await });
            #[derive(diesel::QueryableByName)]
            struct Count { #[diesel(sql_type=diesel::sql_types::BigInt)] n: i64 }
            let observed: Result<(), TestError> = async {
                tokio::time::timeout(std::time::Duration::from_secs(5), async {
                    loop {
                        let wait: LockWait = diesel_async::RunQueryDsl::get_result(
                            diesel::sql_query("SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid))) AS blocked")
                                .bind::<diesel::sql_types::Integer,_>(pid.pid),
                            &mut observer,
                        ).await?;
                        if wait.blocked { return Ok::<(), TestError>(()); }
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                }).await??;
                // The session was valid at initial authentication. Its durable
                // expiry changes only after the real lifecycle lock wait exists.
                observer.batch_execute("UPDATE auth_sessions SET created_at=0,expires_at=1").await?;
                Ok(())
            }.await;
            let released = blocker.batch_execute("ROLLBACK").await;
            let completed = tokio::time::timeout(std::time::Duration::from_secs(5), &mut task).await;
            if completed.is_err() {
                task.abort();
                let _ = task.await;
            }
            observed?;
            released?;
            assert!(matches!(completed??, Err(StoreError::Unauthenticated)));
            assert_eq!(store.conversation_input_queue_state(&input.conversation_id, &input.id).await?, ConversationInputQueueState::Error);
            let count = diesel_async::RunQueryDsl::get_result::<Count>(
                diesel::sql_query("SELECT CAST(count(*) AS BIGINT) AS n FROM computers WHERE desired_state='RUNNING' OR current_request_id<>'lock-stop'"),
                &mut observer,
            ).await?;
            assert_eq!(count.n, 0, "Expired deferred authority must not produce Start intent");
            Ok(())
        })
    })();
    support::combine(
        result,
        control
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .map_err(Into::into),
    )
    .map(|_| ())
}
