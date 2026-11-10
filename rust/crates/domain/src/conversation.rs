//! Conversation-local values. Parsing does not confer owner or execution authority.

use std::{fmt, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid conversation value")]
pub struct InvalidConversationValue;

/// Owner-selected supply, never an implicit PATH/ambient-auth discovery policy.
#[derive(Clone, PartialEq, Eq, Default)]
pub enum HarnessSupply {
    #[default]
    Managed,
    Existing(ExistingCodexPath),
}
impl fmt::Debug for HarnessSupply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HarnessSupply([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExistingCodexPath(String);
impl ExistingCodexPath {
    pub fn parse(value: String) -> Result<Self, InvalidConversationValue> {
        if value.len() > 4096
            || value.chars().any(char::is_control)
            || !Path::new(&value).is_absolute()
        {
            return Err(InvalidConversationValue);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ExistingCodexPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExistingCodexPath([REDACTED])")
    }
}

/// Exact submitted text; validation does not trim or rewrite the user's input.
#[derive(Clone, PartialEq, Eq)]
pub struct ConversationText(String);
impl ConversationText {
    pub fn parse(value: String) -> Result<Self, InvalidConversationValue> {
        if value.trim().is_empty() || value.contains('\0') {
            return Err(InvalidConversationValue);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ConversationText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationText([REDACTED])")
    }
}

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name(String);
        impl $name {
            pub fn parse(value: String) -> Result<Self, InvalidConversationValue> {
                if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
                    return Err(InvalidConversationValue);
                }
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    };
}
opaque_id!(SnapshotId);
opaque_id!(EventOccurrenceId);
opaque_id!(NativeConversationId);

/// Source position is explicit capture order, not ordering inferred from an ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePosition {
    ordinal: i64,
    occurrence: EventOccurrenceId,
}
impl SourcePosition {
    pub fn new(
        ordinal: i64,
        occurrence: EventOccurrenceId,
    ) -> Result<Self, InvalidConversationValue> {
        if ordinal < 1 {
            return Err(InvalidConversationValue);
        }
        Ok(Self {
            ordinal,
            occurrence,
        })
    }
    pub fn ordinal(&self) -> i64 {
        self.ordinal
    }
    pub fn occurrence(&self) -> &EventOccurrenceId {
        &self.occurrence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationSource {
    NotStarted,
    Loading {
        snapshot: SnapshotId,
        position: Option<SourcePosition>,
    },
    Ready {
        snapshot: SnapshotId,
        position: Option<SourcePosition>,
    },
}
impl ConversationSource {
    pub fn snapshot(&self) -> Option<&SnapshotId> {
        match self {
            Self::NotStarted => None,
            Self::Loading { snapshot, .. } | Self::Ready { snapshot, .. } => Some(snapshot),
        }
    }
    pub fn position(&self) -> Option<&SourcePosition> {
        match self {
            Self::NotStarted => None,
            Self::Loading { position, .. } | Self::Ready { position, .. } => position.as_ref(),
        }
    }
    pub fn accepts_input(&self) -> bool {
        !matches!(self, Self::Loading { .. })
    }
}

/// A command retains its original target even after records are replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationTarget {
    NotStarted,
    Snapshot(SnapshotId),
}
impl ConversationTarget {
    pub fn matches(&self, source: &ConversationSource) -> bool {
        match (self, source) {
            (Self::NotStarted, ConversationSource::NotStarted) => true,
            (Self::Snapshot(expected), ConversationSource::Ready { snapshot, .. }) => {
                expected == snapshot
            }
            _ => false,
        }
    }
    pub fn snapshot(&self) -> Option<&SnapshotId> {
        match self {
            Self::NotStarted => None,
            Self::Snapshot(snapshot) => Some(snapshot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supply_is_explicit_and_does_not_rewrite_an_existing_choice()
    -> Result<(), InvalidConversationValue> {
        assert_eq!(HarnessSupply::default(), HarnessSupply::Managed);
        let path = ExistingCodexPath::parse("/tools/codex".to_owned())?;
        assert_eq!(path.as_str(), "/tools/codex");
        assert_ne!(HarnessSupply::Existing(path), HarnessSupply::default());
        for value in ["", "codex", "~/bin/codex", "/tools/codex\n"] {
            assert!(ExistingCodexPath::parse(value.to_owned()).is_err());
        }
        Ok(())
    }

    #[test]
    fn submitted_text_is_retained_verbatim_and_debug_is_field_free()
    -> Result<(), InvalidConversationValue> {
        let original = "  Keep this spacing\n다음 줄  ";
        let text = ConversationText::parse(original.to_owned())?;
        assert_eq!(text.as_str(), original);
        assert_eq!(format!("{text:?}"), "ConversationText([REDACTED])");
        assert!(ConversationText::parse(" \n\t".to_owned()).is_err());
        assert!(ConversationText::parse("bad\0text".to_owned()).is_err());
        Ok(())
    }

    #[test]
    fn original_target_cannot_cross_loading_or_source_replacement()
    -> Result<(), InvalidConversationValue> {
        let snapshot = SnapshotId::parse("snapshot-one".to_owned())?;
        let target = ConversationTarget::Snapshot(snapshot.clone());
        let loading = ConversationSource::Loading {
            snapshot: snapshot.clone(),
            position: None,
        };
        assert!(!loading.accepts_input());
        assert!(!target.matches(&loading));
        assert!(target.matches(&ConversationSource::Ready {
            snapshot,
            position: None
        }));
        assert!(!target.matches(&ConversationSource::Ready {
            snapshot: SnapshotId::parse("snapshot-two".to_owned())?,
            position: None,
        }));
        assert!(ConversationTarget::NotStarted.matches(&ConversationSource::NotStarted));
        assert!(!ConversationTarget::NotStarted.matches(&loading));
        Ok(())
    }

    #[test]
    fn position_preserves_occurrence_without_assuming_sortable_ids()
    -> Result<(), InvalidConversationValue> {
        let position = SourcePosition::new(1, EventOccurrenceId::parse("z-first".to_owned())?)?;
        assert_eq!(position.ordinal(), 1);
        assert_eq!(position.occurrence().as_str(), "z-first");
        assert!(SourcePosition::new(0, EventOccurrenceId::parse("a".to_owned())?).is_err());
        assert!(SnapshotId::parse("bad\nsource".to_owned()).is_err());
        Ok(())
    }
}
