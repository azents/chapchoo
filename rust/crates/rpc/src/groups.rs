//! Runtime-selected public service union. PeerRelay has a separate factory.
use crate::{
    AccountServices, ComponentAuthenticationServices, ComponentConnectionServices, ComputerServices,
};
use tonic::{Status, service::Routes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceGroup {
    Client,
    Admin,
    Control,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceGroups {
    client: bool,
    admin: bool,
    control: bool,
}
impl ServiceGroups {
    pub fn new(groups: impl IntoIterator<Item = ServiceGroup>) -> Self {
        let mut selected = Self::default();
        for group in groups {
            match group {
                ServiceGroup::Client => selected.client = true,
                ServiceGroup::Admin => selected.admin = true,
                ServiceGroup::Control => selected.control = true,
            }
        }
        selected
    }
    pub fn contains(self, group: ServiceGroup) -> bool {
        match group {
            ServiceGroup::Client => self.client,
            ServiceGroup::Admin => self.admin,
            ServiceGroup::Control => self.control,
        }
    }
    pub fn is_empty(self) -> bool {
        !self.client && !self.admin && !self.control
    }
    pub fn service_names(self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.client || self.admin {
            names.extend([
                "chapchoo.accounts.Authentication",
                "chapchoo.accounts.PersonalPreferences",
            ]);
        }
        if self.client {
            names.extend([
                "chapchoo.accounts.AccountLifecycle",
                "chapchoo.accounts.TeamManagement",
                "chapchoo.accounts.InvitationManagement",
                "chapchoo.computers.Agents",
                "chapchoo.computers.Computers",
                "chapchoo.computers.ProviderCatalog",
            ]);
        }
        if self.admin {
            names.extend([
                "chapchoo.accounts.Installation",
                "chapchoo.accounts.AccountGrants",
                "chapchoo.accounts.DisplayDefaults",
                "chapchoo.accounts.AdminWorkspaces",
                "chapchoo.computers.ProviderInventory",
            ]);
        }
        if self.control {
            names.extend([
                "chapchoo.control.ProviderAuthentication",
                "chapchoo.control.ConnectorAuthentication",
                "chapchoo.control.ProviderConnection",
                "chapchoo.control.ConnectorConnection",
            ]);
        }
        names
    }
    /// One registration for shared Authentication/PersonalPreferences regardless
    /// of duplicate group selection. Components expose no internal peer service.
    pub fn routes(
        self,
        accounts: Option<&AccountServices>,
        computers: Option<&ComputerServices>,
        components: Option<(
            &ComponentAuthenticationServices,
            &ComponentConnectionServices,
        )>,
    ) -> Result<Routes, Status> {
        if self.is_empty() {
            return Err(Status::invalid_argument(
                "At least one service group is required",
            ));
        }
        let mut routes = Routes::builder();
        if self.client || self.admin {
            let accounts = accounts
                .ok_or_else(|| Status::failed_precondition("Account services are required"))?;
            let computers = computers
                .ok_or_else(|| Status::failed_precondition("Computer services are required"))?;
            routes
                .add_service(accounts.authentication_server())
                .add_service(accounts.personal_preferences_server());
            if self.client {
                routes
                    .add_service(accounts.account_lifecycle_server())
                    .add_service(accounts.teams_server())
                    .add_service(accounts.invitations_server());
                routes
                    .add_service(computers.agents_server())
                    .add_service(computers.computers_server())
                    .add_service(computers.catalog_server());
            }
            if self.admin {
                routes
                    .add_service(accounts.installation_server())
                    .add_service(accounts.account_grants_server())
                    .add_service(accounts.display_defaults_server());
                routes.add_service(accounts.admin_workspaces_server());
                routes.add_service(computers.inventory_server());
            }
        }
        if self.control {
            let (authentication, connections) = components
                .ok_or_else(|| Status::failed_precondition("Control services are required"))?;
            routes
                .add_service(authentication.provider_server())
                .add_service(authentication.connector_server())
                .add_service(connections.provider_server())
                .add_service(connections.connector_server());
        }
        Ok(routes.routes())
    }
}
