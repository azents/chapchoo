//! Native resource values. These types describe identity, never mutation authority.

use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid native Computer resource")]
pub struct InvalidNativeResource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Kubernetes,
    Docker,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kubernetes => "KUBERNETES",
            Self::Docker => "DOCKER",
        }
    }

    pub fn saved_resource_kind(self) -> ManagedResourceKind {
        match self {
            Self::Kubernetes => ManagedResourceKind::Pvc,
            Self::Docker => ManagedResourceKind::DockerVolume,
        }
    }

    pub fn execution_resource_kind(self) -> ManagedResourceKind {
        match self {
            Self::Kubernetes => ManagedResourceKind::Pod,
            Self::Docker => ManagedResourceKind::DockerContainer,
        }
    }

    pub fn required_resource_kinds(self) -> &'static [ManagedResourceKind] {
        match self {
            Self::Kubernetes => &[
                ManagedResourceKind::Pod,
                ManagedResourceKind::BootstrapSecret,
                ManagedResourceKind::Pvc,
            ],
            Self::Docker => &[
                ManagedResourceKind::DockerContainer,
                ManagedResourceKind::DockerVolume,
            ],
        }
    }
}

impl FromStr for ProviderKind {
    type Err = InvalidNativeResource;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "KUBERNETES" => Ok(Self::Kubernetes),
            "DOCKER" => Ok(Self::Docker),
            _ => Err(InvalidNativeResource),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceRole {
    Execution,
    SavedEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedResourceKind {
    Pod,
    Pvc,
    BootstrapSecret,
    DockerContainer,
    DockerVolume,
}

impl ManagedResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pod => "POD",
            Self::Pvc => "PVC",
            Self::BootstrapSecret => "BOOTSTRAP_SECRET",
            Self::DockerContainer => "DOCKER_CONTAINER",
            Self::DockerVolume => "DOCKER_VOLUME",
        }
    }

    pub fn provider_kind(self) -> ProviderKind {
        match self {
            Self::Pod | Self::Pvc | Self::BootstrapSecret => ProviderKind::Kubernetes,
            Self::DockerContainer | Self::DockerVolume => ProviderKind::Docker,
        }
    }

    pub fn role(self) -> ResourceRole {
        match self {
            Self::Pvc | Self::DockerVolume => ResourceRole::SavedEnvironment,
            Self::Pod | Self::BootstrapSecret | Self::DockerContainer => ResourceRole::Execution,
        }
    }
}

impl FromStr for ManagedResourceKind {
    type Err = InvalidNativeResource;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "POD" => Ok(Self::Pod),
            "PVC" => Ok(Self::Pvc),
            "BOOTSTRAP_SECRET" => Ok(Self::BootstrapSecret),
            "DOCKER_CONTAINER" => Ok(Self::DockerContainer),
            "DOCKER_VOLUME" => Ok(Self::DockerVolume),
            _ => Err(InvalidNativeResource),
        }
    }
}

fn bounded_token(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value.chars().all(|c| !c.is_control() && !c.is_whitespace())
}

macro_rules! native_value {
    ($name:ident, $validate:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: String) -> Result<Self, InvalidNativeResource> {
                if ($validate)(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidNativeResource)
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

// Daemon IDs are opaque comparison evidence, not Kubernetes namespace strings.
native_value!(DockerDaemonId, |v: &str| bounded_token(v, 1024));
native_value!(KubernetesUid, |v: &str| bounded_token(v, 1024));
native_value!(DockerContainerId, |v: &str| v.len() == 64
    && v.bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));

/// A Docker Provider is usable only after the server accepts its daemon binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSubstrate {
    Kubernetes,
    Docker(DockerDaemonId),
}

impl ProviderSubstrate {
    pub fn kind(&self) -> ProviderKind {
        match self {
            Self::Kubernetes => ProviderKind::Kubernetes,
            Self::Docker(_) => ProviderKind::Docker,
        }
    }

    pub fn docker_daemon_id(&self) -> Option<&str> {
        match self {
            Self::Kubernetes => None,
            Self::Docker(daemon) => Some(daemon.as_str()),
        }
    }

    pub fn accepts_scope(&self, scope: &NativeScope) -> bool {
        match (self, scope) {
            (Self::Kubernetes, NativeScope::KubernetesNamespace(namespace)) => {
                kubernetes_name(namespace, 63)
            }
            (Self::Docker(bound), NativeScope::DockerDaemon(observed)) => bound == observed,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeScope {
    KubernetesNamespace(String),
    DockerDaemon(DockerDaemonId),
}

impl NativeScope {
    pub fn parse(kind: &str, value: String) -> Result<Self, InvalidNativeResource> {
        match kind {
            "KUBERNETES_NAMESPACE" => Self::kubernetes(value),
            "DOCKER_DAEMON" => Ok(Self::DockerDaemon(DockerDaemonId::parse(value)?)),
            _ => Err(InvalidNativeResource),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::KubernetesNamespace(_) => "KUBERNETES_NAMESPACE",
            Self::DockerDaemon(_) => "DOCKER_DAEMON",
        }
    }

    pub fn kubernetes(namespace: String) -> Result<Self, InvalidNativeResource> {
        if kubernetes_name(&namespace, 63) {
            Ok(Self::KubernetesNamespace(namespace))
        } else {
            Err(InvalidNativeResource)
        }
    }

    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            Self::KubernetesNamespace(_) => ProviderKind::Kubernetes,
            Self::DockerDaemon(_) => ProviderKind::Docker,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::KubernetesNamespace(namespace) => namespace,
            Self::DockerDaemon(daemon) => daemon.as_str(),
        }
    }
}

// Preserve the existing reservation grammar; adapters also enforce native rules.
fn kubernetes_name(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.'))
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn docker_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// A reserved native address. It does not imply allocation or an accepted effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeReservation {
    kind: ManagedResourceKind,
    scope: NativeScope,
    name: String,
}

impl NativeReservation {
    pub fn new(
        kind: ManagedResourceKind,
        scope: NativeScope,
        name: String,
    ) -> Result<Self, InvalidNativeResource> {
        let valid_scope = match &scope {
            NativeScope::KubernetesNamespace(namespace) => kubernetes_name(namespace, 63),
            NativeScope::DockerDaemon(_) => true,
        };
        let valid_name = match kind.provider_kind() {
            ProviderKind::Kubernetes => kubernetes_name(&name, 253),
            ProviderKind::Docker => docker_name(&name),
        };
        if kind.provider_kind() != scope.provider_kind() || !valid_scope || !valid_name {
            return Err(InvalidNativeResource);
        }
        Ok(Self { kind, scope, name })
    }

    pub fn kind(&self) -> ManagedResourceKind {
        self.kind
    }

    pub fn scope(&self) -> &NativeScope {
        &self.scope
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Original native observations, not a synthetic volume UID or delete precondition.
/// Owner labels are compared by the native adapter against the reservation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerVolumeIdentity {
    created_at: Option<String>,
    labels: BTreeMap<String, String>,
}

impl DockerVolumeIdentity {
    /// Only an ordinary local volume with no driver options is in this contract.
    pub fn local(
        driver: &str,
        scope: &str,
        options: &BTreeMap<String, String>,
        created_at: Option<String>,
        labels: BTreeMap<String, String>,
    ) -> Result<Self, InvalidNativeResource> {
        if driver != "local"
            || scope != "local"
            || !options.is_empty()
            || created_at.as_ref().is_some_and(|v| !bounded_token(v, 128))
            || labels.len() > 64
            || labels.iter().any(|(key, value)| {
                !bounded_token(key, 255)
                    || value.len() > 4096
                    || value.chars().any(char::is_control)
            })
        {
            return Err(InvalidNativeResource);
        }
        Ok(Self { created_at, labels })
    }

    pub fn created_at(&self) -> Option<&str> {
        self.created_at.as_deref()
    }

    pub fn labels(&self) -> &BTreeMap<String, String> {
        &self.labels
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeIdentity {
    KubernetesObject(KubernetesUid),
    DockerContainer(DockerContainerId),
    DockerVolume(DockerVolumeIdentity),
}

impl NativeIdentity {
    pub fn matches_kind(&self, kind: ManagedResourceKind) -> bool {
        match self {
            Self::KubernetesObject(_) => kind.provider_kind() == ProviderKind::Kubernetes,
            Self::DockerContainer(_) => kind == ManagedResourceKind::DockerContainer,
            Self::DockerVolume(_) => kind == ManagedResourceKind::DockerVolume,
        }
    }
}

/// Immutable original allocation bound to its reserved address.
/// Source authorization and sticky presence/absence remain store responsibilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAllocation {
    reservation: NativeReservation,
    identity: NativeIdentity,
}

impl NativeAllocation {
    pub fn new(
        reservation: NativeReservation,
        identity: NativeIdentity,
    ) -> Result<Self, InvalidNativeResource> {
        if !identity.matches_kind(reservation.kind()) {
            return Err(InvalidNativeResource);
        }
        Ok(Self {
            reservation,
            identity,
        })
    }

    pub fn reservation(&self) -> &NativeReservation {
        &self.reservation
    }

    pub fn identity(&self) -> &NativeIdentity {
        &self.identity
    }
}

#[cfg(test)]
mod tests;
