use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use root_lockfile::{get_root_dir, LockedPackage, RootLock};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub reason: String,
    pub packages: Vec<LockedPackage>,
}

impl Snapshot {
    pub fn create(reason: &str, current_lock: &RootLock) -> Result<Self> {
        let now = Utc::now();
        let id = format!("snap_{}", now.format("%Y%m%d_%H%M%S_%f"));

        let snapshot = Snapshot {
            id: id.clone(),
            created_at: now,
            reason: reason.to_string(),
            // Clone packages. It assumes LockedPackage derives Clone, wait... does it?
            // I'll manually implement clone for it in root-lockfile, or just serialize/deserialize
            packages: serde_json::from_str(&serde_json::to_string(&current_lock.packages)?)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list() {
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

        let list = list_snapshots().unwrap();
        assert!(list.iter().any(|s| s.id == snap.id));

        let read_snap = Snapshot::read(&snap.id).unwrap();
        assert_eq!(read_snap.reason, "test snapshot");
    }
}
