# Root — Agent Guide

Root is a deterministic package manager (built on Nix, no Nix knowledge required).  
Binary: `crates/root-cli/src/main.rs` → produces `root` CLI.

## Commands

```bash
cargo build
cargo test --all                          # all workspace tests
cargo test -p root-core                   # single crate
cargo fmt --all -- --check                # formatting check (no config, uses defaults)
cargo clippy --all-targets --all-features -- -D warnings
```

CI order: `fmt` → `clippy` → `test` (`.github/workflows/ci.yml`).

Release: tag `v*` triggers cross-compile for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`.

## Testing quirks

- `MockNixAdapter` (in `root-nix`) lets unit tests run without real Nix.
- Tests that set `ROOT_DIR` env var **must** be serialized with `TEST_MUTEX: Mutex<()>` (see `root-core/src/lib.rs`). Cross-crate test pollution risk.
- Mock recognizes special packages: `"missing_pkg"` → `NotFound`, `"bad_platform_pkg"` → `PlatformMissing`.

## Architecture

8 crates. No async (all `std::process::Command`).

```
root-cli (bin) → root-core → root-lockfile, root-nix, root-snapshot, root-doctor, root-verify
root-agent                   # defined but NOT wired into CLI yet
```

`NixAdapter` trait in `root-nix` has two impls: `RealNixAdapter` (shells out to `nix` CLI) and `MockNixAdapter` (in-memory).

## CLI

Every command supports `--json` flag for structured output. Exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic failure |
| 2 | Invalid arguments / unsupported import |
| 3 | Package not found |
| 4 | Verification failed |
| 5 | Drift detected |
| 6 | Rollback failed |
| 7 | Nix unavailable |
| 8 | Platform missing |

## Config

- `~/.root/Rootfile` — user TOML (package → version mappings)
- `~/.root/root.lock` — auto-generated JSON lockfile
- `~/.root/snapshots/` — JSON snapshot files

## Design docs

`Docs/Core/` has the full spec chain (PRD → TECH_SPEC → ARCHITECTURE → UX_FLOWS → IMPLEMENTATION_PLAN → AGENT_SKILL_SPEC).

## Skill packs

`skills/` contains agent onboarding files for Codex (`skills/codex/AGENTS.md`), Claude (`skills/claude/SKILL.md`), Cursor (`skills/cursor/root.mdc`), and generic agents (`skills/generic/ROOT_AGENT_PROTOCOL.md`). These teach agents to use Root instead of Brew/curl for system tool installs.
