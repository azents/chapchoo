//! Managed local MCP names shared by declaration and captured approval projection.

pub const LEGACY_EXTERNAL_PREFIX: &str = "chapchoo_ext_";
const SHORT_PREFIX: &str = "cx";
const MAX_SLOT: u64 = 9_999_999_999;

pub fn short_external_name(slot: u64) -> Option<String> {
    (1..=MAX_SLOT)
        .contains(&slot)
        .then(|| format!("{SHORT_PREFIX}{slot}"))
}

pub fn valid_integration_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}

/// Name classification only. Native request and tool call authority are
/// checked independently against the exact applied declaration.
pub fn is_external_name(name: &str) -> bool {
    name.strip_prefix(SHORT_PREFIX).is_some_and(|number| {
        !number.starts_with('0')
            && number.bytes().all(|byte| byte.is_ascii_digit())
            && number
                .parse::<u64>()
                .is_ok_and(|slot| (1..=MAX_SLOT).contains(&slot))
    }) || name
        .strip_prefix(LEGACY_EXTERNAL_PREFIX)
        .is_some_and(valid_integration_id)
}
