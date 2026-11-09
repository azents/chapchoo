use crate::Error;
use std::{
    fmt, fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

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
        let root = saved.join(".chapchoo-native");
        private_directory(&root)?;
        let storage = Self {
            saved: saved.into(),
            root,
            saved_handle: Arc::new(
                nix::fcntl::open(
                    saved,
                    nix::fcntl::OFlag::O_RDONLY
                        | nix::fcntl::OFlag::O_DIRECTORY
                        | nix::fcntl::OFlag::O_NOFOLLOW
                        | nix::fcntl::OFlag::O_CLOEXEC,
                    nix::sys::stat::Mode::empty(),
                )
                .map(fs::File::from)
                .map_err(|_| Error::Storage)?,
            ),
        };
        for class in [
            "installations",
            "home",
            "state",
            "tmp",
            "workspace",
            "capture",
        ] {
            private_directory(&storage.root.join(class))?;
        }
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
        self.root.join("home")
    }
    pub fn state(&self) -> PathBuf {
        self.root.join("state")
    }
    pub fn temporary(&self) -> PathBuf {
        self.root.join("tmp")
    }
    pub fn workspace(&self) -> PathBuf {
        self.root.join("workspace")
    }
    pub(crate) fn installations(&self) -> PathBuf {
        self.root.join("installations")
    }
    pub(crate) fn capture_root(&self) -> PathBuf {
        self.root.join("capture")
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
pub(crate) fn private_directory(path: &Path) -> Result<(), Error> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
        Ok(_) => return Err(Error::Storage),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(|_| Error::Storage)?;
        }
        Err(_) => return Err(Error::Storage),
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| Error::Storage)
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
