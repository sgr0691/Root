# Root v0.1 Technical Spec

## Technical Goal

Implement a Rust CLI that wraps Nix for one golden install path:

```bash
root install ffmpeg
```

and supports:

```bash
root history
root rollback
root doctor
```

## Language

Rust

## CLI Framework

Recommended crates:

- `clap` for CLI parsing.
- `serde` and `serde_json` for event storage.
- `chrono` or `time` for timestamps.
- `anyhow` for application errors.
- `thiserror` for typed core errors if needed.
- `dirs` for data directory resolution.
- `uuid` for event/snapshot ids.
- `owo-colors` or `colored` for readable terminal output.

## v0.1 Repository Structure

```text
root/
├── Cargo.toml
├── crates/
│   ├── root-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── doctor.rs
│   │       │   ├── history.rs
│   │       │   ├── install.rs
│   │       │   └── rollback.rs
│   │       └── output.rs
│   │
│   └── root-core/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── nix.rs
│           ├── events.rs
│           ├── snapshots.rs
│           ├── doctor.rs
│           ├── packages.rs
│           └── paths.rs
│
├── docs/
│   ├── PRD.md
│   ├── TECH_SPEC.md
│   └── ROADMAP.md
│
└── README.md
```

For speed, this can initially be a single crate, but the workspace structure is recommended if the repo is still early.

## Data Directory

Root stores local state under:

```text
~/.root/
├── events.jsonl
├── snapshots/
│   └── <snapshot-id>.json
├── profiles/
│   └── default/
└── root.lock
```

Alternative macOS-friendly path:

```text
~/Library/Application Support/root/
```

For v0.1, `~/.root` is simpler and easier to explain.

## Event Store

Use JSON Lines for v0.1.

File:

```text
~/.root/events.jsonl
```

Each line is one event.

Example install event:

```json
{
  "id": "evt_01HX...",
  "timestamp": "2026-06-03T14:22:00Z",
  "type": "install",
  "command": "root install ffmpeg",
  "package": "ffmpeg",
  "status": "verified",
  "snapshot_id": "snap_01HX...",
  "details": {
    "binary": "ffmpeg",
    "verify_command": "ffmpeg -version"
  }
}
```

Example rollback event:

```json
{
  "id": "evt_01HY...",
  "timestamp": "2026-06-03T14:31:00Z",
  "type": "rollback",
  "command": "root rollback",
  "status": "completed",
  "restored_snapshot_id": "snap_01HX..."
}
```

## Snapshot Model

v0.1 snapshots should represent Root-managed package state, not the entire OS.

Snapshot file:

```text
~/.root/snapshots/snap_01HX.json
```

Example:

```json
{
  "id": "snap_01HX...",
  "timestamp": "2026-06-03T14:22:00Z",
  "managed_packages": [
    {
      "name": "ffmpeg",
      "nix_attr": "nixpkgs#ffmpeg",
      "version": null,
      "binary": "ffmpeg"
    }
  ]
}
```

For v0.1, rollback can restore the previous Root-managed package list and rebuild the Root profile.

## Nix Integration

v0.1 should shell out to the Nix CLI instead of using Nix internals.

Required checks:

```bash
nix --version
```

Install flow can use one of two strategies.

### Strategy A: Nix Profile

Use:

```bash
nix profile install nixpkgs#ffmpeg --profile ~/.root/profiles/default
```

Rollback can rebuild the Root profile based on previous snapshot state.

### Strategy B: Generated Profile Environment

Root maintains a `root.lock` and runs Nix commands based on the full desired state.

For v0.1, Strategy A is faster.

## Package Allowlist

v0.1 should intentionally support only an allowlist.

```rust
pub struct PackageSpec {
    pub name: &'static str,
    pub nix_attr: &'static str,
    pub binary: &'static str,
    pub verify_args: &'static [&'static str],
}

pub const SUPPORTED_PACKAGES: &[PackageSpec] = &[
    PackageSpec {
        name: "ffmpeg",
        nix_attr: "nixpkgs#ffmpeg",
        binary: "ffmpeg",
        verify_args: &["-version"],
    },
];
```

If the user runs:

```bash
root install postgres
```

Root should respond:

```text
Root v0.1 currently supports:
  ffmpeg

More packages are coming soon.
```

## Command Behavior

### `root doctor`

Checks:

1. Nix CLI exists.
2. Root data directory exists or can be created.
3. Events file is writable.
4. Snapshot directory is writable.
5. Root profile directory exists or can be created.
6. Supported package metadata is available.

Exit codes:

- `0` healthy
- `1` unhealthy

### `root install ffmpeg`

Flow:

1. Validate package is supported.
2. Run doctor preflight.
3. Load current managed state.
4. Create pre-install snapshot.
5. Run Nix profile install.
6. Verify binary.
7. Record install event.
8. Print summary.

### `root history`

Flow:

1. Read events JSONL.
2. Sort descending by timestamp.
3. Print concise timeline.
4. If no events, print empty state.

### `root rollback`

Flow:

1. Find latest usable snapshot before latest install.
2. Restore Root-managed package state from snapshot.
3. Rebuild profile.
4. Record rollback event.
5. Print summary.

## Exit Codes

Recommended:

```text
0 success
1 general error
2 invalid command usage
3 preflight failed
4 install failed
5 verification failed
6 rollback unavailable
```

## JSON Output

Not required for first public v0.1, but CLI internals should be structured so `--json` can be added quickly.

Recommended hidden/internal design:

- Commands return typed structs.
- Output renderer formats structs to human text.
- Future JSON renderer serializes same structs.

## Error Philosophy

Root must be honest.

If rollback only applies to Root-managed packages, say that.

If Nix failed, say that Root could not complete the install through Nix.

If verification failed, say that install may have completed but Root could not verify it.

## Test Strategy

Unit tests:

- Package allowlist resolution.
- Event serialization/deserialization.
- Snapshot selection for rollback.
- Doctor result rendering.
- History rendering.

Integration tests:

- `root doctor` on mocked environment.
- `root install unsupported-package`.
- Event append/read cycle.
- Snapshot create/read cycle.

Manual smoke test:

```bash
cargo run -- doctor
cargo run -- install ffmpeg
cargo run -- history
cargo run -- rollback
cargo run -- history
```
