use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum NixError {
    #[error("Nix is not installed or not available on PATH")]
    NotInstalled,
    #[error("This package is not available for your Mac architecture.\nTry `root search {0}` to find alternatives.")]
    PlatformMissing(String),
    #[error("Package '{0}' not found in nixpkgs")]
    NotFound(String),
    #[error("Nix command failed: {0}")]
    Generic(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileEntry {
    pub index: u64,
    pub attr_path: Option<String>,
    pub original_url: Option<String>,
    pub installable: Option<String>,
    pub store_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlakeMetadata {
    pub original_url: String,
    pub locked_url: Option<String>,
    pub rev: Option<String>,
    pub nar_hash: Option<String>,
    pub last_modified: Option<u64>,
    pub raw_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageMetadata {
    pub package: String,
    pub installable: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub raw_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildOutputPath {
    pub output_name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivationInfo {
    pub package: String,
    pub installable: String,
    pub derivation_path: PathBuf,
    pub output_paths: Vec<BuildOutputPath>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathInfo {
    pub path: PathBuf,
    pub nar_hash: Option<String>,
    pub nar_size: Option<u64>,
    pub closure_size: Option<u64>,
    pub references: Vec<PathBuf>,
    pub raw_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackageResolution {
    pub package: String,
    pub installable: String,
    pub metadata: PackageMetadata,
    pub derivation: DerivationInfo,
    pub outputs: Vec<BuildOutputPath>,
    pub path_info: Vec<PathInfo>,
}

pub trait NixAdapter {
    fn check_availability(&self) -> Result<bool, NixError>;
    fn search(&self, package: &str) -> Result<String, NixError>;
    fn install(&self, package: &str) -> Result<(), NixError>;
    fn install_installable(&self, package: &str, installable: &str) -> Result<(), NixError>;
    fn list(&self) -> Result<String, NixError>;
    fn remove(&self, package_or_index: &str) -> Result<(), NixError>;
    fn profile_list_json(&self) -> Result<String, NixError>;
    fn flake_metadata(&self, flake_ref: &str) -> Result<FlakeMetadata, NixError>;
    fn eval_package_metadata(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<PackageMetadata, NixError>;
    fn build_output_paths(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<Vec<BuildOutputPath>, NixError>;
    fn derivation_path(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<DerivationInfo, NixError>;
    fn path_info(&self, path_or_installable: &str) -> Result<PathInfo, NixError>;

    fn resolve_locked_package(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<LockedPackageResolution, NixError> {
        let installable = installable_for(package, pinned_installable);
        let metadata = self.eval_package_metadata(package, pinned_installable)?;
        let derivation = self.derivation_path(package, pinned_installable)?;
        let outputs = self.build_output_paths(package, pinned_installable)?;
        let path_info = outputs
            .iter()
            .map(|output| self.path_info(&output.path.to_string_lossy()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LockedPackageResolution {
            package: package.to_string(),
            installable,
            metadata,
            derivation,
            outputs,
            path_info,
        })
    }
}

pub struct RealNixAdapter {
    profile_path: PathBuf,
}

impl Default for RealNixAdapter {
    fn default() -> Self {
        Self::new_default()
    }
}

impl RealNixAdapter {
    pub fn new(profile_path: PathBuf) -> Self {
        Self { profile_path }
    }

    pub fn new_default() -> Self {
        let home = dirs::home_dir().expect("Could not determine home directory");
        Self {
            profile_path: home.join(".root").join("profiles").join("default"),
        }
    }

    fn run_command(
        args: &[&str],
        extra_args: &[&str],
        package_context: Option<&str>,
    ) -> Result<String, NixError> {
        let output = Command::new("nix")
            .arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .args(args)
            .args(extra_args)
            .output()
            .map_err(|_| NixError::NotInstalled)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            Self::normalize_error(&stderr, package_context.unwrap_or("unknown"))
        }
    }

    fn normalize_error(stderr: &str, package_context: &str) -> Result<String, NixError> {
        if stderr.contains("attribute") && stderr.contains("missing from derivation") {
            // E.g. "attribute 'aarch64-darwin' missing from derivation"
            return Err(NixError::PlatformMissing(package_context.to_string()));
        }
        if stderr.contains("error: no outputs found") {
            return Err(NixError::NotFound(package_context.to_string()));
        }
        if stderr.contains("experimental feature") && stderr.contains("is not enabled") {
            return Err(NixError::Generic(
                "Nix experimental features 'nix-command' and 'flakes' are required.\n\
                 To enable them, add this to ~/.config/nix/nix.conf:\n\
                 experimental-features = nix-command flakes\n\n\
                 Or edit /etc/nix/nix.conf to include the same line."
                    .to_string(),
            ));
        }
        if stderr.contains("error: reading symbolic link") || stderr.contains("Invalid argument") {
            return Err(NixError::Generic(
                "Nix profile path issue detected.\n\
                 This can happen when Root's profile path (~/.root/profiles/default)\n\
                 conflicts with Nix's symlink management.\n\n\
                 Run:  root doctor\n\
                 To repair, try:  rm -rf ~/.root/profiles/default && root init"
                    .to_string(),
            ));
        }
        Err(NixError::Generic(stderr.trim().to_string()))
    }

    fn eval_json_attr(
        package: &str,
        installable: &str,
        attr: &str,
    ) -> Result<Option<String>, NixError> {
        let expr = format!("{}.{}", installable, attr);
        match Self::run_command(&["eval", "--json", &expr], &[], Some(package)) {
            Ok(stdout) => Ok(json_string_value(stdout.trim())),
            Err(NixError::Generic(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl NixAdapter for RealNixAdapter {
    fn check_availability(&self) -> Result<bool, NixError> {
        match Self::run_command(&["--version"], &[], None) {
            Ok(_) => Ok(true),
            Err(NixError::NotInstalled) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn search(&self, package: &str) -> Result<String, NixError> {
        Self::run_command(&["search", "nixpkgs", package], &[], Some(package))
    }

    fn install(&self, package: &str) -> Result<(), NixError> {
        let pkg_arg = format!("nixpkgs#{}", package);
        let profile_str = self.profile_path.to_str().unwrap();
        Self::run_command(
            &["profile", "add", &pkg_arg],
            &["--profile", profile_str],
            Some(package),
        )
        .map(|_| ())
    }

    fn install_installable(&self, package: &str, installable: &str) -> Result<(), NixError> {
        let profile_str = self.profile_path.to_str().unwrap();
        Self::run_command(
            &["profile", "add", installable],
            &["--profile", profile_str],
            Some(package),
        )
        .map(|_| ())
    }

    fn list(&self) -> Result<String, NixError> {
        let profile_str = self.profile_path.to_str().unwrap();
        Self::run_command(&["profile", "list"], &["--profile", profile_str], None)
    }

    fn remove(&self, package_or_index: &str) -> Result<(), NixError> {
        let profile_str = self.profile_path.to_str().unwrap();
        Self::run_command(
            &["profile", "remove", package_or_index],
            &["--profile", profile_str],
            Some(package_or_index),
        )
        .map(|_| ())
    }

    fn profile_list_json(&self) -> Result<String, NixError> {
        let profile_str = self.profile_path.to_str().unwrap();
        Self::run_command(
            &["profile", "list", "--json"],
            &["--profile", profile_str],
            None,
        )
    }

    fn flake_metadata(&self, flake_ref: &str) -> Result<FlakeMetadata, NixError> {
        let raw_json = Self::run_command(&["flake", "metadata", "--json", flake_ref], &[], None)?;
        Ok(FlakeMetadata {
            original_url: json_field_string(&raw_json, "originalUrl")
                .or_else(|| json_field_string(&raw_json, "url"))
                .unwrap_or_else(|| flake_ref.to_string()),
            locked_url: json_field_string(&raw_json, "lockedUrl"),
            rev: json_field_string(&raw_json, "rev"),
            nar_hash: json_field_string(&raw_json, "narHash"),
            last_modified: json_field_u64(&raw_json, "lastModified"),
            raw_json,
        })
    }

    fn eval_package_metadata(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<PackageMetadata, NixError> {
        let installable = installable_for(package, pinned_installable);
        let raw_json = Self::run_command(
            &["eval", "--json", &format!("{}.meta", installable)],
            &[],
            Some(package),
        )?;
        Ok(PackageMetadata {
            package: package.to_string(),
            installable: installable.clone(),
            name: Self::eval_json_attr(package, &installable, "name")?,
            version: Self::eval_json_attr(package, &installable, "version")?,
            description: json_field_string(&raw_json, "description"),
            raw_json,
        })
    }

    fn build_output_paths(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<Vec<BuildOutputPath>, NixError> {
        let installable = installable_for(package, pinned_installable);
        let raw_json = Self::run_command(
            &[
                "build",
                "--no-link",
                "--print-out-paths",
                "--json",
                &installable,
            ],
            &[],
            Some(package),
        )?;
        let mut outputs = Vec::new();
        for path in json_store_paths(&raw_json) {
            outputs.push(BuildOutputPath {
                output_name: if outputs.is_empty() {
                    "out".to_string()
                } else {
                    format!("out{}", outputs.len())
                },
                path: PathBuf::from(path),
            });
        }
        Ok(outputs)
    }

    fn derivation_path(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<DerivationInfo, NixError> {
        let installable = installable_for(package, pinned_installable);
        let drv_path = Self::run_command(
            &["eval", "--raw", &format!("{}.drvPath", installable)],
            &[],
            Some(package),
        )?;
        let output_paths = self.build_output_paths(package, pinned_installable)?;
        Ok(DerivationInfo {
            package: package.to_string(),
            installable,
            derivation_path: PathBuf::from(drv_path.trim()),
            output_paths,
        })
    }

    fn path_info(&self, path_or_installable: &str) -> Result<PathInfo, NixError> {
        let raw_json = Self::run_command(
            &["path-info", "--json", "--closure-size", path_or_installable],
            &[],
            Some(path_or_installable),
        )?;
        Ok(PathInfo {
            path: json_store_paths(&raw_json)
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(path_or_installable)),
            nar_hash: json_field_string(&raw_json, "narHash"),
            nar_size: json_field_u64(&raw_json, "narSize"),
            closure_size: json_field_u64(&raw_json, "closureSize"),
            references: json_array_strings(&raw_json, "references")
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            raw_json,
        })
    }
}

pub struct MockNixAdapter {
    pub installed: bool,
    pub installed_packages: std::sync::Mutex<Vec<String>>,
}

impl MockNixAdapter {
    pub fn new(installed: bool) -> Self {
        Self {
            installed,
            installed_packages: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn ensure_available_package(&self, package: &str) -> Result<(), NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        if package == "bad_platform_pkg" {
            return Err(NixError::PlatformMissing(package.to_string()));
        }
        if package == "missing_pkg" {
            return Err(NixError::NotFound(package.to_string()));
        }
        Ok(())
    }
}

impl NixAdapter for MockNixAdapter {
    fn check_availability(&self) -> Result<bool, NixError> {
        if self.installed {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn search(&self, package: &str) -> Result<String, NixError> {
        self.ensure_available_package(package)?;
        Ok(format!("* nixpkgs#{0} (1.0)\n  {0} description", package))
    }

    fn install(&self, package: &str) -> Result<(), NixError> {
        self.ensure_available_package(package)?;
        self.installed_packages
            .lock()
            .unwrap()
            .push(package.to_string());
        Ok(())
    }

    fn install_installable(&self, package: &str, installable: &str) -> Result<(), NixError> {
        self.ensure_available_package(package)?;
        self.installed_packages
            .lock()
            .unwrap()
            .push(installable.to_string());
        Ok(())
    }

    fn list(&self) -> Result<String, NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let pkgs = self.installed_packages.lock().unwrap();
        if pkgs.is_empty() {
            Ok(String::new())
        } else {
            let mut res = String::new();
            for (i, p) in pkgs.iter().enumerate() {
                res.push_str(&format!("Index: {} - {}\n", i, display_installable(p)));
            }
            Ok(res)
        }
    }

    fn remove(&self, package_or_index: &str) -> Result<(), NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let mut pkgs = self.installed_packages.lock().unwrap();
        pkgs.retain(|p| p != package_or_index && package_from_installable(p) != package_or_index);
        Ok(())
    }

    fn profile_list_json(&self) -> Result<String, NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let pkgs = self.installed_packages.lock().unwrap();
        let entries = pkgs
            .iter()
            .enumerate()
            .map(|(index, package)| {
                format!(
                    "{{\"index\":{},\"attrPath\":\"{}\",\"originalUrl\":\"flake:nixpkgs\",\"installable\":\"{}\",\"storePaths\":[\"{}\"]}}",
                    index,
                    json_escape(&package_from_installable(package)),
                    json_escape(package),
                    json_escape(&mock_store_path(package))
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!("[{}]", entries))
    }

    fn flake_metadata(&self, flake_ref: &str) -> Result<FlakeMetadata, NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let locked_url = format!("github:NixOS/nixpkgs/{}", mock_rev_for(flake_ref));
        let raw_json = format!(
            "{{\"originalUrl\":\"{}\",\"lockedUrl\":\"{}\",\"rev\":\"{}\",\"narHash\":\"{}\",\"lastModified\":1700000000}}",
            json_escape(flake_ref),
            json_escape(&locked_url),
            mock_rev_for(flake_ref),
            mock_nar_hash(flake_ref)
        );
        Ok(FlakeMetadata {
            original_url: flake_ref.to_string(),
            locked_url: Some(locked_url),
            rev: Some(mock_rev_for(flake_ref)),
            nar_hash: Some(mock_nar_hash(flake_ref)),
            last_modified: Some(1_700_000_000),
            raw_json,
        })
    }

    fn eval_package_metadata(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<PackageMetadata, NixError> {
        self.ensure_available_package(package)?;
        let installable = installable_for(package, pinned_installable);
        let version = mock_version(package);
        let name = format!("{}-{}", package, version);
        let description = format!("Deterministic mock package metadata for {}", package);
        let raw_json = format!(
            "{{\"description\":\"{}\",\"homepage\":\"https://example.invalid/root/mock/{}\",\"license\":{{\"spdxId\":\"MIT\"}}}}",
            json_escape(&description),
            json_escape(package)
        );
        Ok(PackageMetadata {
            package: package.to_string(),
            installable,
            name: Some(name),
            version: Some(version),
            description: Some(description),
            raw_json,
        })
    }

    fn build_output_paths(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<Vec<BuildOutputPath>, NixError> {
        self.ensure_available_package(package)?;
        let installable = installable_for(package, pinned_installable);
        Ok(vec![BuildOutputPath {
            output_name: "out".to_string(),
            path: PathBuf::from(mock_store_path(&installable)),
        }])
    }

    fn derivation_path(
        &self,
        package: &str,
        pinned_installable: Option<&str>,
    ) -> Result<DerivationInfo, NixError> {
        self.ensure_available_package(package)?;
        let installable = installable_for(package, pinned_installable);
        let output_paths = self.build_output_paths(package, pinned_installable)?;
        Ok(DerivationInfo {
            package: package.to_string(),
            installable: installable.clone(),
            derivation_path: PathBuf::from(format!(
                "/nix/store/{}-{}.drv",
                deterministic_token(&installable),
                sanitize_store_name(package)
            )),
            output_paths,
        })
    }

    fn path_info(&self, path_or_installable: &str) -> Result<PathInfo, NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let path = if path_or_installable.starts_with("/nix/store/") {
            path_or_installable.to_string()
        } else {
            mock_store_path(path_or_installable)
        };
        let nar_hash = mock_nar_hash(&path);
        let raw_json = format!(
            "{{\"path\":\"{}\",\"narHash\":\"{}\",\"narSize\":4096,\"closureSize\":8192,\"references\":[\"{}\"]}}",
            json_escape(&path),
            json_escape(&nar_hash),
            json_escape(&path)
        );
        Ok(PathInfo {
            path: PathBuf::from(&path),
            nar_hash: Some(nar_hash),
            nar_size: Some(4_096),
            closure_size: Some(8_192),
            references: vec![PathBuf::from(&path)],
            raw_json,
        })
    }
}

fn package_from_installable(installable: &str) -> String {
    installable
        .rsplit_once('#')
        .map(|(_, package)| package)
        .unwrap_or(installable)
        .to_string()
}

fn display_installable(value: &str) -> String {
    if value.contains('#') {
        value.to_string()
    } else {
        format!("nixpkgs#{}", value)
    }
}

fn installable_for(package: &str, pinned_installable: Option<&str>) -> String {
    pinned_installable
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("nixpkgs#{}", package))
}

fn json_string_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(unescape_json_string(&trimmed[1..trimmed.len() - 1]))
    } else {
        None
    }
}

fn json_field_string(json: &str, field: &str) -> Option<String> {
    let marker = format!("\"{}\"", field);
    let start = json.find(&marker)? + marker.len();
    let after_colon = json[start..].find(':')? + start + 1;
    let value = json[after_colon..].trim_start();
    if !value.starts_with('"') {
        return None;
    }
    let end = find_json_string_end(value)?;
    Some(unescape_json_string(&value[1..end]))
}

fn json_field_u64(json: &str, field: &str) -> Option<u64> {
    let marker = format!("\"{}\"", field);
    let start = json.find(&marker)? + marker.len();
    let after_colon = json[start..].find(':')? + start + 1;
    let value = json[after_colon..].trim_start();
    let digits = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn json_array_strings(json: &str, field: &str) -> Vec<String> {
    let marker = format!("\"{}\"", field);
    let Some(start) = json.find(&marker).map(|start| start + marker.len()) else {
        return Vec::new();
    };
    let Some(after_colon) = json[start..].find(':').map(|offset| offset + start + 1) else {
        return Vec::new();
    };
    let value = json[after_colon..].trim_start();
    if !value.starts_with('[') {
        return Vec::new();
    }
    let Some(end) = value.find(']') else {
        return Vec::new();
    };
    extract_json_strings(&value[..=end])
}

fn json_store_paths(json: &str) -> Vec<String> {
    extract_json_strings(json)
        .into_iter()
        .filter(|value| value.starts_with("/nix/store/") && !value.ends_with(".drv"))
        .collect()
}

fn extract_json_strings(json: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut rest = json;
    while let Some(start) = rest.find('"') {
        let after_quote = &rest[start + 1..];
        let Some(end) = find_json_string_end(after_quote) else {
            break;
        };
        strings.push(unescape_json_string(&after_quote[..end]));
        rest = &after_quote[end + 1..];
    }
    strings
}

fn find_json_string_end(value: &str) -> Option<usize> {
    let mut escaped = false;
    for (index, ch) in value.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Some(index);
        }
    }
    None
}

fn unescape_json_string(value: &str) -> String {
    value
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\/", "/")
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn mock_version(package: &str) -> String {
    format!("0.1.{}", deterministic_number(package) % 1000)
}

fn mock_store_path(package: &str) -> String {
    format!(
        "/nix/store/{}-{}-{}",
        deterministic_token(package),
        sanitize_store_name(package),
        mock_version(package)
    )
}

fn mock_rev_for(value: &str) -> String {
    format!("{:040x}", deterministic_number(value))
}

fn mock_nar_hash(value: &str) -> String {
    format!("sha256-{:052x}", deterministic_number(value))
}

fn deterministic_token(value: &str) -> String {
    format!("{:032x}", deterministic_number(value))
}

fn deterministic_number(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn sanitize_store_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "package".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_normalization() {
        let err = RealNixAdapter::normalize_error(
            "error: attribute 'aarch64-darwin' missing from derivation",
            "poppler",
        )
        .unwrap_err();
        assert_eq!(err, NixError::PlatformMissing("poppler".to_string()));

        let err2 =
            RealNixAdapter::normalize_error("error: no outputs found", "missing_pkg").unwrap_err();
        assert_eq!(err2, NixError::NotFound("missing_pkg".to_string()));
    }

    #[test]
    fn test_mock_adapter() {
        let mock = MockNixAdapter::new(true);
        assert!(mock.check_availability().unwrap());

        mock.install("poppler").unwrap();
        let list = mock.list().unwrap();
        assert!(list.contains("poppler"));

        let err = mock.install("bad_platform_pkg").unwrap_err();
        assert_eq!(
            err,
            NixError::PlatformMissing("bad_platform_pkg".to_string())
        );

        mock.remove("poppler").unwrap();
        let list2 = mock.list().unwrap();
        assert!(!list2.contains("poppler"));
    }

    #[test]
    fn test_mock_profile_list_json_is_deterministic() {
        let mock = MockNixAdapter::new(true);
        mock.install("ripgrep").unwrap();
        mock.install("fd").unwrap();

        let first = mock.profile_list_json().unwrap();
        let second = mock.profile_list_json().unwrap();

        assert_eq!(first, second);
        assert!(first.contains("\"attrPath\":\"ripgrep\""));
        assert!(first.contains("/nix/store/"));
        assert!(!first.contains("latest"));
        assert!(!first.contains("placeholder"));
    }

    #[test]
    fn test_mock_metadata_methods_are_deterministic() {
        let mock = MockNixAdapter::new(true);
        mock.install("ripgrep").unwrap();

        let metadata = mock
            .eval_package_metadata("ripgrep", Some("github:NixOS/nixpkgs/abcdef#ripgrep"))
            .unwrap();
        assert_eq!(
            metadata.installable,
            "github:NixOS/nixpkgs/abcdef#ripgrep".to_string()
        );
        assert_eq!(metadata.version, Some(mock_version("ripgrep")));
        assert!(!metadata.raw_json.contains("placeholder"));

        let outputs = mock
            .build_output_paths("ripgrep", Some("github:NixOS/nixpkgs/abcdef#ripgrep"))
            .unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].output_name, "out");
        assert!(outputs[0].path.starts_with("/nix/store"));

        let derivation = mock
            .derivation_path("ripgrep", Some("github:NixOS/nixpkgs/abcdef#ripgrep"))
            .unwrap();
        assert_eq!(derivation.output_paths, outputs);
        assert!(derivation
            .derivation_path
            .to_string_lossy()
            .ends_with("ripgrep.drv"));

        let path_info = mock.path_info(outputs[0].path.to_str().unwrap()).unwrap();
        assert_eq!(path_info.path, outputs[0].path);
        assert_eq!(path_info.references, vec![outputs[0].path.clone()]);
    }

    #[test]
    fn test_mock_resolve_locked_package() {
        let mock = MockNixAdapter::new(true);
        let resolution = mock
            .resolve_locked_package("fd", Some("github:NixOS/nixpkgs/abcdef#fd"))
            .unwrap();

        assert_eq!(resolution.package, "fd");
        assert_eq!(resolution.outputs.len(), 1);
        assert_eq!(resolution.path_info.len(), 1);
        assert_eq!(resolution.outputs, resolution.derivation.output_paths);
        assert_eq!(resolution.path_info[0].path, resolution.outputs[0].path);
    }

    #[test]
    fn test_mock_metadata_special_package_errors() {
        let mock = MockNixAdapter::new(true);

        let err = mock.eval_package_metadata("missing_pkg", None).unwrap_err();
        assert_eq!(err, NixError::NotFound("missing_pkg".to_string()));

        let err = mock
            .build_output_paths("bad_platform_pkg", None)
            .unwrap_err();
        assert_eq!(
            err,
            NixError::PlatformMissing("bad_platform_pkg".to_string())
        );
    }

    #[test]
    fn test_json_helpers() {
        let json = r#"{"path":"/nix/store/abc-ripgrep","narHash":"sha256-abcd","narSize":123,"references":["/nix/store/ref-one","/nix/store/ref-two"]}"#;

        assert_eq!(
            json_field_string(json, "narHash"),
            Some("sha256-abcd".to_string())
        );
        assert_eq!(json_field_u64(json, "narSize"), Some(123));
        assert_eq!(
            json_array_strings(json, "references"),
            vec![
                "/nix/store/ref-one".to_string(),
                "/nix/store/ref-two".to_string()
            ]
        );
        assert_eq!(
            json_store_paths(json),
            vec![
                "/nix/store/abc-ripgrep".to_string(),
                "/nix/store/ref-one".to_string(),
                "/nix/store/ref-two".to_string()
            ]
        );
    }

    #[test]
    fn test_json_store_paths_filters_drv_paths() {
        let json = r#"[{"drvPath":"/nix/store/abc-ffmpeg-8.1.drv","outputs":{"out":"/nix/store/xyz-ffmpeg-8.1"}}]"#;

        let paths = json_store_paths(json);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/nix/store/xyz-ffmpeg-8.1");
        assert!(!paths[0].ends_with(".drv"));
    }

    #[test]
    fn test_json_store_paths_multiple_outputs_with_drv() {
        let json = r#"[{"drvPath":"/nix/store/abc-pkg.drv","outputs":{"out":"/nix/store/out-pkg","dev":"/nix/store/dev-pkg"}}]"#;

        let paths = json_store_paths(json);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "/nix/store/out-pkg");
        assert_eq!(paths[1], "/nix/store/dev-pkg");
        for p in &paths {
            assert!(!p.ends_with(".drv"));
        }
    }
}
