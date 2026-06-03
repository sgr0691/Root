# Root v0.1 Data Model

## PackageSpec

Represents a supported package.

```rust
pub struct PackageSpec {
    pub name: String,
    pub nix_attr: String,
    pub binary: String,
    pub verify_args: Vec<String>,
}
```

Example:

```json
{
  "name": "ffmpeg",
  "nix_attr": "nixpkgs#ffmpeg",
  "binary": "ffmpeg",
  "verify_args": ["-version"]
}
```

## ManagedPackage

Represents a package currently managed by Root.

```rust
pub struct ManagedPackage {
    pub name: String,
    pub nix_attr: String,
    pub binary: String,
    pub installed_at: String
}
```

## RootSnapshot

Represents managed state at a point in time.

```rust
pub struct RootSnapshot {
    pub id: String,
    pub timestamp: String,
    pub managed_packages: Vec<ManagedPackage>
}
```

## RootEvent

Represents a machine event.

```rust
pub struct RootEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: RootEventType,
    pub command: String,
    pub status: RootEventStatus,
    pub package: Option<String>,
    pub snapshot_id: Option<String>,
    pub restored_snapshot_id: Option<String>,
    pub message: Option<String>
}
```

## RootEventType

```rust
pub enum RootEventType {
    Doctor,
    Install,
    VerificationFailed,
    Rollback
}
```

## RootEventStatus

```rust
pub enum RootEventStatus {
    Started,
    Completed,
    Failed,
    Verified
}
```

## Storage

Events:

```text
~/.root/events.jsonl
```

Snapshots:

```text
~/.root/snapshots/<snapshot-id>.json
```

Root lock:

```text
~/.root/root.lock
```

## Why JSONL

JSONL is appropriate for v0.1 because:

- It is easy to inspect.
- It is append-only.
- It does not require a database.
- It supports the machine ledger concept.
- It can migrate to SQLite later.
