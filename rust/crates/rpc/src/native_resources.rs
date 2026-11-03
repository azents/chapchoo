//! Normalize existing Kubernetes fields and explicit Docker payloads at wire ingress.
use chapchoo_domain::{
    DockerContainerId, DockerDaemonId, DockerVolumeIdentity, KubernetesUid, ManagedResourceKind,
    NativeIdentity, NativeReservation, NativeScope, ProviderKind,
};
use chapchoo_protocol::{computers as inventory, control as wire};
use chapchoo_service::{ManagedResource, ResourceEvidence};
use tonic::Status;

fn invalid() -> Status {
    Status::invalid_argument("Native resource payload is invalid")
}

pub(crate) fn kind(value: i32) -> Result<ManagedResourceKind, Status> {
    match wire::ResourceKind::try_from(value) {
        Ok(wire::ResourceKind::Pod) => Ok(ManagedResourceKind::Pod),
        Ok(wire::ResourceKind::Pvc) => Ok(ManagedResourceKind::Pvc),
        Ok(wire::ResourceKind::BootstrapSecret) => Ok(ManagedResourceKind::BootstrapSecret),
        Ok(wire::ResourceKind::DockerContainer) => Ok(ManagedResourceKind::DockerContainer),
        Ok(wire::ResourceKind::DockerVolume) => Ok(ManagedResourceKind::DockerVolume),
        _ => Err(invalid()),
    }
}

fn wire_kind(value: ManagedResourceKind) -> i32 {
    let kind = match value {
        ManagedResourceKind::Pod => wire::ResourceKind::Pod,
        ManagedResourceKind::Pvc => wire::ResourceKind::Pvc,
        ManagedResourceKind::BootstrapSecret => wire::ResourceKind::BootstrapSecret,
        ManagedResourceKind::DockerContainer => wire::ResourceKind::DockerContainer,
        ManagedResourceKind::DockerVolume => wire::ResourceKind::DockerVolume,
    };
    kind as i32
}

pub(crate) fn volume_from_wire(
    value: &wire::DockerVolumeIdentity,
) -> Result<DockerVolumeIdentity, Status> {
    DockerVolumeIdentity::local(
        &value.driver,
        &value.scope,
        &value
            .options
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        value.created_at.clone(),
        value
            .labels
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
    .map_err(|_| invalid())
}

fn volume_to_wire(value: &DockerVolumeIdentity) -> wire::DockerVolumeIdentity {
    wire::DockerVolumeIdentity {
        driver: "local".into(),
        scope: "local".into(),
        options: Default::default(),
        labels: value
            .labels()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        created_at: value.created_at().map(str::to_owned),
    }
}

pub(crate) struct DecodedResource {
    pub native: NativeReservation,
    pub identity: Option<NativeIdentity>,
    pub execution_id: Option<String>,
    pub absence_confirmed: bool,
}

pub(crate) fn decode(value: &wire::ManagedResource) -> Result<DecodedResource, Status> {
    let kind = kind(value.kind)?;
    let (scope, identity) = match (kind, &value.native) {
        (
            ManagedResourceKind::Pod
            | ManagedResourceKind::Pvc
            | ManagedResourceKind::BootstrapSecret,
            None,
        ) => {
            let scope = NativeScope::kubernetes(value.namespace.clone()).map_err(|_| invalid())?;
            let identity = value
                .uid
                .as_ref()
                .map(|uid| {
                    KubernetesUid::parse(uid.clone())
                        .map(NativeIdentity::KubernetesObject)
                        .map_err(|_| invalid())
                })
                .transpose()?;
            (scope, identity)
        }
        (
            ManagedResourceKind::DockerContainer,
            Some(wire::managed_resource::Native::DockerContainer(native)),
        ) => {
            if !value.namespace.is_empty() || value.uid.is_some() {
                return Err(invalid());
            }
            let scope = NativeScope::DockerDaemon(
                DockerDaemonId::parse(native.daemon_id.clone()).map_err(|_| invalid())?,
            );
            let identity = native
                .container_id
                .as_ref()
                .map(|id| {
                    DockerContainerId::parse(id.clone())
                        .map(NativeIdentity::DockerContainer)
                        .map_err(|_| invalid())
                })
                .transpose()?;
            (scope, identity)
        }
        (
            ManagedResourceKind::DockerVolume,
            Some(wire::managed_resource::Native::DockerVolume(native)),
        ) => {
            if !value.namespace.is_empty() || value.uid.is_some() {
                return Err(invalid());
            }
            let scope = NativeScope::DockerDaemon(
                DockerDaemonId::parse(native.daemon_id.clone()).map_err(|_| invalid())?,
            );
            let identity = native
                .identity
                .as_ref()
                .map(|value| volume_from_wire(value).map(NativeIdentity::DockerVolume))
                .transpose()?;
            (scope, identity)
        }
        _ => return Err(invalid()),
    };
    Ok(DecodedResource {
        native: NativeReservation::new(kind, scope, value.name.clone()).map_err(|_| invalid())?,
        identity,
        execution_id: value.execution_id.clone(),
        absence_confirmed: value.absence_confirmed,
    })
}

/// Input is a sealed store read model whose constructor verified native identity.
pub(crate) fn encode(value: &ManagedResource) -> wire::ManagedResource {
    let scope = value.native().scope();
    let native = match value.kind() {
        ManagedResourceKind::Pod
        | ManagedResourceKind::Pvc
        | ManagedResourceKind::BootstrapSecret => None,
        ManagedResourceKind::DockerContainer => Some(
            wire::managed_resource::Native::DockerContainer(wire::DockerContainerNative {
                daemon_id: scope.as_str().to_owned(),
                container_id: match value.identity() {
                    Some(NativeIdentity::DockerContainer(id)) => Some(id.as_str().to_owned()),
                    _ => None,
                },
            }),
        ),
        ManagedResourceKind::DockerVolume => Some(wire::managed_resource::Native::DockerVolume(
            wire::DockerVolumeNative {
                daemon_id: scope.as_str().to_owned(),
                identity: match value.identity() {
                    Some(NativeIdentity::DockerVolume(identity)) => Some(volume_to_wire(identity)),
                    _ => None,
                },
            },
        )),
    };
    wire::ManagedResource {
        kind: wire_kind(value.kind()),
        namespace: if value.kind().provider_kind() == ProviderKind::Kubernetes {
            scope.as_str().to_owned()
        } else {
            String::new()
        },
        name: value.native().name().to_owned(),
        uid: value.kubernetes_uid().map(str::to_owned),
        execution_id: value.execution_id.clone(),
        absence_confirmed: value.absence_confirmed_at.is_some(),
        native,
    }
}

pub(crate) enum DecodedEvidence {
    Present(NativeIdentity),
    Absent(Option<NativeIdentity>),
    Unknown,
}

impl DecodedEvidence {
    pub fn store(&self) -> ResourceEvidence<'_> {
        match self {
            Self::Present(identity) => ResourceEvidence::OwnedPresent { identity },
            Self::Absent(identity) => ResourceEvidence::DefinitivelyAbsent {
                expected_identity: identity.as_ref(),
            },
            Self::Unknown => ResourceEvidence::Unknown,
        }
    }
}

pub(crate) fn evidence(
    value: &wire::resource_evidence::Evidence,
    kind: ManagedResourceKind,
) -> Result<DecodedEvidence, Status> {
    let result = match value {
        wire::resource_evidence::Evidence::Unknown(_) => DecodedEvidence::Unknown,
        wire::resource_evidence::Evidence::Present(value) => {
            let identity = match (kind.provider_kind(), &value.native_identity) {
                (ProviderKind::Kubernetes, None) => NativeIdentity::KubernetesObject(
                    KubernetesUid::parse(value.uid.clone()).map_err(|_| invalid())?,
                ),
                (
                    ProviderKind::Docker,
                    Some(wire::owned_present::NativeIdentity::DockerContainerId(id)),
                ) if value.uid.is_empty() => NativeIdentity::DockerContainer(
                    DockerContainerId::parse(id.clone()).map_err(|_| invalid())?,
                ),
                (
                    ProviderKind::Docker,
                    Some(wire::owned_present::NativeIdentity::DockerVolume(volume)),
                ) if value.uid.is_empty() => {
                    NativeIdentity::DockerVolume(volume_from_wire(volume)?)
                }
                _ => return Err(invalid()),
            };
            if !identity.matches_kind(kind) {
                return Err(invalid());
            }
            DecodedEvidence::Present(identity)
        }
        wire::resource_evidence::Evidence::Absent(value) => {
            let identity = match (kind.provider_kind(), &value.expected_native_identity) {
                (ProviderKind::Kubernetes, None) => value
                    .expected_uid
                    .as_ref()
                    .map(|uid| {
                        KubernetesUid::parse(uid.clone())
                            .map(NativeIdentity::KubernetesObject)
                            .map_err(|_| invalid())
                    })
                    .transpose()?,
                (ProviderKind::Docker, None) if value.expected_uid.is_none() => None,
                (
                    ProviderKind::Docker,
                    Some(wire::definitively_absent::ExpectedNativeIdentity::DockerContainerId(id)),
                ) if value.expected_uid.is_none() => Some(NativeIdentity::DockerContainer(
                    DockerContainerId::parse(id.clone()).map_err(|_| invalid())?,
                )),
                (
                    ProviderKind::Docker,
                    Some(wire::definitively_absent::ExpectedNativeIdentity::DockerVolume(volume)),
                ) if value.expected_uid.is_none() => {
                    Some(NativeIdentity::DockerVolume(volume_from_wire(volume)?))
                }
                _ => return Err(invalid()),
            };
            if identity.as_ref().is_some_and(|id| !id.matches_kind(kind)) {
                return Err(invalid());
            }
            DecodedEvidence::Absent(identity)
        }
    };
    Ok(result)
}

pub(crate) fn hello_scope(value: &wire::ProviderHello) -> Result<NativeScope, Status> {
    match (
        wire::ProviderKind::try_from(value.kind),
        &value.docker_daemon_id,
    ) {
        (Ok(wire::ProviderKind::Kubernetes), None) => {
            if !value
                .namespace
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err(invalid());
            }
            NativeScope::kubernetes(value.namespace.clone()).map_err(|_| invalid())
        }
        (Ok(wire::ProviderKind::Docker), Some(daemon)) if value.namespace.is_empty() => {
            Ok(NativeScope::DockerDaemon(
                DockerDaemonId::parse(daemon.clone()).map_err(|_| invalid())?,
            ))
        }
        _ => Err(invalid()),
    }
}

pub(crate) fn cursor(value: &NativeReservation) -> inventory::ResourceCursor {
    let kubernetes = value.kind().provider_kind() == ProviderKind::Kubernetes;
    inventory::ResourceCursor {
        kind: value.kind().as_str().into(),
        name: value.name().into(),
        namespace: if kubernetes {
            value.scope().as_str().into()
        } else {
            String::new()
        },
        native_scope_kind: if kubernetes {
            String::new()
        } else {
            value.scope().kind_str().into()
        },
        native_scope: if kubernetes {
            String::new()
        } else {
            value.scope().as_str().into()
        },
    }
}

pub(crate) fn decode_cursor(value: inventory::ResourceCursor) -> Result<NativeReservation, Status> {
    let kind = value
        .kind
        .parse::<ManagedResourceKind>()
        .map_err(|_| invalid())?;
    let scope = match kind.provider_kind() {
        ProviderKind::Kubernetes
            if value.native_scope.is_empty() && value.native_scope_kind.is_empty() =>
        {
            NativeScope::kubernetes(value.namespace).map_err(|_| invalid())?
        }
        ProviderKind::Docker
            if value.namespace.is_empty() && value.native_scope_kind == "DOCKER_DAEMON" =>
        {
            NativeScope::parse(&value.native_scope_kind, value.native_scope)
                .map_err(|_| invalid())?
        }
        _ => return Err(invalid()),
    };
    NativeReservation::new(kind, scope, value.name).map_err(|_| invalid())
}

#[cfg(test)]
mod tests;
