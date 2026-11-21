//! Native operations owned by the admitted Codex thread.
use serde::Serialize;

#[derive(Clone, Copy)]
pub(in crate::runtime) enum Command<'a> {
    Compact,
    Review(ReviewTarget<'a>),
    Rename(&'a str),
    Invalid,
}

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(in crate::runtime) enum ReviewTarget<'a> {
    UncommittedChanges,
    Custom { instructions: &'a str },
    BaseBranch { branch: &'a str },
    Commit { sha: &'a str },
}

impl<'a> Command<'a> {
    pub(in crate::runtime) fn parse(name: &str, rest: &'a str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "compact" => Self::Compact,
            "review" if rest.is_empty() => Self::Review(ReviewTarget::UncommittedChanges),
            "review" => Self::Review(ReviewTarget::Custom { instructions: rest }),
            "review-branch" if !rest.is_empty() => {
                Self::Review(ReviewTarget::BaseBranch { branch: rest })
            }
            "review-commit" if !rest.is_empty() => Self::Review(ReviewTarget::Commit { sha: rest }),
            "rename" if super::tools::visible(rest) && !rest.chars().any(char::is_control) => {
                Self::Rename(rest)
            }
            "review-branch" | "review-commit" | "rename" => Self::Invalid,
            _ => return None,
        })
    }

    pub(in crate::runtime) fn starts_turn(self) -> bool {
        matches!(self, Self::Compact | Self::Review(_))
    }
}
