use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
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

/// The root.lock JSON format.
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
}
