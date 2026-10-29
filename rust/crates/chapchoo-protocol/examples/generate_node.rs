//! Run the locked Node generator with the same vendored protoc as Rust builds.
use std::{error::Error, ffi::OsString, io, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args_os().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "An output directory is required",
        )
    })?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let plugin =
        root.join("typescript/packages/chapchoo-rpc-client/node_modules/.bin/protoc-gen-ts_proto");
    let mut plugin_argument = OsString::from("--plugin=protoc-gen-ts_proto=");
    plugin_argument.push(plugin);
    let mut output_argument = OsString::from("--ts_proto_out=");
    output_argument.push(output);
    let status = Command::new(protoc_bin_vendored::protoc_bin_path()?)
        .arg(plugin_argument)
        .arg(output_argument)
        .arg("--ts_proto_opt=outputServices=grpc-js,env=node,esModuleInterop=true,importSuffix=.js,forceLong=bigint,useDate=false,outputJsonMethods=false")
        .arg("-I")
        .arg(root.join("proto/chapchoo"))
        .arg("-I")
        .arg(protoc_bin_vendored::include_path()?)
        .arg(root.join("proto/chapchoo/accounts.proto"))
        .status()?;
    if !status.success() {
        return Err(io::Error::other("Node protocol generation failed").into());
    }
    Ok(())
}
