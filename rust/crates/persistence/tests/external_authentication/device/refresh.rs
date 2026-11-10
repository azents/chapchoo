use super::*;
use chapchoo_persistence::{
    AuthenticationRefreshAdmission as Admission, AuthenticationRefreshState as RefreshState,
};
fn expiring() -> Result<DeviceAuthenticationTokens, TestError> {
    Ok(DeviceAuthenticationTokens {
        expires_at: Some(NOW + 1001),
        ..tokens(Some("refresh-account"))?
    })
}
pub(super) async fn exercise(
    store: &Store,
    owner: &SecretDigest,
    stranger: &SecretDigest,
) -> Result<(), TestError> {
    let scope = AuthenticationScope::Personal;
    let device = exchange(store, owner, &scope, "refresh-original", None).await?;
    let id = registered(
        store
            .complete_external_device_authentication(owner, &device, &expiring()?, NOW + 1001)
            .await?,
    )?;
    let foreign = store
        .begin_external_authentication_refresh(stranger, &scope, &id, NOW + 1001)
        .await;
    assert!(
        matches!(&foreign, Err(StoreError::NotFound)),
        "Unexpected foreign refresh admission: {foreign:?}"
    );
    let (one, two) = tokio::join!(
        store.begin_external_authentication_refresh(owner, &scope, &id, NOW + 1001),
        store.begin_external_authentication_refresh(owner, &scope, &id, NOW + 1001),
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
        value => return Err(format!("Expected one refresh grant: {value:?}").into()),
    };
    assert_eq!(original.secret.expose(), b"synthetic-oauth-token-bundle");
    assert_eq!(original.account_id, "refresh-account");
    // Alias changes do not rotate the underlying protected source.
    store
        .rename_external_authentication(
            owner,
            &RenameAuthentication {
                scope: scope.clone(),
                request_id: "refresh-alias".into(),
                authentication_id: id.clone(),
                expected_request_id: "refresh-original".into(),
                name: "Renamed while refreshing".into(),
            },
            NOW + 1002,
        )
        .await?;
    let renewed = DeviceAuthenticationTokens {
        secret: key("renewed-refresh-bundle")?,
        expires_at: Some(NOW + 3_600_000),
        ..tokens(Some("refresh-account"))?
    };
    assert!(matches!(
        store
            .complete_external_authentication_refresh(
                owner,
                &scope,
                &step,
                &tokens(Some("wrong-account"))?,
                NOW + 1002
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    store
        .complete_external_authentication_refresh(owner, &scope, &step, &renewed, NOW + 1002)
        .await?;
    // A delayed duplicate cannot overwrite a successful source, including with changed tokens.
    assert!(matches!(
        store
            .complete_external_authentication_refresh(
                owner,
                &scope,
                &step,
                &expiring()?,
                NOW + 1003
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let Admission::Current(current) = store
        .begin_external_authentication_refresh(owner, &scope, &id, NOW + 1003)
        .await?
    else {
        return Err("Fresh tokens caused a second refresh".into());
    };
    assert_ne!(current.source, original.source);
    assert_eq!(current.secret.expose(), b"renewed-refresh-bundle");
    let metadata = store
        .external_authentication(owner, &scope, &id, NOW + 1003)
        .await?;
    assert_eq!(metadata.current_request_id, "refresh-alias");
    assert_eq!(metadata.name, "Renamed while refreshing");
    // Reauthentication replaces the protected source and fences old results.
    let reauth = exchange(
        store,
        owner,
        &scope,
        "refresh-reauth",
        Some((&id, "refresh-alias")),
    )
    .await?;
    registered(
        store
            .complete_external_device_authentication(owner, &reauth, &expiring()?, NOW + 1003)
            .await?,
    )?;
    let Admission::Granted { step: stale, .. } = store
        .begin_external_authentication_refresh(owner, &scope, &id, NOW + 1003)
        .await?
    else {
        return Err("New source has no refresh grant".into());
    };
    let reauth = exchange(
        store,
        owner,
        &scope,
        "refresh-manual-replacement",
        Some((&id, "refresh-reauth")),
    )
    .await?;
    registered(
        store
            .complete_external_device_authentication(owner, &reauth, &expiring()?, NOW + 1004)
            .await?,
    )?;
    assert!(matches!(
        store
            .complete_external_authentication_refresh(owner, &scope, &stale, &renewed, NOW + 1004)
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .fail_external_authentication_refresh(&stale, RefreshState::Uncertain, NOW + 1004)
        .await?;
    let Admission::Granted {
        step: uncertain, ..
    } = store
        .begin_external_authentication_refresh(owner, &scope, &id, NOW + 1004)
        .await?
    else {
        return Err("Manual replacement was fenced by an older source".into());
    };
    store
        .fail_external_authentication_refresh(&uncertain, RefreshState::Uncertain, NOW + 1005)
        .await?;
    assert!(matches!(
        store
            .begin_external_authentication_refresh(owner, &scope, &id, NOW + 1005)
            .await?,
        Admission::Existing {
            state: RefreshState::Uncertain,
            ..
        }
    ));
    assert!(matches!(
        store
            .complete_external_authentication_refresh(
                owner,
                &scope,
                &uncertain,
                &renewed,
                NOW + 1005
            )
            .await,
        Err(StoreError::Conflict)
    ));
    // Applied-only records will not become a new catalogue deletion blocker.
    assert!(matches!(
        store
            .delete_external_authentication(
                owner,
                &DeleteAuthentication {
                    scope: scope.clone(),
                    authentication_id: id.clone(),
                    expected_request_id: "refresh-manual-replacement".into(),
                    request_id: "delete-refresh-record".into(),
                },
                NOW + 1005
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert!(matches!(
        store
            .begin_external_authentication_refresh(owner, &scope, &id, NOW + 1005)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .complete_external_authentication_refresh(
                owner,
                &scope,
                &uncertain,
                &renewed,
                NOW + 1005
            )
            .await,
        Err(StoreError::NotFound)
    ));
    store
        .fail_external_authentication_refresh(&uncertain, RefreshState::Uncertain, NOW + 1005)
        .await?;
    let race_device = exchange(store, owner, &scope, "refresh-settlement-race", None).await?;
    let race_id = registered(
        store
            .complete_external_device_authentication(owner, &race_device, &expiring()?, NOW + 1001)
            .await?,
    )?;
    let Admission::Granted { step: raced, .. } = store
        .begin_external_authentication_refresh(owner, &scope, &race_id, NOW + 1001)
        .await?
    else {
        return Err("Missing race admission".into());
    };
    let (published, failed) = tokio::join!(
        store.complete_external_authentication_refresh(owner, &scope, &raced, &renewed, NOW + 1002),
        store.fail_external_authentication_refresh(&raced, RefreshState::Uncertain, NOW + 1002),
    );
    failed?;
    match published {
        Ok(_) => assert!(matches!(
            store
                .begin_external_authentication_refresh(owner, &scope, &race_id, NOW + 1003)
                .await?,
            Admission::Current(_)
        )),
        Err(StoreError::Conflict) => assert!(matches!(
            store
                .begin_external_authentication_refresh(owner, &scope, &race_id, NOW + 1003)
                .await?,
            Admission::Existing {
                state: RefreshState::Uncertain,
                ..
            }
        )),
        Err(error) => return Err(error.into()),
    }
    store
        .delete_external_authentication(
            owner,
            &DeleteAuthentication {
                scope,
                authentication_id: race_id,
                expected_request_id: "refresh-settlement-race".into(),
                request_id: "delete-refresh-race".into(),
            },
            NOW + 1003,
        )
        .await?;
    Ok(())
}
