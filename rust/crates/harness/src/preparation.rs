use crate::{Error, NativeStorage};
use chapchoo_domain::HarnessSupply;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt, fs,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::Stdio,
    sync::{Arc, Weak},
    time::Duration,
};

#[cfg(test)]
mod tests;

const REGISTRY: &str = "https://registry.npmjs.org/";
const ADAPTER: &str = "@agentclientprotocol/codex-acp";
const CODEX: &str = "@openai/codex";
const MAX_ARCHIVE: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Current,
    TransientLookupFailure,
}

/// An exact complete combination, retained until physical retirement.
#[derive(Clone)]
pub struct AppliedSupply {
    pub(crate) lease: Arc<PathBuf>,
    pub(crate) node: PathBuf,
    pub(crate) adapter: PathBuf,
    pub(crate) codex: PathBuf,
    pub freshness: Freshness,
}
impl fmt::Debug for AppliedSupply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppliedSupply")
            .field("freshness", &self.freshness)
            .finish_non_exhaustive()
    }
}

/// Single execution owner serializes prepare, acquire and collection.
pub struct Preparer {
    storage: NativeStorage,
    client: reqwest::Client,
    leases: BTreeMap<PathBuf, Weak<PathBuf>>,
}
impl fmt::Debug for Preparer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Preparer([REDACTED])")
    }
}
#[derive(Deserialize)]
struct NodeRelease {
    version: String,
    files: Vec<String>,
}
#[derive(Deserialize)]
struct Package {
    version: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Complete {
    platform: String,
    node_version: String,
    node_sha256: String,
    adapter_version: String,
    codex_version: String,
    files: BTreeMap<PathBuf, String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageLock {
    lockfile_version: u32,
    packages: BTreeMap<String, LockedPackage>,
}
#[derive(Deserialize)]
struct LockedPackage {
    resolved: Option<String>,
    integrity: Option<String>,
}

impl Preparer {
    pub fn new(storage: NativeStorage) -> Result<Self, Error> {
        use rustls_platform_verifier::BuilderVerifierExt;
        let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|_| Error::Process)?
        .with_platform_verifier()
        .map_err(|_| Error::Process)?
        .with_no_client_auth();
        let client = reqwest::Client::builder()
            .use_preconfigured_tls(tls)
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|_| Error::Process)?;
        Ok(Self {
            storage,
            client,
            leases: BTreeMap::new(),
        })
    }

    /// Called only for a genuine authorized new native start.
    pub async fn prepare(&mut self, supply: &HarnessSupply) -> Result<AppliedSupply, Error> {
        if let HarnessSupply::Existing(path) = supply
            && !Path::new(path.as_str()).is_file()
        {
            return Err(Error::Incompatible);
        }
        let platform = platform()?;
        let prepared = self.acquire(platform).await;
        let mut applied = self.eligible_result(prepared, platform)?;
        if let HarnessSupply::Existing(absolute_path) = supply {
            let path = Path::new(absolute_path.as_str());
            if !path.is_absolute() || !path.is_file() {
                return Err(Error::Incompatible);
            }
            let version = probe_version(&self.storage, path, &applied.node, false).await?;
            if !compatible_codex(&version) {
                return Err(Error::Incompatible);
            }
            applied.codex = path.to_owned();
        }
        self.collect(&applied.lease)?;
        Ok(applied)
    }

    fn eligible_result(
        &mut self,
        prepared: Result<AppliedSupply, Error>,
        platform: &str,
    ) -> Result<AppliedSupply, Error> {
        match prepared {
            Ok(value) => Ok(value),
            Err(Error::Transient) => {
                let latest = self.storage.installations().join("latest.json");
                let name: String =
                    serde_json::from_slice(&fs::read(latest).map_err(|_| Error::Transient)?)
                        .map_err(|_| Error::Integrity)?;
                if !owned_name(&name) {
                    return Err(Error::Integrity);
                }
                let mut cached = self.load(&self.storage.installations().join(name), platform)?;
                cached.freshness = Freshness::TransientLookupFailure;
                Ok(cached)
            }
            Err(error) => Err(error),
        }
    }
    async fn acquire(&mut self, platform: &str) -> Result<AppliedSupply, Error> {
        let bytes = download(
            &self.client,
            "https://nodejs.org/dist/index.json",
            4 * 1024 * 1024,
        )
        .await?;
        let releases: Vec<NodeRelease> =
            serde_json::from_slice(&bytes).map_err(|_| Error::Integrity)?;
        let release = releases
            .into_iter()
            .filter_map(|r| {
                Version::parse(r.version.trim_start_matches('v'))
                    .ok()
                    .map(|v| (v, r))
            })
            .filter(|(v, r)| {
                v.major == 24 && v.pre.is_empty() && r.files.iter().any(|f| f == platform)
            })
            .max_by(|a, b| a.0.cmp(&b.0))
            .ok_or(Error::Incompatible)?
            .1;
        let node_version = release.version;
        let archive_name = format!("node-{node_version}-{platform}.tar.gz");
        let base = format!("https://nodejs.org/dist/{node_version}");
        let sums = download(&self.client, &format!("{base}/SHASUMS256.txt"), 128 * 1024).await?;
        let sums = std::str::from_utf8(&sums).map_err(|_| Error::Integrity)?;
        let digest = sums
            .lines()
            .find_map(|line| {
                let (hash, name) = line.split_once("  ")?;
                (name == archive_name
                    && hash.len() == 64
                    && hash.bytes().all(|c| c.is_ascii_hexdigit()))
                .then_some(hash)
            })
            .ok_or(Error::Integrity)?;
        let archive =
            download(&self.client, &format!("{base}/{archive_name}"), MAX_ARCHIVE).await?;
        if sha256(&archive) != digest {
            return Err(Error::Integrity);
        }
        let stage = tempfile::Builder::new()
            .prefix("preparing-")
            .tempdir_in(self.storage.installations())
            .map_err(|_| Error::Storage)?;
        extract_node(
            &archive,
            stage.path(),
            &format!("node-{node_version}-{platform}"),
        )?;
        let node_root = stage.path().join(format!("node-{node_version}-{platform}"));
        let node = node_root.join("bin/node");
        let npm = node_root.join("lib/node_modules/npm/bin/npm-cli.js");
        if !node.is_file() || !npm.is_file() {
            return Err(Error::Integrity);
        }
        fs::set_permissions(&node, fs::Permissions::from_mode(0o755))
            .map_err(|_| Error::Storage)?;
        fs::write(stage.path().join("empty.npmrc"), "").map_err(|_| Error::Storage)?;
        fs::write(stage.path().join("global.npmrc"), "").map_err(|_| Error::Storage)?;
        fs::write(stage.path().join("package.json"), r#"{"private":true}"#)
            .map_err(|_| Error::Storage)?;
        let log =
            fs::File::create(stage.path().join("npm-result.json")).map_err(|_| Error::Storage)?;
        let mut command = self.storage.command(&node);
        command
            .current_dir(stage.path())
            .env("PATH", node_root.join("bin"))
            .env("NPM_CONFIG_USERCONFIG", stage.path().join("empty.npmrc"))
            .env("NPM_CONFIG_GLOBALCONFIG", stage.path().join("global.npmrc"))
            .env("NPM_CONFIG_CACHE", stage.path().join("npm-cache"))
            .env("NPM_CONFIG_REGISTRY", REGISTRY)
            .arg(&npm)
            .args([
                "install",
                "--ignore-scripts",
                "--engine-strict",
                "--no-audit",
                "--no-fund",
                "--omit=dev",
                "--json",
            ])
            .arg(format!("{ADAPTER}@^2.5.0"))
            .arg(format!("{CODEX}@^0.165.0"))
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(|_| Error::Process)?;
        let status = match tokio::time::timeout(Duration::from_secs(180), child.wait()).await {
            Ok(status) => status.map_err(|_| Error::Process)?,
            Err(_) => {
                child.kill().await.map_err(|_| Error::RetirementUnknown)?;
                return Err(Error::Transient);
            }
        };
        if !status.success() {
            #[derive(Deserialize)]
            struct NpmFailure {
                error: NpmError,
            }
            #[derive(Deserialize)]
            struct NpmError {
                code: String,
            }
            let path = stage.path().join("npm-result.json");
            if fs::metadata(&path).map_err(|_| Error::Storage)?.len() > 1024 * 1024 {
                return Err(Error::Integrity);
            }
            let failure: NpmFailure =
                serde_json::from_slice(&fs::read(path).map_err(|_| Error::Storage)?)
                    .map_err(|_| Error::Integrity)?;
            return Err(match failure.error.code.as_str() {
                "ETIMEDOUT" | "ECONNRESET" | "ENETUNREACH" | "EAI_AGAIN" => Error::Transient,
                "EBADENGINE" => Error::Incompatible,
                _ => Error::Integrity,
            });
        }
        validate_lock(stage.path())?;
        let adapter = read_package(stage.path(), ADAPTER)?;
        let codex = read_package(stage.path(), CODEX)?;
        if !compatible_adapter(&adapter.version) || !compatible_codex(&codex.version) {
            return Err(Error::Incompatible);
        }
        let adapter_path = stage
            .path()
            .join(format!("node_modules/{ADAPTER}/dist/index.js"));
        let codex_path = stage
            .path()
            .join(format!("node_modules/{CODEX}/bin/codex.js"));
        if !adapter_path.is_file() || !codex_path.is_file() {
            return Err(Error::Integrity);
        }
        // npm supplies executable bits without running package lifecycle scripts.
        if !compatible_codex(&probe_version(&self.storage, &codex_path, &node, true).await?) {
            return Err(Error::Incompatible);
        }
        fs::remove_dir_all(stage.path().join("npm-cache")).map_err(|_| Error::Storage)?;
        let complete = Complete {
            platform: platform.into(),
            node_version,
            node_sha256: digest.into(),
            adapter_version: adapter.version,
            codex_version: codex.version,
            files: file_digests(stage.path())?,
        };
        fs::write(
            stage.path().join("complete.json"),
            serde_json::to_vec(&complete).map_err(|_| Error::Integrity)?,
        )
        .map_err(|_| Error::Storage)?;
        let name = format!(
            "complete-{}",
            chapchoo_crypto::BearerSecret::generate()
                .map_err(|_| Error::Process)?
                .digest()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let target = self.storage.installations().join(&name);
        fs::rename(stage.path(), &target).map_err(|_| Error::Storage)?;
        let latest = self.storage.installations().join("next.json");
        fs::write(
            &latest,
            serde_json::to_vec(&name).map_err(|_| Error::Integrity)?,
        )
        .map_err(|_| Error::Storage)?;
        fs::rename(latest, self.storage.installations().join("latest.json"))
            .map_err(|_| Error::Storage)?;
        self.load(&target, platform)
    }

    fn load(&mut self, root: &Path, platform: &str) -> Result<AppliedSupply, Error> {
        if fs::canonicalize(root).map_err(|_| Error::Integrity)? != root {
            return Err(Error::Integrity);
        }
        let complete: Complete = serde_json::from_slice(
            &fs::read(root.join("complete.json")).map_err(|_| Error::Integrity)?,
        )
        .map_err(|_| Error::Integrity)?;
        if complete.platform != platform
            || !compatible_adapter(&complete.adapter_version)
            || !compatible_codex(&complete.codex_version)
            || Version::parse(complete.node_version.trim_start_matches('v'))
                .map_err(|_| Error::Integrity)?
                .major
                != 24
            || complete.files != file_digests(root)?
            || complete.node_sha256.len() != 64
            || !complete.node_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || read_package(root, ADAPTER)?.version != complete.adapter_version
            || read_package(root, CODEX)?.version != complete.codex_version
        {
            return Err(Error::Integrity);
        }
        validate_lock(root)?;
        for path in [
            root.join(format!(
                "node-{}-{platform}/bin/node",
                complete.node_version
            )),
            root.join(format!("node_modules/{ADAPTER}/dist/index.js")),
            root.join(format!("node_modules/{CODEX}/bin/codex.js")),
        ] {
            if !path.is_file() {
                return Err(Error::Integrity);
            }
        }
        let lease = self
            .leases
            .get(root)
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| Arc::new(root.to_owned()));
        self.leases.insert(root.into(), Arc::downgrade(&lease));
        Ok(AppliedSupply {
            lease,
            node: root.join(format!(
                "node-{}-{platform}/bin/node",
                complete.node_version
            )),
            adapter: root.join(format!("node_modules/{ADAPTER}/dist/index.js")),
            codex: root.join(format!("node_modules/{CODEX}/bin/codex.js")),
            freshness: Freshness::Current,
        })
    }

    fn collect(&mut self, latest: &Path) -> Result<(), Error> {
        for entry in fs::read_dir(self.storage.installations()).map_err(|_| Error::Storage)? {
            let entry = entry.map_err(|_| Error::Storage)?;
            let path = entry.path();
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return Err(Error::Storage);
            };
            if !owned_name(name)
                || path == latest
                || self.leases.get(&path).and_then(Weak::upgrade).is_some()
            {
                continue;
            }
            if entry.file_type().map_err(|_| Error::Storage)?.is_symlink() {
                return Err(Error::Storage);
            }
            fs::remove_dir_all(path).map_err(|_| Error::Storage)?;
        }
        self.leases.retain(|_, lease| lease.strong_count() != 0);
        Ok(())
    }
}
fn owned_name(name: &str) -> bool {
    name.strip_prefix("complete-")
        .is_some_and(|suffix| suffix.len() == 64 && suffix.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn platform() -> Result<&'static str, Error> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("linux-x64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        _ => Err(Error::UnsupportedRuntime),
    }
}
fn compatible_adapter(value: &str) -> bool {
    Version::parse(value)
        .is_ok_and(|v| v.major == 2 && v >= Version::new(2, 5, 0) && v.pre.is_empty())
}
fn compatible_codex(value: &str) -> bool {
    Version::parse(value).is_ok_and(|v| v.major == 0 && v.minor == 165 && v.pre.is_empty())
}
async fn download(client: &reqwest::Client, url: &str, limit: usize) -> Result<Vec<u8>, Error> {
    let mut response = client.get(url).send().await.map_err(acquisition_failure)?;
    if response.status().is_server_error()
        || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err(Error::Transient);
    }
    if !response.status().is_success() {
        return Err(Error::Integrity);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(acquisition_failure)? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err(Error::Integrity);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
fn acquisition_failure(error: reqwest::Error) -> Error {
    use std::error::Error as _;
    let mut source = error.source();
    while let Some(cause) = source {
        if cause.downcast_ref::<rustls::Error>().is_some() {
            return Error::Integrity;
        }
        source = cause.source();
    }
    if error.is_timeout() || error.is_connect() || error.is_body() {
        Error::Transient
    } else {
        Error::Integrity
    }
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn extract_node(bytes: &[u8], directory: &Path, prefix: &str) -> Result<(), Error> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    let mut size = 0u64;
    for entry in archive.entries().map_err(|_| Error::Integrity)? {
        let mut entry = entry.map_err(|_| Error::Integrity)?;
        let path = entry.path().map_err(|_| Error::Integrity)?.into_owned();
        if !path.starts_with(prefix)
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(Error::Integrity);
        }
        let kind = entry.header().entry_type();
        size = size
            .checked_add(entry.size())
            .filter(|s| *s <= 1024 * 1024 * 1024)
            .ok_or(Error::Integrity)?;
        if !(kind.is_file() || kind.is_dir() || kind.is_symlink()) {
            return Err(Error::Integrity);
        }
        if kind.is_symlink() {
            let link = entry
                .link_name()
                .map_err(|_| Error::Integrity)?
                .ok_or(Error::Integrity)?;
            let mut depth = path.components().count().saturating_sub(1);
            for c in link.components() {
                match c {
                    Component::Normal(_) => depth += 1,
                    Component::ParentDir if depth > 1 => depth -= 1,
                    Component::CurDir => {}
                    _ => return Err(Error::Integrity),
                }
            }
        }
        if !entry.unpack_in(directory).map_err(|_| Error::Integrity)? {
            return Err(Error::Integrity);
        }
    }
    Ok(())
}
fn read_package(root: &Path, package: &str) -> Result<Package, Error> {
    serde_json::from_slice(
        &fs::read(root.join(format!("node_modules/{package}/package.json")))
            .map_err(|_| Error::Integrity)?,
    )
    .map_err(|_| Error::Integrity)
}
fn validate_lock(root: &Path) -> Result<(), Error> {
    let lock: PackageLock = serde_json::from_slice(
        &fs::read(root.join("package-lock.json")).map_err(|_| Error::Integrity)?,
    )
    .map_err(|_| Error::Integrity)?;
    if lock.lockfile_version != 3 {
        return Err(Error::Integrity);
    }
    for (name, package) in lock.packages {
        if name.is_empty() {
            continue;
        }
        if package
            .resolved
            .as_deref()
            .is_none_or(|r| !r.starts_with(REGISTRY))
            || package
                .integrity
                .as_deref()
                .is_none_or(|i| !i.starts_with("sha512-"))
        {
            return Err(Error::Integrity);
        }
    }
    Ok(())
}
fn file_digests(root: &Path) -> Result<BTreeMap<PathBuf, String>, Error> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, String>) -> Result<(), Error> {
        for entry in fs::read_dir(path).map_err(|_| Error::Integrity)? {
            let entry = entry.map_err(|_| Error::Integrity)?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|_| Error::Integrity)?
                .to_owned();
            if relative == Path::new("complete.json") {
                continue;
            }
            let kind = entry.file_type().map_err(|_| Error::Integrity)?;
            if kind.is_dir() {
                visit(root, &path, files)?;
            } else if kind.is_file() {
                let mut file = fs::File::open(&path).map_err(|_| Error::Integrity)?;
                let mut hash = Sha256::new();
                let mut chunk = [0; 65536];
                loop {
                    let n = file.read(&mut chunk).map_err(|_| Error::Integrity)?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&chunk[..n]);
                }
                files.insert(relative, format!("{:x}", hash.finalize()));
            } else if kind.is_symlink() {
                let target = fs::canonicalize(&path).map_err(|_| Error::Integrity)?;
                if !target.starts_with(root) {
                    return Err(Error::Integrity);
                }
                files.insert(
                    relative,
                    format!(
                        "link:{}",
                        fs::read_link(path).map_err(|_| Error::Integrity)?.display()
                    ),
                );
            } else {
                return Err(Error::Integrity);
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files)?;
    Ok(files)
}
async fn probe_version(
    storage: &NativeStorage,
    executable: &Path,
    node: &Path,
    script: bool,
) -> Result<String, Error> {
    let log = tempfile::NamedTempFile::new_in(storage.temporary()).map_err(|_| Error::Storage)?;
    let mut command = storage.command(if script { node } else { executable });
    command.env("PATH", super::runtime::explicit_path(node)?);
    if script {
        command.arg(executable);
    }
    command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(log.reopen().map_err(|_| Error::Storage)?)
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| Error::Incompatible)?;
    let status = match tokio::time::timeout(Duration::from_secs(10), child.wait()).await {
        Ok(v) => v.map_err(|_| Error::Process)?,
        Err(_) => {
            child.kill().await.map_err(|_| Error::RetirementUnknown)?;
            return Err(Error::Incompatible);
        }
    };
    if !status.success() || log.as_file().metadata().map_err(|_| Error::Storage)?.len() > 256 {
        return Err(Error::Incompatible);
    }
    let bytes = fs::read(log.path()).map_err(|_| Error::Storage)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::Incompatible)?
        .trim();
    Ok(text
        .strip_prefix("codex-cli ")
        .ok_or(Error::Incompatible)?
        .into())
}
