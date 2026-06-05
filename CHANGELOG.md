# Changelog

All notable changes to Root are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-06-05

### Fixed

- **Install script SHA256 verification.** The computed hash included a
  trailing `-` (stdin indicator) because `sha256sum` output was not piped
  through `awk`. Both `sha256sum` and `shasum` are now grouped with parens
  so the pipe always applies.
- **`nix-command` and `flakes` experimental features.** All `nix` CLI
  invocations now automatically pass
  `--extra-experimental-features nix-command flakes`, so Root works on
  fresh Nix installations without manual `nix.conf` configuration.

## [0.1.3] - 2026-06-05

### Added

- **Expanded curated package catalog.** From 4 to 24 packages across four
  categories (`media`, `search`, `dev`, `net`). New packages: fd, bat, eza,
  fzf, git-lfs, gh, httpie, just, tree, sqlite, imagemagick, wget, curl,
  gnumake, pkg-config, openssl, python3, nodejs, bun, uv.
- **Rich `PackageSpec` metadata structure.** Each package now defines
  aliases, Nix attribute, expected binaries, per-binary verification
  commands, category, and description. The catalog lives in a single
  `SUPPORTED_PACKAGES` const slice that is easy to extend.
- **`root catalog` command.** Lists all supported packages grouped by
  category. Supports `--json` for structured output.
- **Better `root plan install`.** The plan command now shows a complete
  step-by-step preview including supported package check, Nix metadata
  resolution, snapshot creation, lockfile update, history event recording,
  and rollback availability. Unsupported packages are rejected before any
  Nix calls.
- **Categorized unsupported-package errors.** The error message for
  unsupported packages now groups packages by category, helping users
  discover alternatives.
- **User-friendly error messages.** Custom error formatter wraps common
  failure modes (Nix missing, package not found, platform missing, stale
  lockfile, rollback unavailable) with clear next-step instructions. Raw
  Nix output is not dumped unless `--json` is used.
- **Verification coverage for all packages.** Every supported package has
  at least one verification command. `root verify <pkg>` checks binaries
  from the Root-managed profile binary path, not the user's global PATH.
  Added `openssl` verification override (uses `version` instead of
  `--version`).
- **Package catalog tests.** Validates: unique names, non-empty Nix
  attributes, at least one binary per package, at least one verify command
  per package, verify binary matches expected binaries, aliases don't
  collide with package names, unsupported packages rejected before Nix
  calls, catalog output includes all packages, resolve-by-alias works.
- **Alias resolution.** `resolve_package` now matches both canonical names
  and aliases (e.g., `rg` resolves to `ripgrep`, `node` to `nodejs`).
- **CHANGELOG.md** — this file.
- **Release smoke test docs.** See `Docs/Release/V0_1_3_SMOKE_TEST.md`.

### Changed

- **README updated.** Full supported package table, "Why curated packages
  first?" explanation, "Try Root in 60 seconds" section, and example flow
  with `root catalog`.
- **`root plan install` output.** Changed `verify_args` to `verify_commands`
  showing per-binary verification (e.g., `ffmpeg -version`).
- **Init/doctor/history output.** Updated to reference `root catalog`
  instead of listing packages inline.
- **Doctor suggestions improved.** Error and warning messages now point to
  concrete first commands (`root init --install-nix`, `root install ffmpeg`).
- **Alias canonicalization.** `root install rg` now correctly installs
  `ripgrep`, `root install node` installs `nodejs`, etc. The lockfile
  stores the canonical name and preserves the original input as
  `requested`.
- **Poppler verification.** Changed from `-h` to `-v` to match actual binary
  behavior.

## [0.1.2] - 2026-06-01

### Added

- **Deterministic Nix metadata resolution.** `nix build --print-out-paths`,
  `nix eval`, and `nix flake metadata --json` capture real Nix store paths,
  package versions, and pinned nixpkgs revision. "latest" is never written
  to root.lock.
- **RootLockV2 schema.** New JSON format with `installable`, `drv_path`,
  `store_paths`, `outputs`, `meta`, and `content_hash` fields.
- **Snapshot v2.** Snapshots store the full RootLockV2 state, enabling
  rollback by locked installable rather than by package name.
- **Rollback by locked state.** `root rollback --last` uses saved
  installables (e.g., `github:NixOS/nixpkgs/<rev>#<attr>`) instead of
  resolving `nixpkgs#<pkg>`.
- **Legacy detection.** `root doctor` detects v1 locks, "latest" versions,
  placeholder store paths, and unknown nixpkgs revisions.
- **Nix metadata verification.** Post-install and post-rollback checks
  that profile store paths match locked store paths.
- **Event recording.** Operations are recorded to `~/.root/events.jsonl`
  with type, status, package, and snapshot IDs.

### Changed

- **Lockfile format.** v1 locks are migrated to v2 on write. The `lock`
  subcommand regenerates deterministic metadata for all Rootfile entries.
- **`sync` refuses v2 lockfiles.** v0.1.2 manages profile state
  automatically during install and rollback; `sync` is deprecated.
- **Test infrastructure.** Deterministic mock Nix adapter produces stable
  store paths, nar hashes, and package versions.

### Fixed

- Placeholder store paths are never written to root.lock.
- Rollback verifies profile store paths match locked paths.
- Nix error normalization catches "attribute missing from derivation" and
  "no outputs found" cases.

## Known Limitations

- Curated catalog only (24 packages). Arbitrary `root install <anything>`
  is not yet supported. Unsupported packages are rejected with a clear
  categorized message.
- Rollback applies only to Root-managed packages. Cannot undo Homebrew or
  manual changes.
- Nix must be pre-installed or installed via `root init --install-nix`.
- Stale lockfile (`~/.root/root.lockfile`) must be deleted manually if Root
  crashes during a mutation.
- macOS only (Apple Silicon and Intel). Linux is detected but not officially
  supported.

## Upgrade Notes

### Upgrading from 0.1.1 to 0.1.2

- Existing v1 lockfiles are automatically migrated to v2 on the next
  `root install` or `root lock` command.
- After upgrading, run `root lock` to regenerate deterministic metadata
  for all packages in Rootfile.
- `root sync` no longer works with v2 lockfiles. Use `root install` and
  `root rollback` instead.

### Upgrading from 0.1.2 to 0.1.3

- No breaking changes. Existing v2 lockfiles, snapshots, and events are
  fully compatible.
- The curated catalog expanded from 4 to 24 packages. Run `root catalog`
  to browse the full list.
- Error messages are now user-friendly. Use `--json` to see raw error
  details if needed.
