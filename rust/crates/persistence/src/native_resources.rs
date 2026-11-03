//! Typed native row ingress. Serialized volume metadata is confined to this boundary.

use crate::StoreError;
use chapchoo_domain::{
    DockerContainerId, DockerVolumeIdentity, KubernetesUid, ManagedResourceKind, NativeIdentity,
    NativeReservation, NativeScope, ResourcePresence, ResourceRole,
};
use diesel::sql_types::{BigInt, Nullable, Text};
use std::{collections::BTreeMap, fmt};

#[derive(Clone, diesel::QueryableByName)]
pub(crate) struct ResourceRow {
    #[diesel(sql_type = Text)]
    resource_kind: String,
    #[diesel(sql_type = Text)]
    native_scope_kind: String,
    #[diesel(sql_type = Text)]
    native_scope: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    resource_uid: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    docker_container_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    docker_volume_metadata: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    execution_id: Option<String>,
    #[diesel(sql_type = Text)]
    presence: String,
    #[diesel(sql_type = Nullable<BigInt>)]
    observed_at: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    absence_confirmed_at: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    source_connection_id: Option<String>,
}

impl fmt::Debug for ResourceRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ResourceRow([REDACTED])")
    }
}

#[derive(Debug, Clone)]
pub struct ManagedResource {
    pub(crate) native: NativeReservation,
    pub(crate) identity: Option<NativeIdentity>,
    pub execution_id: Option<String>,
    pub presence: ResourcePresence,
    pub observed_at: Option<i64>,
    pub absence_confirmed_at: Option<i64>,
    pub source_connection_id: Option<String>,
}

impl ManagedResource {
    pub fn native(&self) -> &NativeReservation {
        &self.native
    }

    pub fn identity(&self) -> Option<&NativeIdentity> {
        self.identity.as_ref()
    }

    pub fn kind(&self) -> ManagedResourceKind {
        self.native.kind()
    }

    pub fn kubernetes_uid(&self) -> Option<&str> {
        match &self.identity {
            Some(NativeIdentity::KubernetesObject(uid)) => Some(uid.as_str()),
            _ => None,
        }
    }
}

impl TryFrom<ResourceRow> for ManagedResource {
    type Error = StoreError;

    fn try_from(row: ResourceRow) -> Result<Self, Self::Error> {
        let kind = row
            .resource_kind
            .parse::<ManagedResourceKind>()
            .map_err(|_| StoreError::Integrity)?;
        let scope = NativeScope::parse(&row.native_scope_kind, row.native_scope)
            .map_err(|_| StoreError::Integrity)?;
        let native =
            NativeReservation::new(kind, scope, row.name).map_err(|_| StoreError::Integrity)?;
        if (kind.role() == ResourceRole::SavedEnvironment) != row.execution_id.is_none() {
            return Err(StoreError::Integrity);
        }
        let identity = match kind {
            ManagedResourceKind::Pod
            | ManagedResourceKind::Pvc
            | ManagedResourceKind::BootstrapSecret => {
                if row.docker_container_id.is_some() || row.docker_volume_metadata.is_some() {
                    return Err(StoreError::Integrity);
                }
                row.resource_uid
                    .map(|uid| {
                        KubernetesUid::parse(uid)
                            .map(NativeIdentity::KubernetesObject)
                            .map_err(|_| StoreError::Integrity)
                    })
                    .transpose()?
            }
            ManagedResourceKind::DockerContainer => {
                if row.resource_uid.is_some() || row.docker_volume_metadata.is_some() {
                    return Err(StoreError::Integrity);
                }
                row.docker_container_id
                    .map(|id| {
                        DockerContainerId::parse(id)
                            .map(NativeIdentity::DockerContainer)
                            .map_err(|_| StoreError::Integrity)
                    })
                    .transpose()?
            }
            ManagedResourceKind::DockerVolume => {
                if row.resource_uid.is_some() || row.docker_container_id.is_some() {
                    return Err(StoreError::Integrity);
                }
                row.docker_volume_metadata
                    .map(|metadata| decode_volume(&metadata).map(NativeIdentity::DockerVolume))
                    .transpose()?
            }
        };
        let presence = match row.presence.as_str() {
            "PRESENT" => ResourcePresence::Present,
            "ABSENT" => ResourcePresence::Absent,
            "UNKNOWN" => ResourcePresence::Unknown,
            _ => return Err(StoreError::Integrity),
        };
        if kind.provider_kind() == chapchoo_domain::ProviderKind::Docker
            && presence == ResourcePresence::Present
            && identity.is_none()
        {
            return Err(StoreError::Integrity);
        }
        Ok(Self {
            native,
            identity,
            execution_id: row.execution_id,
            presence,
            observed_at: row.observed_at,
            absence_confirmed_at: row.absence_confirmed_at,
            source_connection_id: row.source_connection_id,
        })
    }
}

/// Absence requires exact native retirement/quiescence, never a bare 404.
#[derive(Debug, Clone, Copy)]
pub enum ResourceEvidence<'a> {
    OwnedPresent {
        identity: &'a NativeIdentity,
    },
    DefinitivelyAbsent {
        expected_identity: Option<&'a NativeIdentity>,
    },
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceReport<'a> {
    pub native: &'a NativeReservation,
    pub execution_id: Option<&'a str>,
    pub evidence: ResourceEvidence<'a>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct VolumeMetadata {
    driver: String,
    scope: String,
    options: BTreeMap<String, String>,
    created_at: Option<String>,
    labels: BTreeMap<String, String>,
}

fn decode_volume(value: &str) -> Result<DockerVolumeIdentity, StoreError> {
    if value.len() > 32768 {
        return Err(StoreError::Integrity);
    }
    let metadata: VolumeMetadata =
        serde_json::from_str(value).map_err(|_| StoreError::Integrity)?;
    DockerVolumeIdentity::local(
        &metadata.driver,
        &metadata.scope,
        &metadata.options,
        metadata.created_at,
        metadata.labels,
    )
    .map_err(|_| StoreError::Integrity)
}

pub(crate) struct IdentityColumns {
    pub uid: Option<String>,
    pub container_id: Option<String>,
    pub volume_metadata: Option<String>,
}

pub(crate) fn identity_columns(
    identity: Option<&NativeIdentity>,
) -> Result<IdentityColumns, StoreError> {
    let mut columns = IdentityColumns {
        uid: None,
        container_id: None,
        volume_metadata: None,
    };
    match identity {
        None => {}
        Some(NativeIdentity::KubernetesObject(uid)) => columns.uid = Some(uid.as_str().to_owned()),
        Some(NativeIdentity::DockerContainer(id)) => {
            columns.container_id = Some(id.as_str().to_owned())
        }
        Some(NativeIdentity::DockerVolume(volume)) => {
            let value = serde_json::to_string(&VolumeMetadata {
                driver: "local".into(),
                scope: "local".into(),
                options: BTreeMap::new(),
                created_at: volume.created_at().map(str::to_owned),
                labels: volume.labels().clone(),
            })
            .map_err(|_| StoreError::InvalidInput)?;
            if value.len() > 32768 {
                return Err(StoreError::InvalidInput);
            }
            columns.volume_metadata = Some(value);
        }
    }
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    type Error = Box<dyn std::error::Error + Send + Sync>;

    #[test]
    fn original_local_volume_observations_round_trip_and_reject_other_authority()
    -> Result<(), Error> {
        let original = DockerVolumeIdentity::local(
            "local",
            "local",
            &BTreeMap::new(),
            Some("2026-11-03T00:00:00Z".into()),
            BTreeMap::from([("computer".into(), "original".into())]),
        )?;
        let identity = NativeIdentity::DockerVolume(original.clone());
        let encoded = identity_columns(Some(&identity))?;
        assert!(encoded.uid.is_none());
        assert!(encoded.container_id.is_none());
        assert_eq!(
            decode_volume(
                encoded
                    .volume_metadata
                    .as_deref()
                    .ok_or("Missing volume metadata")?
            )?,
            original
        );
        for invalid in [
            r#"{"driver":"plugin","scope":"local","options":{},"created_at":null,"labels":{}}"#,
            r#"{"driver":"local","scope":"local","options":{"device":"/host"},"created_at":null,"labels":{}}"#,
            r#"{"driver":"local","scope":"local","options":{},"created_at":null,"labels":{},"uid":"fake"}"#,
            r#"{"driver":"local","scope":"global","options":{},"created_at":null,"labels":{}}"#,
        ] {
            assert!(decode_volume(invalid).is_err());
        }
        let large = DockerVolumeIdentity::local(
            "local",
            "local",
            &BTreeMap::new(),
            None,
            (0..40)
                .map(|i| (format!("label-{i}"), "x".repeat(1024)))
                .collect(),
        )?;
        assert!(identity_columns(Some(&NativeIdentity::DockerVolume(large))).is_err());
        Ok(())
    }

    fn candidate() -> ResourceRow {
        ResourceRow {
            resource_kind: "DOCKER_CONTAINER".into(),
            native_scope_kind: "DOCKER_DAEMON".into(),
            native_scope: "daemon:original".into(),
            name: "execution-original".into(),
            resource_uid: None,
            docker_container_id: None,
            docker_volume_metadata: None,
            execution_id: Some("execution".into()),
            presence: "UNKNOWN".into(),
            observed_at: None,
            absence_confirmed_at: None,
            source_connection_id: None,
        }
    }

    #[test]
    fn native_row_decoder_rejects_mixed_or_unallocated_present_evidence() -> Result<(), Error> {
        assert!(ManagedResource::try_from(candidate())?.identity.is_none());
        let mut row = candidate();
        row.resource_uid = Some("not-a-container-id".into());
        assert!(ManagedResource::try_from(row).is_err());
        let mut row = candidate();
        row.presence = "PRESENT".into();
        assert!(ManagedResource::try_from(row).is_err());
        let mut row = candidate();
        row.native_scope_kind = "KUBERNETES_NAMESPACE".into();
        assert!(ManagedResource::try_from(row).is_err());
        let mut row = candidate();
        row.execution_id = None;
        assert!(ManagedResource::try_from(row).is_err());
        let mut row = candidate();
        row.resource_kind = "UNSPECIFIED".into();
        assert!(ManagedResource::try_from(row).is_err());
        let mut row = candidate();
        row.docker_container_id = Some("a".repeat(64));
        row.presence = "PRESENT".into();
        let allocated = ManagedResource::try_from(row)?;
        assert_eq!(allocated.presence, ResourcePresence::Present);
        assert!(matches!(
            allocated.identity,
            Some(NativeIdentity::DockerContainer(_))
        ));
        assert!(allocated.kubernetes_uid().is_none());
        Ok(())
    }
}
