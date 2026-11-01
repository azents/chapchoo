use chapchoo_protocol::accounts as wire;
use chapchoo_service::AccountsService;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::Status;

mod authentication;
use authentication::AuthenticationAdapter;
mod personal_preferences;
use personal_preferences::PersonalPreferencesAdapter;
mod account_lifecycle;
use account_lifecycle::AccountLifecycleAdapter;
mod teams;
use teams::TeamsAdapter;
mod invitations;
use invitations::InvitationsAdapter;
mod installation;
use installation::InstallationAdapter;
mod account_grants;
use account_grants::AccountGrantsAdapter;
mod display_defaults;
use display_defaults::DisplayDefaultsAdapter;

#[derive(Debug, Clone)]
pub(super) struct HandlerContext {
    service: Arc<AccountsService>,
    admission: Arc<AtomicBool>,
}
impl HandlerContext {
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Service is not admitting new requests"))
        }
    }
}

/// Composition factory; each domain has a separate generated service adapter.
#[derive(Debug, Clone)]
pub struct AccountServices {
    context: HandlerContext,
}
impl AccountServices {
    pub fn new(service: Arc<AccountsService>) -> Self {
        Self {
            context: HandlerContext {
                service,
                admission: Arc::new(AtomicBool::new(true)),
            },
        }
    }
    pub fn with_admission(mut self, admission: Arc<AtomicBool>) -> Self {
        self.context.admission = admission;
        self
    }
    pub fn authentication_server(
        &self,
    ) -> wire::authentication_server::AuthenticationServer<AuthenticationAdapter> {
        wire::authentication_server::AuthenticationServer::new(AuthenticationAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn personal_preferences_server(
        &self,
    ) -> wire::personal_preferences_server::PersonalPreferencesServer<PersonalPreferencesAdapter>
    {
        wire::personal_preferences_server::PersonalPreferencesServer::new(
            PersonalPreferencesAdapter {
                context: self.context.clone(),
            },
        )
        .max_decoding_message_size(64 * 1024)
    }
    pub fn account_lifecycle_server(
        &self,
    ) -> wire::account_lifecycle_server::AccountLifecycleServer<AccountLifecycleAdapter> {
        wire::account_lifecycle_server::AccountLifecycleServer::new(AccountLifecycleAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn teams_server(&self) -> wire::team_management_server::TeamManagementServer<TeamsAdapter> {
        wire::team_management_server::TeamManagementServer::new(TeamsAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn invitations_server(
        &self,
    ) -> wire::invitation_management_server::InvitationManagementServer<InvitationsAdapter> {
        wire::invitation_management_server::InvitationManagementServer::new(InvitationsAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn installation_server(
        &self,
    ) -> wire::installation_server::InstallationServer<InstallationAdapter> {
        wire::installation_server::InstallationServer::new(InstallationAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn account_grants_server(
        &self,
    ) -> wire::account_grants_server::AccountGrantsServer<AccountGrantsAdapter> {
        wire::account_grants_server::AccountGrantsServer::new(AccountGrantsAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
    pub fn display_defaults_server(
        &self,
    ) -> wire::display_defaults_server::DisplayDefaultsServer<DisplayDefaultsAdapter> {
        wire::display_defaults_server::DisplayDefaultsServer::new(DisplayDefaultsAdapter {
            context: self.context.clone(),
        })
        .max_decoding_message_size(64 * 1024)
    }
}
