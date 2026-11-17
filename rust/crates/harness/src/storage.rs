use crate::Error;
use nix::{
    errno::Errno,
    fcntl::{AtFlags, OFlag, openat},
    sys::stat::{Mode, SFlag, fchmod, fstat, fstatat, mkdirat},
    unistd::getuid,
};
use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

/// Every product-owned private class is admitted together at saved-volume open.
/// Nested native/user content is deliberately outside mode normalization.
#[derive(Clone, Copy)]
pub(crate) enum PrivateClass {
    Installations,
    Home,
    State,
    Temporary,
    Workspace,
    Capture,
    ProjectSource,
}
impl PrivateClass {
    const ALL: [Self; 7] = [
        Self::Installations,
        Self::Home,
        Self::State,
        Self::Temporary,
        Self::Workspace,
        Self::Capture,
        Self::ProjectSource,
    ];
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Installations => "installations",
            Self::Home => "home",
            Self::State => "state",
            Self::Temporary => "tmp",
            Self::Workspace => "workspace",
            Self::Capture => "capture",
            Self::ProjectSource => "project-source",
        }
    }
}

/// Canonical, explicit existing saved storage; never inferred from HOME.
#[derive(Clone)]
pub struct NativeStorage {
    saved: PathBuf,
    root: PathBuf,
    pub(crate) saved_handle: Arc<fs::File>,
}
impl fmt::Debug for NativeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeStorage([REDACTED])")
    }
}
impl NativeStorage {
    /// Validate the supplied saved mount and prepare private product-owned classes.
    pub fn open(saved: &Path) -> Result<Self, Error> {
        if !saved.is_absolute()
            || saved == Path::new("/")
            || saved
                .components()
                .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
            || fs::canonicalize(saved).map_err(|_| Error::Storage)? != saved
        {
            return Err(Error::Storage);
        }
        let saved_handle = Arc::new(
            nix::fcntl::open(
                saved,
                OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
                Mode::empty(),
            )
            .map(fs::File::from)
            .map_err(|_| Error::Storage)?,
        );
        let root_handle = private_directory(saved_handle.as_ref(), ".chapchoo-native")?;
        for class in PrivateClass::ALL {
            private_directory(&root_handle, class.name())?;
        }
        let storage = Self {
            saved: saved.into(),
            root: saved.join(".chapchoo-native"),
            saved_handle,
        };
        // An actual write is stronger than intended mount configuration.
        let probe = tempfile::NamedTempFile::new_in(storage.root()).map_err(|_| Error::Storage)?;
        probe.close().map_err(|_| Error::Storage)?;
        Ok(storage)
    }
    pub fn saved_root(&self) -> &Path {
        &self.saved
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn home(&self) -> PathBuf {
        self.root.join(PrivateClass::Home.name())
    }
    pub fn state(&self) -> PathBuf {
        self.root.join(PrivateClass::State.name())
    }
    pub fn temporary(&self) -> PathBuf {
        self.root.join(PrivateClass::Temporary.name())
    }
    pub fn workspace(&self) -> PathBuf {
        self.root.join(PrivateClass::Workspace.name())
    }
    pub(crate) fn installations(&self) -> PathBuf {
        self.root.join(PrivateClass::Installations.name())
    }
    pub(crate) fn capture_root(&self) -> PathBuf {
        self.root.join(PrivateClass::Capture.name())
    }
    pub(crate) fn command(&self, executable: &Path) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(executable);
        command
            .env_clear()
            .current_dir(self.workspace())
            .env("HOME", self.home())
            .env("CODEX_HOME", self.state())
            .env("XDG_CONFIG_HOME", self.home().join(".config"))
            .env("XDG_CACHE_HOME", self.home().join(".cache"))
            .env("XDG_DATA_HOME", self.home().join(".local/share"))
            .env("TMPDIR", self.temporary())
            .env("LANG", "C.UTF-8");
        command
    }
}
fn private_directory(parent: &fs::File, name: &str) -> Result<fs::File, Error> {
    match mkdirat(parent, name, Mode::from_bits_truncate(0o700)) {
        Ok(()) => parent.sync_all().map_err(|_| Error::Storage)?,
        Err(Errno::EEXIST) => {}
        Err(_) => return Err(Error::Storage),
    }
    let directory = fs::File::from(
        openat(
            parent,
            name,
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| Error::Storage)?,
    );
    let opened = fstat(&directory).map_err(|_| Error::Storage)?;
    if opened.st_uid != getuid().as_raw()
        || SFlag::from_bits_truncate(opened.st_mode) & SFlag::S_IFMT != SFlag::S_IFDIR
    {
        return Err(Error::Storage);
    }
    let current =
        fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW).map_err(|_| Error::Storage)?;
    if current.st_dev != opened.st_dev || current.st_ino != opened.st_ino {
        return Err(Error::Storage);
    }
    // Kubelet fsGroup remount can add group/setgid bits. Restore the existing
    // private invariant only on this verified owned descriptor, never by path.
    fchmod(&directory, Mode::from_bits_truncate(0o700)).map_err(|_| Error::Storage)?;
    directory.sync_all().map_err(|_| Error::Storage)?;
    let current =
        fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW).map_err(|_| Error::Storage)?;
    if current.st_dev != opened.st_dev
        || current.st_ino != opened.st_ino
        || current.st_uid != getuid().as_raw()
        || current.st_mode & 0o7777 != 0o700
    {
        return Err(Error::Storage);
    }
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    #[test]
    fn explicit_storage_rejects_aliases_and_keeps_classes_separate()
    -> Result<(), Box<dyn std::error::Error>> {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        assert!(storage.workspace().is_dir());
        assert_ne!(storage.state(), storage.installations());
        assert!(NativeStorage::open(&saved.path().join("..")).is_err());
        let alias = saved.path().join("alias");
        std::os::unix::fs::symlink(storage.home(), &alias)?;
        assert!(NativeStorage::open(&alias).is_err());
        Ok(())
    }
    #[test]
    fn owned_directory_cannot_be_redirected() -> Result<(), Box<dyn std::error::Error>> {
        let saved = tempfile::tempdir()?;
        let foreign = tempfile::tempdir()?;
        std::os::unix::fs::symlink(foreign.path(), saved.path().join(".chapchoo-native"))?;
        assert!(NativeStorage::open(saved.path()).is_err());
        assert_eq!(fs::read_dir(foreign.path())?.count(), 0);
        Ok(())
    }

    #[test]
    fn repeated_volume_reentry_restores_all_private_classes_and_preserves_content()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::{CaptureLog, NativeCredentials};
        use chapchoo_crypto::ExternalSecret;
        use chapchoo_domain::NativeConversationId;
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let original = b"original instructions\xff\n";
        fs::write(storage.workspace().join("AGENTS.md"), original)?;
        storage.publish_project_instructions("first selection")?;
        let archive = storage.root.join("project-source/original-AGENTS.md");
        let marker = storage.root.join("project-source/managed");
        let session = storage.state().join("sessions");
        fs::create_dir(&session)?;
        let native_file = session.join("retained.jsonl");
        fs::write(&native_file, b"existing native history\n")?;
        let executable = storage.installations().join("existing-executable");
        fs::write(&executable, b"retained installation")?;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))?;
        let user_file = storage.workspace().join("user.txt");
        fs::write(&user_file, b"user original")?;
        let credentials = NativeCredentials::api_key(ExternalSecret::new(
            b"synthetic-storage-regression".to_vec(),
        )?)?;
        storage.publish_credentials(&credentials)?;
        let capture = CaptureLog::create(&storage)?;
        let native = NativeConversationId::parse("restored-native".into())?;
        capture.bind_native(&native)?;
        let raw = r#"{"method":"session/update","params":{"sessionId":"restored-native"}}"#;
        capture.append(raw, None)?;
        let snapshot = capture.snapshot()?;
        let captured_bytes = fs::read(storage.capture_root().join("current/source.jsonl"))?;
        drop(capture);
        let archived_inode = fs::metadata(&archive)?.ino();
        let marker_inode = fs::metadata(&marker)?.ino();
        drop(storage);

        for selection in ["second selection", "third selection"] {
            let root = saved.path().join(".chapchoo-native");
            // Kubelet fsGroup reentry adds group access/setgid recursively.
            for class in PrivateClass::ALL {
                fs::set_permissions(root.join(class.name()), fs::Permissions::from_mode(0o2770))?;
            }
            fs::set_permissions(&root, fs::Permissions::from_mode(0o2770))?;
            for file in [
                &archive,
                &marker,
                &native_file,
                &user_file,
                &root.join("state/auth.json"),
            ] {
                fs::set_permissions(file, fs::Permissions::from_mode(0o660))?;
            }
            let reopened = NativeStorage::open(saved.path())?;
            assert_eq!(fs::metadata(&root)?.mode() & 0o7777, 0o700);
            for class in PrivateClass::ALL {
                assert_eq!(
                    fs::metadata(root.join(class.name()))?.mode() & 0o7777,
                    0o700
                );
            }
            // Secure consumers stay strict after initialization; the owned
            // restoration boundary, not a relaxed per-call check, repairs modes.
            reopened.publish_project_instructions(selection)?;
            assert_eq!(reopened.project_instructions()?.as_deref(), Some(selection));
            assert_eq!(fs::read(&archive)?, original);
            assert_eq!(fs::metadata(&archive)?.ino(), archived_inode);
            assert_eq!(fs::metadata(&marker)?.ino(), marker_inode);
            assert_eq!(fs::read(&native_file)?, b"existing native history\n");
            assert_eq!(fs::read(&user_file)?, b"user original");
            assert_eq!(fs::metadata(&user_file)?.mode() & 0o777, 0o660);
            assert_eq!(fs::metadata(&executable)?.mode() & 0o777, 0o755);
            reopened.publish_credentials(&credentials)?;
            assert_eq!(
                fs::metadata(root.join("state/auth.json"))?.mode() & 0o777,
                0o600
            );
            let capture = CaptureLog::open(&reopened)?;
            assert_eq!(capture.snapshot()?, snapshot);
            assert_eq!(capture.native_session()?.as_ref(), Some(&native));
            assert_eq!(
                fs::read(reopened.capture_root().join("current/source.jsonl"))?,
                captured_bytes
            );
        }
        Ok(())
    }

    #[test]
    fn every_private_class_rejects_redirection_and_special_nodes_without_mutating_referents()
    -> Result<(), Box<dyn std::error::Error>> {
        for class in PrivateClass::ALL {
            for special in ["symlink", "file", "fifo"] {
                let saved = tempfile::tempdir()?;
                let foreign = tempfile::tempdir()?;
                let storage = NativeStorage::open(saved.path())?;
                let class = storage.root.join(class.name());
                fs::remove_dir(&class)?;
                fs::set_permissions(foreign.path(), fs::Permissions::from_mode(0o750))?;
                match special {
                    "symlink" => std::os::unix::fs::symlink(foreign.path(), &class)?,
                    "file" => fs::write(&class, b"not a private directory")?,
                    _ => nix::unistd::mkfifo(&class, Mode::from_bits_truncate(0o600))?,
                }
                assert!(NativeStorage::open(saved.path()).is_err());
                assert_eq!(fs::metadata(foreign.path())?.mode() & 0o777, 0o750);
                assert_eq!(fs::read_dir(foreign.path())?.count(), 0);
            }
        }
        Ok(())
    }
}
