//! Explicit synthetic credential prerequisite for new-Agent fixture acceptance.
use chapchoo_crypto::{ExternalSecret, SecretDigest};
use chapchoo_persistence::{
    AuthenticationAcceptedResult, AuthenticationScope, RegisterApiKey, Store,
};

type TestError = Box<dyn std::error::Error + Send + Sync>;

pub async fn personal(store: &Store, actor: &SecretDigest, now: i64) -> Result<String, TestError> {
    match store
        .register_external_api_key(
            actor,
            &RegisterApiKey {
                scope: AuthenticationScope::Personal,
                request_id: "fixture-agent-authentication".into(),
                name: "Fixture authentication".into(),
                key: ExternalSecret::new(b"synthetic-fixture-api-key".to_vec())?,
            },
            now,
        )
        .await?
    {
        AuthenticationAcceptedResult::Record { authentication_id } => Ok(authentication_id),
        _ => Err("Missing fixture authentication identity".into()),
    }
}
