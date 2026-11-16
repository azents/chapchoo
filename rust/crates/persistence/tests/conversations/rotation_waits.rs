//! Exercise fresh queued authority after observable PostgreSQL lock waits.
use super::*;
use chapchoo_persistence::ConversationInputAuthority;

#[derive(diesel::QueryableByName)]
struct BackendPid {
    #[diesel(sql_type=diesel::sql_types::Integer)]
    pid: i32,
}
#[derive(diesel::QueryableByName)]
struct LockWait {
    #[diesel(sql_type=diesel::sql_types::Bool)]
    blocked: bool,
}

#[test]
fn postgres_queued_authority_rechecks_after_user_and_computer_waits() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!("rotation_waits_{}", std::process::id());
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
            let receipt = store.accept_conversation_input(&f.owner.digest(), &AcceptConversationInput {
                agent_id: f.agent.id.clone(), conversation_id: None,
                request_id: "rotation-wait-input".into(), target: ConversationTarget::NotStarted,
                text: ConversationText::parse("Preserved original input".into())?,
            }, now()?).await?;
            let authority = ConversationInputAuthority {
                agent_id: f.agent.id.clone(), conversation_id: receipt.conversation_id.clone(),
                input_id: receipt.id.clone(), claim_id: f.claim(&store, &receipt.conversation_id, &receipt.id).await?,
            };
            let mut blocker = AsyncPgConnection::establish(&url).await?;
            let mut observer = AsyncPgConnection::establish(&url).await?;
            let pid: BackendPid = diesel_async::RunQueryDsl::get_result(
                diesel::sql_query("SELECT pg_backend_pid() AS pid"), &mut blocker,
            ).await?;
            for owner_lock in ["user", "computer"] {
                for mutation in ["rotation", "expiry", "revocation"] {
                    observer.batch_execute(&format!("UPDATE auth_sessions SET expires_at={},revoked_at=NULL WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-wait-input')", now()? + 60_000)).await?;
                    let query = if owner_lock == "user" {
                        "BEGIN; SELECT u.id FROM users u JOIN auth_sessions s ON s.user_id=u.id JOIN conversation_inputs i ON i.submitting_session_id=s.id WHERE i.request_id='rotation-wait-input' FOR NO KEY UPDATE OF u"
                    } else {
                        "BEGIN; SELECT c.id FROM computers c JOIN agents a ON a.computer_id=c.id JOIN conversations v ON v.agent_id=a.id JOIN conversation_inputs i ON i.conversation_id=v.id WHERE i.request_id='rotation-wait-input' FOR NO KEY UPDATE OF c"
                    };
                    blocker.batch_execute(query).await?;
                    let processing = store.clone();
                    let bound = authority.clone();
                    let mut task = tokio::spawn(async move { processing.queued_conversation_input_pending(&bound).await });
                    let observed: Result<(), TestError> = async {
                        tokio::time::timeout(std::time::Duration::from_secs(5), async {
                            loop {
                                let wait: LockWait = diesel_async::RunQueryDsl::get_result(
                                    diesel::sql_query("SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid))) AS blocked")
                                        .bind::<diesel::sql_types::Integer,_>(pid.pid), &mut observer,
                                ).await?;
                                if wait.blocked { return Ok::<(), TestError>(()); }
                                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            }
                        }).await??;
                        // Only this isolated persistence fixture changes authority
                        // while the exact production operation is visibly waiting.
                        match mutation {
                            "rotation" => {
                                let latest = BearerSecret::generate()?;
                                diesel_async::RunQueryDsl::execute(diesel::sql_query("UPDATE auth_sessions SET current_hash=$1,previous_hash=NULL,previous_valid_until=NULL WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-wait-input')")
                                    .bind::<diesel::sql_types::Binary,_>(latest.digest().as_bytes().as_slice()), &mut blocker).await?;
                            }
                            "expiry" => blocker.batch_execute("UPDATE auth_sessions SET created_at=0,expires_at=1 WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-wait-input')").await?,
                            "revocation" => blocker.batch_execute("UPDATE auth_sessions SET revoked_at=1 WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-wait-input')").await?,
                            _ => return Err("Unknown fixture mutation".into()),
                        }
                        Ok(())
                    }.await;
                    let released = blocker.batch_execute("COMMIT").await;
                    let completed = tokio::time::timeout(std::time::Duration::from_secs(5), &mut task).await;
                    if completed.is_err() { task.abort(); let _ = task.await; }
                    observed?;
                    released?;
                    let completed = completed??;
                    if mutation == "rotation" {
                        assert!(completed?, "Valid bound session must survive {owner_lock} wait rotation");
                    } else {
                        assert!(matches!(completed, Err(StoreError::Unauthenticated)), "Withdrawal during {owner_lock} wait must remain terminal");
                    }
                }
            }
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
