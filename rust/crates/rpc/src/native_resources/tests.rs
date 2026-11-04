use super::*;
type Error = Box<dyn std::error::Error + Send + Sync>;

fn container() -> wire::ManagedResource {
    wire::ManagedResource {
        kind: wire::ResourceKind::DockerContainer as i32,
        name: "container-original".into(),
        execution_id: Some("execution".into()),
        allocation: Some(wire::NativeAllocationWitness {
            computer_id: "original-computer".into(),
            provider_id: "original-provider".into(),
            execution_id: Some("original-execution".into()),
        }),
        native: Some(wire::managed_resource::Native::DockerContainer(
            wire::DockerContainerNative {
                daemon_id: format!("daemon:{}", "a".repeat(100)),
                container_id: Some("b".repeat(64)),
            },
        )),
        ..Default::default()
    }
}

fn local_volume() -> wire::DockerVolumeIdentity {
    wire::DockerVolumeIdentity {
        driver: "local".into(),
        scope: "local".into(),
        labels: [("provider".into(), "provider-1".into())]
            .into_iter()
            .collect(),
        created_at: Some("2026-11-03T00:00:00Z".into()),
        ..Default::default()
    }
}

#[test]
fn decode_rejects_mixed_unknown_or_wrong_substrate_payloads() -> Result<(), Error> {
    let value = container();
    let decoded = decode(&value)?;
    assert_eq!(decoded.native.kind(), ManagedResourceKind::DockerContainer);
    assert!(decoded.native.scope().as_str().len() > 63);
    assert!(matches!(
        decoded.identity,
        Some(NativeIdentity::DockerContainer(_))
    ));
    let mut mixed = value.clone();
    mixed.namespace = "fake-kubernetes".into();
    assert!(decode(&mixed).is_err());
    mixed = value.clone();
    mixed.uid = Some("fake-uid".into());
    assert!(decode(&mixed).is_err());
    mixed = value.clone();
    mixed.kind = wire::ResourceKind::Pod as i32;
    mixed.namespace = "fixture".into();
    assert!(decode(&mixed).is_err());
    mixed = value.clone();
    mixed.native = Some(wire::managed_resource::Native::DockerVolume(
        wire::DockerVolumeNative {
            daemon_id: "daemon:original".into(),
            identity: Some(local_volume()),
        },
    ));
    assert!(decode(&mixed).is_err());
    mixed = value.clone();
    mixed.native = None;
    assert!(decode(&mixed).is_err());
    mixed = value;
    mixed.kind = 900;
    assert!(decode(&mixed).is_err());
    let kubernetes = wire::ManagedResource {
        kind: wire::ResourceKind::Pvc as i32,
        namespace: "fixture".into(),
        name: "original-pvc".into(),
        uid: Some("original-uid".into()),
        allocation: Some(wire::NativeAllocationWitness {
            computer_id: "original-computer".into(),
            provider_id: "original-provider".into(),
            execution_id: None,
        }),
        ..Default::default()
    };
    assert_eq!(decode(&kubernetes)?.native.kind(), ManagedResourceKind::Pvc);
    Ok(())
}

#[test]
fn allocation_witness_is_required_validated_and_independent_of_correlation_identity()
-> Result<(), Error> {
    let value = container();
    let decoded = decode(&value)?;
    assert_eq!(decoded.allocation.execution(), Some("original-execution"));
    assert_eq!(decoded.execution_id.as_deref(), Some("execution"));
    for variant in 0..5 {
        let mut invalid = value.clone();
        match variant {
            0 => invalid.allocation = None,
            1 => invalid
                .allocation
                .as_mut()
                .ok_or("Witness")?
                .provider_id
                .clear(),
            2 => invalid.allocation.as_mut().ok_or("Witness")?.computer_id = "../invalid".into(),
            3 => invalid.allocation.as_mut().ok_or("Witness")?.execution_id = None,
            _ => invalid.execution_id = None,
        }
        assert_eq!(
            decode(&invalid)
                .err()
                .ok_or("Accepted invalid witness")?
                .code(),
            tonic::Code::InvalidArgument
        );
    }
    Ok(())
}

#[test]
fn native_evidence_preserves_original_variant_and_local_volume_observations() -> Result<(), Error> {
    let present = wire::resource_evidence::Evidence::Present(wire::OwnedPresent {
        native_identity: Some(wire::owned_present::NativeIdentity::DockerVolume(
            local_volume(),
        )),
        ..Default::default()
    });
    assert!(matches!(
        evidence(&present, ManagedResourceKind::DockerVolume)?,
        DecodedEvidence::Present(NativeIdentity::DockerVolume(_))
    ));
    assert!(evidence(&present, ManagedResourceKind::DockerContainer).is_err());
    assert!(evidence(&present, ManagedResourceKind::Pvc).is_err());
    let mixed = wire::resource_evidence::Evidence::Present(wire::OwnedPresent {
        uid: "fake-native-uid".into(),
        native_identity: Some(wire::owned_present::NativeIdentity::DockerContainerId(
            "a".repeat(64),
        )),
    });
    assert!(evidence(&mixed, ManagedResourceKind::DockerContainer).is_err());
    let absent = wire::resource_evidence::Evidence::Absent(wire::DefinitivelyAbsent {
        expected_native_identity: Some(
            wire::definitively_absent::ExpectedNativeIdentity::DockerContainerId("a".repeat(64)),
        ),
        ..Default::default()
    });
    assert!(matches!(
        evidence(&absent, ManagedResourceKind::DockerContainer)?,
        DecodedEvidence::Absent(Some(NativeIdentity::DockerContainer(_)))
    ));
    assert!(evidence(&absent, ManagedResourceKind::DockerVolume).is_err());
    let mut invalid_volume = local_volume();
    invalid_volume
        .options
        .insert("device".into(), "/host".into());
    assert!(volume_from_wire(&invalid_volume).is_err());
    invalid_volume = local_volume();
    invalid_volume.driver = "plugin".into();
    assert!(volume_from_wire(&invalid_volume).is_err());
    assert_eq!(
        volume_to_wire(&volume_from_wire(&local_volume())?),
        local_volume()
    );
    Ok(())
}

#[test]
fn cursor_uses_canonical_native_scope_without_truncation_or_dual_authority() -> Result<(), Error> {
    let native = decode(&container())?.native;
    let encoded = cursor(&native);
    assert!(encoded.namespace.is_empty());
    assert_eq!(encoded.native_scope_kind, "DOCKER_DAEMON");
    assert!(encoded.native_scope.len() > 63);
    assert_eq!(decode_cursor(encoded.clone())?, native);
    let mut mixed = encoded.clone();
    mixed.namespace = "fake".into();
    assert!(decode_cursor(mixed).is_err());
    mixed = encoded;
    mixed.native_scope_kind = "KUBERNETES_NAMESPACE".into();
    assert!(decode_cursor(mixed).is_err());
    let kubernetes = NativeReservation::new(
        ManagedResourceKind::Pvc,
        NativeScope::kubernetes("fixture".into())?,
        "original-pvc".into(),
    )?;
    let encoded = cursor(&kubernetes);
    assert!(encoded.native_scope.is_empty());
    assert!(encoded.native_scope_kind.is_empty());
    assert_eq!(decode_cursor(encoded)?, kubernetes);
    Ok(())
}

#[test]
fn hello_requires_explicit_kind_and_one_coherent_scope() -> Result<(), Error> {
    let mut hello = wire::ProviderHello {
        kind: wire::ProviderKind::Docker as i32,
        docker_daemon_id: Some("daemon:original".into()),
        ..Default::default()
    };
    assert_eq!(hello_scope(&hello)?.provider_kind(), ProviderKind::Docker);
    hello.namespace = "fake".into();
    assert!(hello_scope(&hello).is_err());
    hello.namespace.clear();
    hello.kind = wire::ProviderKind::Unspecified as i32;
    assert!(hello_scope(&hello).is_err());
    hello.kind = wire::ProviderKind::Kubernetes as i32;
    assert!(hello_scope(&hello).is_err());
    hello.docker_daemon_id = None;
    hello.namespace = "fixture".into();
    assert_eq!(
        hello_scope(&hello)?.provider_kind(),
        ProviderKind::Kubernetes
    );
    Ok(())
}
