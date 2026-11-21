//! Shared permission facts for a sandbox consent offer and its selected grant.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SandboxPermissionProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_system: Option<FileSystem>,
}

impl std::fmt::Debug for SandboxPermissionProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SandboxPermissionProfile([REDACTED])")
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Network {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FileSystem {
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    write: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<Entry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    glob_scan_max_depth: Option<usize>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: Path,
    access: Access,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Access {
    Read,
    Write,
    Deny,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Path {
    #[serde(rename = "path")]
    Literal {
        path: String,
    },
    GlobPattern {
        pattern: String,
    },
    Special {
        value: Special,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Special {
    Root,
    Minimal,
    ProjectRoots { subpath: Option<String> },
    Tmpdir,
    SlashTmp,
}

impl SandboxPermissionProfile {
    /// Render the same typed profile that would be granted, without rendering
    /// native paths or explanation as markup.
    pub fn consent_description(
        &self,
        cwd: &str,
        environment: Option<&str>,
        reason: Option<&str>,
    ) -> Option<String> {
        if !single_line(cwd)
            || environment.is_some_and(|id| {
                id.is_empty() || id.len() > 256 || !id.bytes().all(|byte| byte.is_ascii_graphic())
            })
            || reason.is_some_and(|text| !displayable(text, true))
        {
            return None;
        }
        let mut has_scope = self
            .network
            .as_ref()
            .is_some_and(|network| network.enabled.is_some());
        if let Some(files) = &self.file_system {
            if files.glob_scan_max_depth == Some(0)
                || [&files.read, &files.write]
                    .into_iter()
                    .flatten()
                    .any(|paths| paths.len() > 64 || paths.iter().any(|path| !single_line(path)))
                || files.entries.as_ref().is_some_and(|entries| {
                    entries.len() > 64 || entries.iter().any(|entry| !entry.path.valid())
                })
            {
                return None;
            }
            has_scope |= files.read.as_ref().is_some_and(|paths| !paths.is_empty())
                || files.write.as_ref().is_some_and(|paths| !paths.is_empty())
                || files
                    .entries
                    .as_ref()
                    .is_some_and(|paths| !paths.is_empty());
        }
        if !has_scope {
            return None;
        }
        let profile = serde_json::to_string_pretty(self).ok()?;
        let mut result = format!(
            "Directory:\n\n{}\n\nRequested permissions:\n\n{}",
            literal(cwd),
            literal(&profile)
        );
        if let Some(id) = environment {
            result.push_str(&format!("\n\nEnvironment:\n\n{}", literal(id)));
        }
        if let Some(reason) = reason.filter(|text| !text.trim().is_empty()) {
            result.push_str(&format!(
                "\n\nReason from the harness:\n\n{}",
                literal(reason)
            ));
        }
        (result.len() <= 8192).then_some(result)
    }
}

fn displayable(value: &str, multiline: bool) -> bool {
    value.len() <= 8192 && !value.chars().any(|c| {
        (c.is_control() && !(multiline && matches!(c, '\n' | '\t')))
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    })
}

fn single_line(value: &str) -> bool {
    !value.trim().is_empty() && displayable(value, false)
}

fn literal(content: &str) -> String {
    let ticks = content.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(ticks + 1));
    format!("{fence}\n{content}\n{fence}")
}

impl Path {
    fn valid(&self) -> bool {
        match self {
            Self::Literal { path } => single_line(path),
            Self::GlobPattern { pattern } => single_line(pattern),
            Self::Special {
                value: Special::ProjectRoots { subpath },
            } => subpath.as_deref().is_none_or(single_line),
            Self::Special { .. } => true,
        }
    }
}
