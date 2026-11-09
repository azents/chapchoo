//! Finite raw-event assembly. Capacity follows bytes through final DB publication.
use chapchoo_domain::{SnapshotId, SourcePosition};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const MAX_EVENT: u32 = 16 * 1024 * 1024;
const MAX_PIECE: usize = 32 * 1024;
const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const TRANSFERS: usize = 4;
const LIFETIME: Duration = Duration::from_secs(30);

#[derive(Clone, PartialEq, Eq)]
pub(super) struct EventIdentity {
    pub session: String,
    pub control: String,
    pub connection: String,
    pub conversation: String,
    pub snapshot: SnapshotId,
    pub position: SourcePosition,
    pub transfer: String,
    pub total: u32,
}
type Key = (String, String, String);
impl EventIdentity {
    fn key(&self) -> Key {
        (
            self.control.clone(),
            self.connection.clone(),
            self.transfer.clone(),
        )
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Error {
    Invalid,
    Conflict,
    Capacity,
    Missing,
    Unavailable,
}
struct Entry {
    identity: EventIdentity,
    bytes: Vec<u8>,
    expires: Instant,
    _bytes: OwnedSemaphorePermit,
    _slot: OwnedSemaphorePermit,
}
pub(super) struct ReceivedEvent {
    pub identity: EventIdentity,
    pub bytes: Vec<u8>,
    _bytes: OwnedSemaphorePermit,
    _slot: OwnedSemaphorePermit,
}
pub(super) enum Progress {
    Accepted(u32),
    Complete(Box<ReceivedEvent>),
}
pub(super) struct Assembler {
    entries: Mutex<BTreeMap<Key, Entry>>,
    bytes: Arc<Semaphore>,
    slots: Arc<Semaphore>,
}
impl std::fmt::Debug for Assembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Assembler([REDACTED])")
    }
}
impl Default for Assembler {
    fn default() -> Self {
        Self {
            entries: Mutex::default(),
            bytes: Arc::new(Semaphore::new(TOTAL_BYTES)),
            slots: Arc::new(Semaphore::new(TRANSFERS)),
        }
    }
}
impl Assembler {
    pub(super) fn append(
        &self,
        identity: EventIdentity,
        offset: u32,
        content: &[u8],
        now: Instant,
    ) -> Result<Progress, Error> {
        let length = u32::try_from(content.len()).map_err(|_| Error::Invalid)?;
        let end = offset.checked_add(length).ok_or(Error::Invalid)?;
        if identity.total == 0
            || identity.total > MAX_EVENT
            || content.is_empty()
            || content.len() > MAX_PIECE
            || end > identity.total
        {
            return Err(Error::Invalid);
        }
        let key = identity.key();
        let mut entries = self.entries.lock().map_err(|_| Error::Unavailable)?;
        // A dead incomplete transfer never permanently consumes another caller's
        // budget. This does not affect a completed event already owned by ingest.
        entries.retain(|_, entry| entry.expires > now);
        if !entries.contains_key(&key) {
            if offset != 0 {
                return Err(Error::Missing);
            }
            let slot = self
                .slots
                .clone()
                .try_acquire_owned()
                .map_err(|_| Error::Capacity)?;
            let budget = self
                .bytes
                .clone()
                .try_acquire_many_owned(identity.total)
                .map_err(|_| Error::Capacity)?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(identity.total as usize)
                .map_err(|_| Error::Capacity)?;
            entries.insert(
                key.clone(),
                Entry {
                    identity: identity.clone(),
                    bytes,
                    expires: now + LIFETIME,
                    _bytes: budget,
                    _slot: slot,
                },
            );
        }
        let entry = entries.get_mut(&key).ok_or(Error::Unavailable)?;
        if entry.identity != identity {
            return Err(Error::Conflict);
        }
        let accepted = u32::try_from(entry.bytes.len()).map_err(|_| Error::Unavailable)?;
        if offset < accepted {
            if end > accepted || entry.bytes.get(offset as usize..end as usize) != Some(content) {
                return Err(Error::Conflict);
            }
            return Ok(Progress::Accepted(accepted));
        }
        if offset != accepted {
            return Err(Error::Conflict);
        }
        entry.bytes.extend_from_slice(content);
        if end < identity.total {
            return Ok(Progress::Accepted(end));
        }
        let entry = entries.remove(&key).ok_or(Error::Unavailable)?;
        Ok(Progress::Complete(Box::new(ReceivedEvent {
            identity: entry.identity,
            bytes: entry.bytes,
            _bytes: entry._bytes,
            _slot: entry._slot,
        })))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chapchoo_domain::EventOccurrenceId;
    type TestError = Box<dyn std::error::Error>;
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for Error {}
    fn identity(transfer: &str, total: u32) -> Result<EventIdentity, TestError> {
        Ok(EventIdentity {
            session: "session".into(),
            control: "control".into(),
            connection: "connection".into(),
            conversation: "conversation".into(),
            snapshot: SnapshotId::parse("snapshot".into())?,
            position: SourcePosition::new(1, EventOccurrenceId::parse("occurrence".into())?)?,
            transfer: transfer.into(),
            total,
        })
    }
    #[test]
    fn pieces_are_exact_and_only_the_complete_event_leaves_assembly() -> Result<(), TestError> {
        let assembler = Assembler::default();
        let now = Instant::now();
        let raw = "한글 🦀 original".as_bytes();
        let meta = identity("transfer", u32::try_from(raw.len())?)?;
        assert!(matches!(
            assembler.append(meta.clone(), 0, &raw[..5], now)?,
            Progress::Accepted(5)
        ));
        assert!(matches!(
            assembler.append(meta.clone(), 0, &raw[..5], now)?,
            Progress::Accepted(5)
        ));
        assert!(matches!(
            assembler.append(meta.clone(), 0, b"other", now),
            Err(Error::Conflict)
        ));
        let changed = EventIdentity {
            snapshot: SnapshotId::parse("other".into())?,
            ..meta.clone()
        };
        assert!(matches!(
            assembler.append(changed, 5, &raw[5..], now),
            Err(Error::Conflict)
        ));
        assert!(matches!(
            assembler.append(meta.clone(), 6, &raw[6..], now),
            Err(Error::Conflict)
        ));
        let Progress::Complete(event) = assembler.append(meta.clone(), 5, &raw[5..], now)? else {
            return Err("Incomplete event".into());
        };
        assert_eq!(event.bytes, raw);
        assert_eq!(event.identity.position, meta.position);
        assert!(matches!(
            assembler.append(meta, 5, &raw[5..], now),
            Err(Error::Missing)
        ));
        Ok(())
    }
    #[test]
    fn final_publication_retains_capacity_and_expiry_reaps_only_incomplete_bytes()
    -> Result<(), TestError> {
        let assembler = Assembler::default();
        let now = Instant::now();
        let mut completed = Vec::new();
        for index in 0..TRANSFERS {
            let Progress::Complete(event) =
                assembler.append(identity(&format!("complete-{index}"), 1)?, 0, b"x", now)?
            else {
                return Err("Expected complete".into());
            };
            completed.push(event);
        }
        assert!(matches!(
            assembler.append(identity("overloaded", 1)?, 0, b"x", now + LIFETIME),
            Err(Error::Capacity)
        ));
        completed.clear();
        assert!(matches!(
            assembler.append(identity("partial", 2)?, 0, b"x", now)?,
            Progress::Accepted(1)
        ));
        assert!(matches!(
            assembler.append(identity("partial", 2)?, 1, b"y", now + LIFETIME),
            Err(Error::Missing)
        ));
        assert_eq!(assembler.bytes.available_permits(), TOTAL_BYTES);
        assert_eq!(assembler.slots.available_permits(), TRANSFERS);
        assert!(matches!(
            assembler.append(identity("again", 1)?, 0, b"x", now + LIFETIME)?,
            Progress::Complete(_)
        ));
        Ok(())
    }
    #[test]
    fn byte_budget_and_piece_limits_cannot_be_bypassed_by_count_bound() -> Result<(), TestError> {
        let assembler = Assembler::default();
        let now = Instant::now();
        for index in 0..2 {
            assert!(matches!(
                assembler.append(
                    identity(&format!("large-{index}"), MAX_EVENT)?,
                    0,
                    b"x",
                    now
                )?,
                Progress::Accepted(1)
            ));
        }
        assert!(matches!(
            assembler.append(identity("byte-overflow", 1)?, 0, b"x", now),
            Err(Error::Capacity)
        ));
        assert!(matches!(
            assembler.append(identity("too-large", MAX_EVENT + 1)?, 0, b"x", now),
            Err(Error::Invalid)
        ));
        assert!(matches!(
            assembler.append(
                identity("piece", MAX_EVENT)?,
                0,
                &vec![b'x'; MAX_PIECE + 1],
                now
            ),
            Err(Error::Invalid)
        ));
        Ok(())
    }
}
