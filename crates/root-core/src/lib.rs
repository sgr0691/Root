use anyhow::{Context, Result};
use root_lockfile::{get_root_dir, LockedPackage, NixpkgsConfig, RootLock, Rootfile};
use root_nix::NixAdapter;
use root_snapshot::{list_snapshots, Snapshot};
use serde::Serialize;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PackageSpec {
    pub name: &'static str,
    pub nix_attr: &'static str,
    pub binary: &'static str,
    pub verify_args: &'static [&'static str],
}

pub const SUPPORTED_PACKAGES: &[PackageSpec] = &[PackageSpec {
    name: "ffmpeg",
    nix_attr: "nixpkgs#ffmpeg",
    binary: "ffmpeg",
    verify_args: &["-version"],
}];

fn resolve_package(name: &str) -> Option<&'static PackageSpec> {
    SUPPORTED_PACKAGES.iter().find(|p| p.name == name)
}

/// A simple file-based mutex guard for mutation commands.
/// Acquires the lock by atomically creating `~/.root/root.lockfile`.
/// Released on Drop.
struct MutationGuard {
    lock_path: PathBuf,
}

impl MutationGuard {
    fn acquire() -> anyhow::Result<Self> {
        let dir = root_lockfile::init_root_dir()?;
        let lock_path = dir.join("root.lockfile");

        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    anyhow::anyhow!(
                        "Another Root mutation is in progress.\n\
                         If this is unexpected, delete ~/.root/root.lockfile and try again."
                    )
                } else {
                    anyhow::anyhow!("Failed to acquire lock: {}", e)
                }
            })?;

        Ok(Self { lock_path })
    }
}

impl Drop for MutationGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

pub mod brew;
pub mod events;

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
            platform: root_lockfile::detect_platform()?,
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
    // v0.1 allowlist enforcement
    let _spec = resolve_package(pkg).ok_or_else(|| {
        let supported: Vec<&str> = SUPPORTED_PACKAGES.iter().map(|p| p.name).collect();
        anyhow::anyhow!(
            "Root v0.1 does not support \"{}\" yet.\n\nSupported packages:\n  {}\n\nMore packages are coming soon.",
            pkg,
            supported.join("\n  ")
        )
    })?;
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
    // v0.1 allowlist enforcement
    let _spec = resolve_package(pkg).ok_or_else(|| {
        let supported: Vec<&str> = SUPPORTED_PACKAGES.iter().map(|p| p.name).collect();
        anyhow::anyhow!(
            "Root v0.1 does not support \"{}\" yet.\n\nSupported packages:\n  {}\n\nMore packages are coming soon.",
            pkg,
            supported.join("\n  ")
        )
    })?;
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
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

    let _ = events::record_event(
        events::RootEventType::Install,
        events::RootEventStatus::Verified,
        &format!("root install {}", pkg),
        Some(pkg.to_string()),
        Some(snapshot_id.clone()),
        None,
        Some("Package installed successfully".to_string()),
    )?;

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
    root_lockfile::init_root_dir()?;
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
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
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

pub fn history() -> Result<events::HistoryOutput> {
    root_lockfile::init_root_dir()?;
    events::read_events().map(|events| events::HistoryOutput { events })
}

pub fn rollback_last(adapter: &impl NixAdapter) -> Result<RollbackReport> {
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
    let snaps = list_snapshots()?;
    if snaps.is_empty() {
        return Err(anyhow::anyhow!("No snapshots available for rollback."));
    }

    let last_snap = &snaps[0];
    let current_lock = get_or_create_lock()?;

    // Step 1: Compute rollback plan
    let mut packages_to_remove = Vec::new();
    for curr_pkg in &current_lock.packages {
        if !last_snap.packages.iter().any(|p| p.name == curr_pkg.name) {
            packages_to_remove.push(curr_pkg.name.clone());
        }
    }

    let mut packages_to_install = Vec::new();
    for old_pkg in &last_snap.packages {
        if !current_lock.packages.iter().any(|p| p.name == old_pkg.name) {
            packages_to_install.push(old_pkg.name.clone());
        }
    }

    // Step 2: Create a pre-rollback snapshot (for safety)
    let pre_rollback_snap = root_snapshot::Snapshot::create(
        &format!("before rollback to {}", last_snap.id),
        &current_lock,
    )?;

    // Step 3: Execute Nix profile changes FIRST
    for pkg in &packages_to_remove {
        adapter.remove(pkg).map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Failed to remove package '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback failed to remove '{}': {}", pkg, e)
        })?;
    }

    for pkg in &packages_to_install {
        adapter.install(pkg).map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Failed to install package '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback failed to install '{}': {}", pkg, e)
        })?;
    }

    // Step 4: ONLY NOW update Rootfile and root.lock (after Nix succeeded)
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

    // Step 5: Record rollback event
    let _ = events::record_event(
        events::RootEventType::Rollback,
        events::RootEventStatus::Completed,
        "root rollback --last",
        None,
        Some(pre_rollback_snap.id.clone()),
        Some(last_snap.id.clone()),
        Some(format!(
            "Removed: {}. Restored: {}.",
            packages_to_remove.join(", "),
            packages_to_install.join(", ")
        )),
    )?;

    Ok(RollbackReport {
        success: true,
        from_snapshot: last_snap.id.clone(),
        packages_removed: packages_to_remove,
        packages_restored: packages_to_install,
    })
}

pub fn doctor(adapter: &impl NixAdapter) -> Result<root_doctor::DoctorReport> {
    root_lockfile::init_root_dir()?;
    let report = root_doctor::run_diagnostics(adapter)?;
    let _ = events::record_event(
        events::RootEventType::Doctor,
        events::RootEventStatus::Completed,
        "root doctor",
        None,
        None,
        None,
        if report.healthy {
            Some("System healthy".to_string())
        } else {
            Some(format!("Issues found: {}", report.issues.len()))
        },
    )?;
    Ok(report)
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
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
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
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
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

        // Manually set up an initial package (bypasses allowlist)
        adapter.install("test-pkg-1").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "test-pkg-1".into(),
            requested: "test-pkg-1".into(),
            version: "latest".into(),
            attribute: "test-pkg-1".into(),
            store_path: root_lockfile::derive_store_path("test-pkg-1", "latest"),
            binaries: vec!["test-pkg-1".into()],
        });
        save_lock(&lock).unwrap();
        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile
            .packages
            .insert("test-pkg-1".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        // Use core::install for the only supported package
        install(&adapter, "ffmpeg").unwrap();

        let hist = history().unwrap();
        assert!(hist
            .events
            .iter()
            .any(|e| e.package.as_deref() == Some("ffmpeg")));

        let res = rollback_last(&adapter).unwrap();
        assert!(res.success);

        let rf = get_or_create_rootfile().unwrap();
        // Rollback reverts the ffmpeg install, leaving test-pkg-1
        assert!(rf.packages.contains_key("test-pkg-1"));
        assert!(!rf.packages.contains_key("ffmpeg"));

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
