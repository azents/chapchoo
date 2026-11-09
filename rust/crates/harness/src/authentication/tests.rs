use super::*;
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

type TestError = Box<dyn std::error::Error + Send + Sync>;
#[test]
fn maximum_raw_key_survives_native_file_json_expansion() -> Result<(), TestError> {
    #[derive(Deserialize)]
    struct FileKey {
        #[serde(rename = "OPENAI_API_KEY")]
        key: String,
    }
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let raw = "\\\"".repeat(32768);
    assert_eq!(raw.len(), 65536);
    let key = NativeCredentials::api_key(ExternalSecret::new(raw.as_bytes().to_vec())?)?;
    storage.publish_credentials(&key)?;
    let file = fs::read(storage.state().join("auth.json"))?;
    assert!(file.len() > 65536 && file.len() <= 2 * 65536 + 256);
    assert_eq!(serde_json::from_slice::<FileKey>(&file)?.key, raw);
    assert_eq!(
        fs::metadata(storage.state().join("auth.json"))?
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    Ok(())
}
#[test]
fn explicit_initial_key_cleanup_removes_only_the_owned_auth_file() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    fs::write(storage.state().join("retained-session"), b"native-context")?;
    storage.publish_credentials(&key("old-key")?)?;
    storage.clear_credentials()?;
    assert!(!storage.state().join("auth.json").exists());
    assert_eq!(
        fs::read(storage.state().join("retained-session"))?,
        b"native-context"
    );
    storage.clear_credentials()?;
    Ok(())
}
fn key(value: &str) -> Result<NativeCredentials, TestError> {
    Ok(NativeCredentials::api_key(ExternalSecret::new(
        value.as_bytes().to_vec(),
    )?)?)
}
fn tokens(account: &str, refresh: &str) -> Result<ExternalSecret, TestError> {
    Ok(ExternalSecret::new(serde_json::to_vec(&json!({
        "auth_mode":"chatgptAuthTokens","OPENAI_API_KEY":null,
        "tokens":{"id_token":"synthetic.access.jwt","access_token":"synthetic.access.jwt",
            "refresh_token":refresh,"account_id":account}
    }))?)?)
}
#[test]
fn ingress_rejects_refresh_authority_wrong_accounts_and_ambiguous_material() -> Result<(), TestError>
{
    assert!(
        NativeCredentials::external_token_file(tokens("account", "server-refresh")?, "account")
            .is_err()
    );
    assert!(NativeCredentials::external_token_file(tokens("other", "")?, "account").is_err());
    let unexpected = ExternalSecret::new(br#"{"auth_mode":"chatgptAuthTokens","OPENAI_API_KEY":null,"tokens":{"id_token":"x","access_token":"x","refresh_token":"","account_id":"account"},"server_bundle":"forbidden"}"#.to_vec())?;
    assert!(NativeCredentials::external_token_file(unexpected, "account").is_err());
    let mixed = ExternalSecret::new(br#"{"auth_mode":"chatgptAuthTokens","OPENAI_API_KEY":"key","tokens":{"id_token":"x","access_token":"x","refresh_token":"","account_id":"account"}}"#.to_vec())?;
    assert!(NativeCredentials::external_token_file(mixed, "account").is_err());
    for value in [" \t ", "key\nsecret", "key with spaces", "키"] {
        assert!(key(value).is_err());
    }
    assert!(NativeCredentials::api_key(ExternalSecret::new(vec![0xff])?).is_err());
    let accepted = NativeCredentials::external_token_file(tokens("account", "")?, "account")?;
    assert!(accepted.initial_api_key().is_none());
    assert_eq!(format!("{accepted:?}"), "NativeCredentials([REDACTED])");
    Ok(())
}
#[test]
fn publication_is_private_atomic_and_retains_native_context() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let context = storage.state().join("sessions");
    fs::create_dir(&context)?;
    fs::write(context.join("retained.jsonl"), "retained native context")?;
    let credentials = key("synthetic-original-key")?;
    assert_eq!(
        credentials.initial_api_key(),
        Some("synthetic-original-key")
    );
    storage.publish_credentials(&credentials)?;
    let original = File::open(storage.state().join("auth.json"))?;
    let original_stat = fstat(&original)?;
    assert_eq!(original_stat.st_mode & 0o777, 0o600);
    assert_eq!(
        fs::read(storage.state().join("auth.json"))?,
        br#"{"auth_mode":"apiKey","OPENAI_API_KEY":"synthetic-original-key"}"#
    );
    let newer = NativeCredentials::external_token_file(tokens("account", "")?, "account")?;
    storage.publish_credentials(&newer)?;
    let updated = File::open(storage.state().join("auth.json"))?;
    assert_ne!(fstat(&updated)?.st_ino, original_stat.st_ino);
    assert_eq!(fstat(&updated)?.st_mode & 0o777, 0o600);
    assert_eq!(
        fs::read(context.join("retained.jsonl"))?,
        b"retained native context"
    );
    assert!(
        !fs::read_to_string(storage.state().join("auth.json"))?.contains("synthetic-original-key")
    );
    assert_eq!(fs::read_dir(storage.state())?.count(), 2);
    assert_eq!(fs::read_dir(storage.workspace())?.count(), 0);
    assert_eq!(fs::read_dir(storage.capture_root())?.count(), 0);
    Ok(())
}
#[test]
fn target_aliases_fail_without_mutating_their_referent_or_leaking_temporary_files()
-> Result<(), TestError> {
    for hard_link in [false, true] {
        let saved = tempfile::tempdir()?;
        let foreign = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let referent = foreign.path().join("keep");
        fs::write(&referent, b"unmodified")?;
        let target = storage.state().join("auth.json");
        if hard_link {
            fs::hard_link(&referent, &target)?;
        } else {
            std::os::unix::fs::symlink(&referent, &target)?;
        }
        assert!(storage.publish_credentials(&key("synthetic-key")?).is_err());
        assert_eq!(fs::read(&referent)?, b"unmodified");
        assert_eq!(fs::read_dir(storage.state())?.count(), 1);
    }
    Ok(())
}
#[test]
fn replacement_of_an_owned_directory_is_rejected_at_handoff() -> Result<(), TestError> {
    for class in [".chapchoo-native", ".chapchoo-native/state"] {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let foreign = tempfile::tempdir()?;
        let original = saved.path().join(class);
        fs::rename(&original, saved.path().join("retained"))?;
        std::os::unix::fs::symlink(foreign.path(), &original)?;
        assert!(storage.publish_credentials(&key("synthetic-key")?).is_err());
        assert_eq!(fs::read_dir(foreign.path())?.count(), 0);
    }
    Ok(())
}
#[test]
fn descriptor_handoff_fences_directory_substitution_after_open() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let root = directory(storage.saved_handle.as_ref(), ".chapchoo-native")?;
    let state = directory(&root, "state")?;
    // This explicit boundary models replacement after opening the retained
    // directory, without scheduling sleeps or a probabilistic filesystem race.
    fs::rename(storage.state(), storage.root().join("old-state"))?;
    fs::create_dir(storage.state())?;
    fs::set_permissions(storage.state(), fs::Permissions::from_mode(0o700))?;
    assert!(same_directory(&root, "state", &state).is_err());
    assert_eq!(fs::read_dir(storage.state())?.count(), 0);
    Ok(())
}
#[test]
fn relaxed_directory_permissions_deny_secret_publication() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    fs::set_permissions(storage.state(), fs::Permissions::from_mode(0o755))?;
    assert!(storage.publish_credentials(&key("synthetic-key")?).is_err());
    assert_eq!(fs::read_dir(storage.state())?.count(), 0);
    Ok(())
}
#[test]
fn temporary_file_substitution_is_rejected_before_atomic_handoff() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let root = directory(storage.saved_handle.as_ref(), ".chapchoo-native")?;
    let state = directory(&root, "state")?;
    let pending = PendingFile::create(&state)?;
    pending.check()?;
    let path = storage.state().join(&pending.name);
    fs::remove_file(&path)?;
    std::os::unix::fs::symlink(saved.path().join("foreign"), &path)?;
    assert!(pending.check().is_err());
    pending.finish()?;
    assert_eq!(fs::read_dir(storage.state())?.count(), 0);
    Ok(())
}
