use super::*;

#[test]
fn catalogue_assets_are_logical_local_keys_not_paths_or_urls() -> Result<(), StoreError> {
    let id = EntityId::generate(1000)?;
    let row = |asset: &str| CatalogueRow {
        id,
        item_type: "AGENT".into(),
        internal_name: "Installation image".into(),
        source_kind: "BUILTIN".into(),
        asset_key: asset.into(),
    };
    for valid in ["03-corgi", "installation-dog", "06-border-collie"] {
        row(valid).validate()?;
    }
    for invalid in [
        "",
        "../image",
        "https://external.invalid/image",
        "file/name",
        "03-corgi\n",
        "Uppercase",
    ] {
        assert!(row(invalid).validate().is_err());
    }
    assert!(row(&"a".repeat(129)).validate().is_err());
    Ok(())
}
