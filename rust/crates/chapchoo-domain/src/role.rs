/// Stored membership role. OWNER is derived from the Team owner reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipRole {
    Manager,
    Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamKind {
    Personal,
    Collaborative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRole {
    Owner,
    Manager,
    Member,
}

impl TeamRole {
    pub fn effective(stored: MembershipRole, is_owner: bool) -> Self {
        if is_owner {
            Self::Owner
        } else {
            match stored {
                MembershipRole::Manager => Self::Manager,
                MembershipRole::Member => Self::Member,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_is_derived_without_a_second_stored_owner_role() {
        for stored in [MembershipRole::Member, MembershipRole::Manager] {
            assert_eq!(TeamRole::effective(stored, true), TeamRole::Owner);
        }
        assert_eq!(
            TeamRole::effective(MembershipRole::Manager, false),
            TeamRole::Manager
        );
        assert_eq!(
            TeamRole::effective(MembershipRole::Member, false),
            TeamRole::Member
        );
    }
}
