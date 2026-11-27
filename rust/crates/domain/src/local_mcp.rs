//! Managed local MCP names shared by declaration and captured approval projection.

pub const EXTERNAL_PREFIX: &str = "chapchoo_ext_";

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
    name.strip_prefix(EXTERNAL_PREFIX)
        .is_some_and(valid_integration_id)
}
