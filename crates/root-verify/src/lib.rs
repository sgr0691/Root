use anyhow::{Context, Result};
use root_lockfile::{get_root_dir, RootLock};
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct BinaryResult {
    pub binary: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct VerificationReport {
    pub package: String,
    pub success: bool,
    pub binaries: Vec<BinaryResult>,
}

/// Verification Strategy override entry
struct VerifyOverride {
    pub args: &'static [&'static str],
    pub expected_contains: Option<&'static str>,
}

/// Get custom verification arguments/assertions for specific tools.
fn get_override(binary: &str) -> Option<VerifyOverride> {
    match binary {
        // Specific tools overrides can go here
        "pdftotext" | "pdfinfo" => Some(VerifyOverride {
            args: &["-h"],
            expected_contains: Some("usage"),
        }),
        _ => None,
    }
}

pub fn verify_package(pkg_name: &str) -> Result<VerificationReport> {
    let root_dir = get_root_dir().context("Could not determine ~/.root path")?;
    let lock_path = root_dir.join("root.lock");
    if !lock_path.exists() {
        return Err(anyhow::anyhow!(
            "root.lock does not exist. Run 'root install' first."
        ));
    }

    let lock = RootLock::read_from_file(&lock_path)?;
    let locked_pkg = lock
        .packages
        .iter()
        .find(|p| p.name == pkg_name)
        .ok_or_else(|| anyhow::anyhow!("Package '{}' is not found in root.lock", pkg_name))?;

    let mut binaries_results = Vec::new();

    for binary in &locked_pkg.binaries {
        let mut bin_res = BinaryResult {
            binary: binary.clone(),
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error_message: None,
        };

        if let Some(over) = get_override(binary) {
            // Run with package-specific override
            match run_binary(binary, over.args) {
                Ok((code, out, err)) => {
                    bin_res.exit_code = Some(code);
                    bin_res.stdout = out.clone();
                    bin_res.stderr = err.clone();

                    let mut check = code == 0;
                    if let Some(expected) = over.expected_contains {
                        if out.to_lowercase().contains(expected)
                            || err.to_lowercase().contains(expected)
                        {
                            check = true;
                        }
                    }
                    bin_res.success = check;
                }
                Err(e) => {
                    bin_res.error_message = Some(e.to_string());
                }
            }
        } else {
            // Generic strategy: try --help, then -h, then --version
            let strategies = vec![vec!["--help"], vec!["-h"], vec!["--version"]];

            let mut last_error = None;
            for args in strategies {
                match run_binary(binary, &args) {
                    Ok((code, out, err)) => {
                        bin_res.exit_code = Some(code);
                        bin_res.stdout = out.clone();
                        bin_res.stderr = err.clone();

                        let lower_out = out.to_lowercase();
                        let lower_err = err.to_lowercase();

                        // Consider successful if exit code is 0, or output matches typical help terms
                        let has_help_keywords = lower_out.contains("usage")
                            || lower_out.contains("options")
                            || lower_out.contains("help")
                            || lower_out.contains("version")
                            || lower_err.contains("usage")
                            || lower_err.contains("options")
                            || lower_err.contains("help")
                            || lower_err.contains("version");

                        if code == 0 || has_help_keywords {
                            bin_res.success = true;
                            break;
                        }
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                    }
                }
            }

            if !bin_res.success {
                if let Some(err) = last_error {
                    bin_res.error_message = Some(err);
                }
            }
        }

        binaries_results.push(bin_res);
    }

    let all_success = !binaries_results.is_empty() && binaries_results.iter().all(|r| r.success);

    Ok(VerificationReport {
        package: pkg_name.to_string(),
        success: all_success,
        binaries: binaries_results,
    })
}

fn run_binary(binary: &str, args: &[&str]) -> Result<(i32, String, String)> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .context(format!("Failed to execute binary '{}'", binary))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    Ok((code, stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use root_lockfile::{self, LockedPackage, NixpkgsConfig};
    use std::fs;
    use std::path::PathBuf;

    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_home(test_name: &str) -> (PathBuf, std::sync::MutexGuard<'static, ()>) {
        let guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_homes")
            .join(test_name);
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        fs::create_dir_all(&temp_dir).unwrap();
        std::env::set_var("HOME", &temp_dir);
        (temp_dir, guard)
    }

    #[test]
    fn test_verify_missing_lockfile() {
        let (_temp_home, _guard) = setup_test_home("test_verify_missing_lockfile");
        let res = verify_package("poppler");
        assert!(res.is_err());
    }

    #[test]
    fn test_verify_missing_package() {
        let (_temp_home, _guard) = setup_test_home("test_verify_missing_package");
        let root_dir = root_lockfile::init_root_dir().unwrap();
        let lock = RootLock {
            version: 1,
            platform: "aarch64-darwin".into(),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![],
        };
        lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        let res = verify_package("poppler");
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("is not found in root.lock"));
    }

    #[test]
    fn test_verify_nonexistent_binary() {
        let (_temp_home, _guard) = setup_test_home("test_verify_nonexistent_binary");
        let root_dir = root_lockfile::init_root_dir().unwrap();
        let lock = RootLock {
            version: 1,
            platform: "aarch64-darwin".into(),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![LockedPackage {
                name: "nonexistent-pkg".to_string(),
                requested: "nonexistent-pkg".to_string(),
                version: "latest".to_string(),
                attribute: "nonexistent-pkg".to_string(),
                store_path: root_lockfile::derive_store_path("nonexistent-pkg", "latest"),
                binaries: vec!["some_garbage_bin_name_123".to_string()],
            }],
        };
        lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        let report = verify_package("nonexistent-pkg").unwrap();
        assert!(!report.success);
        assert_eq!(report.binaries.len(), 1);
        assert!(!report.binaries[0].success);
        assert!(report.binaries[0].error_message.is_some());
    }
}
