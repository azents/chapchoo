//! Explicit server-side bootstrap proof issuance. Neither stdout nor Provider runtime receives the root.
use crate::{
    AppError,
    config::{ConfigError, ProviderCredentialOptions},
};
use chapchoo_crypto::{CredentialCipher, ProviderBootstrapCredential, RootKey};
use k8s_openapi::{
    ByteString, api::core::v1::Secret, apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{Api, Client, Config, api::PostParams};
use std::{collections::BTreeMap, fs::OpenOptions, io::Write};
use zeroize::Zeroizing;

fn validate_existing(cipher: &CredentialCipher, id: &str, value: String) -> Result<(), AppError> {
    cipher.verify_provider_bootstrap(id, &ProviderBootstrapCredential::parse(value)?)?;
    Ok(())
}
pub async fn run(options: ProviderCredentialOptions) -> Result<(), AppError> {
    let root = std::env::var("CHAPCHOO_SECRET_KEY")
        .map_err(|_| ConfigError::Variable("CHAPCHOO_SECRET_KEY"))?;
    let cipher = CredentialCipher::new(RootKey::parse(root)?)?;
    if let Some(path) = options.output_file {
        if !path.is_absolute() {
            return Err("Credential output must be an absolute private path".into());
        }
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                if !meta.file_type().is_file() || meta.len() > 272 {
                    return Err("Credential output is not a private regular file".into());
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if meta.permissions().mode() & 0o077 != 0 {
                        return Err("Credential output permissions are not private".into());
                    }
                }
                return validate_existing(
                    &cipher,
                    &options.provider_id,
                    std::fs::read_to_string(path)?,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let proof = cipher.issue_provider_bootstrap(&options.provider_id)?;
        let mut flags = OpenOptions::new();
        flags.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            flags.mode(0o600);
        }
        let mut output = flags.open(path)?;
        output.write_all(proof.expose().as_bytes())?;
        output.sync_all()?;
        return Ok(());
    }
    if !options.in_cluster {
        return Err("Select a credential output target".into());
    }
    let namespace = options
        .namespace
        .ok_or("Credential target namespace is required")?;
    let name = options
        .secret_name
        .ok_or("Credential target Secret is required")?;
    let config = Config::incluster()?;
    let api: Api<Secret> = Api::namespaced(Client::try_from(config)?, &namespace);
    if let Some(secret) = api.get_opt(&name).await? {
        return validate_secret(&cipher, &options.provider_id, &secret);
    }
    let proof = cipher.issue_provider_bootstrap(&options.provider_id)?;
    let secret = Secret {
        metadata: ObjectMeta {
            name: Some(name.clone()),
            namespace: Some(namespace),
            annotations: Some(BTreeMap::from([
                ("helm.sh/resource-policy".into(), "keep".into()),
                (
                    "argocd.argoproj.io/sync-options".into(),
                    "Prune=false,Delete=false".into(),
                ),
            ])),
            ..Default::default()
        },
        immutable: Some(true),
        type_: Some("Opaque".into()),
        data: Some(BTreeMap::from([(
            "credential".into(),
            ByteString(proof.expose().as_bytes().to_vec()),
        )])),
        ..Default::default()
    };
    match api.create(&PostParams::default(), &secret).await {
        Ok(created) => validate_secret(&cipher, &options.provider_id, &created),
        Err(kube::Error::Api(error)) if error.code == 409 => {
            validate_secret(&cipher, &options.provider_id, &api.get(&name).await?)
        }
        Err(error) => Err(error.into()),
    }
}
fn validate_secret(cipher: &CredentialCipher, id: &str, secret: &Secret) -> Result<(), AppError> {
    if secret
        .metadata
        .owner_references
        .as_ref()
        .is_some_and(|v| !v.is_empty())
    {
        return Err("Credential Secret has an unexpected owner".into());
    }
    if secret.immutable != Some(true) || secret.type_.as_deref() != Some("Opaque") {
        return Err("Credential Secret is not immutable Opaque data".into());
    }
    let annotations = secret
        .metadata
        .annotations
        .as_ref()
        .ok_or("Credential Secret retention is missing")?;
    if annotations
        .get("helm.sh/resource-policy")
        .map(String::as_str)
        != Some("keep")
        || annotations
            .get("argocd.argoproj.io/sync-options")
            .map(String::as_str)
            != Some("Prune=false,Delete=false")
    {
        return Err("Credential Secret retention is invalid".into());
    }
    let data = secret.data.as_ref().ok_or("Credential Secret is empty")?;
    if data.len() != 1 {
        return Err("Credential Secret has unexpected fields".into());
    }
    let value = data
        .get("credential")
        .ok_or("Credential field is missing")?;
    let text = Zeroizing::new(
        String::from_utf8(value.0.clone()).map_err(|_| "Credential field is invalid")?,
    );
    validate_existing(cipher, id, text.to_string())
}
