use std::str::FromStr;

/// A persisted Computer enum must have an explicit supported value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid Computer domain value")]
pub struct InvalidComputerValue;

macro_rules! persisted_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }
        impl FromStr for $name {
            type Err = InvalidComputerValue;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($value => Ok(Self::$variant)),+, _ => Err(InvalidComputerValue) }
            }
        }
    };
}

persisted_enum!(ComputerKind { Cloud => "CLOUD", Local => "LOCAL" });
persisted_enum!(DesiredState {
    Running => "RUNNING", Stopped => "STOPPED", Deleting => "DELETING"
});
persisted_enum!(ObservedPower {
    Running => "RUNNING", Stopped => "STOPPED", Unknown => "UNKNOWN"
});
persisted_enum!(LifecycleAction {
    Create => "CREATE", Start => "START", Stop => "STOP", Restart => "RESTART", Delete => "DELETE"
});
persisted_enum!(LifecyclePhase {
    Accepted => "ACCEPTED", Retiring => "RETIRING", Starting => "STARTING",
    Deleting => "DELETING", Completed => "COMPLETED", Blocked => "BLOCKED", Uncertain => "UNCERTAIN"
});

impl LifecycleAction {
    pub fn desired_state(self) -> DesiredState {
        match self {
            Self::Create | Self::Start | Self::Restart => DesiredState::Running,
            Self::Stop => DesiredState::Stopped,
            Self::Delete => DesiredState::Deleting,
        }
    }
}

/// Connectivity does not prove the state of an external execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePresence {
    Present,
    Absent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedEnvironment {
    Verified,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerCapabilities {
    pub start: bool,
    pub stop: bool,
    pub restart: bool,
}

/// Current operation context, not a second persisted UI lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleProgress {
    pub action: LifecycleAction,
    pub phase: LifecyclePhase,
    pub initial_start_dispatch_ms: Option<i64>,
}

impl LifecycleProgress {
    pub fn pending(self) -> bool {
        self.phase != LifecyclePhase::Completed
    }

    pub fn start_delayed(self, now_ms: i64) -> bool {
        self.start_delayed_after(now_ms, 300_000)
    }
    pub fn start_delayed_after(self, now_ms: i64, threshold_ms: i64) -> bool {
        matches!(
            self.action,
            LifecycleAction::Create | LifecycleAction::Start | LifecycleAction::Restart
        ) && self.pending()
            && self.initial_start_dispatch_ms.is_some_and(|initial| {
                now_ms
                    .checked_sub(initial)
                    .is_some_and(|elapsed| elapsed >= threshold_ms)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBlockReason {
    Unsupported,
    Deleting,
    ConflictingLifecycle,
    ManagementUnavailable,
    PowerUnknown,
    RequiresStopped,
    RequiresRunning,
    SavedEnvironmentUnverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionAvailability {
    Available,
    Blocked(ActionBlockReason),
}

/// Facts must already be authorized and freshness-qualified by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputerActionContext {
    pub desired: DesiredState,
    pub power: ObservedPower,
    pub management: ConnectionStatus,
    pub capabilities: PowerCapabilities,
    pub lifecycle: Option<LifecycleProgress>,
}

impl ComputerActionContext {
    pub fn power_action(self, action: LifecycleAction) -> ActionAvailability {
        use ActionAvailability::{Available, Blocked};
        use ActionBlockReason as Reason;
        let supported = match action {
            LifecycleAction::Start => self.capabilities.start,
            LifecycleAction::Stop => self.capabilities.stop,
            LifecycleAction::Restart => self.capabilities.restart,
            LifecycleAction::Create | LifecycleAction::Delete => false,
        };
        if !supported {
            return Blocked(Reason::Unsupported);
        }
        if self.desired == DesiredState::Deleting {
            return Blocked(Reason::Deleting);
        }
        if self.lifecycle.is_some_and(LifecycleProgress::pending) {
            return Blocked(Reason::ConflictingLifecycle);
        }
        if self.management != ConnectionStatus::Connected {
            return Blocked(Reason::ManagementUnavailable);
        }
        match (action, self.power) {
            (_, ObservedPower::Unknown) => Blocked(Reason::PowerUnknown),
            (LifecycleAction::Start, ObservedPower::Stopped)
            | (LifecycleAction::Stop | LifecycleAction::Restart, ObservedPower::Running) => {
                Available
            }
            (LifecycleAction::Start, _) => Blocked(Reason::RequiresStopped),
            _ => Blocked(Reason::RequiresRunning),
        }
    }

    /// An explicit force-delete is available throughout deletion, without a timer.
    pub fn force_delete_available(self) -> bool {
        self.desired == DesiredState::Deleting
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestorationBlockReason {
    NotRunningIntent,
    ConflictingLifecycle,
    PriorExecutionNotAbsent,
    SavedEnvironmentUnverified,
    OwnershipUnverified,
}

/// All independent evidence predicates must hold; disconnection is not absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestorationEvidence {
    pub desired: DesiredState,
    pub conflicting_lifecycle: bool,
    pub prior_execution: ResourcePresence,
    pub saved_environment: SavedEnvironment,
    pub ownership_verified: bool,
}

impl RestorationEvidence {
    pub fn check(self) -> Result<(), RestorationBlockReason> {
        if self.desired != DesiredState::Running {
            return Err(RestorationBlockReason::NotRunningIntent);
        }
        if self.conflicting_lifecycle {
            return Err(RestorationBlockReason::ConflictingLifecycle);
        }
        if self.prior_execution != ResourcePresence::Absent {
            return Err(RestorationBlockReason::PriorExecutionNotAbsent);
        }
        if self.saved_environment != SavedEnvironment::Verified {
            return Err(RestorationBlockReason::SavedEnvironmentUnverified);
        }
        if !self.ownership_verified {
            return Err(RestorationBlockReason::OwnershipUnverified);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available() -> ComputerActionContext {
        ComputerActionContext {
            desired: DesiredState::Running,
            power: ObservedPower::Running,
            management: ConnectionStatus::Connected,
            capabilities: PowerCapabilities {
                start: true,
                stop: true,
                restart: true,
            },
            lifecycle: None,
        }
    }

    #[test]
    fn power_admission_uses_management_and_power_not_connector_connectivity() {
        let context = available();
        assert_eq!(
            context.power_action(LifecycleAction::Stop),
            ActionAvailability::Available
        );
        assert_eq!(
            context.power_action(LifecycleAction::Restart),
            ActionAvailability::Available
        );
        assert_eq!(
            context.power_action(LifecycleAction::Start),
            ActionAvailability::Blocked(ActionBlockReason::RequiresStopped)
        );
        let stopped = ComputerActionContext {
            power: ObservedPower::Stopped,
            desired: DesiredState::Stopped,
            ..context
        };
        assert_eq!(
            stopped.power_action(LifecycleAction::Start),
            ActionAvailability::Available
        );
        let unknown = ComputerActionContext {
            power: ObservedPower::Unknown,
            ..context
        };
        assert_eq!(
            unknown.power_action(LifecycleAction::Stop),
            ActionAvailability::Blocked(ActionBlockReason::PowerUnknown)
        );
        let disconnected = ComputerActionContext {
            management: ConnectionStatus::Disconnected,
            ..context
        };
        assert_eq!(
            disconnected.power_action(LifecycleAction::Stop),
            ActionAvailability::Blocked(ActionBlockReason::ManagementUnavailable)
        );
    }

    #[test]
    fn unresolved_operation_blocks_conflicts_but_force_deletion_has_no_wait() {
        for phase in [
            LifecyclePhase::Accepted,
            LifecyclePhase::Retiring,
            LifecyclePhase::Starting,
            LifecyclePhase::Blocked,
            LifecyclePhase::Uncertain,
        ] {
            let context = ComputerActionContext {
                lifecycle: Some(LifecycleProgress {
                    action: LifecycleAction::Restart,
                    phase,
                    initial_start_dispatch_ms: None,
                }),
                ..available()
            };
            assert_eq!(
                context.power_action(LifecycleAction::Stop),
                ActionAvailability::Blocked(ActionBlockReason::ConflictingLifecycle)
            );
        }
        let deleting = ComputerActionContext {
            desired: DesiredState::Deleting,
            management: ConnectionStatus::Disconnected,
            power: ObservedPower::Unknown,
            ..available()
        };
        assert!(deleting.force_delete_available());
        assert_eq!(
            deleting.power_action(LifecycleAction::Start),
            ActionAvailability::Blocked(ActionBlockReason::Deleting)
        );
    }

    #[test]
    fn start_delay_uses_original_dispatch_not_observation_or_reconnect() {
        let progress = LifecycleProgress {
            action: LifecycleAction::Restart,
            phase: LifecyclePhase::Starting,
            initial_start_dispatch_ms: Some(10_000),
        };
        assert!(!progress.start_delayed(309_999));
        assert!(progress.start_delayed(310_000));
        assert!(!progress.start_delayed(9_999));
        assert!(
            !LifecycleProgress {
                phase: LifecyclePhase::Completed,
                ..progress
            }
            .start_delayed(400_000)
        );
        assert!(
            !LifecycleProgress {
                initial_start_dispatch_ms: None,
                ..progress
            }
            .start_delayed(400_000)
        );
    }

    #[test]
    fn restoration_requires_every_independent_safety_predicate() {
        let safe = RestorationEvidence {
            desired: DesiredState::Running,
            conflicting_lifecycle: false,
            prior_execution: ResourcePresence::Absent,
            saved_environment: SavedEnvironment::Verified,
            ownership_verified: true,
        };
        assert_eq!(safe.check(), Ok(()));
        assert_eq!(
            RestorationEvidence {
                desired: DesiredState::Stopped,
                ..safe
            }
            .check(),
            Err(RestorationBlockReason::NotRunningIntent)
        );
        assert_eq!(
            RestorationEvidence {
                desired: DesiredState::Deleting,
                ..safe
            }
            .check(),
            Err(RestorationBlockReason::NotRunningIntent)
        );
        assert_eq!(
            RestorationEvidence {
                conflicting_lifecycle: true,
                ..safe
            }
            .check(),
            Err(RestorationBlockReason::ConflictingLifecycle)
        );
        for presence in [ResourcePresence::Present, ResourcePresence::Unknown] {
            assert_eq!(
                RestorationEvidence {
                    prior_execution: presence,
                    ..safe
                }
                .check(),
                Err(RestorationBlockReason::PriorExecutionNotAbsent)
            );
        }
        for saved in [SavedEnvironment::Missing, SavedEnvironment::Unknown] {
            assert_eq!(
                RestorationEvidence {
                    saved_environment: saved,
                    ..safe
                }
                .check(),
                Err(RestorationBlockReason::SavedEnvironmentUnverified)
            );
        }
        assert_eq!(
            RestorationEvidence {
                ownership_verified: false,
                ..safe
            }
            .check(),
            Err(RestorationBlockReason::OwnershipUnverified)
        );
    }

    #[test]
    fn persisted_values_reject_unknowns_instead_of_defaulting() -> Result<(), InvalidComputerValue>
    {
        assert_eq!(
            DesiredState::from_str(DesiredState::Deleting.as_str())?,
            DesiredState::Deleting
        );
        assert_eq!(ComputerKind::from_str("LOCAL")?, ComputerKind::Local);
        assert!(ObservedPower::from_str("DISCONNECTED").is_err());
        assert!(LifecyclePhase::from_str("FUTURE").is_err());
        assert_eq!(
            LifecycleAction::Restart.desired_state(),
            DesiredState::Running
        );
        assert_eq!(
            LifecycleAction::Delete.desired_state(),
            DesiredState::Deleting
        );
        Ok(())
    }
}
