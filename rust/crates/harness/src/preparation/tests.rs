use super::*;
type TestError = Box<dyn std::error::Error + Send + Sync>;
#[test]
fn transient_cache_fallback_never_masks_integrity_or_compatibility_failure() -> Result<(), TestError>
{
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut preparer = Preparer::new(storage.clone())?;
    let cache = fixture(&storage.installations(), 'a')?;
    fs::write(
        storage.installations().join("latest.json"),
        serde_json::to_vec(
            cache
                .file_name()
                .and_then(|v| v.to_str())
                .ok_or("Cache name")?,
        )?,
    )?;
    assert_eq!(
        preparer
            .eligible_result(Err(Error::Transient), "linux-x64")?
            .freshness,
        Freshness::TransientLookupFailure
    );
    assert!(matches!(
        preparer.eligible_result(Err(Error::Integrity), "linux-x64"),
        Err(Error::Integrity)
    ));
    assert!(matches!(
        preparer.eligible_result(Err(Error::Incompatible), "linux-x64"),
        Err(Error::Incompatible)
    ));
    fs::remove_file(cache.join("complete.json"))?;
    assert!(matches!(
        preparer.eligible_result(Err(Error::Transient), "linux-x64"),
        Err(Error::Integrity)
    ));
    Ok(())
}
fn fixture(root: &Path, marker: char) -> Result<PathBuf, TestError> {
    let path = root.join(format!("complete-{}", marker.to_string().repeat(64)));
    fs::create_dir(&path)?;
    for (name,bytes) in [
        ("node-v24.0.0-linux-x64/bin/node",b"node".as_slice()),
        ("node_modules/@agentclientprotocol/codex-acp/dist/index.js",b"adapter".as_slice()),
        ("node_modules/@agentclientprotocol/codex-acp/package.json",br#"{"version":"2.5.0"}"#.as_slice()),
        ("node_modules/@openai/codex/bin/codex.js",b"codex".as_slice()),
        ("node_modules/@openai/codex/package.json",br#"{"version":"0.165.0"}"#.as_slice()),
        ("package-lock.json",br#"{"lockfileVersion":3,"packages":{"":{},"node_modules/@openai/codex":{"resolved":"https://registry.npmjs.org/@openai/codex/-/codex-0.165.0.tgz","integrity":"sha512-fixture"}}}"#.as_slice()),
    ] {
        let target = path.join(name);
        fs::create_dir_all(target.parent().ok_or("Missing parent")?)?;
        fs::write(target,bytes)?;
    }
    let complete = Complete {
        platform: "linux-x64".into(),
        node_version: "v24.0.0".into(),
        node_sha256: "a".repeat(64),
        adapter_version: "2.5.0".into(),
        codex_version: "0.165.0".into(),
        files: file_digests(&path)?,
    };
    fs::write(path.join("complete.json"), serde_json::to_vec(&complete)?)?;
    Ok(path)
}
#[test]
fn collection_protects_applied_and_latest_but_not_inactive_superseded_cache()
-> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut preparer = Preparer::new(storage.clone())?;
    let old = fixture(&storage.installations(), 'a')?;
    let latest = fixture(&storage.installations(), 'b')?;
    let unused = fixture(&storage.installations(), 'c')?;
    fs::write(storage.state().join("auth.json"), b"secret-marker")?;
    fs::write(storage.workspace().join("source.txt"), b"workspace")?;
    let live = preparer.load(&old, "linux-x64")?;
    preparer.collect(&latest)?;
    assert!(old.is_dir());
    assert!(latest.is_dir());
    assert!(!unused.exists());
    assert_eq!(
        fs::read(storage.state().join("auth.json"))?,
        b"secret-marker"
    );
    assert_eq!(
        fs::read(storage.workspace().join("source.txt"))?,
        b"workspace"
    );
    drop(live);
    preparer.collect(&latest)?;
    assert!(!old.exists());
    assert!(latest.is_dir());
    Ok(())
}
#[test]
fn cached_complete_rejects_modified_payload_and_symlink_escape() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut preparer = Preparer::new(storage.clone())?;
    let cache = fixture(&storage.installations(), 'a')?;
    let _ = preparer.load(&cache, "linux-x64")?;
    fs::write(
        cache.join("node_modules/@openai/codex/bin/codex.js"),
        b"changed",
    )?;
    assert!(matches!(
        preparer.load(&cache, "linux-x64"),
        Err(Error::Integrity)
    ));
    let outside = tempfile::tempdir()?;
    fs::write(outside.path().join("keep"), b"original")?;
    let alias = storage
        .installations()
        .join(format!("complete-{}", "f".repeat(64)));
    std::os::unix::fs::symlink(outside.path(), &alias)?;
    assert!(matches!(preparer.collect(&cache), Err(Error::Storage)));
    assert_eq!(fs::read(outside.path().join("keep"))?, b"original");
    Ok(())
}
#[test]
fn archive_rejects_external_links_without_writing_outside_stage() -> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let mut bytes = Vec::new();
    {
        let gzip = flate2::write::GzEncoder::new(&mut bytes, flate2::Compression::default());
        let mut archive = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o777);
        archive.append_link(&mut header, "node-fixture/bin/escape", "../../../outside")?;
        archive.into_inner()?.finish()?;
    }
    assert!(matches!(
        extract_node(&bytes, saved.path(), "node-fixture"),
        Err(Error::Integrity)
    ));
    assert!(!saved.path().join("node-fixture/bin/escape").exists());
    Ok(())
}
#[test]
fn publication_checks_full_lock_sources_and_platform_aliases_remain_owned_by_npm()
-> Result<(), TestError> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("package-lock.json"),br#"{"lockfileVersion":3,"packages":{"":{},"node_modules/native":{"resolved":"http://untrusted.invalid/native.tgz","integrity":"sha512-fixture"}}}"#)?;
    assert!(matches!(validate_lock(root.path()), Err(Error::Integrity)));
    assert!(compatible_codex("0.165.0"));
    assert!(!compatible_codex("0.166.0"));
    assert!(!compatible_adapter("3.0.0"));
    // Native optional-package aliases are npm's ecosystem resolution, not parsed
    // as the selected stable CLI version by our compatibility boundary.
    assert!(compatible_adapter("2.5.1"));
    Ok(())
}
#[tokio::test]
async fn missing_explicit_cli_is_rejected_before_any_managed_acquisition() -> Result<(), TestError>
{
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let mut preparer = Preparer::new(storage.clone())?;
    let supply = HarnessSupply::Existing(chapchoo_domain::ExistingCodexPath::parse(
        saved.path().join("missing").to_string_lossy().into_owned(),
    )?);
    assert!(matches!(
        preparer.prepare(&supply).await,
        Err(Error::Incompatible)
    ));
    assert_eq!(fs::read_dir(storage.installations())?.count(), 0);
    Ok(())
}
