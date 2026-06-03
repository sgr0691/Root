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

pub trait NixAdapter {
    fn check_availability(&self) -> Result<bool, NixError>;
    fn search(&self, package: &str) -> Result<String, NixError>;
    fn install(&self, package: &str) -> Result<(), NixError>;
    fn list(&self) -> Result<String, NixError>;
    fn remove(&self, package_or_index: &str) -> Result<(), NixError>;
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
        Err(NixError::Generic(stderr.trim().to_string()))
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
            &["profile", "install", &pkg_arg],
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
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        if package == "missing_pkg" {
            Err(NixError::NotFound(package.to_string()))
        } else if package == "bad_platform_pkg" {
            Err(NixError::PlatformMissing(package.to_string()))
        } else {
            Ok(format!("* nixpkgs#{0} (1.0)\n  {0} description", package))
        }
    }

    fn install(&self, package: &str) -> Result<(), NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        if package == "bad_platform_pkg" {
            return Err(NixError::PlatformMissing(package.to_string()));
        }
        if package == "missing_pkg" {
            return Err(NixError::NotFound(package.to_string()));
        }
        self.installed_packages
            .lock()
            .unwrap()
            .push(package.to_string());
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
                res.push_str(&format!("Index: {} - nixpkgs#{}\n", i, p));
            }
            Ok(res)
        }
    }

    fn remove(&self, package_or_index: &str) -> Result<(), NixError> {
        if !self.installed {
            return Err(NixError::NotInstalled);
        }
        let mut pkgs = self.installed_packages.lock().unwrap();
        pkgs.retain(|p| p != package_or_index);
        Ok(())
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
}
