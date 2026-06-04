use anyhow::{Context, Result};
use root_lockfile::{
    get_root_dir, LockProfile, LockedPackage, LockedPackageOutput, LockedPackageV2, NixRuntime,
    NixpkgsConfig, NixpkgsConfigV2, RootLock, RootLockV2, Rootfile, ROOT_LOCK_SCHEMA_VERSION,
};
use root_nix::NixAdapter;
use root_snapshot::{list_snapshot_summaries, list_snapshots, Snapshot, SnapshotSummary};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PackageSpec {
    pub name: &'static str,
    pub nix_attr: &'static str,
    pub binaries: &'static [&'static str],
    pub verify_args: &'static [&'static str],
}

pub const SUPPORTED_PACKAGES: &[PackageSpec] = &[
    PackageSpec {
        name: "ffmpeg",
        nix_attr: "nixpkgs#ffmpeg",
        binaries: &["ffmpeg"],
        verify_args: &["-version"],
    },
    PackageSpec {
        name: "poppler",
        nix_attr: "nixpkgs#poppler",
        binaries: &["pdftotext", "pdfinfo"],
        verify_args: &["-h"],
    },
];

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

#[derive(Debug, Serialize)]
pub struct HistoryOutput {
    pub snapshots: Vec<SnapshotSummary>,
    pub events: Vec<events::RootEvent>,
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
            version: ROOT_LOCK_SCHEMA_VERSION,
            platform: root_lockfile::detect_platform()?,
            nixpkgs: NixpkgsConfig {
                rev: "unknown".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: Vec::new(),
        })
    }
}

#[cfg(test)]
fn save_lock(lock: &RootLock) -> Result<()> {
    let path = get_root_dir()?.join("root.lock");
    lock.write_to_file(&path)
}

fn get_or_create_lock_v2() -> Result<RootLockV2> {
    let dir = get_root_dir()?;
    let path = dir.join("root.lock");
    if path.exists() {
        RootLockV2::read_from_file(&path)
            .or_else(|_| RootLock::read_from_file(&path).map(|lock| lock.to_v2()))
    } else {
        Ok(get_or_create_lock()?.to_v2())
    }
}

fn save_lock_v2(lock: &RootLockV2) -> Result<()> {
    let path = get_root_dir()?.join("root.lock");
    lock.write_to_file(&path)
}

fn legacy_lock_from_v2(lock: &RootLockV2) -> RootLock {
    RootLock {
        version: lock.version,
        platform: lock.platform.clone(),
        nixpkgs: NixpkgsConfig {
            rev: lock.nixpkgs.rev.clone(),
            source: lock.nixpkgs.source.clone(),
        },
        packages: lock.packages.iter().map(legacy_package_from_v2).collect(),
    }
}

fn locked_package_changed(current: &LockedPackageV2, target: &LockedPackageV2) -> bool {
    serde_json::to_value(current).ok() != serde_json::to_value(target).ok()
}

fn locked_installable_for(
    adapter: &impl NixAdapter,
    pkg: &str,
) -> Result<(root_nix::FlakeMetadata, String)> {
    let flake = adapter
        .flake_metadata("nixpkgs")
        .map_err(|e| anyhow::anyhow!(e))?;
    let locked_ref = flake
        .locked_url
        .clone()
        .or_else(|| {
            flake
                .rev
                .as_ref()
                .map(|rev| format!("github:NixOS/nixpkgs/{}", rev))
        })
        .unwrap_or_else(|| "nixpkgs".to_string());
    Ok((flake, format!("{}#{}", locked_ref, pkg)))
}

fn deterministic_package_from_resolution(
    pkg: &str,
    installable: &str,
    resolution: &root_nix::LockedPackageResolution,
) -> LockedPackageV2 {
    let version = resolution
        .metadata
        .version
        .clone()
        .or_else(|| {
            resolution.metadata.name.as_ref().and_then(|name| {
                name.strip_prefix(&format!("{}-", pkg))
                    .map(|value| value.to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut outputs = BTreeMap::new();
    let mut store_paths = BTreeMap::new();
    for output in &resolution.outputs {
        let path = output.path.to_string_lossy().to_string();
        let path_info = resolution
            .path_info
            .iter()
            .find(|info| info.path == output.path);
        outputs.insert(
            output.output_name.clone(),
            LockedPackageOutput {
                store_path: path.clone(),
                content_hash: None,
                nar_hash: path_info.and_then(|info| info.nar_hash.clone()),
                references: path_info
                    .map(|info| {
                        info.references
                            .iter()
                            .map(|reference| reference.to_string_lossy().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            },
        );
        store_paths.insert(output.output_name.clone(), path);
    }

    let primary_store_path = store_paths
        .get("out")
        .cloned()
        .or_else(|| store_paths.values().next().cloned())
        .unwrap_or_default();

    let mut meta = BTreeMap::new();
    if let Some(name) = &resolution.metadata.name {
        meta.insert("name".to_string(), serde_json::Value::String(name.clone()));
    }
    if let Some(description) = &resolution.metadata.description {
        meta.insert(
            "description".to_string(),
            serde_json::Value::String(description.clone()),
        );
    }

    let binaries = resolve_package(pkg)
        .map(|spec| {
            spec.binaries
                .iter()
                .map(|binary| (*binary).to_string())
                .collect()
        })
        .unwrap_or_else(|| vec![pkg.to_string()]);

    let mut package = LockedPackageV2 {
        name: pkg.to_string(),
        requested: pkg.to_string(),
        version,
        attribute: pkg.to_string(),
        store_path: primary_store_path,
        binaries,
        installable: Some(installable.to_string()),
        flake_attribute: Some(pkg.to_string()),
        drv_path: Some(
            resolution
                .derivation
                .derivation_path
                .to_string_lossy()
                .to_string(),
        ),
        outputs,
        store_paths,
        meta,
        content_hash: None,
    };

    let hash_input = serde_json::to_vec(&package).unwrap_or_default();
    package.content_hash = Some(root_lockfile::compute_sha256(&hash_input));
    package
}

fn legacy_package_from_v2(package: &LockedPackageV2) -> LockedPackage {
    LockedPackage {
        name: package.name.clone(),
        requested: package.requested.clone(),
        version: package.version.clone(),
        attribute: package.attribute.clone(),
        store_path: package.store_path.clone(),
        binaries: package.binaries.clone(),
    }
}

fn build_v2_lock(
    old_lock: &RootLock,
    flake: &root_nix::FlakeMetadata,
    packages: Vec<LockedPackageV2>,
) -> Result<RootLockV2> {
    let root_dir = get_root_dir()?;
    let now = chrono::Utc::now().to_rfc3339();
    Ok(RootLockV2 {
        version: ROOT_LOCK_SCHEMA_VERSION,
        root_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        created_at: Some(now.clone()),
        updated_at: Some(now),
        platform: old_lock.platform.clone(),
        nix: NixRuntime {
            version: None,
            system: Some(old_lock.platform.clone()),
            store_dir: Some("/nix/store".to_string()),
            sandbox: None,
        },
        nixpkgs: NixpkgsConfigV2 {
            rev: flake
                .rev
                .clone()
                .unwrap_or_else(|| old_lock.nixpkgs.rev.clone()),
            source: flake.original_url.clone(),
            flake_ref: flake.locked_url.clone(),
            nar_hash: flake.nar_hash.clone(),
            last_modified: flake.last_modified.map(|value| value.to_string()),
            system: Some(old_lock.platform.clone()),
            config: BTreeMap::new(),
            overlays: Vec::new(),
        },
        profile: LockProfile {
            name: "default".to_string(),
            path: Some(
                root_dir
                    .join("profiles")
                    .join("default")
                    .to_string_lossy()
                    .to_string(),
            ),
            generation: None,
        },
        packages,
    })
}

fn verify_profile_contains_outputs(
    adapter: &impl NixAdapter,
    outputs: &BTreeMap<String, String>,
) -> Result<()> {
    let profile_json = adapter
        .profile_list_json()
        .map_err(|e| anyhow::anyhow!(e))?;
    for store_path in outputs.values() {
        if !profile_json.contains(store_path) {
            return Err(anyhow::anyhow!(
                "Installed profile did not contain locked Nix store path {}",
                store_path
            ));
        }
    }
    Ok(())
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
    let lock = get_or_create_lock_v2()?;
    let before_packages: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let (flake, installable) = locked_installable_for(adapter, pkg)?;
    let resolution = adapter
        .resolve_locked_package(pkg, Some(&installable))
        .map_err(|e| anyhow::anyhow!(e))?;
    let locked_package = deterministic_package_from_resolution(pkg, &installable, &resolution);

    let snapshot = Snapshot::create_from_v2(&format!("before install {}", pkg), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter
        .install_installable(pkg, &installable)
        .map_err(|e| anyhow::anyhow!(e))?;
    verify_profile_contains_outputs(adapter, &locked_package.store_paths)?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile
        .packages
        .insert(pkg.to_string(), locked_package.version.clone());
    save_rootfile(&rootfile)?;

    let mut v2_packages: Vec<LockedPackageV2> = lock
        .packages
        .iter()
        .filter(|package| package.name != pkg)
        .cloned()
        .collect();
    v2_packages.push(locked_package.clone());
    let legacy_lock = legacy_lock_from_v2(&lock);
    let v2_lock = build_v2_lock(&legacy_lock, &flake, v2_packages)?;
    save_lock_v2(&v2_lock)?;

    let _ = events::record_event(
        events::RootEventType::Install,
        events::RootEventStatus::Verified,
        &format!("root install {}", pkg),
        Some(pkg.to_string()),
        Some(snapshot_id.clone()),
        None,
        Some("Package installed successfully".to_string()),
    )?;

    let after_packages: Vec<String> = v2_lock.packages.iter().map(|p| p.name.clone()).collect();

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
    let mut lock = get_or_create_lock_v2()?;

    // Create snapshot before mutation
    let snapshot = Snapshot::create_from_v2(&format!("before remove {}", pkg), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter.remove(pkg).map_err(|e| anyhow::anyhow!(e))?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.remove(pkg);
    save_rootfile(&rootfile)?;

    lock.packages.retain(|p| p.name != pkg);
    lock.updated_at = Some(chrono::Utc::now().to_rfc3339());
    save_lock_v2(&lock)?;

    let _ = events::record_event(
        events::RootEventType::Remove,
        events::RootEventStatus::Completed,
        &format!("root remove {}", pkg),
        Some(pkg.to_string()),
        Some(snapshot_id.clone()),
        None,
        Some("Package removed successfully".to_string()),
    )?;

    Ok(RemoveReport {
        success: true,
        package: pkg.to_string(),
        snapshot_id,
        rollback_available: true,
    })
}

pub fn history() -> Result<HistoryOutput> {
    root_lockfile::init_root_dir()?;
    Ok(HistoryOutput {
        snapshots: list_snapshot_summaries()?,
        events: events::read_events()?,
    })
}

pub fn rollback_last(adapter: &impl NixAdapter) -> Result<RollbackReport> {
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
    let snaps = list_snapshots()?;
    if snaps.is_empty() {
        return Err(anyhow::anyhow!("No snapshots available for rollback."));
    }

    let last_snap = &snaps[0];
    let current_lock = get_or_create_lock_v2()?;
    let target_lock = last_snap.restored_lock();

    // Step 1: Compute rollback plan
    let mut packages_to_remove = Vec::new();
    for curr_pkg in &current_lock.packages {
        let target_pkg = target_lock
            .packages
            .iter()
            .find(|package| package.name == curr_pkg.name);
        if target_pkg
            .map(|target_pkg| locked_package_changed(curr_pkg, target_pkg))
            .unwrap_or(true)
        {
            packages_to_remove.push(curr_pkg.name.clone());
        }
    }

    let mut packages_to_install = Vec::new();
    for target_pkg in &target_lock.packages {
        let current_pkg = current_lock
            .packages
            .iter()
            .find(|package| package.name == target_pkg.name);
        if current_pkg
            .map(|current_pkg| locked_package_changed(current_pkg, target_pkg))
            .unwrap_or(true)
        {
            packages_to_install.push(target_pkg.name.clone());
        }
    }

    // Step 2: Create a pre-rollback snapshot (for safety)
    let pre_rollback_snap = root_snapshot::Snapshot::create_from_v2(
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
        let locked_pkg = target_lock
            .packages
            .iter()
            .find(|package| package.name == *pkg)
            .ok_or_else(|| {
                anyhow::anyhow!("Snapshot package '{}' is missing lock metadata", pkg)
            })?;
        let install_result = if let Some(installable) = locked_pkg.installable.as_deref() {
            adapter.install_installable(pkg, installable)
        } else {
            adapter.install(pkg)
        };
        install_result.map_err(|e| {
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

        // Verify profile contains the locked store paths from the snapshot
        verify_profile_contains_outputs(adapter, &locked_pkg.store_paths).map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Rollback verification failed for '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback verification failed for '{}': {}", pkg, e)
        })?;
    }

    // Step 4: ONLY NOW update Rootfile and root.lock (after Nix succeeded)
    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();

    for pkg in &target_lock.packages {
        rootfile
            .packages
            .insert(pkg.name.clone(), pkg.version.clone());
    }

    save_rootfile(&rootfile)?;
    save_lock_v2(&target_lock)?;

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
    let report = root_doctor::run_diagnostics(adapter)?;
    if get_root_dir().map(|path| path.exists()).unwrap_or(false) {
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
        );
    }
    Ok(report)
}

pub fn verify(pkg: &str) -> Result<root_verify::VerificationReport> {
    match root_verify::verify_package(pkg) {
        Ok(report) => {
            let event_type = if report.success {
                events::RootEventType::Verification
            } else {
                events::RootEventType::VerificationFailed
            };
            let status = if report.success {
                events::RootEventStatus::Verified
            } else {
                events::RootEventStatus::Failed
            };
            let message = if report.success {
                Some(format!("Verified package '{}'.", pkg))
            } else {
                Some(format!(
                    "Verification failed for '{}': {}",
                    pkg,
                    report.errors.join("; ")
                ))
            };
            let _ = events::record_event(
                event_type,
                status,
                &format!("root verify {}", pkg),
                Some(pkg.to_string()),
                None,
                None,
                message,
            );
            Ok(report)
        }
        Err(err) => {
            let _ = events::record_event(
                events::RootEventType::VerificationFailed,
                events::RootEventStatus::Failed,
                &format!("root verify {}", pkg),
                Some(pkg.to_string()),
                None,
                None,
                Some(err.to_string()),
            );
            Err(err)
        }
    }
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

    let mut packages_locked = Vec::new();
    let mut packages_removed = Vec::new();
    let mut v2_packages = Vec::new();
    let mut flake_for_lock = None;

    // Build a deterministic lock from Rootfile intent by resolving Nix metadata.
    for name in rootfile.packages.keys() {
        let (flake, installable) = locked_installable_for(adapter, name)?;
        let resolution = adapter
            .resolve_locked_package(name, Some(&installable))
            .map_err(|e| anyhow::anyhow!(e))?;
        let locked_package = deterministic_package_from_resolution(name, &installable, &resolution);
        flake_for_lock = Some(flake);
        packages_locked.push(name.clone());
        v2_packages.push(locked_package);
    }

    // Detect packages that were in old lock but not in new lock
    for old_pkg in &old_lock.packages {
        if !v2_packages.iter().any(|p| p.name == old_pkg.name) {
            packages_removed.push(old_pkg.name.clone());
        }
    }

    let flake = match flake_for_lock {
        Some(flake) => flake,
        None => adapter
            .flake_metadata("nixpkgs")
            .map_err(|e| anyhow::anyhow!(e))?,
    };
    let new_lock = build_v2_lock(&old_lock, &flake, v2_packages)?;
    save_lock_v2(&new_lock)?;

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

    // sync is experimental and does not support v2 deterministic locks.
    // Detect v2 lock and refuse rather than operating on a lossy v1 projection.
    let root_dir = get_root_dir()?;
    let lock_path = root_dir.join("root.lock");
    if lock_path.exists() {
        if let Ok(v2_lock) = RootLockV2::read_from_file(&lock_path) {
            if v2_lock.version >= ROOT_LOCK_SCHEMA_VERSION {
                return Err(anyhow::anyhow!(
                    "`root sync` does not support v2 lockfiles. \
                     v0.1.2 manages profile state automatically during `root install` and `root rollback`. \
                     Use `root install` or `root rollback` instead."
                ));
            }
        }
    }

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
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    /// Serializes tests that mutate process-global env vars (ROOT_DIR).
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn test_tmp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("root_test_{}_{}_{}", name, std::process::id(), n))
    }

    fn write_fake_binary(root_dir: &std::path::Path, name: &str, body: &str) {
        let bin_dir = root_dir.join("profiles").join("default").join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let path = bin_dir.join(name);
        std::fs::write(&path, body).unwrap();
        #[cfg(unix)]
        {
            let mut permissions = std::fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).unwrap();
        }
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

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let deterministic_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let ffmpeg = deterministic_lock
            .packages
            .iter()
            .find(|package| package.name == "ffmpeg")
            .unwrap();
        assert_eq!(deterministic_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_ne!(ffmpeg.version, "latest");
        assert!(ffmpeg.installable.as_deref().unwrap().contains("#ffmpeg"));
        assert!(ffmpeg.drv_path.as_deref().unwrap().ends_with(".drv"));
        assert!(ffmpeg.store_path.starts_with("/nix/store/"));
        assert!(!ffmpeg.has_placeholder_store_path());

        let hist = history().unwrap();
        assert!(hist
            .snapshots
            .iter()
            .any(|snapshot| snapshot.reason == "before install ffmpeg"));
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

        let restored_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(restored_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert!(restored_lock
            .packages
            .iter()
            .any(|package| package.name == "test-pkg-1"));
        assert!(!restored_lock
            .packages
            .iter()
            .any(|package| package.name == "ffmpeg"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_remove_preserves_v2_lock_and_records_snapshot() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("remove_v2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "ffmpeg").unwrap();
        let remove_report = remove(&adapter, "ffmpeg").unwrap();
        assert!(remove_report.success);

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert!(lock.packages.is_empty());

        let hist = history().unwrap();
        assert!(hist
            .snapshots
            .iter()
            .any(|snapshot| snapshot.id == remove_report.snapshot_id));
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Remove
                && event.package.as_deref() == Some("ffmpeg")
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_poppler_install_writes_binary_metadata() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("poppler_binaries");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "poppler").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let poppler = lock
            .packages
            .iter()
            .find(|package| package.name == "poppler")
            .unwrap();
        assert_eq!(
            poppler.binaries,
            vec!["pdftotext".to_string(), "pdfinfo".to_string()]
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_verify_records_success_event() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("verify_event");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let root_dir = root_lockfile::init_root_dir().unwrap();
        std::env::set_var(
            "PATH",
            root_dir.join("profiles").join("default").join("bin"),
        );
        write_fake_binary(
            &root_dir,
            "ffmpeg",
            "#!/bin/sh\necho 'ffmpeg version root'\n",
        );

        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "ffmpeg".into(),
            requested: "ffmpeg".into(),
            version: "7.1".into(),
            attribute: "ffmpeg".into(),
            store_path: root_lockfile::derive_store_path("ffmpeg", "7.1"),
            binaries: vec!["ffmpeg".into()],
        });
        save_lock(&lock).unwrap();

        let report = verify("ffmpeg").unwrap();
        assert!(report.success);

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Verification
                && event.status == events::RootEventStatus::Verified
                && event.package.as_deref() == Some("ffmpeg")
        }));

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

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let deterministic_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let locked_package = deterministic_lock
            .packages
            .iter()
            .find(|p| p.name == "ripgrep")
            .unwrap();
        assert_eq!(deterministic_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_ne!(deterministic_lock.nixpkgs.rev, "unknown");
        assert_ne!(locked_package.version, "latest");
        assert!(locked_package.store_path.starts_with("/nix/store/"));
        assert!(!locked_package.has_placeholder_store_path());

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

        let mut lock = RootLock {
            version: 1,
            platform: root_lockfile::detect_platform().unwrap_or_else(|_| "aarch64-darwin".into()),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![],
        };
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
        lock.write_to_file(&root_lockfile::get_root_dir().unwrap().join("root.lock"))
            .unwrap();

        let report = sync(&adapter).unwrap();
        assert!(report.success);
        assert!(report.installed.contains(&"pkg-b".to_string()));
        assert!(report.removed.contains(&"pkg-c".to_string()));
        assert!(report.unchanged.contains(&"pkg-a".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sync_refuses_v2_lock() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("sync_refuse_v2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let root_dir = root_lockfile::init_root_dir().unwrap();
        let adapter = MockNixAdapter::new(true);

        // Create a v2 lock with deterministic metadata
        adapter.install("ripgrep").unwrap();
        let (flake, installable) = locked_installable_for(&adapter, "ripgrep").unwrap();
        let resolution = adapter
            .resolve_locked_package("ripgrep", Some(&installable))
            .unwrap();
        let locked_pkg =
            deterministic_package_from_resolution("ripgrep", &installable, &resolution);
        let v2_lock = build_v2_lock(
            &RootLock {
                version: 1,
                platform: root_lockfile::detect_platform()
                    .unwrap_or_else(|_| "aarch64-darwin".into()),
                nixpkgs: NixpkgsConfig {
                    rev: "some-rev".into(),
                    source: "github:NixOS/nixpkgs".into(),
                },
                packages: vec![],
            },
            &flake,
            vec![locked_pkg],
        )
        .unwrap();
        v2_lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        let err = sync(&adapter).unwrap_err();
        assert!(err.to_string().contains("does not support v2 lockfiles"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rollback_v2_verifies_store_paths() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("rollback_v2_verify");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Set up an initial v1 package (simulates pre-v1.2 state)
        adapter.install("ripgrep").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "ripgrep".into(),
            requested: "ripgrep".into(),
            version: "latest".into(),
            attribute: "ripgrep".into(),
            store_path: root_lockfile::derive_store_path("ripgrep", "latest"),
            binaries: vec!["rg".into()],
        });
        save_lock(&lock).unwrap();
        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        // Install v2 package
        install(&adapter, "ffmpeg").unwrap();

        // Rollback should succeed (profile contains expected paths)
        let res = rollback_last(&adapter).unwrap();
        assert!(res.success);
        assert!(res.packages_removed.contains(&"ffmpeg".to_string()));

        // Verify final state: ripgrep present, ffmpeg absent
        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let restored_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert!(restored_lock.packages.iter().any(|p| p.name == "ripgrep"));
        assert!(!restored_lock.packages.iter().any(|p| p.name == "ffmpeg"));

        let rf = get_or_create_rootfile().unwrap();
        assert!(rf.packages.contains_key("ripgrep"));
        assert!(!rf.packages.contains_key("ffmpeg"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_on_v1_lock_migrates_to_v2() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_migrate_v1");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Create a v1 lock with "latest" version
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

        // Install ffmpeg (v2 deterministically)
        install(&adapter, "ffmpeg").unwrap();

        // Verify lock is now v2 with real metadata
        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let migrated_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(migrated_lock.version, ROOT_LOCK_SCHEMA_VERSION);

        // Old package preserved (v1 fields carry over; v2-only fields may be empty)
        let old_pkg = migrated_lock
            .packages
            .iter()
            .find(|p| p.name == "test-pkg-1")
            .unwrap();
        assert_eq!(old_pkg.version, "latest");
        assert!(!old_pkg.store_path.is_empty());

        // New package has deterministic metadata
        let new_pkg = migrated_lock
            .packages
            .iter()
            .find(|p| p.name == "ffmpeg")
            .unwrap();
        assert_ne!(new_pkg.version, "latest");
        assert!(new_pkg.store_path.starts_with("/nix/store/"));
        assert!(new_pkg.drv_path.as_deref().unwrap().ends_with(".drv"));
        assert!(!new_pkg.store_paths.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
