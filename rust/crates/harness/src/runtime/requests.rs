use super::*;
mod forms;
pub(super) fn supported_form(request: &schema::CreateElicitationRequest) -> bool {
    let schema::ElicitationMode::Form(form) = &request.mode else {
        return false;
    };
    if !matches!(form.scope, schema::ElicitationScope::Session(_))
        || form.requested_schema.properties.len() > 64
    {
        return false;
    }
    form.requested_schema.required.iter().flatten().all(|name| {
        form.requested_schema
            .properties
            .get(name)
            .is_some_and(forms::supported)
    })
}
pub(super) fn supported_permission(request: &schema::RequestPermissionRequest) -> bool {
    let mut options = std::collections::BTreeSet::new();
    !request.options.is_empty()
        && request
            .options
            .iter()
            .all(|option| options.insert(option.option_id.to_string()))
}

pub(super) async fn respond(
    requests: &Requests,
    session: &schema::SessionId,
    response: InteractiveResponse,
) -> Result<(), Error> {
    let id = match &response {
        InteractiveResponse::Permission { id, .. } | InteractiveResponse::Form { id, .. } => id,
    };
    let mut pending = requests.lock().await;
    let (_, request) = pending.get(id).ok_or(Error::NotAdmitted)?;
    if request.cancelled() {
        return Err(Error::NotAdmitted);
    }
    match (request, &response) {
        (Pending::Permission(request, _), InteractiveResponse::Permission { response, .. }) => {
            if &request.session_id != session {
                return Err(Error::NotAdmitted);
            }
            match &response.outcome {
                schema::RequestPermissionOutcome::Selected(selected)
                    if request
                        .options
                        .iter()
                        .any(|option| option.option_id == selected.option_id) => {}
                schema::RequestPermissionOutcome::Cancelled(_) => {}
                _ => return Err(Error::NotAdmitted),
            }
        }
        (Pending::Form(request, _), InteractiveResponse::Form { response, .. }) => {
            let schema::ElicitationMode::Form(form) = &request.mode else {
                return Err(Error::NotAdmitted);
            };
            let schema::ElicitationScope::Session(scope) = &form.scope else {
                return Err(Error::NotAdmitted);
            };
            if &scope.session_id != session || !forms::response(form, response) {
                return Err(Error::NotAdmitted);
            }
        }
        _ => return Err(Error::NotAdmitted),
    }
    match (pending.remove(id), response) {
        (
            Some((_, Pending::Permission(_, r))),
            InteractiveResponse::Permission { response, .. },
        ) => r.respond(response).map_err(|_| Error::Protocol),
        (Some((_, Pending::Form(_, r))), InteractiveResponse::Form { response, .. }) => {
            r.respond(response).map_err(|_| Error::Protocol)
        }
        _ => Err(Error::NotAdmitted),
    }
}
