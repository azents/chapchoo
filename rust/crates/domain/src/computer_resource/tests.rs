use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn provider_binding_and_required_resources_are_substrate_specific() -> TestResult {
    let bound = ProviderSubstrate::Docker(DockerDaemonId::parse("daemon:original".into())?);
    let current = NativeScope::parse("DOCKER_DAEMON", "daemon:original".into())?;
    let replacement = NativeScope::parse("DOCKER_DAEMON", "daemon:other".into())?;
    let namespace = NativeScope::parse("KUBERNETES_NAMESPACE", "fixture".into())?;
    assert!(bound.accepts_scope(&current));
    assert!(!bound.accepts_scope(&replacement));
    assert!(!bound.accepts_scope(&namespace));
    assert!(ProviderSubstrate::Kubernetes.accepts_scope(&namespace));
    assert!(!ProviderSubstrate::Kubernetes.accepts_scope(&current));
    assert_eq!(bound.docker_daemon_id(), Some("daemon:original"));
    assert_eq!(current.kind_str(), "DOCKER_DAEMON");
    assert!(NativeScope::parse("UNSPECIFIED", "fixture".into()).is_err());
    assert_eq!(
        ProviderKind::Docker.required_resource_kinds(),
        &[
            ManagedResourceKind::DockerContainer,
            ManagedResourceKind::DockerVolume
        ]
    );
    for kind in [ProviderKind::Kubernetes, ProviderKind::Docker] {
        assert_eq!(
            kind.saved_resource_kind().role(),
            ResourceRole::SavedEnvironment
        );
        assert_eq!(
            kind.execution_resource_kind().role(),
            ResourceRole::Execution
        );
        assert!(
            kind.required_resource_kinds()
                .iter()
                .all(|r| r.provider_kind() == kind)
        );
    }
    Ok(())
}

fn daemon() -> Result<NativeScope, InvalidNativeResource> {
    Ok(NativeScope::DockerDaemon(DockerDaemonId::parse(format!(
        "daemon:{}",
        "a".repeat(100)
    ))?))
}

#[test]
fn kind_decode_has_one_exact_vocabulary_and_derived_roles() -> TestResult {
    for (value, provider, role) in [
        ("POD", ProviderKind::Kubernetes, ResourceRole::Execution),
        (
            "PVC",
            ProviderKind::Kubernetes,
            ResourceRole::SavedEnvironment,
        ),
        (
            "BOOTSTRAP_SECRET",
            ProviderKind::Kubernetes,
            ResourceRole::Execution,
        ),
        (
            "DOCKER_CONTAINER",
            ProviderKind::Docker,
            ResourceRole::Execution,
        ),
        (
            "DOCKER_VOLUME",
            ProviderKind::Docker,
            ResourceRole::SavedEnvironment,
        ),
    ] {
        let kind: ManagedResourceKind = value.parse()?;
        assert_eq!(kind.as_str(), value);
        assert_eq!(kind.provider_kind(), provider);
        assert_eq!(kind.role(), role);
        assert_eq!(provider.as_str().parse::<ProviderKind>()?, provider);
    }
    for invalid in ["", "UNSPECIFIED", "SECRET", "pod", "POD ", "VOLUME"] {
        assert!(invalid.parse::<ManagedResourceKind>().is_err());
    }
    for invalid in ["", "docker", "KUBERNETES ", "OTHER"] {
        assert!(invalid.parse::<ProviderKind>().is_err());
    }
    Ok(())
}

#[test]
fn reservation_rejects_wrong_substrate_and_retains_long_opaque_daemon() -> TestResult {
    let docker = NativeReservation::new(
        ManagedResourceKind::DockerContainer,
        daemon()?,
        "computer-1".into(),
    )?;
    assert!(docker.scope().as_str().len() > 63);
    assert_eq!(docker.name(), "computer-1");
    assert_eq!(docker.scope().provider_kind(), ProviderKind::Docker);
    assert!(NativeReservation::new(ManagedResourceKind::Pod, daemon()?, "pod-1".into(),).is_err());
    assert!(
        NativeReservation::new(
            ManagedResourceKind::DockerVolume,
            NativeScope::kubernetes("namespace".into())?,
            "volume-1".into(),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn native_bounds_reject_invalid_or_truncated_addresses() -> TestResult {
    for invalid in ["", "daemon\nother", "daemon other", &"x".repeat(1025)] {
        assert!(DockerDaemonId::parse(invalid.into()).is_err());
    }
    assert!(DockerDaemonId::parse("x".repeat(1024)).is_ok());
    assert!(NativeScope::kubernetes("n".repeat(63)).is_ok());
    for invalid in ["", "-namespace", "namespace-", "UPPER", &"n".repeat(64)] {
        assert!(NativeScope::kubernetes(invalid.into()).is_err());
    }
    for invalid in ["", "/volume", "volume/other", &"x".repeat(256)] {
        assert!(
            NativeReservation::new(ManagedResourceKind::DockerVolume, daemon()?, invalid.into(),)
                .is_err()
        );
    }
    // Public enum construction cannot bypass reservation validation.
    assert!(
        NativeReservation::new(
            ManagedResourceKind::Pod,
            NativeScope::KubernetesNamespace("bad\nnamespace".into()),
            "pod".into(),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn allocated_container_requires_full_native_id_and_matching_variant() -> TestResult {
    for invalid in [
        "",
        "abcdef",
        &"A".repeat(64),
        &"a".repeat(65),
        &"g".repeat(64),
    ] {
        assert!(DockerContainerId::parse(invalid.into()).is_err());
    }
    let reservation = NativeReservation::new(
        ManagedResourceKind::DockerContainer,
        daemon()?,
        "computer-1".into(),
    )?;
    let original = NativeAllocation::new(
        reservation.clone(),
        NativeIdentity::DockerContainer(DockerContainerId::parse("a".repeat(64))?),
    )?;
    let replacement = NativeAllocation::new(
        reservation.clone(),
        NativeIdentity::DockerContainer(DockerContainerId::parse("b".repeat(64))?),
    )?;
    assert_ne!(original, replacement);
    assert_eq!(original.reservation(), &reservation);
    assert!(
        NativeAllocation::new(
            reservation,
            NativeIdentity::KubernetesObject(KubernetesUid::parse("uid-1".into())?),
        )
        .is_err()
    );
    let other_daemon = NativeReservation::new(
        ManagedResourceKind::DockerContainer,
        NativeScope::DockerDaemon(DockerDaemonId::parse("other-daemon".into())?),
        "computer-1".into(),
    )?;
    assert_ne!(
        original,
        NativeAllocation::new(other_daemon, original.identity().clone())?
    );
    Ok(())
}

#[test]
fn volume_observations_keep_original_metadata_without_synthetic_uid() -> TestResult {
    let labels = BTreeMap::from([
        ("provider".into(), "provider-1".into()),
        ("computer".into(), "computer-1".into()),
    ]);
    let original = DockerVolumeIdentity::local(
        "local",
        "local",
        &BTreeMap::new(),
        Some("2026-11-03T00:00:00Z".into()),
        labels.clone(),
    )?;
    assert_eq!(original.created_at(), Some("2026-11-03T00:00:00Z"));
    assert_eq!(original.labels(), &labels);
    let changed = DockerVolumeIdentity::local(
        "local",
        "local",
        &BTreeMap::new(),
        Some("2026-11-04T00:00:00Z".into()),
        labels.clone(),
    )?;
    assert_ne!(original, changed);
    let unavailable_time =
        DockerVolumeIdentity::local("local", "local", &BTreeMap::new(), None, labels.clone())?;
    assert_ne!(original, unavailable_time);
    let reservation = NativeReservation::new(
        ManagedResourceKind::DockerVolume,
        daemon()?,
        "saved-1".into(),
    )?;
    let allocation = NativeAllocation::new(reservation, NativeIdentity::DockerVolume(original))?;
    assert_eq!(
        allocation.reservation().kind().role(),
        ResourceRole::SavedEnvironment
    );
    for (driver, scope, options) in [
        ("plugin", "local", BTreeMap::new()),
        ("local", "global", BTreeMap::new()),
        (
            "local",
            "local",
            BTreeMap::from([("device".into(), "/host".into())]),
        ),
    ] {
        assert!(
            DockerVolumeIdentity::local(driver, scope, &options, None, labels.clone()).is_err()
        );
    }
    assert!(
        DockerVolumeIdentity::local(
            "local",
            "local",
            &BTreeMap::new(),
            Some("invalid\nmetadata".into()),
            labels,
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn kubernetes_original_address_and_uid_are_unchanged() -> TestResult {
    let reservation = NativeReservation::new(
        ManagedResourceKind::Pvc,
        NativeScope::kubernetes("existing.namespace".into())?,
        "original-pvc".into(),
    )?;
    let uid = KubernetesUid::parse("original-uid-1".into())?;
    let allocation =
        NativeAllocation::new(reservation, NativeIdentity::KubernetesObject(uid.clone()))?;
    assert_eq!(
        allocation.reservation().scope().as_str(),
        "existing.namespace"
    );
    assert_eq!(allocation.reservation().name(), "original-pvc");
    assert_eq!(
        allocation.identity(),
        &NativeIdentity::KubernetesObject(uid)
    );
    assert!(
        !NativeIdentity::DockerContainer(DockerContainerId::parse("a".repeat(64))?)
            .matches_kind(ManagedResourceKind::Pvc)
    );
    Ok(())
}
