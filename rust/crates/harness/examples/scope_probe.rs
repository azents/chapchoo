//! Explicit model-free acceptance probe for a disposable restricted Computer.
use chapchoo_harness::{NativeStorage, Preparer, Retirement, ScopeEvidence};
use nix::{
    sys::signal::{SigSet, Signal},
    unistd::setsid,
};
use std::{io::Write, path::Path, process::Stdio, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("descendant") => {
            setsid()?;
            let mut blocked = SigSet::empty();
            blocked.add(Signal::SIGTERM);
            blocked.thread_block()?;
            std::fs::write(args.get(2).ok_or("Missing ready path")?,std::process::id().to_string())?;
            loop { std::thread::park(); }
        },
        Some("parent") => {
            let _child = std::process::Command::new(std::env::current_exe()?).arg("descendant")
                .arg(args.get(2).ok_or("Missing ready path")?)
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
            Ok(())
        },
        _ => tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
            let storage = NativeStorage::open(Path::new(args.get(2).ok_or("Missing saved root")?))?;
            let scope = ScopeEvidence::verify(&storage)?;
            if args.get(1).map(String::as_str) == Some("native") {
                let owner = chapchoo_harness::NativeOwner::new(storage)?;
                let started = owner.start(chapchoo_domain::HarnessSupply::Managed,None,std::sync::Arc::new(|_|Ok(()))).await;
                let cleanup = owner.finish().await?;
                if cleanup != Retirement::Quiescent { return Err("Native retirement was not verified".into()); }
                let _context = started?;
                std::io::stdout().write_all(b"Strict ACP v2 native session initialization and whole-scope retirement verified; no prompt/model call.\n")?;
                return Ok(());
            }
            if args.get(1).map(String::as_str) == Some("prepare") {
                let mut preparer = Preparer::new(storage)?;
                let applied = preparer.prepare(&chapchoo_domain::HarnessSupply::Managed).await?;
                drop(applied);
                if scope.retire().await? != Retirement::Quiescent { return Err("Retirement was not verified".into()); }
                std::io::stdout().write_all(b"Managed preparation and restricted-scope cleanup verified; no model call.\n")?;
                return Ok(());
            }
            let ready = storage.temporary().join("detached-ready");
            match std::fs::remove_file(&ready) {
                Ok(())=>{},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
                Err(e)=>return Err(e.into()),
            }
            let mut parent = tokio::process::Command::new(std::env::current_exe()?)
                .arg("parent").arg(&ready).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
            if !parent.wait().await?.success() { return Err("Fixture parent failed".into()); }
            tokio::time::timeout(Duration::from_secs(10),async {
                loop {
                    if ready.is_file() { break; }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await?;
            let detached = std::fs::read_to_string(ready)?;
            if !Path::new("/proc").join(&detached).is_dir() { return Err("Detached fixture was not alive".into()); }
            if args.get(1).map(String::as_str) == Some("owner-death") {
                // Returning from namespace PID1 is the kernel teardown boundary.
                std::io::stdout().write_all(b"Detached descendant alive at namespace-init exit.\n")?;
                return Ok(());
            }
            if scope.retire().await? != Retirement::Quiescent || Path::new("/proc").join(detached).exists() {
                return Err("Detached descendant retirement was not verified".into());
            }
            std::io::stdout().write_all(b"Restricted PID1 scope, detached escalation and quiescent reaping verified.\n")?;
            Ok(())
        }),
    }
}
