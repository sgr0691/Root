# Codex Build Prompt — Root v0.1 Believable Ship

You are building Root v0.1.

Root is a deterministic machine manager for developers and AI coding agents.

Do not build the full future roadmap.
Do not build agent runtime.
Do not build permissions.
Do not build sandboxes.
Do not build desktop.
Do not build cloud.
Do not build team features.

Build the smallest believable v0.1:

```bash
root doctor
root install ffmpeg
root history
root rollback
```

## Product Requirements

Root v0.1 must prove:

1. Root can install `ffmpeg` through Nix.
2. Root can create a snapshot before installation.
3. Root can verify the install.
4. Root can record history events.
5. Root can roll back the latest Root-managed change.
6. Root can diagnose whether the environment is healthy.

## Technical Requirements

Use Rust.

Recommended structure:

```text
crates/
  root-cli/
  root-core/
```

Use:

- clap for CLI parsing
- serde/serde_json for JSONL events and snapshots
- anyhow for error handling
- dirs for home directory resolution
- uuid for event/snapshot ids

Root local state:

```text
~/.root/
  events.jsonl
  snapshots/
  profiles/default
```

## Commands

### root doctor

Checks:

- Nix exists using `nix --version`
- Root data dir writable
- Events file writable
- Snapshots dir writable
- Profile dir writable
- ffmpeg package spec exists

### root install ffmpeg

Flow:

1. Reject unsupported packages.
2. Run doctor preflight.
3. Load current Root-managed package state.
4. Create pre-install snapshot.
5. Install with:

```bash
nix profile install nixpkgs#ffmpeg --profile ~/.root/profiles/default
```

6. Verify:

```bash
ffmpeg -version
```

Prefer the Root profile bin path when verifying.

7. Record install event.
8. Print a concise summary.

### root history

Reads:

```text
~/.root/events.jsonl
```

Prints a readable timeline.

### root rollback

Flow:

1. Find latest previous snapshot.
2. Restore Root-managed package state from that snapshot.
3. Rebuild or update the Root profile.
4. Record rollback event.
5. Print what was restored.

Important:

Rollback only applies to Root-managed state.
Do not imply full system rollback.

## Package Allowlist

v0.1 only supports:

```text
ffmpeg
```

If the user tries another package, print:

```text
Root v0.1 does not support "<package>" yet.

Supported packages:
  ffmpeg

More packages are coming soon.
```

## Output Style

Root should feel calm and trustworthy.

Do not dump raw Nix output unless in debug mode.

Example install output:

```text
Root install

Package:
  ffmpeg

Snapshot:
  created snap_...

Installing with Nix...
Verifying ffmpeg...

Done.

Changed:
  + ffmpeg

Verified:
  ffmpeg -version succeeded

Rollback:
  root rollback
```

## Tests

Add tests for:

- package resolution
- unsupported package rejection
- event serialization
- event append/read
- snapshot create/read
- doctor output rendering
- rollback snapshot selection

## Acceptance Criteria

The following manual flow must work:

```bash
cargo run -- doctor
cargo run -- install ffmpeg
ffmpeg -version
cargo run -- history
cargo run -- rollback
cargo run -- history
```

Ship only when this works.

## Non-Goals

Do not implement:

- arbitrary package install
- Homebrew migration
- GUI app support
- AGENTS.md
- Claude skill
- Codex integration
- Cursor rules
- permissions
- sandboxes
- sync
- desktop app
- cloud

Focus exclusively on the v0.1 trust loop.
