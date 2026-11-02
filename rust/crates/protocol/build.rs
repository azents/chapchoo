use std::{error::Error, path::PathBuf};

#[allow(clippy::print_stdout, reason = "Cargo build-script change tracking")]
fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let protos = manifest.join("../../../proto/chapchoo");
    let sources = [
        protos.join("accounts.proto"),
        protos.join("control.proto"),
        protos.join("computers.proto"),
    ];
    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.display());
    }
    let mut prost = tonic_prost_build::Config::new();
    prost.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    // Protocol values can contain credentials and personal data. Every message
    // uses an explicit field-free Debug implementation in the public module.
    prost.skip_debug(["."]);
    let mut generator = tonic_prost_build::configure();
    // A wildcard enum attribute also reaches oneof payloads. Those carry keys
    // and trusted authority and must use the field-free policy instead.
    for name in [
        "chapchoo.accounts.TeamKind",
        "chapchoo.accounts.TeamRole",
        "chapchoo.accounts.MembershipRole",
        "chapchoo.accounts.InvitationStatus",
        "chapchoo.accounts.InvitationAction",
        "chapchoo.accounts.GrantKind",
        "chapchoo.accounts.DomainReason",
        "chapchoo.control.OwnerAuthority",
        "chapchoo.control.ResourceKind",
        "chapchoo.control.ExecutionFact",
        "chapchoo.control.CommandOutcome",
        "chapchoo.control.ForwardingOutcome",
        "chapchoo.computers.ComputerKind",
        "chapchoo.computers.DesiredState",
        "chapchoo.computers.Power",
        "chapchoo.computers.Connectivity",
        "chapchoo.computers.SavedEnvironment",
        "chapchoo.computers.Action",
        "chapchoo.computers.Phase",
        "chapchoo.computers.BlockReason",
    ] {
        generator = generator.enum_attribute(name, "#[derive(Debug)]");
    }
    generator.compile_with_config(
        prost,
        &sources,
        &[protos, protoc_bin_vendored::include_path()?],
    )?;
    Ok(())
}
