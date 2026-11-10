//! Execution-only material and bounded native file replacement. No server authority.
use crate::{Error, NativeStorage};
use chapchoo_crypto::{BearerSecret, ExternalSecret};
use nix::{
    errno::Errno,
    fcntl::{AtFlags, OFlag, openat, renameat},
    sys::stat::{Mode, SFlag, fchmod, fstat, fstatat},
    unistd::{UnlinkatFlags, getuid, unlinkat},
};
use serde::{Deserialize, Serialize};
use std::{fmt, fs::File, future::Future, io::Write, os::fd::AsFd, pin::Pin};

/// Equality data echoing server admission, not standalone authorization.
#[derive(Clone, PartialEq, Eq)]
pub struct NativeAuthenticationBinding {
    pub agent_id: String,
    pub conversation_id: String,
    pub operation_id: String,
    pub execution_id: String,
    pub authentication_id: String,
    pub source: [u8; 24],
}
pub struct NativeAuthentication {
    pub binding: NativeAuthenticationBinding,
    pub credentials: NativeCredentials,
}
pub type NativeAuthenticationFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>;
/// Composition supplies actual route/record authority. Acquisition is invoked
/// only after the actor selects a real launch, never already-live Setup.
pub trait NativeAuthenticationProvider: Send + Sync {
    fn acquire(&self) -> NativeAuthenticationFuture<'_, NativeAuthentication>;
    fn check<'a>(
        &'a self,
        binding: &'a NativeAuthenticationBinding,
    ) -> NativeAuthenticationFuture<'a, ()>;
}
impl fmt::Debug for NativeAuthenticationBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeAuthenticationBinding([REDACTED])")
    }
}
impl fmt::Debug for NativeAuthentication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeAuthentication([REDACTED])")
    }
}

/// Validated short-lived native input, never a server refresh bundle.
/// API keys require explicit ACP login on initial application. Their file form
/// is for same-record replacement, not an alternative initial login mechanism.
pub struct NativeCredentials(Material);
enum Material {
    ApiKey(ExternalSecret),
    ExternalTokens(ExternalSecret),
}
impl fmt::Debug for NativeCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeCredentials([REDACTED])")
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalFile<'a> {
    auth_mode: &'a str,
    #[serde(rename = "OPENAI_API_KEY")]
    api_key: Option<&'a str>,
    #[serde(borrow)]
    tokens: ExternalTokens<'a>,
    last_refresh: &'a str,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalTokens<'a> {
    id_token: &'a str,
    access_token: &'a str,
    refresh_token: &'a str,
    account_id: &'a str,
}
#[derive(Serialize)]
struct ApiKeyFile<'a> {
    auth_mode: &'static str,
    #[serde(rename = "OPENAI_API_KEY")]
    api_key: &'a str,
}
impl NativeCredentials {
    pub(crate) fn payload_len(&self) -> usize {
        match &self.0 {
            Material::ApiKey(value) | Material::ExternalTokens(value) => value.expose().len(),
        }
    }
    pub fn api_key(secret: ExternalSecret) -> Result<Self, Error> {
        let key = std::str::from_utf8(secret.expose()).map_err(|_| Error::Authentication)?;
        if !key.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(Error::Authentication);
        }
        Ok(Self(Material::ApiKey(secret)))
    }
    /// Validate the exact server-converted external-token shape at ingress.
    /// Account/JWT authorization belongs to the server; this is not signature
    /// verification, a refreshed source grant or proof of native adoption.
    pub fn external_token_file(
        file: ExternalSecret,
        expected_account: &str,
    ) -> Result<Self, Error> {
        let value: ExternalFile<'_> =
            serde_json::from_slice(file.expose()).map_err(|_| Error::Authentication)?;
        if value.auth_mode != "chatgptAuthTokens"
            || value.last_refresh.parse::<jiff::Timestamp>().is_err()
            || value.api_key.is_some()
            || expected_account.is_empty()
            || expected_account.len() > 4096
            || value.tokens.account_id != expected_account
            || value.tokens.access_token.is_empty()
            || value.tokens.id_token != value.tokens.access_token
            || !value.tokens.refresh_token.is_empty()
        {
            return Err(Error::Authentication);
        }
        Ok(Self(Material::ExternalTokens(file)))
    }
    /// Exact explicit key for initial ACP metadata. Never place it in argv/env.
    pub fn initial_api_key(&self) -> Option<&str> {
        match &self.0 {
            Material::ApiKey(key) => std::str::from_utf8(key.expose()).ok(),
            Material::ExternalTokens(_) => None,
        }
    }
    fn file(&self) -> Result<zeroize::Zeroizing<Vec<u8>>, Error> {
        // A full 64-KiB key may double under JSON quote/backslash escaping.
        // Native file encoding has its own finite bound, not the raw-key bound.
        let bytes = match &self.0 {
            Material::ApiKey(_) => serde_json::to_vec(&ApiKeyFile {
                auth_mode: "apiKey",
                api_key: self.initial_api_key().ok_or(Error::Authentication)?,
            })
            .map_err(|_| Error::Authentication)?,
            Material::ExternalTokens(file) => file.expose().to_vec(),
        };
        let bytes = zeroize::Zeroizing::new(bytes);
        if bytes.len() > 2 * 65536 + 256 {
            return Err(Error::Authentication);
        }
        Ok(bytes)
    }
}
impl NativeStorage {
    /// Remove only residual owned credentials before explicit initial key login.
    pub fn clear_credentials(&self) -> Result<(), Error> {
        let root = directory(self.saved_handle.as_ref(), ".chapchoo-native")?;
        let state = directory(&root, "state")?;
        same_directory(self.saved_handle.as_ref(), ".chapchoo-native", &root)?;
        same_directory(&root, "state", &state)?;
        target(&state)?;
        match unlinkat(&state, "auth.json", UnlinkatFlags::NoRemoveDir) {
            Ok(()) | Err(Errno::ENOENT) => {}
            Err(_) => return Err(Error::Storage),
        }
        state.sync_all().map_err(|_| Error::Storage)
    }
    /// File publication only. The native actor must serialize this operation and
    /// revalidate exact execution/applied-record/source authority before calling.
    /// A completed rename does not assert cache reload, login or model success.
    /// This finite blocking operation must remain owned beyond a cancelled waiter.
    pub fn publish_credentials(&self, credentials: &NativeCredentials) -> Result<(), Error> {
        let file = credentials.file()?;
        let root = directory(self.saved_handle.as_ref(), ".chapchoo-native")?;
        let state = directory(&root, "state")?;
        let mut pending = PendingFile::create(&state)?;
        let result = (|| {
            pending.file.write_all(&file).map_err(|_| Error::Storage)?;
            pending.file.sync_all().map_err(|_| Error::Storage)?;
            // Refuse a substituted class or special target. All actual writes
            // and rename stay relative to retained directories, never a symlink.
            same_directory(self.saved_handle.as_ref(), ".chapchoo-native", &root)?;
            same_directory(&root, "state", &state)?;
            pending.check()?;
            target(&state)?;
            renameat(&state, pending.name.as_str(), &state, "auth.json")
                .map_err(|_| Error::Storage)?;
            state.sync_all().map_err(|_| Error::Storage)?;
            Ok(())
        })();
        let cleanup = pending.finish();
        match (result, cleanup) {
            (Err(primary), _) => Err(primary),
            (Ok(()), cleanup) => cleanup,
        }
    }
}
fn directory(parent: &impl AsFd, name: &str) -> Result<File, Error> {
    let directory = File::from(
        openat(
            parent,
            name,
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| Error::Storage)?,
    );
    let stat = fstat(&directory).map_err(|_| Error::Storage)?;
    if stat.st_uid != getuid().as_raw() || stat.st_mode & 0o777 != 0o700 {
        return Err(Error::Storage);
    }
    Ok(directory)
}
fn same_directory(parent: &impl AsFd, name: &str, opened: &File) -> Result<(), Error> {
    let current =
        fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW).map_err(|_| Error::Storage)?;
    let retained = fstat(opened).map_err(|_| Error::Storage)?;
    if current.st_dev != retained.st_dev
        || current.st_ino != retained.st_ino
        || current.st_uid != getuid().as_raw()
        || current.st_mode & 0o777 != 0o700
        || SFlag::from_bits_truncate(current.st_mode) & SFlag::S_IFMT != SFlag::S_IFDIR
    {
        return Err(Error::Storage);
    }
    Ok(())
}
fn target(directory: &File) -> Result<(), Error> {
    match fstatat(directory, "auth.json", AtFlags::AT_SYMLINK_NOFOLLOW) {
        Err(Errno::ENOENT) => Ok(()),
        Ok(stat)
            if SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT == SFlag::S_IFREG
                && stat.st_uid == getuid().as_raw()
                && stat.st_nlink == 1 =>
        {
            Ok(())
        }
        _ => Err(Error::Storage),
    }
}
struct PendingFile<'a> {
    directory: &'a File,
    name: String,
    file: File,
}
impl<'a> PendingFile<'a> {
    fn create(directory: &'a File) -> Result<Self, Error> {
        let nonce = BearerSecret::generate().map_err(|_| Error::Storage)?;
        let name = format!(".auth-{}", nonce.expose());
        // Creation never truncates an existing object or follows an alias.
        let file = File::from(
            openat(
                directory,
                name.as_str(),
                OFlag::O_WRONLY
                    | OFlag::O_CREAT
                    | OFlag::O_EXCL
                    | OFlag::O_NOFOLLOW
                    | OFlag::O_CLOEXEC,
                Mode::from_bits_truncate(0o600),
            )
            .map_err(|_| Error::Storage)?,
        );
        let pending = Self {
            directory,
            name,
            file,
        };
        // The process umask cannot weaken the promised final file mode.
        fchmod(&pending.file, Mode::from_bits_truncate(0o600)).map_err(|_| Error::Storage)?;
        Ok(pending)
    }
    fn check(&self) -> Result<(), Error> {
        let current = fstatat(
            self.directory,
            self.name.as_str(),
            AtFlags::AT_SYMLINK_NOFOLLOW,
        )
        .map_err(|_| Error::Storage)?;
        let retained = fstat(&self.file).map_err(|_| Error::Storage)?;
        if current.st_dev != retained.st_dev
            || current.st_ino != retained.st_ino
            || SFlag::from_bits_truncate(current.st_mode) & SFlag::S_IFMT != SFlag::S_IFREG
            || current.st_nlink != 1
            || current.st_uid != getuid().as_raw()
            || current.st_mode & 0o777 != 0o600
        {
            return Err(Error::Storage);
        }
        Ok(())
    }
    fn finish(self) -> Result<(), Error> {
        self.remove()
    }
    fn remove(&self) -> Result<(), Error> {
        match unlinkat(
            self.directory,
            self.name.as_str(),
            UnlinkatFlags::NoRemoveDir,
        ) {
            Ok(()) | Err(Errno::ENOENT) => Ok(()),
            Err(_) => Err(Error::Storage),
        }
    }
}
impl Drop for PendingFile<'_> {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

#[cfg(test)]
mod tests;
