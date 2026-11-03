#[allow(clippy::print_stdout, reason = "Cargo build-script change tracking")]
fn main() {
    // Embedded SQL and newly added migration directories must invalidate the
    // library, including when no Rust source changed alongside the migration.
    println!("cargo:rerun-if-changed=migrations");
}
