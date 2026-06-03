# Root v0.1 Architecture

## High-Level Architecture

```text
User
 ↓
Root CLI
 ↓
Root Core
 ├─ Doctor
 ├─ Package Allowlist
 ├─ Snapshot Store
 ├─ Event Store
 ├─ Nix Runner
 └─ Verification
 ↓
Nix CLI
 ↓
Root Managed Profile
```

## Root Managed State

Root v0.1 manages only its own profile and state directory.

It does not claim to manage the whole operating system.

```text
~/.root/
├── events.jsonl
├── snapshots/
├── profiles/
│   └── default/
└── root.lock
```

## Root CLI Layer

Responsibilities:

- Parse commands.
- Call root-core.
- Render human output.
- Map errors to useful messages.
- Return consistent exit codes.

## Root Core Layer

Responsibilities:

- Detect environment readiness.
- Resolve supported package specs.
- Create snapshots.
- Append events.
- Invoke Nix.
- Verify installs.
- Restore snapshots.

## Nix Runner

The Nix runner shells out to the Nix CLI.

v0.1 does not use Nix internals.

Example:

```bash
nix profile install nixpkgs#ffmpeg --profile ~/.root/profiles/default
```

## Snapshot Store

Snapshots represent Root-managed package state.

Example:

```json
{
  "id": "snap_01HX",
  "timestamp": "2026-06-03T14:22:00Z",
  "managed_packages": []
}
```

After installing ffmpeg:

```json
{
  "id": "snap_01HY",
  "timestamp": "2026-06-03T14:25:00Z",
  "managed_packages": [
    {
      "name": "ffmpeg",
      "nix_attr": "nixpkgs#ffmpeg",
      "binary": "ffmpeg"
    }
  ]
}
```

## Event Store

Events explain what happened.

Events are stored as JSONL to make v0.1 simple, inspectable, and easy to debug.

## Rollback Architecture

Rollback uses snapshots to restore Root-managed state.

Important limitation:

> Root rollback only applies to Root-managed packages.

This must be clear in README and CLI output.

## Why This Architecture Is Enough

v0.1 does not need:

- SQLite
- Daemons
- Background services
- Desktop app
- Cloud sync
- Permissions engine

Those would slow down the first credible release.

The first release only needs a trustworthy local loop.
