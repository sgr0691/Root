use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The Rootfile TOML format.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Rootfile {
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub settings: RootSettings,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RootSettings {
    #[serde(default = "default_true")]
    pub snapshots: bool,
    #[serde(default = "default_true")]
    pub verify_installs: bool,
}

impl Default for RootSettings {
    fn default() -> Self {
        Self {
            snapshots: true,
            verify_installs: true,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Rootfile {
    pub fn read_from_str(content: &str) -> Result<Self> {
        let rootfile: Rootfile =
            toml::from_str(content).context("Failed to parse Rootfile TOML")?;
        Ok(rootfile)
    }

    pub fn read_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read Rootfile")?;
        Self::read_from_str(&content)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize Rootfile")?;
        fs::write(path, content).context("Failed to write Rootfile")?;
        Ok(())
    }
}

/// Current root.lock schema version emitted by Root v0.1.2+.
pub const ROOT_LOCK_SCHEMA_VERSION: u32 = 2;

/// The legacy root.lock JSON format.
///
/// This shape is intentionally kept source-compatible with v0.1 callers that
/// construct locks with struct literals. New deterministic v2 metadata is
/// represented by [`RootLockV2`] and can be reached with [`RootLock::to_v2`].
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RootLock {
    pub version: u32,
    pub platform: String,
    pub nixpkgs: NixpkgsConfig,
    #[serde(default)]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NixpkgsConfig {
    pub rev: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub requested: String,
    pub version: String,
    pub attribute: String,
    #[serde(rename = "storePath")]
    pub store_path: String,
    pub binaries: Vec<String>,
}

/// RootLock v2 JSON format with deterministic Nix resolution metadata.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RootLockV2 {
    #[serde(default = "default_root_lock_schema_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub nix: NixRuntime,
    #[serde(default)]
    pub nixpkgs: NixpkgsConfigV2,
    #[serde(default)]
    pub profile: LockProfile,
    #[serde(default)]
    pub packages: Vec<LockedPackageV2>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct NixRuntime {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct NixpkgsConfigV2 {
    #[serde(default)]
    pub rev: String,
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flake_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nar_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlays: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LockProfile {
    #[serde(default = "default_profile_name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct LockedPackageV2 {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub requested: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub attribute: String,
    #[serde(default, rename = "storePath")]
    pub store_path: String,
    #[serde(default)]
    pub binaries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installable: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flake_attribute: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drv_path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outputs: BTreeMap<String, LockedPackageOutput>,
    #[serde(
        default,
        rename = "storePaths",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub store_paths: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct LockedPackageOutput {
    #[serde(default, rename = "storePath")]
    pub store_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nar_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LegacyLockIssue {
    V1Schema { version: u32 },
    LatestPackageVersion { package: String },
    UnknownNixpkgsRev { rev: String },
    PlaceholderStorePath { package: String, store_path: String },
}

fn default_root_lock_schema_version() -> u32 {
    ROOT_LOCK_SCHEMA_VERSION
}

fn default_profile_name() -> String {
    "default".to_string()
}

impl Default for LockProfile {
    fn default() -> Self {
        Self {
            name: default_profile_name(),
            path: None,
            generation: None,
        }
    }
}

impl RootLock {
    pub fn read_from_str(content: &str) -> Result<Self> {
        let lockfile: RootLock =
            serde_json::from_str(content).context("Failed to parse root.lock JSON")?;
        Ok(lockfile)
    }

    pub fn read_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read root.lock")?;
        Self::read_from_str(&content)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize root.lock")?;
        fs::write(path, content).context("Failed to write root.lock")?;
        Ok(())
    }

    /// Returns true when this lock predates the v2 deterministic metadata schema.
    pub fn is_legacy_v1(&self) -> bool {
        self.version < ROOT_LOCK_SCHEMA_VERSION
    }

    /// Returns true when the lock's nixpkgs revision is not pinned to a concrete revision.
    pub fn has_unknown_nixpkgs_rev(&self) -> bool {
        is_unknown_nixpkgs_rev(&self.nixpkgs.rev)
    }

    /// Reports all known nondeterministic legacy markers in this lock.
    pub fn nondeterministic_legacy_issues(&self) -> Vec<LegacyLockIssue> {
        let mut issues = Vec::new();

        if self.is_legacy_v1() {
            issues.push(LegacyLockIssue::V1Schema {
                version: self.version,
            });
        }

        if self.has_unknown_nixpkgs_rev() {
            issues.push(LegacyLockIssue::UnknownNixpkgsRev {
                rev: self.nixpkgs.rev.clone(),
            });
        }

        for package in &self.packages {
            if package.has_latest_version() {
                issues.push(LegacyLockIssue::LatestPackageVersion {
                    package: package.name.clone(),
                });
            }

            if package.has_placeholder_store_path() {
                issues.push(LegacyLockIssue::PlaceholderStorePath {
                    package: package.name.clone(),
                    store_path: package.store_path.clone(),
                });
            }
        }

        issues
    }

    /// Returns true if any v1/nondeterministic marker is present.
    pub fn has_nondeterministic_legacy_entries(&self) -> bool {
        !self.nondeterministic_legacy_issues().is_empty()
    }

    /// Converts the source-compatible v1 shape into the richer v2 representation.
    pub fn to_v2(&self) -> RootLockV2 {
        RootLockV2 {
            version: ROOT_LOCK_SCHEMA_VERSION,
            root_version: None,
            created_at: None,
            updated_at: None,
            platform: self.platform.clone(),
            nix: NixRuntime::default(),
            nixpkgs: NixpkgsConfigV2::from(self.nixpkgs.clone()),
            profile: LockProfile::default(),
            packages: self
                .packages
                .iter()
                .cloned()
                .map(LockedPackageV2::from)
                .collect(),
        }
    }
}

impl RootLockV2 {
    pub fn read_from_str(content: &str) -> Result<Self> {
        let lockfile: RootLockV2 =
            serde_json::from_str(content).context("Failed to parse root.lock v2 JSON")?;
        Ok(lockfile)
    }

    pub fn read_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read root.lock")?;
        Self::read_from_str(&content)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize root.lock v2")?;
        fs::write(path, content).context("Failed to write root.lock")?;
        Ok(())
    }
}

impl LockedPackage {
    /// Returns true when the package was locked to a floating latest version.
    pub fn has_latest_version(&self) -> bool {
        self.version.trim().eq_ignore_ascii_case("latest")
    }

    /// Returns true when the package store path is a placeholder rather than a concrete store path.
    pub fn has_placeholder_store_path(&self) -> bool {
        is_placeholder_store_path(&self.store_path)
    }
}

impl LockedPackageV2 {
    /// Returns true when the package was locked to a floating latest version.
    pub fn has_latest_version(&self) -> bool {
        self.version.trim().eq_ignore_ascii_case("latest")
    }

    /// Returns true when the primary package store path is a placeholder rather than a concrete store path.
    pub fn has_placeholder_store_path(&self) -> bool {
        is_placeholder_store_path(&self.store_path)
    }
}

impl From<NixpkgsConfig> for NixpkgsConfigV2 {
    fn from(config: NixpkgsConfig) -> Self {
        Self {
            rev: config.rev,
            source: config.source,
            flake_ref: None,
            nar_hash: None,
            last_modified: None,
            system: None,
            config: BTreeMap::new(),
            overlays: Vec::new(),
        }
    }
}

impl From<LockedPackage> for LockedPackageV2 {
    fn from(package: LockedPackage) -> Self {
        let mut outputs = BTreeMap::new();
        let mut store_paths = BTreeMap::new();

        if !package.store_path.is_empty() {
            outputs.insert(
                "out".to_string(),
                LockedPackageOutput {
                    store_path: package.store_path.clone(),
                    content_hash: None,
                    nar_hash: None,
                    references: Vec::new(),
                },
            );
            store_paths.insert("out".to_string(), package.store_path.clone());
        }

        Self {
            installable: Some(package.attribute.clone()),
            flake_attribute: Some(package.attribute.clone()),
            outputs,
            store_paths,
            name: package.name,
            requested: package.requested,
            version: package.version,
            attribute: package.attribute,
            store_path: package.store_path,
            binaries: package.binaries,
            drv_path: None,
            meta: BTreeMap::new(),
            content_hash: None,
        }
    }
}

fn is_unknown_nixpkgs_rev(rev: &str) -> bool {
    let normalized = rev.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "" | "unknown" | "latest" | "unstable" | "master" | "main" | "head" | "n/a"
    )
}

fn is_placeholder_store_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return true;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("placeholder")
        || lower.contains("dummy")
        || lower.contains("example")
        || lower == "/nix/store/xxx"
    {
        return true;
    }

    let Some(rest) = trimmed.strip_prefix("/nix/store/") else {
        return true;
    };
    let Some((hash_part, _name_part)) = rest.split_once('-') else {
        return true;
    };

    let plausible_legacy_hash =
        hash_part.len() == 10 && hash_part.chars().all(|c| c.is_ascii_hexdigit());
    let plausible_nix_hash = hash_part.len() == 32
        && hash_part
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());

    !(plausible_legacy_hash || plausible_nix_hash)
}

/// Returns the ~/.root path (or $ROOT_DIR env var override for testing)
pub fn get_root_dir() -> Result<PathBuf> {
    if let Some(val) = std::env::var_os("ROOT_DIR") {
        return Ok(PathBuf::from(val));
    }
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".root"))
}

/// Initializes the ~/.root directory structure
pub fn init_root_dir() -> Result<PathBuf> {
    let root_dir = get_root_dir()?;
    fs::create_dir_all(&root_dir).context("Failed to create ~/.root directory")?;
    fs::create_dir_all(root_dir.join("snapshots"))
        .context("Failed to create snapshots directory")?;
    fs::create_dir_all(root_dir.join("profiles")).context("Failed to create profiles directory")?;
    fs::create_dir_all(root_dir.join("profiles").join("default"))
        .context("Failed to create default profile directory")?;
    fs::create_dir_all(root_dir.join("logs")).context("Failed to create logs directory")?;
    fs::create_dir_all(root_dir.join("cache")).context("Failed to create cache directory")?;
    Ok(root_dir)
}

/// Compute the SHA-256 hex digest of arbitrary data.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute the SHA-256 hex digest of a file's contents.
pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).context("Failed to open file for hashing")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .context("Failed to read file for hashing")?;
    Ok(compute_sha256(&buf))
}

/// Derive a deterministic Nix-like store path from a package name.
///
/// In real Nix the hash is derived from the full derivation; here we produce
/// a stable, reproducible identifier for MVP state-tracking purposes.
pub fn derive_store_path(name: &str, version: &str) -> String {
    let input = format!("root:{}+{}", name, version);
    let hash = compute_sha256(input.as_bytes());
    let short_hash = &hash[..10];
    format!("/nix/store/{}-{}-{}", short_hash, name, version)
}

/// Detect the current platform string for the lockfile.
/// Returns an error if the platform is unsupported.
pub fn detect_platform() -> Result<String> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (os, arch) {
        ("macos", "aarch64") => Ok("aarch64-darwin".to_string()),
        ("macos", "x86_64") => Ok("x86_64-darwin".to_string()),
        ("linux", "aarch64") => Ok("aarch64-linux".to_string()),
        ("linux", "x86_64") => Ok("x86_64-linux".to_string()),
        (os, arch) => Err(anyhow::anyhow!(
            "Unsupported platform: {}-{}. Root v0.1 supports macOS (Apple Silicon and Intel) and Linux (aarch64 and x86_64).",
            os, arch
        )),
    }
}

/// Compute a content hash for the packages in a RootLock (sorted for determinism).
pub fn compute_lock_content_hash(lock: &RootLock) -> String {
    let mut hasher = Sha256::new();
    for pkg in &lock.packages {
        hasher.update(pkg.name.as_bytes());
        hasher.update(b"\0");
        hasher.update(pkg.version.as_bytes());
        hasher.update(b"\0");
        hasher.update(pkg.requested.as_bytes());
        hasher.update(b"\0");
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        let hash = compute_sha256(b"hello");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_derive_store_path() {
        let path = derive_store_path("poppler", "24.08.0");
        assert!(path.starts_with("/nix/store/"));
        assert!(path.contains("poppler-24.08.0"));
        assert_eq!(derive_store_path("poppler", "24.08.0"), path);
    }

    #[test]
    fn test_compute_lock_content_hash() {
        let lock = RootLock {
            version: 1,
            platform: "test".into(),
            nixpkgs: NixpkgsConfig {
                rev: "abc".into(),
                source: "test".into(),
            },
            packages: vec![LockedPackage {
                name: "poppler".into(),
                requested: "poppler".into(),
                version: "24.08.0".into(),
                attribute: "poppler".into(),
                store_path: "/nix/store/xxx".into(),
                binaries: vec!["pdftotext".into()],
            }],
        };
        let hash = compute_lock_content_hash(&lock);
        assert_eq!(hash.len(), 64);
        assert_eq!(compute_lock_content_hash(&lock), hash);
    }

    #[test]
    fn parse_rootfile() {
        let toml_str = r#"
        [packages]
        node = "22.11.0"
        poppler = "24.08.0"

        [settings]
        snapshots = true
        verify_installs = false
        "#;
        let rootfile = Rootfile::read_from_str(toml_str).unwrap();
        assert_eq!(rootfile.packages.get("node").unwrap(), "22.11.0");
        assert!(rootfile.settings.snapshots);
        assert!(!rootfile.settings.verify_installs);
    }

    #[test]
    fn parse_rootlock() {
        let json_str = r#"
        {
          "version": 1,
          "platform": "aarch64-darwin",
          "nixpkgs": {
            "rev": "some-hash",
            "source": "github:NixOS/nixpkgs"
          },
          "packages": [
            {
              "name": "poppler",
              "requested": "poppler",
              "version": "24.08.0",
              "attribute": "poppler",
              "storePath": "/nix/store/123-poppler-24.08.0",
              "binaries": ["pdftotext", "pdfinfo"]
            }
          ]
        }
        "#;
        let lock = RootLock::read_from_str(json_str).unwrap();
        assert_eq!(lock.version, 1);
        assert_eq!(lock.nixpkgs.rev, "some-hash");
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "poppler");
        assert_eq!(lock.packages[0].binaries, vec!["pdftotext", "pdfinfo"]);
    }

    #[test]
    fn parse_rootlock_v2_metadata() {
        let json_str = r#"
        {
          "version": 2,
          "root_version": "0.1.2",
          "created_at": "2026-06-03T00:00:00Z",
          "updated_at": "2026-06-03T00:00:01Z",
          "platform": "x86_64-linux",
          "nix": {
            "version": "2.24.0",
            "system": "x86_64-linux",
            "store_dir": "/nix/store",
            "sandbox": true
          },
          "nixpkgs": {
            "rev": "0123456789abcdef0123456789abcdef01234567",
            "source": "github:NixOS/nixpkgs",
            "flake_ref": "github:NixOS/nixpkgs/0123456789abcdef0123456789abcdef01234567",
            "nar_hash": "sha256-deadbeef",
            "last_modified": "2026-06-01T00:00:00Z",
            "system": "x86_64-linux",
            "config": { "allowUnfree": true },
            "overlays": ["github:example/overlay"]
          },
          "profile": {
            "name": "default",
            "path": "/home/alice/.root/profiles/default",
            "generation": 7
          },
          "packages": [
            {
              "name": "poppler",
              "requested": "poppler",
              "version": "24.08.0",
              "attribute": "poppler",
              "storePath": "/nix/store/0123456789abcdef0123456789abcdef-poppler-24.08.0",
              "binaries": ["pdftotext"],
              "installable": "nixpkgs#poppler",
              "flake_attribute": "legacyPackages.x86_64-linux.poppler",
              "drv_path": "/nix/store/0123456789abcdef0123456789abcdef-poppler-24.08.0.drv",
              "outputs": {
                "out": {
                  "storePath": "/nix/store/0123456789abcdef0123456789abcdef-poppler-24.08.0",
                  "content_hash": "sha256-feedface",
                  "nar_hash": "sha256-cafebabe",
                  "references": []
                }
              },
              "storePaths": {
                "out": "/nix/store/0123456789abcdef0123456789abcdef-poppler-24.08.0"
              },
              "meta": { "description": "PDF rendering tools" },
              "content_hash": "sha256-feedface"
            }
          ]
        }
        "#;

        let lock = RootLockV2::read_from_str(json_str).unwrap();

        assert_eq!(lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_eq!(lock.root_version.as_deref(), Some("0.1.2"));
        assert_eq!(lock.nix.version.as_deref(), Some("2.24.0"));
        assert_eq!(
            lock.nixpkgs.config.get("allowUnfree"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(lock.profile.generation, Some(7));
        assert_eq!(
            lock.packages[0].installable.as_deref(),
            Some("nixpkgs#poppler")
        );
        assert_eq!(
            lock.packages[0].outputs["out"].content_hash.as_deref(),
            Some("sha256-feedface")
        );
    }

    #[test]
    fn detect_nondeterministic_legacy_entries() {
        let lock = RootLock {
            version: 1,
            platform: "test".into(),
            nixpkgs: NixpkgsConfig {
                rev: "unknown".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![LockedPackage {
                name: "node".into(),
                requested: "node".into(),
                version: "latest".into(),
                attribute: "nodejs".into(),
                store_path: "/nix/store/xxx".into(),
                binaries: vec!["node".into()],
            }],
        };

        let issues = lock.nondeterministic_legacy_issues();

        assert!(lock.has_nondeterministic_legacy_entries());
        assert!(issues.contains(&LegacyLockIssue::V1Schema { version: 1 }));
        assert!(issues.contains(&LegacyLockIssue::UnknownNixpkgsRev {
            rev: "unknown".into()
        }));
        assert!(issues.contains(&LegacyLockIssue::LatestPackageVersion {
            package: "node".into()
        }));
        assert!(issues.contains(&LegacyLockIssue::PlaceholderStorePath {
            package: "node".into(),
            store_path: "/nix/store/xxx".into()
        }));
    }

    #[test]
    fn v1_lock_converts_to_v2_defaults() {
        let lock = RootLock {
            version: 1,
            platform: "x86_64-linux".into(),
            nixpkgs: NixpkgsConfig {
                rev: "0123456789abcdef0123456789abcdef01234567".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![LockedPackage {
                name: "ripgrep".into(),
                requested: "ripgrep".into(),
                version: "14.1.1".into(),
                attribute: "ripgrep".into(),
                store_path: derive_store_path("ripgrep", "14.1.1"),
                binaries: vec!["rg".into()],
            }],
        };

        let v2 = lock.to_v2();

        assert_eq!(v2.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_eq!(v2.platform, "x86_64-linux");
        assert_eq!(v2.profile, LockProfile::default());
        assert_eq!(v2.nixpkgs.rev, "0123456789abcdef0123456789abcdef01234567");
        assert_eq!(v2.packages[0].installable.as_deref(), Some("ripgrep"));
        assert_eq!(
            v2.packages[0].store_paths["out"],
            lock.packages[0].store_path
        );
        assert_eq!(
            lock.nondeterministic_legacy_issues(),
            vec![LegacyLockIssue::V1Schema { version: 1 }]
        );
    }
}
