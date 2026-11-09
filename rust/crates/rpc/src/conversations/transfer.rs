//! Finite owner uploads retain byte/slot capacity through product acceptance.
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tonic::Status;

type Key = ([u8; 32], String, String);
const LIFETIME: Duration = Duration::from_secs(30);
const MAX_BYTES: u32 = 16 * 1024 * 1024;
struct Entry<I> {
    identity: I,
    total: u32,
    bytes: Vec<u8>,
    expires: Instant,
    bytes_permit: OwnedSemaphorePermit,
    slot: OwnedSemaphorePermit,
}
pub(super) struct Complete {
    pub bytes: Vec<u8>,
    _bytes_permit: OwnedSemaphorePermit,
    _slot: OwnedSemaphorePermit,
}
pub(super) enum Progress {
    Uploading(u32),
    Complete(Complete),
}
pub(super) struct Assembler<I> {
    entries: Mutex<BTreeMap<Key, Entry<I>>>,
    bytes: Arc<Semaphore>,
    slots: Arc<Semaphore>,
}
impl<I> std::fmt::Debug for Assembler<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OwnerUploads([REDACTED])")
    }
}
impl<I> Default for Assembler<I> {
    fn default() -> Self {
        Self {
            entries: Mutex::default(),
            bytes: Arc::new(Semaphore::new(32 * 1024 * 1024)),
            slots: Arc::new(Semaphore::new(4)),
        }
    }
}
pub(super) struct Piece<'a, I> {
    pub key: Key,
    pub identity: I,
    pub total: u32,
    pub offset: u32,
    pub bytes: &'a [u8],
}
impl<I: PartialEq> Assembler<I> {
    pub(super) fn append(&self, piece: Piece<'_, I>, now: Instant) -> Result<Progress, Status> {
        let end = piece
            .offset
            .checked_add(u32::try_from(piece.bytes.len()).map_err(|_| invalid())?)
            .ok_or_else(invalid)?;
        if piece.total == 0
            || piece.total > MAX_BYTES
            || piece.bytes.is_empty()
            || piece.bytes.len() > 32 * 1024
            || end > piece.total
        {
            return Err(invalid());
        }
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| Status::unavailable("Owner uploads are unavailable"))?;
        entries.retain(|_, entry| entry.expires > now);
        if let std::collections::btree_map::Entry::Vacant(vacant) = entries.entry(piece.key.clone())
        {
            if piece.offset != 0 {
                return Err(Status::failed_precondition(
                    "Upload is absent; query acceptance or restart complete byte transfer",
                ));
            }
            let slot = self
                .slots
                .clone()
                .try_acquire_owned()
                .map_err(|_| capacity())?;
            let bytes_permit = self
                .bytes
                .clone()
                .try_acquire_many_owned(piece.total)
                .map_err(|_| capacity())?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(piece.total as usize)
                .map_err(|_| capacity())?;
            bytes.extend_from_slice(piece.bytes);
            let entry = Entry {
                identity: piece.identity,
                total: piece.total,
                bytes,
                expires: now + LIFETIME,
                bytes_permit,
                slot,
            };
            if end == piece.total {
                return Ok(Progress::Complete(complete(entry)));
            }
            vacant.insert(entry);
            return Ok(Progress::Uploading(end));
        }
        let entry = entries
            .get_mut(&piece.key)
            .ok_or_else(|| Status::unavailable("Owner upload disappeared"))?;
        if entry.identity != piece.identity || entry.total != piece.total {
            return Err(conflict());
        }
        let accepted = u32::try_from(entry.bytes.len()).map_err(|_| invalid())?;
        if piece.offset < accepted {
            if end > accepted
                || entry.bytes.get(piece.offset as usize..end as usize) != Some(piece.bytes)
            {
                return Err(conflict());
            }
            return Ok(Progress::Uploading(accepted));
        }
        if piece.offset != accepted {
            return Err(conflict());
        }
        entry.bytes.extend_from_slice(piece.bytes);
        if end != piece.total {
            return Ok(Progress::Uploading(end));
        }
        let entry = entries
            .remove(&piece.key)
            .ok_or_else(|| Status::unavailable("Owner upload disappeared"))?;
        Ok(Progress::Complete(complete(entry)))
    }
}
fn complete<I>(entry: Entry<I>) -> Complete {
    Complete {
        bytes: entry.bytes,
        _bytes_permit: entry.bytes_permit,
        _slot: entry.slot,
    }
}
fn invalid() -> Status {
    Status::invalid_argument("Owner upload piece is invalid")
}
fn conflict() -> Status {
    Status::already_exists("Owner upload conflicts with its retained identity or bytes")
}
fn capacity() -> Status {
    Status::resource_exhausted("Owner upload capacity is exhausted")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn piece<'a>(
        request: &str,
        identity: u8,
        total: u32,
        offset: u32,
        bytes: &'a [u8],
    ) -> Piece<'a, u8> {
        Piece {
            key: ([0; 32], "agent".into(), request.into()),
            identity,
            total,
            offset,
            bytes,
        }
    }
    #[test]
    fn complete_utf8_identity_conflicts_and_expiry() -> Result<(), Box<dyn std::error::Error>> {
        let uploads = Assembler::default();
        let now = Instant::now();
        let text = "한글🦀".as_bytes();
        let total = u32::try_from(text.len())?;
        assert!(matches!(
            uploads.append(piece("one", 1, total, 0, &text[..2]), now)?,
            Progress::Uploading(2)
        ));
        assert!(matches!(
            uploads.append(piece("one", 1, total, 0, &text[..2]), now)?,
            Progress::Uploading(2)
        ));
        assert_eq!(
            uploads
                .append(piece("one", 2, total, 2, &text[2..]), now)
                .err()
                .ok_or("Changed target accepted")?
                .code(),
            tonic::Code::AlreadyExists
        );
        assert_eq!(
            uploads
                .append(piece("one", 1, total, 0, b"xx"), now)
                .err()
                .ok_or("Changed bytes accepted")?
                .code(),
            tonic::Code::AlreadyExists
        );
        let Progress::Complete(complete) =
            uploads.append(piece("one", 1, total, 2, &text[2..]), now)?
        else {
            return Err("Expected complete bytes".into());
        };
        assert_eq!(complete.bytes, text);
        assert_eq!(
            uploads.bytes.available_permits(),
            32 * 1024 * 1024 - text.len()
        );
        drop(complete);
        assert_eq!(uploads.bytes.available_permits(), 32 * 1024 * 1024);
        uploads.append(piece("expired", 1, total, 0, &text[..2]), now)?;
        assert_eq!(
            uploads
                .append(piece("expired", 1, total, 2, &text[2..]), now + LIFETIME)
                .err()
                .ok_or("Expired upload resumed")?
                .code(),
            tonic::Code::FailedPrecondition
        );
        assert_eq!(uploads.slots.available_permits(), 4);
        Ok(())
    }
    #[test]
    fn incomplete_and_complete_uploads_share_physical_capacity()
    -> Result<(), Box<dyn std::error::Error>> {
        let uploads = Assembler::default();
        let now = Instant::now();
        for request in ["large-one", "large-two"] {
            uploads.append(piece(request, 1, MAX_BYTES, 0, b"x"), now)?;
        }
        assert_eq!(
            uploads
                .append(piece("overflow", 1, 1, 0, b"x"), now)
                .err()
                .ok_or("Byte budget bypassed")?
                .code(),
            tonic::Code::ResourceExhausted
        );
        assert_eq!(
            uploads
                .append(piece("invalid", 1, MAX_BYTES + 1, 0, b"x"), now)
                .err()
                .ok_or("Size bound bypassed")?
                .code(),
            tonic::Code::InvalidArgument
        );
        let mut completed = Vec::new();
        for index in 0..4 {
            let Progress::Complete(value) = uploads.append(
                piece(&format!("completed-{index}"), 1, 1, 0, b"x"),
                now + LIFETIME,
            )?
            else {
                return Err("Expected complete".into());
            };
            completed.push(value);
        }
        assert_eq!(
            uploads
                .append(piece("no-slot", 1, 1, 0, b"x"), now + LIFETIME)
                .err()
                .ok_or("Slot budget bypassed")?
                .code(),
            tonic::Code::ResourceExhausted
        );
        drop(completed);
        assert_eq!(uploads.slots.available_permits(), 4);
        Ok(())
    }
}
