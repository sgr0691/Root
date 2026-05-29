# Root Implementation Plan

## Build Philosophy

Build Root in narrow, shippable slices.

The MVP should not attempt to replace Brew entirely. It should prove the core promise:

> Install a CLI tool safely, lock machine state, verify it, and roll back.

## Phase 0 — Repo Foundation

### Goals

- Create Rust workspace.
- Establish CLI skeleton.
- Add docs and examples.
- Add CI.

### Tasks

- Create Cargo workspace.
- Add `root-cli` binary crate.
- Add placeholder crates for core modules.
- Add `clap` command structure.
- Add `--json` global flag.
- Add baseline tests.
- Add GitHub Actions for format, clippy, and test.

### Acceptance Criteria

- `cargo test` passes.
- `root --help` works.
- `root doctor --json` returns valid JSON placeholder.

## Phase 1 — Root State and Config

### Goals

- Create `~/.root` directory structure.
- Implement Rootfile and root.lock read/write.
- Define typed schemas.

### Tasks

- Implement `root init`.
- Implement Rootfile parser/writer.
- Implement root.lock parser/writer.
- Add schema versioning.
- Add hash helpers.

### Acceptance Criteria

- `root init` creates expected directory structure.
- Rootfile can be generated and parsed.
- root.lock can be generated and parsed.

## Phase 2 — Nix Adapter

### Goals

- Detect Nix.
- Invoke Nix safely.
- Normalize Nix errors.

### Tasks

- Implement `NixAdapter` trait.
- Implement shell command executor.
- Add `nix --version` detection.
- Add package search/resolve function.
- Add install/remove/list wrappers.
- Add mock adapter for tests.

### Acceptance Criteria

- Root can detect missing/present Nix.
- Tests can run without real Nix using mock adapter.
- Nix errors are normalized into Root errors.

## Phase 3 — Install, List, Remove

### Goals

- Install CLI packages through Root.
- Track installed packages.
- Remove packages.

### Tasks

- Implement `root plan install <pkg>`.
- Implement `root install <pkg>`.
- Implement `root list`.
- Implement `root remove <pkg>`.
- Update Rootfile/root.lock after successful operations.
- Add human and JSON output.

### Acceptance Criteria

- `root install poppler` installs through Nix.
- `root list` shows Root-managed packages.
- `root remove poppler` removes it.
- JSON output is valid.

## Phase 4 — Snapshots and Rollback

### Goals

- Create snapshots before mutations.
- Roll back to previous Root-managed state.

### Tasks

- Implement snapshot metadata.
- Create snapshot before install/remove.
- Implement `root history`.
- Implement `root rollback --last`.
- Implement rollback by reconciling previous lockfile state.

### Acceptance Criteria

- Every install creates a snapshot.
- `root history` lists snapshots.
- `root rollback --last` restores previous state.

## Phase 5 — Doctor and Drift Detection

### Goals

- Detect machine drift.
- Detect PATH conflicts.
- Detect missing binaries.

### Tasks

- Implement `root doctor`.
- Check Nix status.
- Check Root directory status.
- Check Root lockfile status.
- Compare expected packages to actual profile.
- Detect PATH shadowing with `which`.
- Add clear fix suggestions.

### Acceptance Criteria

- Healthy machine reports healthy.
- Drift produces non-zero exit code if requested.
- PATH shadowing is reported.

## Phase 6 — Verification

### Goals

- Verify installed tools are usable.

### Tasks

- Implement `root verify <pkg>`.
- Add package binary metadata.
- Add generic verification strategy.
- Add package-specific overrides for common tools.

### Acceptance Criteria

- `root verify poppler` verifies `pdftotext` and `pdfinfo`.
- Failed verification returns machine-readable error.

## Phase 7 — Brew Import

### Goals

- Help users migrate incrementally.

### Tasks

- Detect Brew.
- Read formula and cask lists.
- Map formula names to nixpkgs attributes.
- Generate migration report.
- Generate `Rootfile.import` candidate.

### Acceptance Criteria

- `root import brew` never mutates packages.
- CLI packages are separated from unsupported casks.
- User receives useful migration report.

## Phase 8 — Agent Skill Pack

### Goals

- Make Root seamless with AI coding agents.

### Tasks

- Create `skills/codex/AGENTS.md`.
- Create `skills/claude/SKILL.md`.
- Create `skills/cursor/root.mdc`.
- Create generic Root Agent Protocol doc.
- Ensure CLI JSON output supports agent flows.

### Acceptance Criteria

- Agent skill instructs agent to avoid Brew/curl global installs.
- Agent follows plan/install/verify/rollback protocol.
- Skill can be copied into a project or global agent config.

## Phase 9 — Packaging

### Goals

- Make Root easy to install.

### Tasks

- Build release binary.
- Create install script.
- Add checksums.
- Add Homebrew tap only as distribution if desired, but Root should not rely on Brew internally.
- Add GitHub release workflow.

### Acceptance Criteria

- User can install Root with one command.
- `root init` works after install.

## Post-MVP Roadmap

### v0.2 — Project Environments

```bash
root shell
root env init
root env sync
```

### v0.3 — Menu Bar App

- Drift status
- Snapshot timeline
- Rollback button
- Agent activity feed

### v0.4 — Agent Permissions

- Install approvals
- Package allow/deny lists
- Per-agent policies
- Audit log

### v0.5 — Team Sync

- Shared Rootfiles
- Team policy packs
- Remote cache support
