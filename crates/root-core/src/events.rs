use anyhow::{Context, Result};
use root_lockfile::get_root_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RootEventType {
    Doctor,
    Install,
    VerificationFailed,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RootEventStatus {
    Started,
    Completed,
    Failed,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: RootEventType,
    pub command: String,
    pub status: RootEventStatus,
    pub package: Option<String>,
    pub snapshot_id: Option<String>,
    pub restored_snapshot_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryOutput {
    pub events: Vec<RootEvent>,
}

fn generate_event_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let micros = now.subsec_micros();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y%m%d_%H%M%S").to_string())
        .unwrap_or_else(|| format!("{}", secs));
    format!("evt_{}_{:06}", datetime, micros)
}

fn now_iso() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{}", secs))
}

fn events_path() -> Result<std::path::PathBuf> {
    Ok(get_root_dir()?.join("events.jsonl"))
}

pub fn append_event(event: &RootEvent) -> Result<()> {
    let path = events_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Failed to open events file at {:?}", path))?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn read_events() -> Result<Vec<RootEvent>> {
    let path = events_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(&path)
        .with_context(|| format!("Failed to open events file at {:?}", path))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<RootEvent>(line) {
            events.push(event);
        }
    }
    events.reverse();
    Ok(events)
}

pub fn create_event(
    event_type: RootEventType,
    status: RootEventStatus,
    command: &str,
    package: Option<String>,
    snapshot_id: Option<String>,
    restored_snapshot_id: Option<String>,
    message: Option<String>,
) -> RootEvent {
    RootEvent {
        id: generate_event_id(),
        timestamp: now_iso(),
        event_type,
        command: command.to_string(),
        status,
        package,
        snapshot_id,
        restored_snapshot_id,
        message,
    }
}

pub fn record_event(
    event_type: RootEventType,
    status: RootEventStatus,
    command: &str,
    package: Option<String>,
    snapshot_id: Option<String>,
    restored_snapshot_id: Option<String>,
    message: Option<String>,
) -> Result<RootEvent> {
    let event = create_event(
        event_type,
        status,
        command,
        package,
        snapshot_id,
        restored_snapshot_id,
        message,
    );
    append_event(&event)?;
    Ok(event)
}
