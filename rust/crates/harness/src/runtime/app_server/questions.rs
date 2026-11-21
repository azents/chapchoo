//! Translate Codex's tool question IDs and choices into the existing form lane.
use crate::runtime::{Error, schema};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const OTHER: &str = "None of the above";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    thread_id: String,
    turn_id: String,
    item_id: String,
    questions: Vec<Question>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Question {
    id: String,
    header: String,
    question: String,
    options: Option<Vec<Label>>,
    #[serde(default)]
    is_other: bool,
    #[serde(default)]
    is_secret: bool,
}

#[derive(Deserialize)]
struct Label {
    label: String,
    description: String,
}

pub(super) struct Questions(Vec<AnswerBinding>);

struct AnswerBinding {
    id: String,
    note: Option<String>,
}

impl Questions {
    pub(super) fn parse(
        root: &str,
        turn: &str,
        params: Value,
    ) -> Option<(Self, schema::ElicitationSchema)> {
        let input: Request = serde_json::from_value(params).ok()?;
        if input.thread_id != root
            || input.turn_id != turn
            || !super::valid_key(turn)
            || !super::valid_key(&input.item_id)
            || input.questions.is_empty()
            || input.questions.len() > 16
        {
            return None;
        }
        let mut keys: BTreeSet<_> = input.questions.iter().map(|q| q.id.clone()).collect();
        if keys.len() != input.questions.len() {
            return None;
        }
        let mut properties = serde_json::Map::new();
        let mut bindings = Vec::new();
        for (index, q) in input.questions.into_iter().enumerate() {
            use super::tools::visible;
            // The existing form lane displays and stores ordinary answers.
            if q.is_secret
                || !super::valid_key(&q.id)
                || !visible(&q.question)
                || (!q.header.is_empty() && !visible(&q.header))
            {
                return None;
            }
            let mut field =
                json!({"type":"string","title":q.question,"description":q.header,"maxLength":8192});
            let mut note = None;
            if let Some(options) = q.options.filter(|options| !options.is_empty()) {
                let mut labels = BTreeSet::new();
                let mut choices = Vec::new();
                if options.len() > 32 {
                    return None;
                }
                for option in options {
                    if !visible(&option.label)
                        || (!option.description.is_empty() && !visible(&option.description))
                        || !labels.insert(option.label.clone())
                    {
                        return None;
                    }
                    choices.push(json!({"const":option.label,"title":option.label,"description":option.description}));
                }
                if q.is_other {
                    if !labels.contains(OTHER) {
                        choices.push(json!({"const":OTHER,"title":OTHER}));
                    }
                    let mut name = format!("chapchoo-note-{index}");
                    while !keys.insert(name.clone()) {
                        name.push('-');
                    }
                    properties.insert(name.clone(), json!({"type":"string","title":"Additional answer or note","maxLength":8192}));
                    note = Some(name);
                }
                field["oneOf"] = json!(choices);
            }
            properties.insert(q.id.clone(), field);
            bindings.push(AnswerBinding { id: q.id, note });
        }
        let required: Vec<&str> = bindings.iter().map(|q| q.id.as_str()).collect();
        let fields = serde_json::from_value(json!({
            "type":"object","properties":properties,"required":required
        }))
        .ok()?;
        Some((Self(bindings), fields))
    }

    pub(super) fn answer(&self, action: schema::ElicitationAction) -> Result<Value, Error> {
        let mut answers = BTreeMap::new();
        match action {
            schema::ElicitationAction::Accept(answer) => {
                let content = answer.content.ok_or(Error::NotAdmitted)?;
                for question in &self.0 {
                    let string = |id: &str| match content.get(id) {
                        Some(schema::ElicitationContentValue::String(value)) => Some(value),
                        _ => None,
                    };
                    let first = string(&question.id).ok_or(Error::NotAdmitted)?;
                    let mut values = vec![first.clone()];
                    if let Some(note) = question
                        .note
                        .as_deref()
                        .and_then(string)
                        .map(|note| note.trim())
                        .filter(|note| !note.is_empty())
                    {
                        values.push(format!("user_note: {note}"));
                    }
                    answers.insert(&question.id, json!({"answers":values}));
                }
            }
            schema::ElicitationAction::Decline | schema::ElicitationAction::Cancel => {}
            _ => return Err(Error::NotAdmitted),
        }
        Ok(json!({"answers":answers}))
    }
}
