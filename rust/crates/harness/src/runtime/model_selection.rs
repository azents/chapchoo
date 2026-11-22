//! Model options observed from the owned strict-v2 session.

use super::*;
use chapchoo_domain::ConversationModelTuning;
use std::collections::HashSet;
use std::{future::Future, pin::Pin};

pub type NativeModelFuture<'a> =
    Pin<Box<dyn Future<Output = Result<NativeModelReservation, Error>> + Send + 'a>>;

#[derive(Debug)]
pub enum NativeModelReservation {
    Ungoverned,
    Reserved {
        model: Option<String>,
        tuning: ConversationModelTuning,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeModelStart {
    pub conversation_id: String,
    pub operation_id: String,
    pub launch_operation_id: String,
    pub target: NativeTextTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeModelStartOutcome {
    Inserted(String),
    NotPublished,
    /// The adapter proved both the reason and that dispatch did not start.
    ModelUnavailableBeforeStart(String),
}

pub trait NativeModelAuthority: Send + Sync {
    fn reserve<'a>(
        &'a self,
        start: &'a NativeModelStart,
        observed: NativeModelObservation,
        capable: bool,
    ) -> NativeModelFuture<'a>;
    /// Retain exact outcomes locally until Control accepts them. Never
    /// acquire a second grant or resend a native request from this receipt.
    fn retain(&self, start: &NativeModelStart, outcome: NativeModelStartOutcome);
}

pub struct NativeModelDispatch {
    pub authority: Arc<dyn NativeModelAuthority>,
    pub route: ContinuationRoute,
}
impl fmt::Debug for NativeModelDispatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeModelDispatch")
            .finish_non_exhaustive()
    }
}

pub(super) enum PromptKind<'a> {
    Ordinary { managed: bool },
    Continuation(&'a NativeNormalEnding),
}

struct BeforePublish<'a> {
    authority: &'a dyn NativeModelAuthority,
    start: &'a NativeModelStart,
    unsent: bool,
}
impl Drop for BeforePublish<'_> {
    fn drop(&mut self) {
        if self.unsent {
            self.authority
                .retain(self.start, NativeModelStartOutcome::NotPublished);
        }
    }
}

/// Match only the vetted adapter's built-in command tokens. Slash-prefixed
/// paths and other ordinary text still go through the governed start.
fn native_command(text: &str) -> bool {
    // The pinned adapter uses ECMAScript trim and /\s+/, which include
    // the BOM and exclude NEL (unlike Rust's Unicode whitespace).
    let whitespace = |c: char| (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}';
    let Some(command) = text.trim_matches(whitespace).strip_prefix('/') else {
        return false;
    };
    command
        .trim_matches(whitespace)
        .split(whitespace)
        .next()
        .is_some_and(|name| {
            matches!(
                name.to_lowercase().as_str(),
                "plan"
                    | "mcp"
                    | "skills"
                    | "status"
                    | "review"
                    | "review-branch"
                    | "review-commit"
                    | "compact"
                    | "goal"
                    | "rename"
                    | "logout"
            )
        })
}

pub(super) async fn prompt(
    live: &Live,
    kind: PromptKind<'_>,
    operation_id: &str,
    text: &ConversationText,
    dispatch: Option<&NativeModelDispatch>,
) -> Result<NativePromptAcceptance, Error> {
    let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
    let observations = &live.observations;
    let direct = matches!(session, Session::AppServer(_));
    let vetted = direct || qualified(&live._supply.versions);
    let builtin = !direct
        && live._supply.versions.adapter == "2.8.0"
        && matches!(kind, PromptKind::Ordinary { .. })
        && native_command(text.as_str());
    let target = observations.current()?.target.ok_or(Error::NotAdmitted)?;
    let start = NativeModelStart {
        conversation_id: live._authentication.conversation_id.clone(),
        operation_id: operation_id.into(),
        launch_operation_id: live._authentication.operation_id.clone(),
        target,
    };
    let mut before = None;
    let mut reserved = false;
    let mut model_for_turn = None;
    let mut tuning_for_turn = ConversationModelTuning::default();
    if let Some(dispatch) = dispatch.filter(|_| !builtin) {
        dispatch
            .route
            .publish(|| observations.check_model_prompt(&kind, vetted))?;
        let observed = observe(live)?;
        before = Some(BeforePublish {
            authority: dispatch.authority.as_ref(),
            start: &start,
            unsent: true,
        });
        match dispatch.authority.reserve(&start, observed, vetted).await? {
            NativeModelReservation::Ungoverned => {
                if let Some(before) = &mut before {
                    before.unsent = false;
                }
            }
            NativeModelReservation::Reserved { model, tuning } => {
                if !vetted || (!direct && !tuning.is_inherited()) {
                    return Err(Error::Protocol);
                }
                reserved = true;
                if direct {
                    model_for_turn = model;
                    tuning_for_turn = tuning;
                } else if let Some(model) = model {
                    let options = observations
                        .model(&start.launch_operation_id, live.api_key_source)?
                        .options
                        .ok_or(Error::NotAdmitted)?;
                    if options
                        .current
                        .as_ref()
                        .is_none_or(|current| current.id != model)
                    {
                        if !options.options.iter().any(|item| item.id == model) {
                            return Err(Error::NotAdmitted);
                        }
                        let request = dispatch
                            .route
                            .publish(|| {
                                observations.with_model_idle(&kind, || {
                                    session.enqueue_model(options.option_id.clone(), &model)
                                })
                            })
                            .map_err(after_admission)??;
                        let seen = request.await?;
                        if seen.option_id != options.option_id
                            || seen
                                .current
                                .as_ref()
                                .is_none_or(|current| current.id != model)
                        {
                            return Err(Error::Protocol);
                        }
                        observations.confirm_model(seen)?;
                    }
                }
            }
        }
    }
    let mut publish = || {
        observations.publish_model_prompt(&kind, reserved, || {
            let pending =
                session.enqueue_prompt(text, model_for_turn.as_deref(), &tuning_for_turn)?;
            if let Some(before) = &mut before {
                before.unsent = false;
            }
            Ok(pending)
        })
    };
    let pending = match dispatch {
        Some(dispatch) => dispatch.route.publish(publish),
        None => publish(),
    }
    .map_err(|error| {
        if reserved {
            after_admission(error)
        } else {
            error
        }
    })?;
    let native_message_id = pending.await?;
    // Settle the ordering fence on exact insertion, not on which turn the
    // harness used or whether it performed inference.
    if reserved && let Some(dispatch) = dispatch {
        dispatch.authority.retain(
            &start,
            NativeModelStartOutcome::Inserted(native_message_id.clone()),
        );
    }
    Ok(NativePromptAcceptance::Inserted { native_message_id })
}

fn after_admission(error: Error) -> Error {
    if matches!(error, Error::EndingDecisionPending) {
        // This operation already reserved a start. Settle as NotPublished;
        // do not requeue an operation that has a terminal reservation.
        Error::NotAdmitted
    } else {
        error
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeModelChoice {
    pub id: String,
    pub label: String,
    pub capabilities: Option<chapchoo_domain::ConversationModelCapabilities>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeModelOptions {
    pub option_id: String,
    pub current: Option<NativeModelChoice>,
    pub options: Vec<NativeModelChoice>,
    pub permission_modes: Vec<chapchoo_domain::ConversationPermissionMode>,
}

#[derive(Clone)]
pub struct NativeModelObservation {
    pub order: i64,
    pub options: Option<NativeModelOptions>,
    pub launch_operation_id: String,
    pub api_key_source: Option<[u8; 24]>,
}

impl fmt::Debug for NativeModelObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeModelObservation")
            .field("order", &self.order)
            .finish_non_exhaustive()
    }
}

/// This release sends the configured model for a plain prompt and reports
/// exact insertion. It does not distinguish new turns from native adoption.
pub(super) fn qualified(versions: &crate::NativeComponentVersions) -> bool {
    versions.adapter == "2.8.0" && versions.codex == "0.165.0"
}

pub(super) fn observe(live: &Live) -> Result<NativeModelObservation, Error> {
    let mut model = live
        .observations
        .model(&live._authentication.operation_id, live.api_key_source)?;
    if let Some(Session::AppServer(thread)) = &live.session {
        model.options = thread.models();
    } else if !qualified(&live._supply.versions) {
        // An installed successor can prove replacement even when its model
        // commands and options are not qualified.
        model.options = None;
    }
    Ok(model)
}

pub(super) fn options(config: &[schema::SessionConfigOption]) -> Option<NativeModelOptions> {
    let mut models = config
        .iter()
        .filter(|option| option.category == Some(schema::SessionConfigOptionCategory::Model));
    let option = models.next()?;
    if models.next().is_some() {
        return None;
    }
    let schema::SessionConfigKind::Select(select) = &option.kind else {
        return None;
    };
    let choices = match &select.options {
        schema::SessionConfigSelectOptions::Ungrouped(items) => items.iter().collect::<Vec<_>>(),
        schema::SessionConfigSelectOptions::Grouped(groups) => {
            groups.iter().flat_map(|group| &group.options).collect()
        }
        _ => return None,
    };
    let mut ids = HashSet::new();
    if choices.len() > 128
        || choices.iter().any(|choice| {
            choice.value.0.is_empty()
                || choice.value.0.len() > 256
                || !choice.value.0.bytes().all(|value| value.is_ascii_graphic())
                || choice.name.trim().is_empty()
                || choice.name.len() > 256
                || choice.name.chars().any(char::is_control)
                || !ids.insert(choice.value.0.as_ref())
        })
    {
        return None;
    }
    let options = choices
        .iter()
        .map(|choice| NativeModelChoice {
            id: choice.value.0.to_string(),
            label: choice.name.clone(),
            capabilities: None,
        })
        .collect::<Vec<_>>();
    let current = options
        .iter()
        .find(|choice| choice.id == select.current_value.0.as_ref())
        .cloned();
    Some(NativeModelOptions {
        option_id: option.config_id.0.to_string(),
        current,
        options,
        permission_modes: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_model_config_and_insertion_pair_is_exact() {
        let mut versions = crate::NativeComponentVersions {
            node: "24.0.0".into(),
            adapter: "2.8.0".into(),
            codex: "0.165.0".into(),
        };
        assert!(qualified(&versions));
        versions.codex = "0.165.1".into();
        assert!(!qualified(&versions));
    }

    #[test]
    fn only_exact_builtin_command_tokens_bypass_model_preselection() {
        for input in [
            "/review",
            " \t/REVIEW --base main",
            "/goal ship",
            "/compact ",
            "\u{feff}/compact",
            "/\u{feff}compact",
            "/compact\u{feff}argument",
        ] {
            assert!(native_command(input), "{input:?}");
        }
        for input in [
            "/workspace/review",
            "/review.log",
            "/review-new",
            "describe /goal",
            "\u{85}/compact",
            "/\u{85}compact",
            "/compact\u{85}argument",
            "/",
        ] {
            assert!(!native_command(input), "{input:?}");
        }
    }
}
