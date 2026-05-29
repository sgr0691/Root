# Root Architecture

## High-Level Architecture

```txt
┌──────────────────────────────┐
│ Human Developer              │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Root CLI                     │
│ human output + JSON output   │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Root Core Engine             │
│ - command orchestration      │
│ - state management           │
│ - error normalization        │
└───────┬───────────┬──────────┘
        │           │
        ▼           ▼
┌──────────────┐ ┌────────────────┐
│ Root Locking │ │ Root Snapshots │
└──────┬───────┘ └───────┬────────┘
       │                 │
       ▼                 ▼
┌──────────────────────────────┐
│ Root Nix Adapter             │
│ controlled Nix CLI execution │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Nix / nixpkgs                │
└──────────────────────────────┘
```

## Agent Architecture

```txt
┌──────────────────────────────┐
│ AI Coding Agent              │
│ Codex / Claude / Cursor      │
└──────────────┬───────────────┘
               │ reads
               ▼
┌──────────────────────────────┐
│ Root Skill / AGENTS.md       │
│ Safe Install Protocol        │
└──────────────┬───────────────┘
               │ runs
               ▼
┌──────────────────────────────┐
│ Root CLI --json              │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Plan → Install → Verify      │
│ Rollback on failure          │
└──────────────────────────────┘
```

## MVP Runtime Flow

```txt
root install poppler
        │
        ▼
Check platform support
        │
        ▼
Check Nix availability
        │
        ▼
Resolve package in nixpkgs
        │
        ▼
Create pre-install snapshot
        │
        ▼
Run Nix install through adapter
        │
        ▼
Update Rootfile/root.lock
        │
        ▼
Verify package binaries
        │
        ▼
Print impact report
```

## Rollback Flow

```txt
root rollback --last
        │
        ▼
Find latest snapshot
        │
        ▼
Compare current state to snapshot
        │
        ▼
Generate rollback plan
        │
        ▼
Reconcile Nix profile
        │
        ▼
Restore Rootfile/root.lock metadata
        │
        ▼
Run doctor check
        │
        ▼
Print rollback summary
```

## Doctor Flow

```txt
root doctor
        │
        ▼
Check Root config directory
        │
        ▼
Check Nix install
        │
        ▼
Check Root-managed profile
        │
        ▼
Check PATH order
        │
        ▼
Compare lockfile to actual binaries
        │
        ▼
Detect unmanaged shadows
        │
        ▼
Emit health report
```

## Component Responsibilities

### root-cli

- Parses CLI args
- Chooses human or JSON renderer
- Handles terminal UX
- Delegates to root-core

### root-core

- Orchestrates commands
- Owns business logic
- Normalizes errors
- Defines typed command result structs

### root-nix

- Wraps Nix CLI commands
- Resolves packages
- Installs/removes packages
- Parses Nix responses where needed

### root-lockfile

- Reads/writes Rootfile
- Reads/writes root.lock
- Computes hashes
- Validates schemas

### root-snapshot

- Creates snapshots before mutations
- Lists history
- Restores snapshots

### root-doctor

- Checks drift
- Checks PATH conflicts
- Checks missing packages
- Checks unsupported platform states

### root-verify

- Finds installed binaries
- Runs verification commands
- Returns success/failure metadata

### root-agent

- Defines JSON contracts
- Houses agent protocol helpers
- May generate skill files in future versions

## Future Menu Bar Architecture

```txt
┌──────────────────────────────┐
│ Root Menu Bar App            │
└──────────────┬───────────────┘
               │ invokes
               ▼
┌──────────────────────────────┐
│ Root CLI / Local Daemon      │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Root State + Snapshots       │
└──────────────────────────────┘
```

The menu bar app is post-MVP. It should not be required for Root v0.1.
