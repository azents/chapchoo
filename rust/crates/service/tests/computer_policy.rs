use chapchoo_service::ComputerPolicy;
#[test]
fn invalid_component_policy_is_rejected_before_admission() {
    let base = ComputerPolicy::default();
    let invalid = [
        ComputerPolicy {
            heartbeat_ms: 0,
            ..base
        },
        ComputerPolicy {
            heartbeat_ms: 60_000,
            ..base
        },
        ComputerPolicy {
            start_delayed_ms: 0,
            ..base
        },
        ComputerPolicy {
            keys: chapchoo_service::ComponentKeyPolicy {
                overlap_ms: 300_001,
                ..base.keys
            },
            ..base
        },
        ComputerPolicy {
            keys: chapchoo_service::ComponentKeyPolicy {
                proactive_after_ms: base.keys.validity_ms,
                ..base.keys
            },
            ..base
        },
        ComputerPolicy {
            directory: chapchoo_service::DirectoryPolicy {
                component_validity_ms: i64::MAX,
                ..base.directory
            },
            ..base
        },
    ];
    for policy in invalid {
        assert!(policy.validate().is_err());
    }
    assert!(base.validate().is_ok());
}
