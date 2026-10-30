use std::{error::Error, path::PathBuf};

#[allow(clippy::print_stdout, reason = "Cargo build-script change tracking")]
fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let protos = manifest.join("../../../proto/chapchoo");
    let source = protos.join("accounts.proto");
    println!("cargo:rerun-if-changed={}", source.display());
    let mut prost = tonic_prost_build::Config::new();
    prost.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    // Protocol values can contain credentials and personal data. Every message
    // uses an explicit field-free Debug implementation in the public module.
    prost.skip_debug(["."]);
    tonic_prost_build::configure()
        .enum_attribute(".", "#[derive(Debug)]")
        .compile_with_config(
            prost,
            &[source],
            &[protos, protoc_bin_vendored::include_path()?],
        )?;
    Ok(())
}
