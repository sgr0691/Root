use anyhow::{Context, Result};
use root_lockfile::{get_root_dir, LockedPackage, NixpkgsConfig, RootLock, Rootfile};
use root_nix::NixAdapter;
use root_snapshot::{list_snapshots, Snapshot};
use serde::Serialize;
use std::path::Path;

pub mod brew;

#[derive(Debug, Serialize)]
pub struct ListPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ListOutput {
    pub packages: Vec<ListPackage>,
    pub nix_profile: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryOutput {
    pub snapshots: Vec<Snapshot>,
}

#[derive(Debug, Serialize)]
pub struct InitReport {
    pub success: bool,
    pub root_dir: String,
    pub nix_detected: bool,
    pub profile_ready: bool,
    pub snapshot_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct InstallReport {
    pub success: bool,
    pub operation: &'static str,
    pub package: String,
    pub changed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
    pub rollback_available: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanReport {
    pub package: String,
    pub found: bool,
    pub description: String,
    pub would_create_snapshot: bool,
    pub attributes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RemoveReport {
    pub success: bool,
    pub package: String,
    pub snapshot_id: String,
    pub rollback_available: bool,
}

#[derive(Debug, Serialize)]
pub struct RollbackReport {
    pub success: bool,
    pub from_snapshot: String,
    pub packages_removed: Vec<String>,
    pub packages_restored: Vec<String>,
}

/// Install Nix using the official multi-user installer.
pub fn install_nix() -> Result<()> {
    let status = std::process::Command::new("sh")
        .args(["-c", "curl -L https://nixos.org/nix/install | sh"])
        .status()
        .context("Failed to run Nix installer")?;

    if !status.success() {
        Err(anyhow::anyhow!(
            "Nix installer exited with code {:?}",
            status.code()
        ))
    } else {
        Ok(())
    }
}

pub fn init(adapter: &impl NixAdapter) -> Result<InitReport> {
    let root_dir =
        root_lockfile::init_root_dir().context("Failed to initialize Root directories")?;
    let nix_detected = adapter
        .check_availability()
        .map_err(|e| anyhow::anyhow!(e))
        .unwrap_or(false);
    Ok(InitReport {
        success: true,
        root_dir: root_dir.to_string_lossy().to_string(),
        nix_detected,
        profile_ready: true,
        snapshot_enabled: true,
    })
}

fn get_or_create_rootfile() -> Result<Rootfile> {
    let dir = get_root_dir()?;
    let path = dir.join("Rootfile");
    if path.exists() {
        Rootfile::read_from_file(&path)
    } else {
        Ok(Rootfile::default())
    }
}

fn save_rootfile(rootfile: &Rootfile) -> Result<()> {
    let path = get_root_dir()?.join("Rootfile");
    rootfile.write_to_file(&path)
}

fn get_or_create_lock() -> Result<RootLock> {
    let dir = get_root_dir()?;
    let path = dir.join("root.lock");
    if path.exists() {
        RootLock::read_from_file(&path)
    } else {
        Ok(RootLock {
            version: 1,
            platform: "aarch64-darwin".into(),
            nixpkgs: NixpkgsConfig {
                rev: "unknown".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: Vec::new(),
        })
    }
}

fn save_lock(lock: &RootLock) -> Result<()> {
    let path = get_root_dir()?.join("root.lock");
    lock.write_to_file(&path)
}

fn parse_attributes(search_output: &str) -> Vec<String> {
    search_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("* nixpkgs#") {
                rest.split_whitespace().next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn plan(adapter: &impl NixAdapter, pkg: &str) -> Result<PlanReport> {
    match adapter.search(pkg) {
        Ok(description) => {
            let attributes = parse_attributes(&description);
            Ok(PlanReport {
                package: pkg.to_string(),
                found: true,
                description,
                would_create_snapshot: true,
                attributes,
            })
        }
        Err(root_nix::NixError::NotFound(_)) => Ok(PlanReport {
            package: pkg.to_string(),
            found: false,
            description: String::new(),
            would_create_snapshot: true,
            attributes: Vec::new(),
        }),
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

pub fn install(adapter: &impl NixAdapter, pkg: &str) -> Result<InstallReport> {
    let lock = get_or_create_lock()?;
    let before_packages: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let snapshot = Snapshot::create(&format!("before install {}", pkg), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter.install(pkg).map_err(|e| anyhow::anyhow!(e))?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile
        .packages
        .insert(pkg.to_string(), "latest".to_string());
    save_rootfile(&rootfile)?;

    let mut lock = lock;
    if !lock.packages.iter().any(|p| p.name == pkg) {
        lock.packages.push(LockedPackage {
            name: pkg.to_string(),
            requested: pkg.to_string(),
            version: "latest".into(),
            attribute: pkg.to_string(),
            store_path: root_lockfile::derive_store_path(pkg, "latest"),
            binaries: vec![pkg.to_string()],
        });
        save_lock(&lock)?;
    }

    let after_packages: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let changed: Vec<String> = after_packages
        .iter()
        .filter(|p| !before_packages.contains(p))
        .cloned()
        .collect();
    let unchanged: Vec<String> = before_packages
        .iter()
        .filter(|p| after_packages.contains(p))
        .cloned()
        .collect();

    Ok(InstallReport {
        success: true,
        operation: "install",
        package: pkg.to_string(),
        changed,
        unchanged,
        snapshot_id,
        rollback_available: true,
        warnings: Vec::new(),
    })
}

pub fn list(adapter: &impl NixAdapter) -> Result<ListOutput> {
    let rootfile = get_or_create_rootfile()?;
    let packages: Vec<ListPackage> = rootfile
        .packages
        .into_iter()
        .map(|(name, version)| ListPackage { name, version })
        .collect();

    let nix_profile = adapter.list().map_err(|e| anyhow::anyhow!(e))?;

    Ok(ListOutput {
        packages,
        nix_profile,
    })
}

pub fn remove(adapter: &impl NixAdapter, pkg: &str) -> Result<RemoveReport> {
    let lock = get_or_create_lock()?;

    // Create snapshot before mutation
    let snapshot = Snapshot::create(&format!("before remove {}", pkg), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter.remove(pkg).map_err(|e| anyhow::anyhow!(e))?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.remove(pkg);
    save_rootfile(&rootfile)?;

    let mut lock = lock;
    lock.packages.retain(|p| p.name != pkg);
    save_lock(&lock)?;

    Ok(RemoveReport {
        success: true,
        package: pkg.to_string(),
        snapshot_id,
        rollback_available: true,
    })
}

pub fn history() -> Result<HistoryOutput> {
    let snaps = list_snapshots()?;
    Ok(HistoryOutput { snapshots: snaps })
}

pub fn rollback_last(adapter: &impl NixAdapter) -> Result<RollbackReport> {
    let snaps = list_snapshots()?;
    if snaps.is_empty() {
        return Err(anyhow::anyhow!("No snapshots available for rollback."));
    }

    let last_snap = &snaps[0];
    let current_lock = get_or_create_lock()?;

    let mut added_since_snap = Vec::new();
    for curr_pkg in &current_lock.packages {
        if !last_snap.packages.iter().any(|p| p.name == curr_pkg.name) {
            added_since_snap.push(curr_pkg.name.clone());
        }
    }

    let mut removed_since_snap = Vec::new();
    for old_pkg in &last_snap.packages {
        if !current_lock.packages.iter().any(|p| p.name == old_pkg.name) {
            removed_since_snap.push(old_pkg.name.clone());
        }
    }

    // Snapshot again before rollback
    Snapshot::create(
        &format!("before rollback to {}", last_snap.id),
        &current_lock,
    )?;

    // Reconcile
    for pkg in &added_since_snap {
        adapter
            .remove(pkg)
            .map_err(|e| anyhow::anyhow!("Rollback remove failed: {}", e))?;
    }
    for pkg in &removed_since_snap {
        adapter
            .install(pkg)
            .map_err(|e| anyhow::anyhow!("Rollback install failed: {}", e))?;
    }

    // Overwrite lockfile & rootfile to match snapshot EXACTLY
    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();
    let mut restored_lock = current_lock;
    restored_lock.packages.clear();

    for pkg in &last_snap.packages {
        rootfile
            .packages
            .insert(pkg.name.clone(), pkg.version.clone());
        restored_lock.packages.push(LockedPackage {
            name: pkg.name.clone(),
            requested: pkg.requested.clone(),
            version: pkg.version.clone(),
            attribute: pkg.attribute.clone(),
            store_path: pkg.store_path.clone(),
            binaries: pkg.binaries.clone(),
        });
    }

    save_rootfile(&rootfile)?;
    save_lock(&restored_lock)?;

    Ok(RollbackReport {
        success: true,
        from_snapshot: last_snap.id.clone(),
        packages_removed: added_since_snap,
        packages_restored: removed_since_snap,
    })
}

pub fn doctor(adapter: &impl NixAdapter) -> Result<root_doctor::DoctorReport> {
    root_doctor::run_diagnostics(adapter)
}

pub fn verify(pkg: &str) -> Result<root_verify::VerificationReport> {
    root_verify::verify_package(pkg)
}

pub fn import_brew(dest_dir: &Path) -> Result<brew::BrewImportReport> {
    brew::import_brew(dest_dir)
}

fn parse_nix_profile_packages(list_output: &str) -> Vec<String> {
    list_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Match mock adapter format: "Index: 0 - nixpkgs#poppler"
            if let Some(idx) = line.find("nixpkgs#") {
                let pkg = line[idx + 8..].trim().to_string();
                if !pkg.is_empty() {
                    return Some(pkg);
                }
            }
            None
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct LockReport {
    pub success: bool,
    pub packages_locked: Vec<String>,
    pub packages_removed: Vec<String>,
}

pub fn lock(adapter: &impl NixAdapter) -> Result<LockReport> {
    let rootfile = get_or_create_rootfile()?;
    let old_lock = get_or_create_lock()?;

    let nix_list = adapter.list().map_err(|e| anyhow::anyhow!(e))?;
    let profile_pkgs = parse_nix_profile_packages(&nix_list);

    let mut new_lock = RootLock {
        version: 1,
        platform: old_lock.platform.clone(),
        nixpkgs: old_lock.nixpkgs.clone(),
        packages: Vec::new(),
    };

    let mut packages_locked = Vec::new();
    let mut packages_removed = Vec::new();

    // Build new lock from Rootfile + profile state
    for (name, version) in &rootfile.packages {
        let store_path = profile_pkgs
            .iter()
            .find(|p| p == &name)
            .map(|p| root_lockfile::derive_store_path(p, version))
            .unwrap_or_else(|| root_lockfile::derive_store_path(name, version));
        new_lock.packages.push(LockedPackage {
            name: name.clone(),
            requested: name.clone(),
            version: version.clone(),
            attribute: name.clone(),
            store_path,
            binaries: vec![name.clone()],
        });
        packages_locked.push(name.clone());
    }

    // Detect packages that were in old lock but not in new lock
    for old_pkg in &old_lock.packages {
        if !new_lock.packages.iter().any(|p| p.name == old_pkg.name) {
            packages_removed.push(old_pkg.name.clone());
        }
    }

    save_lock(&new_lock)?;

    Ok(LockReport {
        success: true,
        packages_locked,
        packages_removed,
    })
}

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub success: bool,
    pub installed: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
}

pub fn sync(adapter: &impl NixAdapter) -> Result<SyncReport> {
    let lock = get_or_create_lock()?;
    let nix_list = adapter.list().map_err(|e| anyhow::anyhow!(e))?;
    let profile_pkgs = parse_nix_profile_packages(&nix_list);

    let locked_names: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let to_install: Vec<String> = locked_names
        .iter()
        .filter(|p| !profile_pkgs.contains(p))
        .cloned()
        .collect();
    let to_remove: Vec<String> = profile_pkgs
        .iter()
        .filter(|p| !locked_names.contains(p))
        .cloned()
        .collect();
    let unchanged: Vec<String> = locked_names
        .iter()
        .filter(|p| profile_pkgs.contains(p))
        .cloned()
        .collect();

    // Snapshot before sync
    let snapshot = Snapshot::create("before sync", &lock)?;
    let snapshot_id = snapshot.id.clone();

    for pkg in &to_install {
        adapter.install(pkg).map_err(|e| anyhow::anyhow!(e))?;
    }
    for pkg in &to_remove {
        adapter.remove(pkg).map_err(|e| anyhow::anyhow!(e))?;
    }

    // Update Rootfile to match lock
    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();
    for pkg in &lock.packages {
        rootfile
            .packages
            .insert(pkg.name.clone(), pkg.version.clone());
    }
    save_rootfile(&rootfile)?;

    Ok(SyncReport {
        success: true,
        installed: to_install,
        removed: to_remove,
        unchanged,
        snapshot_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use root_nix::MockNixAdapter;
    use std::sync::Mutex;

    /// Serializes tests that mutate process-global env vars (ROOT_DIR).
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn test_tmp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("root_test_{}_{}_{}", name, std::process::id(), n))
    }

    #[test]
    fn test_snapshots_and_rollback() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("snapshots");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "test-pkg-1").unwrap();
        install(&adapter, "test-pkg-2").unwrap();

        let hist = history().unwrap();
        assert!(hist
            .snapshots
            .iter()
            .any(|s| s.reason.contains("test-pkg-1")));
        assert!(hist
            .snapshots
            .iter()
            .any(|s| s.reason.contains("test-pkg-2")));

        let res = rollback_last(&adapter).unwrap();
        assert!(res.success);

        let rf = get_or_create_rootfile().unwrap();
        // Rollback reverts the LAST action (which was installing test-pkg-2)
        assert!(rf.packages.contains_key("test-pkg-1"));
        assert!(!rf.packages.contains_key("test-pkg-2"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_lock_generates_lockfile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("lock");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Install something via adapter so it's in the profile
        adapter.install("ripgrep").unwrap();

        // Add to Rootfile
        let mut rf = get_or_create_rootfile().unwrap();
        rf.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rf).unwrap();

        let report = lock(&adapter).unwrap();
        assert!(report.success);
        assert!(report.packages_locked.contains(&"ripgrep".to_string()));

        let lockfile = get_or_create_lock().unwrap();
        assert!(lockfile.packages.iter().any(|p| p.name == "ripgrep"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sync_reconciles_profile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("sync");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Setup: lock says pkg-a and pkg-b, but profile only has pkg-a + pkg-c
        adapter.install("pkg-a").unwrap();
        adapter.install("pkg-c").unwrap();

        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "pkg-a".into(),
            requested: "pkg-a".into(),
            version: "latest".into(),
            attribute: "pkg-a".into(),
            store_path: root_lockfile::derive_store_path("pkg-a", "latest"),
            binaries: vec!["pkg-a".into()],
        });
        lock.packages.push(LockedPackage {
            name: "pkg-b".into(),
            requested: "pkg-b".into(),
            version: "latest".into(),
            attribute: "pkg-b".into(),
            store_path: root_lockfile::derive_store_path("pkg-b", "latest"),
            binaries: vec!["pkg-b".into()],
        });
        save_lock(&lock).unwrap();

        let report = sync(&adapter).unwrap();
        assert!(report.success);
        assert!(report.installed.contains(&"pkg-b".to_string()));
        assert!(report.removed.contains(&"pkg-c".to_string()));
        assert!(report.unchanged.contains(&"pkg-a".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
