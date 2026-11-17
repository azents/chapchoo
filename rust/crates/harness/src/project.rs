//! Exact project instruction publication in the retained default workspace.
use crate::{Error, NativeStorage, storage::PrivateClass};
use chapchoo_crypto::BearerSecret;
use nix::{
    errno::Errno,
    fcntl::{AtFlags, OFlag, RenameFlags, openat, renameat, renameat2},
    sys::stat::{FileStat, Mode, SFlag, fchmod, fstat, fstatat},
    unistd::{UnlinkatFlags, getuid, unlinkat},
};
use std::{
    fs::File,
    io::{Read, Write},
    os::fd::AsFd,
};

/// Complete native project document, including any product-owned sections.
pub const MAX_PROJECT_INSTRUCTION_BYTES: usize = 32 * 1024;
const MAX_ORIGINAL_BYTES: usize = 16 * 1024 * 1024;
const INSTRUCTIONS: &str = "AGENTS.md";
const ORIGINAL: &str = "original-AGENTS.md";
const MANAGED: &str = "managed";

impl NativeStorage {
    /// Publish exact selected document bytes into the existing default project.
    /// The native actor serializes this finite blocking operation and retains
    /// its completion beyond a cancelled waiter. Server evidence is preserved
    /// before this call; publication alone is not native adoption.
    pub fn publish_project_instructions(&self, document: &str) -> Result<(), Error> {
        if document.len() > MAX_PROJECT_INSTRUCTION_BYTES {
            return Err(Error::NotAdmitted);
        }
        let root = directory(self.saved_handle.as_ref(), ".chapchoo-native")?;
        let workspace = directory(&root, "workspace")?;
        let source = directory(&root, PrivateClass::ProjectSource.name())?;
        let check = || {
            same_directory(self.saved_handle.as_ref(), ".chapchoo-native", &root)?;
            same_directory(&root, "workspace", &workspace)?;
            same_directory(&root, PrivateClass::ProjectSource.name(), &source)
        };
        check()?;
        preserve_original(&workspace, &source)?;
        let mut pending = PendingFile::create(&workspace)?;
        let result = (|| {
            pending.write(document.as_bytes())?;
            check()?;
            regular_target(&workspace, INSTRUCTIONS)?;
            pending.check()?;
            renameat(&workspace, pending.name.as_str(), &workspace, INSTRUCTIONS)
                .map_err(|_| Error::Storage)?;
            workspace.sync_all().map_err(|_| Error::Storage)
        })();
        finish(result, pending.finish())
    }

    /// Read exact current project document bytes; absence is distinct from error.
    pub fn project_instructions(&self) -> Result<Option<String>, Error> {
        let root = directory(self.saved_handle.as_ref(), ".chapchoo-native")?;
        let workspace = directory(&root, "workspace")?;
        same_directory(self.saved_handle.as_ref(), ".chapchoo-native", &root)?;
        same_directory(&root, "workspace", &workspace)?;
        read_regular(&workspace, INSTRUCTIONS, MAX_PROJECT_INSTRUCTION_BYTES)?
            .map(|bytes| String::from_utf8(bytes).map_err(|_| Error::Storage))
            .transpose()
    }
}

fn preserve_original(workspace: &File, source: &File) -> Result<(), Error> {
    match read_regular(source, MANAGED, 32)? {
        Some(marker) if marker == b"preserved\n" => {
            if read_regular(source, ORIGINAL, MAX_ORIGINAL_BYTES)?.is_none() {
                return Err(Error::Storage);
            }
            return Ok(());
        }
        Some(marker) if marker == b"absent\n" => return Ok(()),
        Some(_) => return Err(Error::Storage),
        None => {}
    }
    let original = read_regular(workspace, INSTRUCTIONS, MAX_ORIGINAL_BYTES)?;
    let archived = read_regular(source, ORIGINAL, MAX_ORIGINAL_BYTES)?;
    match (&original, archived) {
        (Some(bytes), Some(previous)) if *bytes == previous => {}
        (Some(bytes), None) => publish_once(source, ORIGINAL, bytes)?,
        (None, None) => {}
        _ => return Err(Error::Storage),
    }
    let marker: &[u8] = if original.is_some() {
        b"preserved\n"
    } else {
        b"absent\n"
    };
    publish_once(source, MANAGED, marker)
}

fn publish_once(directory: &File, name: &str, bytes: &[u8]) -> Result<(), Error> {
    let mut pending = PendingFile::create(directory)?;
    let result = (|| {
        pending.write(bytes)?;
        pending.check()?;
        match renameat2(
            directory,
            pending.name.as_str(),
            directory,
            name,
            RenameFlags::RENAME_NOREPLACE,
        ) {
            Ok(()) => {}
            Err(Errno::EEXIST)
                if read_regular(directory, name, bytes.len())?.as_deref() == Some(bytes) => {}
            Err(_) => return Err(Error::Storage),
        }
        directory.sync_all().map_err(|_| Error::Storage)
    })();
    finish(result, pending.finish())?;
    directory.sync_all().map_err(|_| Error::Storage)
}

fn finish(primary: Result<(), Error>, cleanup: Result<(), Error>) -> Result<(), Error> {
    match (primary, cleanup) {
        (Err(primary), _) => Err(primary),
        (Ok(()), cleanup) => cleanup,
    }
}

fn directory(parent: &impl AsFd, name: &str) -> Result<File, Error> {
    let file = File::from(
        openat(
            parent,
            name,
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| Error::Storage)?,
    );
    let stat = fstat(&file).map_err(|_| Error::Storage)?;
    if stat.st_uid != getuid().as_raw() || stat.st_mode & 0o777 != 0o700 {
        return Err(Error::Storage);
    }
    Ok(file)
}

fn same_directory(parent: &impl AsFd, name: &str, retained: &File) -> Result<(), Error> {
    let current =
        fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW).map_err(|_| Error::Storage)?;
    let opened = fstat(retained).map_err(|_| Error::Storage)?;
    if !same_file(&current, &opened)
        || current.st_uid != getuid().as_raw()
        || current.st_mode & 0o777 != 0o700
        || SFlag::from_bits_truncate(current.st_mode) & SFlag::S_IFMT != SFlag::S_IFDIR
    {
        return Err(Error::Storage);
    }
    Ok(())
}

fn same_file(one: &FileStat, two: &FileStat) -> bool {
    one.st_dev == two.st_dev && one.st_ino == two.st_ino
}

fn regular_target(directory: &File, name: &str) -> Result<Option<FileStat>, Error> {
    match fstatat(directory, name, AtFlags::AT_SYMLINK_NOFOLLOW) {
        Err(Errno::ENOENT) => Ok(None),
        Ok(stat)
            if SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT == SFlag::S_IFREG
                && stat.st_uid == getuid().as_raw()
                && stat.st_nlink == 1 =>
        {
            Ok(Some(stat))
        }
        _ => Err(Error::Storage),
    }
}

fn read_regular(directory: &File, name: &str, limit: usize) -> Result<Option<Vec<u8>>, Error> {
    let Some(before) = regular_target(directory, name)? else {
        return Ok(None);
    };
    let mut file = File::from(
        openat(
            directory,
            name,
            OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| Error::Storage)?,
    );
    let opened = fstat(&file).map_err(|_| Error::Storage)?;
    if !same_file(&before, &opened) || opened.st_nlink != 1 {
        return Err(Error::Storage);
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(u64::try_from(limit).map_err(|_| Error::Storage)? + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Storage)?;
    let after = regular_target(directory, name)?.ok_or(Error::Storage)?;
    if bytes.len() > limit
        || !same_file(&opened, &after)
        || before.st_size != after.st_size
        || before.st_mtime != after.st_mtime
        || before.st_mtime_nsec != after.st_mtime_nsec
        || before.st_ctime != after.st_ctime
        || before.st_ctime_nsec != after.st_ctime_nsec
    {
        return Err(Error::Storage);
    }
    Ok(Some(bytes))
}

struct PendingFile<'a> {
    directory: &'a File,
    name: String,
    file: File,
}
impl<'a> PendingFile<'a> {
    fn create(directory: &'a File) -> Result<Self, Error> {
        let nonce = BearerSecret::generate().map_err(|_| Error::Storage)?;
        let name = format!(".project-{}", nonce.expose());
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
        fchmod(&pending.file, Mode::from_bits_truncate(0o600)).map_err(|_| Error::Storage)?;
        Ok(pending)
    }
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.file.write_all(bytes).map_err(|_| Error::Storage)?;
        self.file.sync_all().map_err(|_| Error::Storage)
    }
    fn check(&self) -> Result<(), Error> {
        let current = regular_target(self.directory, self.name.as_str())?.ok_or(Error::Storage)?;
        let retained = fstat(&self.file).map_err(|_| Error::Storage)?;
        if !same_file(&current, &retained) || current.st_mode & 0o777 != 0o600 {
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
