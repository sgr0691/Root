use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use root_lockfile::{compute_sha256, get_root_dir, LockedPackage, RootLock, RootLockV2};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub reason: String,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub package_count: usize,
    #[serde(default)]
    pub lock_content_hash: String,
    #[serde(default = "default_snapshot_lock")]
    pub lock: RootLockV2,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotSummary {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub reason: String,
    pub schema_version: u32,
    pub package_count: usize,
    pub lock_content_hash: String,
}

impl From<&Snapshot> for SnapshotSummary {
    fn from(snapshot: &Snapshot) -> Self {
        Self {
            id: snapshot.id.clone(),
            created_at: snapshot.created_at,
            reason: snapshot.reason.clone(),
            schema_version: snapshot.schema_version,
            package_count: snapshot.package_count,
            lock_content_hash: snapshot.lock_content_hash.clone(),
        }
    }
}

fn default_snapshot_lock() -> RootLockV2 {
    RootLock {
        version: 0,
        platform: "unknown".to_string(),
        nixpkgs: root_lockfile::NixpkgsConfig {
            rev: "unknown".to_string(),
            source: "github:NixOS/nixpkgs".to_string(),
        },
        packages: Vec::new(),
    }
    .to_v2()
}

impl Snapshot {
    pub fn create(reason: &str, current_lock: &RootLock) -> Result<Self> {
        Self::create_from_v2(reason, &current_lock.to_v2())
    }

    pub fn create_from_v2(reason: &str, current_lock: &RootLockV2) -> Result<Self> {
        let now = Utc::now();
        let id = format!("snap_{}", now.format("%Y%m%d_%H%M%S_%f"));
        let packages: Vec<LockedPackage> = current_lock
            .packages
            .iter()
            .map(|package| LockedPackage {
                name: package.name.clone(),
                requested: package.requested.clone(),
                version: package.version.clone(),
                attribute: package.attribute.clone(),
                store_path: package.store_path.clone(),
                binaries: package.binaries.clone(),
            })
            .collect();
        let lock_content = serde_json::to_vec(current_lock)?;

        let snapshot = Snapshot {
            id: id.clone(),
            created_at: now,
            reason: reason.to_string(),
            schema_version: current_lock.version,
            package_count: current_lock.packages.len(),
            lock_content_hash: compute_sha256(&lock_content),
            lock: current_lock.clone(),
            packages,
        };

        let snapshots_dir = get_root_dir()?.join("snapshots");
        fs::create_dir_all(&snapshots_dir)?;

        let path = snapshots_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&snapshot)?;
        fs::write(path, content)?;

        Ok(snapshot)
    }

    pub fn read(id: &str) -> Result<Self> {
        let path = get_root_dir()?
            .join("snapshots")
            .join(format!("{}.json", id));
        let content = fs::read_to_string(path).context("Snapshot not found")?;
        let snapshot = serde_json::from_str(&content)?;
        Ok(snapshot)
    }

    pub fn restored_lock(&self) -> RootLockV2 {
        if self.lock.version == 0 && !self.packages.is_empty() {
            RootLock {
                version: root_lockfile::ROOT_LOCK_SCHEMA_VERSION,
                platform: root_lockfile::detect_platform().unwrap_or_else(|_| "unknown".into()),
                nixpkgs: root_lockfile::NixpkgsConfig {
                    rev: "unknown".into(),
                    source: "github:NixOS/nixpkgs".into(),
                },
                packages: self.packages.clone(),
            }
            .to_v2()
        } else {
            self.lock.clone()
        }
    }
}

pub fn list_snapshots() -> Result<Vec<Snapshot>> {
    let snapshots_dir = get_root_dir()?.join("snapshots");
    if !snapshots_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in fs::read_dir(snapshots_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            if let Ok(snap) = serde_json::from_str::<Snapshot>(&content) {
                snapshots.push(snap);
            }
        }
    }

    snapshots.sort_by_key(|b| std::cmp::Reverse(b.created_at)); // newest first
    Ok(snapshots)
}

pub fn list_snapshot_summaries() -> Result<Vec<SnapshotSummary>> {
    Ok(list_snapshots()?
        .iter()
        .map(SnapshotSummary::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_create_and_list() {
        let _guard = TEST_MUTEX.lock().unwrap();
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!("root_test_snap_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let lock = RootLock {
            version: 1,
            platform: "test".into(),
            nixpkgs: root_lockfile::NixpkgsConfig {
                rev: "abc".into(),
                source: "test".into(),
            },
            packages: vec![],
        };

        let snap = Snapshot::create("test snapshot", &lock).unwrap();
        assert!(snap.id.starts_with("snap_"));
        assert_eq!(snap.schema_version, root_lockfile::ROOT_LOCK_SCHEMA_VERSION);
        assert_eq!(snap.package_count, 0);
        assert!(!snap.lock_content_hash.is_empty());

        let list = list_snapshots().unwrap();
        assert!(list.iter().any(|s| s.id == snap.id));

        let read_snap = Snapshot::read(&snap.id).unwrap();
        assert_eq!(read_snap.reason, "test snapshot");
    }
}
