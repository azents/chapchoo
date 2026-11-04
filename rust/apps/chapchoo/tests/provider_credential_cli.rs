//! Issuance uses server authority without loading product database configuration.
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, ProviderBootstrapCredential, RootKey};
use std::process::Command;
type Error = Box<dyn std::error::Error + Send + Sync>;
#[test]
fn credential_file_is_private_reused_and_bound_without_root_or_db_output() -> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("credential");
    let root = URL_SAFE.encode([93; 32]);
    let command = |id: &str, key: &str| {
        Command::new(env!("CARGO_BIN_EXE_chapchoo"))
            .env_clear()
            .env("CHAPCHOO_SECRET_KEY", key)
            .env("CHAPCHOO_SQLITE_PATH", "invalid-db-selection")
            .env("CHAPCHOO_POSTGRES_URL", "invalid-db-selection")
            .args(["provider-credential", "--provider-id", id, "--output-file"])
            .arg(&path)
            .output()
    };
    let first = command("provider-a", &root)?;
    assert!(first.status.success());
    assert!(first.stdout.is_empty());
    let original = std::fs::read_to_string(&path)?;
    let proof = ProviderBootstrapCredential::parse(original.clone())?;
    CredentialCipher::new(RootKey::parse(root.clone())?)?
        .verify_provider_bootstrap("provider-a", &proof)?;
    let again = command("provider-a", &root)?;
    assert!(again.status.success());
    assert_eq!(std::fs::read_to_string(&path)?, original);
    for (id, key) in [
        ("provider-b", root.clone()),
        ("provider-a", URL_SAFE.encode([94; 32])),
    ] {
        let failed = command(id, &key)?;
        assert!(!failed.status.success());
        assert!(!String::from_utf8_lossy(&failed.stdout).contains(&original));
        assert!(!String::from_utf8_lossy(&failed.stderr).contains(&original));
        assert_eq!(std::fs::read_to_string(&path)?, original);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path)?.permissions().mode() & 0o777,
            0o600
        );
        let link = dir.path().join("symlink");
        std::os::unix::fs::symlink(&path, &link)?;
        let result = Command::new(env!("CARGO_BIN_EXE_chapchoo"))
            .env_clear()
            .env("CHAPCHOO_SECRET_KEY", &root)
            .args([
                "provider-credential",
                "--provider-id",
                "provider-a",
                "--output-file",
            ])
            .arg(link)
            .output()?;
        assert!(!result.status.success());
        assert_eq!(std::fs::read_to_string(&path)?, original);
    }
    Ok(())
}
