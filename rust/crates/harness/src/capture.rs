use crate::{Error, NativeStorage};
use chapchoo_domain::{EventOccurrenceId, NativeConversationId, SnapshotId, SourcePosition};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const MAX_RAW: usize = 16 * 1024 * 1024;
const MAX_RECORD: usize = MAX_RAW * 2 + 4096;

/// Retained, ordered captured copy. It contains no outgoing command journal.
#[derive(Clone)]
pub struct CaptureLog(Arc<Shared>);
struct Shared {
    directory: PathBuf,
    state: Mutex<State>,
    readers: Arc<Semaphore>,
}
struct State {
    manifest: Manifest,
    file: File,
    end: u64,
    last: Option<SourcePosition>,
    observed_native: Option<NativeConversationId>,
    failed: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    snapshot: String,
    native_session: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiskRecord {
    ordinal: i64,
    occurrence: String,
    raw: String,
}
/// The original UTF-8 native envelope, not SDK reserialization.
pub struct CapturedEvent {
    pub position: SourcePosition,
    pub raw: String,
}
pub struct CaptureBoundary {
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
    pub through: Option<SourcePosition>,
}
/// A finite complete-record read. Append after its boundary cannot extend it.
pub struct CapturePass {
    pub boundary: CaptureBoundary,
    reader: BufReader<std::io::Take<File>>,
    last: Option<SourcePosition>,
    _permit: OwnedSemaphorePermit,
}
macro_rules! safe_debug {
    ($($name:ty),+)=>{$(
        impl fmt::Debug for $name {
            fn fmt(&self,f: &mut fmt::Formatter<'_>)->fmt::Result {
                f.write_str(concat!(stringify!($name),"([REDACTED])"))
            }
        }
    )+};
}
safe_debug!(CaptureLog, CapturedEvent, CaptureBoundary, CapturePass);

impl CaptureLog {
    /// First captured copy only. Existing source is never overwritten implicitly.
    pub fn create(storage: &NativeStorage) -> Result<Self, Error> {
        let directory = storage.capture_root().join("current");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .map_err(|_| Error::CaptureUnavailable)?;
        let manifest = Manifest {
            snapshot: opaque("snapshot")?,
            native_session: None,
        };
        let mut file = private_file(&directory.join("source.jsonl"), true)?;
        file.try_lock().map_err(|_| Error::CaptureUnavailable)?;
        file.flush().map_err(|_| Error::CaptureUnavailable)?;
        let mut metadata = private_file(&directory.join("manifest.json"), true)?;
        metadata
            .write_all(&serde_json::to_vec(&manifest).map_err(|_| Error::CaptureCorrupt)?)
            .map_err(|_| Error::CaptureUnavailable)?;
        Ok(Self(Arc::new(Shared {
            directory,
            state: Mutex::new(State {
                manifest,
                file,
                end: 0,
                last: None,
                observed_native: None,
                failed: false,
            }),
            readers: Arc::new(Semaphore::new(2)),
        })))
    }

    /// Existing source restoration never creates a replacement or repairs a gap.
    pub fn open(storage: &NativeStorage) -> Result<Self, Error> {
        let directory = storage.capture_root().join("current");
        if fs::canonicalize(&directory).map_err(|_| Error::CaptureUnavailable)? != directory {
            return Err(Error::CaptureCorrupt);
        }
        let mut metadata = private_file(&directory.join("manifest.json"), false)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut metadata)
            .take(8193)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::CaptureCorrupt)?;
        if bytes.len() > 8192 {
            return Err(Error::CaptureCorrupt);
        }
        let manifest: Manifest =
            serde_json::from_slice(&bytes).map_err(|_| Error::CaptureCorrupt)?;
        let _ = snapshot(&manifest)?;
        let expected = manifest
            .native_session
            .as_ref()
            .map(|v| native(v))
            .transpose()?;
        let file = private_file(&directory.join("source.jsonl"), false)?;
        file.try_lock().map_err(|_| Error::CaptureUnavailable)?;
        let end = file
            .metadata()
            .map_err(|_| Error::CaptureUnavailable)?
            .len();
        let mut reader = BufReader::new(
            file.try_clone()
                .map_err(|_| Error::CaptureUnavailable)?
                .take(end),
        );
        let mut last = None;
        let mut observed_native = expected;
        while let Some(record) = next(&mut reader)? {
            let ordinal = last.as_ref().map_or(Ok(1), |p: &SourcePosition| {
                p.ordinal().checked_add(1).ok_or(Error::CaptureCorrupt)
            })?;
            if record.position.ordinal() != ordinal {
                return Err(Error::CaptureCorrupt);
            }
            let id = event_native(&record.raw)?;
            match &observed_native {
                Some(expected) if expected != &id => return Err(Error::CaptureCorrupt),
                None => observed_native = Some(id),
                _ => {}
            }
            last = Some(record.position);
        }
        Ok(Self(Arc::new(Shared {
            directory,
            state: Mutex::new(State {
                manifest,
                file,
                end,
                last,
                observed_native,
                failed: false,
            }),
            readers: Arc::new(Semaphore::new(2)),
        })))
    }

    pub fn snapshot(&self) -> Result<SnapshotId, Error> {
        snapshot(
            &self
                .0
                .state
                .lock()
                .map_err(|_| Error::CaptureUnavailable)?
                .manifest,
        )
    }
    pub(crate) fn belongs_to(&self, storage: &NativeStorage) -> bool {
        self.0.directory == storage.capture_root().join("current")
    }
    pub(crate) fn is_empty(&self) -> Result<bool, Error> {
        let state = self.0.state.lock().map_err(|_| Error::CaptureUnavailable)?;
        if state.failed {
            return Err(Error::CaptureCorrupt);
        }
        Ok(state.last.is_none())
    }
    pub fn native_session(&self) -> Result<Option<NativeConversationId>, Error> {
        self.0
            .state
            .lock()
            .map_err(|_| Error::CaptureUnavailable)?
            .manifest
            .native_session
            .as_ref()
            .map(|v| native(v))
            .transpose()
    }
    /// Cheap current-copy metadata; only begin() fixes a finite transfer boundary.
    pub fn current_boundary(&self) -> Result<Option<CaptureBoundary>, Error> {
        let state = self.0.state.lock().map_err(|_| Error::CaptureUnavailable)?;
        if state.failed {
            return Err(Error::CaptureCorrupt);
        }
        let Some(native_id) = &state.manifest.native_session else {
            return Ok(None);
        };
        Ok(Some(CaptureBoundary {
            snapshot: snapshot(&state.manifest)?,
            native_session: native(native_id)?,
            through: state.last.clone(),
        }))
    }
    /// Bind the exact setup result, including events received before that result.
    pub fn bind_native(&self, id: &NativeConversationId) -> Result<(), Error> {
        let mut state = self.0.state.lock().map_err(|_| Error::CaptureUnavailable)?;
        if state.failed {
            return Err(Error::CaptureCorrupt);
        }
        if state
            .observed_native
            .as_ref()
            .is_some_and(|current| current != id)
            || state
                .manifest
                .native_session
                .as_ref()
                .is_some_and(|current| current != id.as_str())
        {
            return Err(Error::CaptureConflict);
        }
        if state.manifest.native_session.is_some() {
            return Ok(());
        }
        let manifest = Manifest {
            snapshot: state.manifest.snapshot.clone(),
            native_session: Some(id.as_str().into()),
        };
        let temporary = self.0.directory.join("binding.pending");
        let mut file = private_file(&temporary, true)?;
        file.write_all(&serde_json::to_vec(&manifest).map_err(|_| Error::CaptureCorrupt)?)
            .map_err(|_| Error::CaptureUnavailable)?;
        file.flush().map_err(|_| Error::CaptureUnavailable)?;
        fs::rename(temporary, self.0.directory.join("manifest.json"))
            .map_err(|_| Error::CaptureUnavailable)?;
        state.manifest = manifest;
        state.observed_native = Some(id.clone());
        Ok(())
    }

    /// An actual producer occurrence may be supplied; entity/request IDs are not occurrences.
    pub fn append(
        &self,
        raw: &str,
        occurrence: Option<&EventOccurrenceId>,
    ) -> Result<SourcePosition, Error> {
        if raw.len() > MAX_RAW {
            return Err(Error::CaptureUnavailable);
        }
        let id = event_native(raw)?;
        let mut state = self.0.state.lock().map_err(|_| Error::CaptureUnavailable)?;
        if state.failed {
            return Err(Error::CaptureCorrupt);
        }
        if state
            .observed_native
            .as_ref()
            .is_some_and(|expected| expected != &id)
        {
            return Err(Error::CaptureConflict);
        }
        if let Some(occurrence) = occurrence {
            let mut reader = BufReader::new(
                private_file(&self.0.directory.join("source.jsonl"), false)?.take(state.end),
            );
            while let Some(record) = next(&mut reader)? {
                if record.position.occurrence() == occurrence {
                    return if record.raw == raw {
                        Ok(record.position)
                    } else {
                        Err(Error::CaptureConflict)
                    };
                }
            }
        }
        let occurrence = match occurrence {
            Some(id) => id.clone(),
            None => {
                EventOccurrenceId::parse(opaque("event")?).map_err(|_| Error::CaptureCorrupt)?
            }
        };
        let ordinal = state.last.as_ref().map_or(Ok(1), |p| {
            p.ordinal().checked_add(1).ok_or(Error::CaptureCorrupt)
        })?;
        let position =
            SourcePosition::new(ordinal, occurrence).map_err(|_| Error::CaptureCorrupt)?;
        let record = DiskRecord {
            ordinal,
            occurrence: position.occurrence().as_str().into(),
            raw: raw.into(),
        };
        let mut bytes = serde_json::to_vec(&record).map_err(|_| Error::CaptureCorrupt)?;
        bytes.push(b'\n');
        let end = state
            .end
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Error::CaptureCorrupt)?)
            .ok_or(Error::CaptureCorrupt)?;
        let written = state
            .file
            .write_all(&bytes)
            .and_then(|_| state.file.flush());
        if written.is_err() {
            state.failed = true;
            return Err(Error::CaptureUnavailable);
        }
        state.end = end;
        state.last = Some(position.clone());
        state.observed_native = Some(id);
        Ok(position)
    }

    pub fn begin(
        &self,
        expected: &SnapshotId,
        after: Option<&SourcePosition>,
    ) -> Result<CapturePass, Error> {
        let permit = self
            .0
            .readers
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::NotAdmitted)?;
        let (boundary, end) = {
            let state = self.0.state.lock().map_err(|_| Error::CaptureUnavailable)?;
            if state.failed {
                return Err(Error::CaptureCorrupt);
            }
            let current = snapshot(&state.manifest)?;
            if &current != expected {
                return Err(Error::CaptureUnavailable);
            }
            let native_session = state
                .manifest
                .native_session
                .as_ref()
                .ok_or(Error::CaptureUnavailable)
                .and_then(|v| native(v))?;
            (
                CaptureBoundary {
                    snapshot: current,
                    native_session,
                    through: state.last.clone(),
                },
                state.end,
            )
        };
        let mut file = private_file(&self.0.directory.join("source.jsonl"), false)?;
        file.seek(SeekFrom::Start(0))
            .map_err(|_| Error::CaptureUnavailable)?;
        let mut reader = BufReader::new(file.take(end));
        if let Some(after) = after {
            loop {
                let record = next(&mut reader)?.ok_or(Error::CaptureUnavailable)?;
                if record.position.occurrence() == after.occurrence() {
                    if record.position != *after {
                        return Err(Error::CaptureConflict);
                    }
                    break;
                }
            }
        }
        Ok(CapturePass {
            boundary,
            reader,
            last: after.cloned(),
            _permit: permit,
        })
    }
}
impl CapturePass {
    pub fn next_event(&mut self) -> Result<Option<CapturedEvent>, Error> {
        let record = next(&mut self.reader)?;
        if let Some(record) = &record {
            let expected = self.last.as_ref().map_or(Ok(1), |p| {
                p.ordinal().checked_add(1).ok_or(Error::CaptureCorrupt)
            })?;
            if record.position.ordinal() != expected {
                return Err(Error::CaptureCorrupt);
            }
            self.last = Some(record.position.clone());
        } else if self.last != self.boundary.through {
            return Err(Error::CaptureCorrupt);
        }
        Ok(record)
    }
}
fn private_file(path: &Path, create: bool) -> Result<File, Error> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .append(true)
        .custom_flags(nix::libc::O_NOFOLLOW);
    if create {
        options.create_new(true).mode(0o600);
    }
    let file = options.open(path).map_err(|_| Error::CaptureUnavailable)?;
    if !file
        .metadata()
        .map_err(|_| Error::CaptureUnavailable)?
        .is_file()
    {
        return Err(Error::CaptureCorrupt);
    }
    Ok(file)
}
fn snapshot(manifest: &Manifest) -> Result<SnapshotId, Error> {
    SnapshotId::parse(manifest.snapshot.clone()).map_err(|_| Error::CaptureCorrupt)
}
fn native(id: &str) -> Result<NativeConversationId, Error> {
    NativeConversationId::parse(id.into()).map_err(|_| Error::CaptureCorrupt)
}
fn opaque(prefix: &str) -> Result<String, Error> {
    let digest = chapchoo_crypto::BearerSecret::generate()
        .map_err(|_| Error::Process)?
        .digest();
    Ok(format!(
        "{prefix}-{}",
        digest
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}
fn next(reader: &mut impl BufRead) -> Result<Option<CapturedEvent>, Error> {
    let mut bytes = Vec::new();
    let count = reader
        .take(u64::try_from(MAX_RECORD + 1).map_err(|_| Error::CaptureCorrupt)?)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| Error::CaptureCorrupt)?;
    if count == 0 {
        return Ok(None);
    }
    if count > MAX_RECORD || bytes.last() != Some(&b'\n') {
        return Err(Error::CaptureCorrupt);
    }
    let disk: DiskRecord = serde_json::from_slice(&bytes).map_err(|_| Error::CaptureCorrupt)?;
    if disk.raw.len() > MAX_RAW {
        return Err(Error::CaptureCorrupt);
    }
    let _ = event_native(&disk.raw)?;
    let position = SourcePosition::new(
        disk.ordinal,
        EventOccurrenceId::parse(disk.occurrence).map_err(|_| Error::CaptureCorrupt)?,
    )
    .map_err(|_| Error::CaptureCorrupt)?;
    Ok(Some(CapturedEvent {
        position,
        raw: disk.raw,
    }))
}
#[derive(Deserialize)]
struct EventEnvelope {
    method: String,
    params: EventParams,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventParams {
    session_id: String,
}
fn event_native(raw: &str) -> Result<NativeConversationId, Error> {
    let envelope: EventEnvelope = serde_json::from_str(raw).map_err(|_| Error::CaptureCorrupt)?;
    if !matches!(
        envelope.method.as_str(),
        "session/update" | "session/request_permission" | "elicitation/create"
    ) {
        return Err(Error::CaptureCorrupt);
    }
    native(&envelope.params.session_id)
}

#[cfg(test)]
mod tests;
