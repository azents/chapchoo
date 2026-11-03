use super::*;

type TestError = Box<dyn std::error::Error + Send + Sync>;

fn env(directory: &Path) -> Environment {
    Environment {
        config_dir: directory.to_owned(),
        host: None,
        context: None,
        tls: false,
        tls_verify: false,
        cert_dir: None,
        api_version: None,
    }
}

fn context(directory: &Path, name: &str, host: &str, skip: bool) -> Result<PathBuf, TestError> {
    let digest = format!("{:x}", Sha256::digest(name.as_bytes()));
    let metadata = directory.join("contexts/meta").join(&digest);
    std::fs::create_dir_all(&metadata)?;
    std::fs::write(
        metadata.join("meta.json"),
        serde_json::to_vec(&serde_json::json!({
            "Name": name,
            "Metadata": {"Description": "fixture"},
            "Endpoints": {
                "docker": {"Host": host, "SkipTLSVerify": skip},
                "kubernetes": {"unrelated": "not-a-docker-endpoint"},
            },
        }))?,
    )?;
    let tls = directory.join("contexts/tls").join(digest).join("docker");
    std::fs::create_dir_all(&tls)?;
    Ok(tls)
}

#[tokio::test]
async fn api_version_selection_is_frozen_and_explicit_override_does_not_mix_sources()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let mut source = env(directory.path());
    source.api_version = Some("1.41".into());
    let selected = resolve(&source, None)?;
    let transport = crate::transport::Transport::new(selected).await?;
    assert_eq!(transport.sdk()?.client_version().minor_version, 41);
    let selected = resolve(
        &source,
        Some(Explicit {
            host: "unix:///selected.sock".into(),
            tls: None,
            api_version: Some("1.40".into()),
        }),
    )?;
    assert_eq!(selected.api_version, Some(ApiVersion::parse("1.40")?));
    let selected = resolve(
        &source,
        Some(Explicit {
            host: "unix:///selected.sock".into(),
            tls: None,
            api_version: None,
        }),
    )?;
    assert!(selected.api_version.is_none());
    for invalid in ["", "0.1", "v1.40", "1", "1.2.3", "1.-1", "1. 40"] {
        source.api_version = Some(invalid.into());
        assert!(resolve(&source, None).is_err());
    }
    Ok(())
}

#[test]
fn cli_host_context_and_saved_context_select_one_coherent_source() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    context(directory.path(), "saved", "unix:///saved.sock", false)?;
    context(
        directory.path(),
        "environment",
        "unix:///environment.sock",
        false,
    )?;
    std::fs::write(
        directory.path().join("config.json"),
        br#"{"currentContext":"saved","auths":{"private":"ignored"}}"#,
    )?;
    let mut source = env(directory.path());
    assert!(
        matches!(resolve(&source, None)?.endpoint, Endpoint::Unix(path) if path == Path::new("/saved.sock"))
    );
    source.context = Some("environment".into());
    assert!(
        matches!(resolve(&source, None)?.endpoint, Endpoint::Unix(path) if path == Path::new("/environment.sock"))
    );
    // Verified against CLI resolveContextName and a configuration-only Docker
    // 29.8.2 `context show` oracle: DOCKER_HOST selects the default context.
    source.host = Some("unix:///selected.sock".into());
    assert!(
        matches!(resolve(&source, None)?.endpoint, Endpoint::Unix(path) if path == Path::new("/selected.sock"))
    );
    source.host = Some("invalid-selected-host".into());
    assert!(resolve(&source, None).is_err());
    Ok(())
}

#[test]
fn explicit_override_never_borrows_ambient_tls_or_context_and_debug_is_redacted()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let mut source = env(directory.path());
    source.context = Some("missing-private-context".into());
    source.tls = true;
    source.cert_dir = Some(directory.path().join("missing-private-keys"));
    let explicit = Explicit {
        api_version: None,
        host: "tcp://private-selected-host:2375".into(),
        tls: None,
    };
    assert!(!format!("{explicit:?}").contains("private-selected-host"));
    let resolved = resolve(&source, Some(explicit))?;
    assert!(resolved.tls.is_none());
    assert!(matches!(
        resolved.endpoint,
        Endpoint::Tcp { port: 2375, .. }
    ));
    assert!(resolve(&source, None).is_err());
    Ok(())
}

#[test]
fn selected_context_retains_skip_policy_and_captured_material_without_env_mixing()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let tls = context(directory.path(), "tls", "tcp://selected:2376", true)?;
    std::fs::write(tls.join("ca.pem"), "original-ca")?;
    std::fs::write(tls.join("cert.pem"), "original-cert")?;
    std::fs::write(tls.join("key.pem"), "private-original-key")?;
    let mut source = env(directory.path());
    source.context = Some("tls".into());
    source.tls_verify = true;
    source.cert_dir = Some(directory.path().join("not-this-context"));
    let selected = resolve(&source, None)?;
    let material = selected.tls.ok_or("Missing selected TLS")?;
    assert!(material.skip_verify);
    std::fs::write(tls.join("key.pem"), "changed-key")?;
    assert_eq!(
        material.key.as_deref().map(Vec::as_slice),
        Some(b"private-original-key".as_slice())
    );
    assert!(!format!("{material:?}").contains("private-original-key"));
    std::fs::remove_file(tls.join("cert.pem"))?;
    assert!(resolve(&source, None).is_err());
    Ok(())
}

#[test]
fn missing_or_malformed_selected_metadata_never_falls_back() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let mut source = env(directory.path());
    source.context = Some("missing".into());
    assert!(resolve(&source, None).is_err());
    let digest = format!("{:x}", Sha256::digest(b"missing"));
    let metadata = directory.path().join("contexts/meta").join(digest);
    std::fs::create_dir_all(&metadata)?;
    for payload in [
        b"{}".as_slice(),
        br#"{"Name":"different","Endpoints":{"docker":{"Host":"unix:///other.sock"}}}"#.as_slice(),
    ] {
        std::fs::write(metadata.join("meta.json"), payload)?;
        assert!(resolve(&source, None).is_err());
    }
    Ok(())
}

#[test]
fn tls_ports_ipv6_and_ssh_escaped_values_preserve_selected_semantics() -> Result<(), TestError> {
    let tls = Tls {
        skip_verify: true,
        ca: None,
        certificate: None,
        key: None,
    };
    let selected = validate_tls("tcp://[::1]", Some(tls.clone()))?;
    assert!(matches!(selected.endpoint, Endpoint::Tcp { host, port: 2376 } if host == "::1"));
    let selected = validate_tls("tcp://[::1]:2375", Some(tls))?;
    assert!(matches!(
        selected.endpoint,
        Endpoint::Tcp { port: 2375, .. }
    ));
    assert!(selected.tls.is_some_and(|tls| tls.skip_verify));
    assert!(
        matches!(endpoint("ssh://op%65rator@[::1]/run/user/docker%20socket.sock")?,
        Endpoint::Ssh { host, user: Some(user), socket: Some(socket), .. }
        if host == "::1" && user == "operator" && socket == "/run/user/docker socket.sock")
    );
    assert!(endpoint("ssh://operator@host/run/docker%00.sock").is_err());
    Ok(())
}

#[test]
fn selected_endpoint_shapes_are_explicit_and_never_synthesize_other_hosts() -> Result<(), TestError>
{
    assert!(
        matches!(endpoint("unix:///tmp/user%20docker.sock")?, Endpoint::Unix(path) if path == Path::new("/tmp/user docker.sock"))
    );
    assert!(
        matches!(endpoint("ssh://operator@selected:2222/run/user/1000/docker.sock")?,
        Endpoint::Ssh { user: Some(user), port: Some(2222), socket: Some(socket), .. }
        if user == "operator" && socket == "/run/user/1000/docker.sock")
    );
    for invalid in [
        "unix://other-host/path.sock",
        "unix:///",
        "tcp://user:secret@host:2375",
        "tcp://host:2375/extra",
        "ssh://user:secret@host",
        "tcp://host?token=private",
        "https://silently-different-mode",
        "npipe:////./pipe/docker_engine",
    ] {
        assert!(endpoint(invalid).is_err());
    }
    Ok(())
}
