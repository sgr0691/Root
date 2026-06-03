# Root v0.1 Implementation Plan

## Release Target

A believable v0.1 that can be publicly demoed and tested by developers.

Do not overbuild.
Do not fake safety.
Do not imply Root manages more than it does.

## Phase 0 — Repo Setup

### Tasks

- Confirm Apache 2.0 license.
- Add Rust workspace.
- Add `crates/root-cli`.
- Add `crates/root-core`.
- Add README with current status.
- Add docs folder.
- Add GitHub Actions for:
  - `cargo fmt`
  - `cargo clippy`
  - `cargo test`

### Done When

```bash
cargo test
cargo fmt --check
cargo clippy
```

all pass.

## Phase 1 — CLI Skeleton

### Commands

```bash
root doctor
root install <package>
root history
root rollback
```

### Tasks

- Add `clap`.
- Define command enum.
- Route commands to handlers.
- Add simple output helpers.
- Add consistent error handling.

### Done When

Each command exists and returns a useful placeholder response.

## Phase 2 — Root Paths and Local State

### Data Paths

Use:

```text
~/.root/
├── events.jsonl
├── snapshots/
└── profiles/default
```

### Tasks

- Implement `RootPaths`.
- Create root directory if missing.
- Create snapshots directory if missing.
- Create profile directory if missing.
- Create events file if missing.
- Add tests for path resolution.

### Done When

`root doctor` can create and verify the local Root state directory.

## Phase 3 — Doctor

### Checks

- Nix installed.
- Root directory writable.
- Events file writable.
- Snapshot directory writable.
- Profile directory writable.
- ffmpeg package spec exists.

### Output

Healthy:

```text
Root doctor

✓ Nix installed
✓ Root data directory writable
✓ History store writable
✓ Snapshot directory writable
✓ Root profile available
✓ ffmpeg supported

Root is ready.
```

Unhealthy:

```text
Root doctor

✗ Nix not found

Root needs Nix to install packages safely.

Install Nix:
  https://nixos.org/download

Then run:
  root doctor
```

### Done When

`root doctor` correctly identifies whether the machine is ready for v0.1.

## Phase 4 — Package Allowlist

### Supported Package

`ffmpeg`

### Tasks

- Add `PackageSpec`.
- Add `SUPPORTED_PACKAGES`.
- Implement package lookup.
- Reject unsupported packages with a clear message.

### Done When

```bash
root install ffmpeg
```

is accepted, and:

```bash
root install postgres
```

prints a helpful unsupported package message.

## Phase 5 — Event Store

### Tasks

- Define `RootEvent`.
- Define event types:
  - install
  - rollback
  - doctor
  - verification_failed
- Append event to JSONL.
- Read events from JSONL.
- Render history timeline.

### Done When

`root history` prints events from local storage.

## Phase 6 — Snapshots

### v0.1 Snapshot Meaning

A snapshot is the Root-managed package state before a change.

It is not a full OS snapshot.

### Tasks

- Define `RootSnapshot`.
- Write snapshots to JSON.
- Track managed packages.
- Select previous snapshot for rollback.
- Add tests for snapshot selection.

### Done When

Root can create a snapshot before installing ffmpeg.

## Phase 7 — Nix Install Wrapper

### Tasks

- Implement process runner.
- Run:

```bash
nix profile install nixpkgs#ffmpeg --profile ~/.root/profiles/default
```

- Capture stdout/stderr.
- Convert failures into clean Root errors.
- Avoid dumping raw Nix logs unless `--debug` is set.

### Done When

`root install ffmpeg` installs ffmpeg into the Root-managed profile.

## Phase 8 — Verification

### Verification Command

```bash
ffmpeg -version
```

### Tasks

- Ensure the Root profile bin path is used.
- Run the verify command.
- Mark event as verified or failed.
- Print verification result.

### Done When

After install, Root verifies ffmpeg successfully.

## Phase 9 — History

### Tasks

- Print install events.
- Print rollback events.
- Print failed verification events.
- Empty state if no history exists.
- Add `--limit` later if needed.

### Done When

`root history` gives a readable timeline.

## Phase 10 — Rollback

### v0.1 Rollback Strategy

Rollback restores the previous Root-managed package set.

For v0.1, acceptable approaches:

1. Remove ffmpeg from the Root profile if previous snapshot did not include it.
2. Rebuild the Root profile from the previous snapshot package list.

### Tasks

- Find latest previous snapshot.
- Restore package list.
- Rebuild profile.
- Record rollback event.
- Print summary.

### Done When

After:

```bash
root install ffmpeg
root rollback
```

ffmpeg is no longer available through Root's managed profile.

## Phase 11 — README and Demo

### README Must Include

- What Root is.
- What Root is not.
- v0.1 limitations.
- Install instructions.
- Demo commands.
- Why Root uses Nix.
- Why no Nix knowledge is needed.
- Roadmap.

### Demo Video Script

```bash
root doctor
root install ffmpeg
ffmpeg -version
root history
root rollback
root history
```

### Done When

A developer can clone the repo, read the README, and understand how to test v0.1.
