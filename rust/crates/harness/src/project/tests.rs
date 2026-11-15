use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn publish_replace_and_clear_preserve_project_files() -> TestResult {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let ordinary = storage.workspace().join("ordinary.txt");
    fs::write(&ordinary, "ordinary project work")?;
    assert_eq!(storage.project_instructions()?, None);
    storage.publish_project_instructions("α\nexact bytes\n")?;
    assert_eq!(
        storage.project_instructions()?.as_deref(),
        Some("α\nexact bytes\n")
    );
    storage.publish_project_instructions("changed")?;
    storage.publish_project_instructions("")?;
    assert_eq!(storage.project_instructions()?.as_deref(), Some(""));
    assert_eq!(fs::read_to_string(ordinary)?, "ordinary project work");
    assert_eq!(
        fs::metadata(storage.workspace().join(INSTRUCTIONS))?
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let source = storage.root().join("project-source");
    assert_eq!(fs::read(source.join(MANAGED))?, b"absent\n");
    assert!(!source.join(ORIGINAL).exists());
    assert_eq!(fs::read_dir(storage.workspace())?.count(), 2);
    Ok(())
}

#[test]
fn existing_original_is_preserved_once_outside_the_project() -> TestResult {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let original = b"existing project instructions\xff\n";
    fs::write(storage.workspace().join(INSTRUCTIONS), original)?;
    storage.publish_project_instructions("managed A")?;
    let archive = storage.root().join("project-source").join(ORIGINAL);
    assert_eq!(fs::read(&archive)?, original);
    storage.publish_project_instructions("managed B")?;
    assert_eq!(fs::read(&archive)?, original);
    let reopened = NativeStorage::open(saved.path())?;
    reopened.publish_project_instructions("managed C")?;
    assert_eq!(fs::read(&archive)?, original);
    assert_eq!(
        reopened.project_instructions()?.as_deref(),
        Some("managed C")
    );
    assert_eq!(
        fs::read_dir(storage.root().join("project-source"))?.count(),
        2
    );
    Ok(())
}

#[test]
fn archival_reentry_requires_the_same_original() -> TestResult {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let root = directory(storage.saved_handle.as_ref(), ".chapchoo-native")?;
    mkdirat(&root, "project-source", Mode::from_bits_truncate(0o700))?;
    let source = directory(&root, "project-source")?;
    fs::write(storage.workspace().join(INSTRUCTIONS), b"original")?;
    publish_once(&source, ORIGINAL, b"original")?;
    storage.publish_project_instructions("managed")?;
    assert_eq!(storage.project_instructions()?.as_deref(), Some("managed"));

    let other_saved = tempfile::tempdir()?;
    let other = NativeStorage::open(other_saved.path())?;
    fs::create_dir(other.root().join("project-source"))?;
    fs::set_permissions(
        other.root().join("project-source"),
        fs::Permissions::from_mode(0o700),
    )?;
    fs::write(other.root().join("project-source").join(ORIGINAL), b"other")?;
    fs::write(other.workspace().join(INSTRUCTIONS), b"original")?;
    assert!(other.publish_project_instructions("managed").is_err());
    assert_eq!(fs::read(other.workspace().join(INSTRUCTIONS))?, b"original");
    Ok(())
}

#[test]
fn publication_rejects_aliased_or_special_targets() -> TestResult {
    for kind in ["symlink", "hardlink", "directory", "fifo"] {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let target = storage.workspace().join(INSTRUCTIONS);
        let foreign = saved.path().join("foreign");
        fs::write(&foreign, "do not modify")?;
        match kind {
            "symlink" => std::os::unix::fs::symlink(&foreign, &target)?,
            "hardlink" => fs::hard_link(&foreign, &target)?,
            "directory" => fs::create_dir(&target)?,
            _ => nix::unistd::mkfifo(&target, Mode::from_bits_truncate(0o600))?,
        }
        assert!(storage.publish_project_instructions("managed").is_err());
        assert!(storage.project_instructions().is_err());
        assert_eq!(fs::read_to_string(&foreign)?, "do not modify");
    }
    Ok(())
}

#[test]
fn redirected_project_or_archive_directory_is_rejected() -> TestResult {
    for class in ["workspace", "project-source"] {
        let saved = tempfile::tempdir()?;
        let foreign = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let target = storage.root().join(class);
        if target.exists() {
            fs::remove_dir(&target)?;
        }
        std::os::unix::fs::symlink(foreign.path(), target)?;
        assert!(storage.publish_project_instructions("managed").is_err());
        assert_eq!(fs::read_dir(foreign.path())?.count(), 0);
    }
    Ok(())
}

#[test]
fn complete_document_byte_limit_is_enforced_before_publication() -> TestResult {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    let allowed = "a".repeat(MAX_PROJECT_INSTRUCTION_BYTES);
    storage.publish_project_instructions(&allowed)?;
    assert!(
        storage
            .publish_project_instructions(&"a".repeat(MAX_PROJECT_INSTRUCTION_BYTES + 1))
            .is_err()
    );
    assert_eq!(
        storage.project_instructions()?.as_deref(),
        Some(allowed.as_str())
    );
    Ok(())
}

#[test]
fn corrupted_managed_marker_does_not_replace_instructions() -> TestResult {
    let saved = tempfile::tempdir()?;
    let storage = NativeStorage::open(saved.path())?;
    storage.publish_project_instructions("before")?;
    fs::write(
        storage.root().join("project-source").join(MANAGED),
        "corrupt",
    )?;
    assert!(storage.publish_project_instructions("after").is_err());
    assert_eq!(storage.project_instructions()?.as_deref(), Some("before"));
    Ok(())
}
