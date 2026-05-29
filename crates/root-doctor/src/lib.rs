use anyhow::Result;
use root_lockfile::{get_root_dir, RootLock, Rootfile};
use root_nix::NixAdapter;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum IssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct DoctorIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct DoctorReport {
    pub healthy: bool,
    pub nix_installed: bool,
    pub root_initialized: bool,
    pub issues: Vec<DoctorIssue>,
}

pub fn run_diagnostics(adapter: &impl NixAdapter) -> Result<DoctorReport> {
    let mut report = DoctorReport {
        healthy: true,
        nix_installed: false,
        root_initialized: false,
        issues: Vec::new(),
    };

    // 1. Check Nix Status
    match adapter.check_availability() {
        Ok(available) => {
            report.nix_installed = available;
            if !available {
                report.issues.push(DoctorIssue {
                    severity: IssueSeverity::Error,
                    category: "Nix".to_string(),
                    description: "Nix package manager is not installed or not in PATH.".to_string(),
                    suggestion:
                        "Install Nix by running: curl -L https://nixos.org/nix/install | sh"
                            .to_string(),
                });
            }
        }
        Err(e) => {
            report.issues.push(DoctorIssue {
                severity: IssueSeverity::Error,
                category: "Nix".to_string(),
                description: format!("Failed to check Nix availability: {}", e),
                suggestion: "Ensure Nix is correctly configured on your system.".to_string(),
            });
        }
    }

    // 2. Check Root Directory Status
    let root_dir_res = get_root_dir();
    let mut has_root_dir = false;
    if let Ok(ref root_dir) = root_dir_res {
        if root_dir.exists() && root_dir.is_dir() {
            has_root_dir = true;
            report.root_initialized = true;

            // Check subdirectories
            let subdirs = ["snapshots", "profiles", "logs", "cache"];
            for sub in &subdirs {
                let sub_path = root_dir.join(sub);
                if !sub_path.exists() || !sub_path.is_dir() {
                    report.issues.push(DoctorIssue {
                        severity: IssueSeverity::Warning,
                        category: "Repository".to_string(),
                        description: format!("Subdirectory ~/.root/{} is missing.", sub),
                        suggestion: "Run `root init` to recreate missing directories safely."
                            .to_string(),
                    });
                }
            }
        } else {
            report.issues.push(DoctorIssue {
                severity: IssueSeverity::Error,
                category: "Repository".to_string(),
                description: "Root app directory (~/.root) does not exist.".to_string(),
                suggestion: "Run `root init` to initialize the Root environment.".to_string(),
            });
        }
    } else {
        report.issues.push(DoctorIssue {
            severity: IssueSeverity::Error,
            category: "Repository".to_string(),
            description: "Unable to determine the Root directory (~/.root).".to_string(),
            suggestion: "Ensure your HOME environment variable is set correctly.".to_string(),
        });
    }

    // 3. Lockfile and Configuration Status
    let mut rootfile_opt = None;
    let mut lockfile_opt = None;

    if has_root_dir {
        let root_dir = root_dir_res.unwrap();
        let rootfile_path = root_dir.join("Rootfile");
        let lock_path = root_dir.join("root.lock");

        // Read Rootfile
        if rootfile_path.exists() {
            match Rootfile::read_from_file(&rootfile_path) {
                Ok(rf) => rootfile_opt = Some(rf),
                Err(e) => {
                    report.issues.push(DoctorIssue {
                        severity: IssueSeverity::Error,
                        category: "Config".to_string(),
                        description: format!("Rootfile is corrupted or unparseable: {}", e),
                        suggestion:
                            "Fix the syntax errors in ~/.root/Rootfile or delete and recreate it."
                                .to_string(),
                    });
                }
            }
        } else {
            report.issues.push(DoctorIssue {
                severity: IssueSeverity::Warning,
                category: "Config".to_string(),
                description: "Rootfile (~/.root/Rootfile) is missing.".to_string(),
                suggestion:
                    "Create ~/.root/Rootfile or run `root install <package>` to create one."
                        .to_string(),
            });
        }

        // Read Lockfile
        if lock_path.exists() {
            match RootLock::read_from_file(&lock_path) {
                Ok(lock) => lockfile_opt = Some(lock),
                Err(e) => {
                    report.issues.push(DoctorIssue {
                        severity: IssueSeverity::Error,
                        category: "Config".to_string(),
                        description: format!("root.lock is corrupted or unparseable: {}", e),
                        suggestion:
                            "Run `root init` or reinstall packages to rebuild the lockfile."
                                .to_string(),
                    });
                }
            }
        }
    }

    // 4. Reconcile Configuration and Nix profile (Drift Detection)
    if report.nix_installed && report.root_initialized {
        if let (Some(ref rootfile), Some(ref lockfile)) = (&rootfile_opt, &lockfile_opt) {
            // Check for Rootfile packages missing from lockfile
            for pkg_name in rootfile.packages.keys() {
                if !lockfile.packages.iter().any(|p| p.name == *pkg_name) {
                    report.issues.push(DoctorIssue {
                        severity: IssueSeverity::Warning,
                        category: "Drift".to_string(),
                        description: format!(
                            "Package '{}' in Rootfile is not locked in root.lock.",
                            pkg_name
                        ),
                        suggestion: format!(
                            "Run `root install {}` to lock and install the package.",
                            pkg_name
                        ),
                    });
                }
            }

            // Fetch actual Nix profile packages
            match adapter.list() {
                Ok(nix_list) => {
                    // Compare expected locked packages to actual Nix profile packages
                    for locked_pkg in &lockfile.packages {
                        // Normally `nix profile list` returns containing `nixpkgs#<attr>` or similar.
                        // Let's check if the list stdout contains either the name, the attribute, or nixpkgs#attribute.
                        let is_installed = nix_list
                            .contains(&format!("nixpkgs#{}", locked_pkg.attribute))
                            || nix_list.contains(&locked_pkg.attribute)
                            || nix_list.contains(&locked_pkg.name);

                        if !is_installed {
                            report.issues.push(DoctorIssue {
                                severity: IssueSeverity::Error,
                                category: "Drift".to_string(),
                                description: format!(
                                    "Locked package '{}' is missing from the active Nix profile.",
                                    locked_pkg.name
                                ),
                                suggestion: format!(
                                    "Run `root install {}` to reinstall and repair.",
                                    locked_pkg.name
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    report.issues.push(DoctorIssue {
                        severity: IssueSeverity::Warning,
                        category: "Nix".to_string(),
                        description: format!("Failed to read Nix profile list: {}", e),
                        suggestion: "Ensure nix profile operations are functioning normally."
                            .to_string(),
                    });
                }
            }
        }
    }

    // 5. Check PATH & Shadows
    if let Some(ref lockfile) = lockfile_opt {
        // Collect all Nix-profile bin dirs or default Nix bin path
        let mut nix_in_path = false;
        if let Some(paths) = env::var_os("PATH") {
            for path in env::split_paths(&paths) {
                let path_str = path.to_string_lossy();
                if path_str.contains(".nix-profile") || path_str.contains("nix") {
                    nix_in_path = true;
                    break;
                }
            }
        }

        if !nix_in_path {
            report.issues.push(DoctorIssue {
                severity: IssueSeverity::Warning,
                category: "Environment".to_string(),
                description: "Nix profile binary path (~/.nix-profile/bin) is not found in your PATH environment variable.".to_string(),
                suggestion: "Add '~/.nix-profile/bin' to your PATH in your shell configuration (.zshrc, .bashrc, or .bash_profile).".to_string(),
            });
        }

        for locked_pkg in &lockfile.packages {
            for binary in &locked_pkg.binaries {
                match find_on_path(binary) {
                    Some(bin_path) => {
                        let path_str = bin_path.to_string_lossy();
                        // Check if it's from Nix or shadowed
                        let is_nix = path_str.contains(".nix-profile")
                            || path_str.contains("/nix/var/nix/profiles")
                            || path_str.contains("/nix/store")
                            || path_str.contains("/nix/profile");

                        if !is_nix {
                            report.issues.push(DoctorIssue {
                                severity: IssueSeverity::Warning,
                                category: "Conflict".to_string(),
                                description: format!(
                                    "Binary '{}' from package '{}' is shadowed by another installation at '{}'.",
                                    binary, locked_pkg.name, path_str
                                ),
                                suggestion: format!(
                                    "Uninstall the conflicting package from Brew/system, or re-order your PATH so '~/.nix-profile/bin' precedes '{}'.",
                                    bin_path.parent().unwrap_or(Path::new("")).display()
                                ),
                            });
                        }
                    }
                    None => {
                        report.issues.push(DoctorIssue {
                            severity: IssueSeverity::Warning,
                            category: "Environment".to_string(),
                            description: format!("Binary '{}' from package '{}' is not accessible in your current PATH.", binary, locked_pkg.name),
                            suggestion: "Ensure '~/.nix-profile/bin' is included in your PATH and active.".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Determine healthy status: no errors or warnings
    report.healthy = report
        .issues
        .iter()
        .all(|issue| issue.severity != IssueSeverity::Error);

    Ok(report)
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let bin_path = path.join(binary);
            if bin_path.is_file() {
                return Some(bin_path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use root_lockfile::{self, LockedPackage, NixpkgsConfig};
    use root_nix::MockNixAdapter;
    use std::fs;
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
    fn test_diagnostics_healthy_with_packages() {
        let (_temp_home, _guard) = setup_test_home("test_diagnostics_healthy_with_packages");
        let root_dir = root_lockfile::init_root_dir().unwrap();

        let adapter = MockNixAdapter::new(true);
        adapter.install("poppler").unwrap();

        // Create standard Rootfile and root.lock
        let mut rf = Rootfile::default();
        rf.packages
            .insert("poppler".to_string(), "latest".to_string());
        rf.write_to_file(&root_dir.join("Rootfile")).unwrap();

        let lock = RootLock {
            version: 1,
            platform: "aarch64-darwin".into(),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![LockedPackage {
                name: "poppler".to_string(),
                requested: "poppler".to_string(),
                version: "latest".to_string(),
                attribute: "poppler".to_string(),
                store_path: root_lockfile::derive_store_path("poppler", "latest"),
                binaries: vec!["pdftotext".to_string()],
            }],
        };
        lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        // Run diagnostics
        let report = run_diagnostics(&adapter).unwrap();
        assert!(report.nix_installed);
        assert!(report.root_initialized);
        // It might have a warning about pdftotext not being on PATH or not a Nix binary — correct and expected!
        // But check that no fatal Nix or Config errors were raised.
        let error_count = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .count();
        assert_eq!(error_count, 0);
    }

    #[test]
    fn test_diagnostics_no_nix() {
        let (_temp_home, _guard) = setup_test_home("test_diagnostics_no_nix");
        let adapter = MockNixAdapter::new(false);
        let report = run_diagnostics(&adapter).unwrap();
        assert!(!report.nix_installed);
        assert!(!report.healthy);

        let nix_issue = report.issues.iter().find(|i| i.category == "Nix").unwrap();
        assert_eq!(nix_issue.severity, IssueSeverity::Error);
    }

    #[test]
    fn test_diagnostics_drift() {
        let (_temp_home, _guard) = setup_test_home("test_diagnostics_drift");
        let root_dir = root_lockfile::init_root_dir().unwrap();

        // Mock nix adapter is installed, but we DO NOT install poppler in its actual profile.
        let adapter = MockNixAdapter::new(true);

        // Create standard Rootfile and root.lock claiming poppler is locked
        let mut rf = Rootfile::default();
        rf.packages
            .insert("poppler".to_string(), "latest".to_string());
        rf.write_to_file(&root_dir.join("Rootfile")).unwrap();

        let lock = RootLock {
            version: 1,
            platform: "aarch64-darwin".into(),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![LockedPackage {
                name: "poppler".to_string(),
                requested: "poppler".to_string(),
                version: "latest".to_string(),
                attribute: "poppler".to_string(),
                store_path: root_lockfile::derive_store_path("poppler", "latest"),
                binaries: vec![],
            }],
        };
        lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        // Run diagnostics
        let report = run_diagnostics(&adapter).unwrap();
        assert!(report.nix_installed);
        assert!(report.root_initialized);

        // We should detect drift because poppler is locked but missing from active Nix profile
        let drift_issue = report
            .issues
            .iter()
            .find(|i| i.category == "Drift")
            .unwrap();
        assert_eq!(drift_issue.severity, IssueSeverity::Error);
        assert!(drift_issue
            .description
            .contains("missing from the active Nix profile"));
    }
}
