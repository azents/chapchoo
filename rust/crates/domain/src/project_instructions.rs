//! Selected project instruction originals, independent of a native harness.
use std::fmt;

pub const MAX_OWNER_INSTRUCTION_BYTES: usize = 16 * 1024;
pub const MAX_PROJECT_INSTRUCTION_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionSourceKind {
    Product,
    Function,
    Owner,
    Execution,
}
impl InstructionSourceKind {
    fn heading(self) -> &'static str {
        match self {
            Self::Product => "Product instructions",
            Self::Function => "Function instructions",
            Self::Owner => "Owner instructions",
            Self::Execution => "Execution instructions",
        }
    }
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstructionSource {
    pub kind: InstructionSourceKind,
    pub text: String,
}
impl fmt::Debug for InstructionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InstructionSource([REDACTED])")
    }
}

#[derive(Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectInstructions {
    pub sources: Vec<InstructionSource>,
}
impl fmt::Debug for ProjectInstructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectInstructions([REDACTED])")
    }
}
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[error("Project instruction content exceeds its supported bounds")]
pub struct InvalidProjectInstructions;

impl ProjectInstructions {
    pub fn new(sources: Vec<InstructionSource>) -> Result<Self, InvalidProjectInstructions> {
        let value = Self { sources };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), InvalidProjectInstructions> {
        if self.sources.len() > 32
            || self.sources.iter().any(|source| {
                source.text.len() > MAX_PROJECT_INSTRUCTION_BYTES
                    || (source.kind == InstructionSourceKind::Owner
                        && source.text.len() > MAX_OWNER_INSTRUCTION_BYTES)
                    || source.text.contains('\0')
            })
            || self.document().len() > MAX_PROJECT_INSTRUCTION_BYTES
        {
            return Err(InvalidProjectInstructions);
        }
        Ok(())
    }
    /// Deterministic presentation; original text remains separately retained.
    pub fn document(&self) -> String {
        self.sources
            .iter()
            .filter(|source| !source.text.is_empty())
            .map(|source| format!("# {}\n\n{}\n", source.kind.heading(), source.text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_sources_render_and_clear_without_changing_other_sources()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut value = ProjectInstructions::new(vec![
            InstructionSource {
                kind: InstructionSourceKind::Product,
                text: "Product\n".into(),
            },
            InstructionSource {
                kind: InstructionSourceKind::Owner,
                text: "  소유자\n\n".into(),
            },
        ])?;
        assert!(value.document().contains("  소유자\n\n"));
        assert_eq!(
            serde_json::from_str::<ProjectInstructions>(&serde_json::to_string(&value)?)?,
            value
        );
        value.sources[1].text.clear();
        assert_eq!(value.document(), "# Product instructions\n\nProduct\n\n");
        assert_eq!(format!("{value:?}"), "ProjectInstructions([REDACTED])");
        Ok(())
    }
    #[test]
    fn utf8_limit_and_document_limit_are_byte_bounds() {
        let value = ProjectInstructions {
            sources: vec![InstructionSource {
                kind: InstructionSourceKind::Owner,
                text: "한".repeat(MAX_OWNER_INSTRUCTION_BYTES / 3 + 1),
            }],
        };
        assert!(value.validate().is_err());
        let value = ProjectInstructions {
            sources: vec![InstructionSource {
                kind: InstructionSourceKind::Product,
                text: "p".repeat(MAX_PROJECT_INSTRUCTION_BYTES),
            }],
        };
        assert!(value.validate().is_err());
        assert_eq!(ProjectInstructions::default().document(), "");
    }
}
