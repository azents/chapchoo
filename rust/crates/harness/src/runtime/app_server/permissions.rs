//! Codex-native presets. Availability is reported by the connected App Server.
use super::{call, wire::Client};
use crate::{Error, ProtocolStage};
use chapchoo_domain::ConversationPermissionMode as Mode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Preset {
    permissions: &'static str,
    approval_policy: &'static str,
    approvals_reviewer: &'static str,
}

impl From<Mode> for Preset {
    fn from(mode: Mode) -> Self {
        Self {
            permissions: match mode {
                Mode::FullAccess => ":danger-full-access",
                _ => ":workspace",
            },
            approval_policy: match mode {
                Mode::FullAccess => "never",
                _ => "on-request",
            },
            approvals_reviewer: match mode {
                Mode::ApproveForMe => "auto_review",
                _ => "user",
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
    data: Vec<T>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
struct Profile {
    id: String,
    allowed: bool,
}

#[derive(Deserialize)]
struct Feature {
    name: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct ConfigRequirements {
    requirements: Option<Requirements>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Requirements {
    allowed_approval_policies: Option<Vec<Value>>,
    allowed_approvals_reviewers: Option<Vec<String>>,
}

fn supported(profiles: &[Profile], guardian: bool, requirements: &Requirements) -> Vec<Mode> {
    Mode::DEFAULT_ORDER
        .into_iter()
        .filter(|mode| {
            let preset = Preset::from(*mode);
            (*mode != Mode::ApproveForMe || guardian)
                && profiles
                    .iter()
                    .any(|p| p.id == preset.permissions && p.allowed)
                && requirements
                    .allowed_approval_policies
                    .as_ref()
                    .is_none_or(|rules| {
                        rules
                            .iter()
                            .any(|rule| rule.as_str() == Some(preset.approval_policy))
                    })
                && requirements
                    .allowed_approvals_reviewers
                    .as_ref()
                    .is_none_or(|rules| rules.iter().any(|rule| rule == preset.approvals_reviewer))
        })
        .collect()
}

async fn list<T: DeserializeOwned>(
    link: &Client,
    method: &str,
    mut params: Value,
) -> Result<Vec<T>, Error> {
    let mut result = Vec::new();
    for _ in 0..4 {
        let page: Page<T> =
            serde_json::from_value(call(link, ProtocolStage::Session, method, &params).await?)
                .map_err(|_| Error::Protocol)?;
        result.extend(page.data);
        let Some(cursor) = page.next_cursor else {
            return Ok(result);
        };
        if cursor.is_empty() || cursor.len() > 256 || params["cursor"].as_str() == Some(&cursor) {
            return Err(Error::Protocol);
        }
        params["cursor"] = json!(cursor);
    }
    Err(Error::Protocol)
}

pub(super) async fn discover(link: &Client, cwd: &Path, thread: &str) -> Result<Vec<Mode>, Error> {
    let (profiles, features, requirements) = tokio::try_join!(
        list::<Profile>(
            link,
            "permissionProfile/list",
            json!({"cwd": cwd, "limit": 128})
        ),
        list::<Feature>(
            link,
            "experimentalFeature/list",
            json!({"threadId": thread, "limit": 128})
        ),
        call(
            link,
            ProtocolStage::Session,
            "configRequirements/read",
            json!({})
        ),
    )?;
    let config: ConfigRequirements =
        serde_json::from_value(requirements).map_err(|_| Error::Protocol)?;
    Ok(supported(
        &profiles,
        features
            .iter()
            .any(|f| f.name == "guardian_approval" && f.enabled),
        &config.requirements.unwrap_or_default(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_order_only_uses_eligible_modes() -> Result<(), serde_json::Error> {
        let mut profiles = vec![
            Profile {
                id: ":workspace".into(),
                allowed: true,
            },
            Profile {
                id: ":danger-full-access".into(),
                allowed: true,
            },
        ];
        let requirements = Requirements::default();
        assert_eq!(
            supported(&profiles, true, &requirements),
            Mode::DEFAULT_ORDER
        );
        assert_eq!(
            Mode::default_from(&supported(&profiles, false, &requirements)),
            Some(Mode::FullAccess),
        );
        profiles[1].allowed = false;
        assert_eq!(
            supported(&profiles, false, &requirements),
            [Mode::AskForApproval]
        );
        let restricted = Requirements {
            allowed_approvals_reviewers: Some(vec!["user".into()]),
            ..Requirements::default()
        };
        assert_eq!(
            supported(&profiles, true, &restricted),
            [Mode::AskForApproval]
        );
        let manual = serde_json::to_value(Preset::from(Mode::AskForApproval))?;
        assert_eq!(
            manual,
            json!({
                "permissions": ":workspace",
                "approvalPolicy": "on-request",
                "approvalsReviewer": "user",
            })
        );
        Ok(())
    }
}
