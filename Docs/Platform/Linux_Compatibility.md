# Linux Compatibility Investigation

**Status:** Investigation — not a support announcement.

**Root v0.1.9** is officially macOS-only (Apple Silicon and Intel).  
This document investigates what works on Linux today and what would need to change before Linux could be supported.

---

## What Works Today on Linux

### Cargo build
- `cargo build`, `cargo test --all`, `cargo clippy`, `cargo fmt` all pass on `x86_64-linux` and `aarch64-linux`.

### Platform detection
- `root_lockfile::detect_platform()` in `crates/root-lockfile/src/lib.rs:525-538` correctly handles:
  - `("linux", "aarch64")` → `"aarch64-linux"`
  - `("linux", "x86_64")` → `"x86_64-linux"`
- These platform strings are written into root.lock.

### Nix integration
- `root-nix` runs `nix` CLI commands with `--extra-experimental-features nix-command flakes`.
- All Nix CLI commands (`nix build`, `nix profile add`, `nix profile list`, `nix eval`, `nix path-info`, `nix flake metadata`) work identically on Linux.
- The `NormalizeError` path for `"aarch64-darwin"` in `crates/root-nix/src/lib.rs:179` has a Linux counterpart (`"x86_64-linux"` / `"aarch64-linux"`) — the error message text would differ but the pattern match `"attribute" + "missing from derivation"` catches all arches.

### Mock adapter
- `MockNixAdapter` is platform-agnostic. All unit tests pass on Linux.

### Lockfile schema
- Lockfile is JSON — platform-independent.
- Drv paths, store paths, and output paths use `/nix/store/...` paths which are the same on both platforms.

---

## What Is macOS-Specific

### Homebrew import (`crates/root-core/src/brew.rs`)
- `root import brew` shells out to `brew list --formula` and `brew list --cask`.
- This is macOS-only by definition. On Linux, `brew` won't be installed.
- The code handles this gracefully: if `brew --version` fails, it returns `brew_detected: false`.

### Nix installer (`install_nix()` in `crates/root-core/src/lib.rs:690-704`)
- Runs `curl -L https://nixos.org/nix/install | sh`.
- The official Nix installer handles Linux and macOS, so this is **not** macOS-specific.
- However, the installer URL is the multi-user installer which works on both.

### Profile path default (`~/.root/profiles/default`)
- Default profile path is derived from `$HOME/.root/profiles/default`.
- `dirs::home_dir()` resolves correctly on Linux.
- This is **not** macOS-specific.

### Binary path resolution in root-verify
- `resolve_binary_path` checks `~/.root/profiles/default/bin` and the profile's `bin/` subdirectory.
- On Linux, Nix profile store paths are the same format (`/nix/store/<hash>-<name>-<version>/bin/<binary>`).
- This is **not** macOS-specific.

### Doctor PATH checks
- `root doctor` checks if `~/.root/profiles/default/bin` is in `$PATH`.
- PATH format differs (`:` separator on both; but macOS also has `/usr/bin:/bin:/usr/sbin:/sbin`).
- The split/join logic uses `std::env::split_paths` / `std::env::join_paths` which are cross-platform.
- This is **not** macOS-specific.

---

## Assumptions That Depend on macOS

### Test infrastructure
- `#[cfg(unix)]` is used in test code for `PermissionsExt::set_mode` — this works on Linux too.
- All tests use `setup_test_home` which sets `HOME` and `ROOT_DIR`.
- **No test assumes macOS.**

### Nix profile JSON format
- `nix profile list --json` output format is the same on both platforms.
- The `parse_profile_entries` function handles both `{ "elements": [...] }` and `[...]` formats.

### Store path pattern
- All mock store paths use `/nix/store/<hash>-<name>-<version>`.
- This is the same on both platforms.

---

## What Would Need to Change Before Linux Is Supported

### 1. Smoke test on real Linux
- Run the full smoke test suite on `x86_64-linux` and `aarch64-linux`.
- CI would need Linux runners (GitHub Actions `ubuntu-latest` and `arm64` runners).

### 2. Nix installer on Linux
- `root init --install-nix` uses `curl -L https://nixos.org/nix/install | sh`.
- This works on Linux but the installer is different (multi-user install on systemd vs macOS launchd).
- The installer works on both platforms but the post-install steps differ (daemon reload, service start).

### 3. Profile path differences
- On macOS, Nix stores its own profiles in `/nix/var/nix/profiles/`.
- Root uses `~/.root/profiles/default` as a `--profile` argument.
- This works on Linux too; `nix profile add --profile ~/.root/profiles/default` is cross-platform.

### 4. Error message normalization
- `normalize_error` currently checks for `"aarch64-darwin"` in `"attribute 'aarch64-darwin' missing from derivation"`.
- On Linux the error would contain `"x86_64-linux"` or `"aarch64-linux"` instead.
- The current check looks for `"attribute" && "missing from derivation"` which catches all architectures.
- **Already works on Linux.** No change needed.

### 5. Event system
- Events are written to `~/.root/events.jsonl`.
- The `chrono` crate used for timestamps works on Linux.
- **No change needed.**

### 6. `root doctor` checks
- Doctor checks for `~/.root/profiles/default/bin` in `$PATH`.
- On Linux, PATH may not include `~/.root/profiles/default/bin` — same as on macOS.
- This check works identically on both platforms.
- **No change needed.**

---

## Recommended Tests to Pass Before Linux Support

### Unit tests (all pass today)
- `cargo test --all` on `x86_64-unknown-linux-gnu`
- `cargo test --all` on `aarch64-unknown-linux-gnu`

### Integration tests (need Linux machine)
1. `rm -rf ~/.root && root doctor` — no crash, detects missing Root dir
2. `root init` — creates directories successfully
3. `root init --install-nix` — installs Nix
4. `root doctor` — reports healthy
5. `root install ffmpeg` — installs and creates lockfile
6. `root verify ffmpeg` — verifies binary
7. `root history` — shows events
8. `root rollback --last` — restores previous state
9. `root install ripgrep` — second package install
10. `root list` — shows managed packages

### Edge cases
- Nix not installed (should give clear error)
- `$HOME` not writable
- System with only `x86_64-linux` binary cache
- `aarch64-linux` (Raspberry Pi, AWS Graviton)

---

## Summary

| Area | macOS | Linux | Gap |
|------|-------|-------|-----|
| Core Rust code | ✓ | ✓ | None |
| Nix CLI commands | ✓ | ✓ | None |
| Platform detection | ✓ | ✓ | None |
| Lockfile schema | ✓ | ✓ | None |
| Profile paths | ✓ | ✓ | None |
| Verification | ✓ | ✓ | None |
| Doctor checks | ✓ | ✓ | None |
| Homebrew import | ✓ | ✗ | N/A (brew not on Linux) |
| Nix installer | ✓ | ✓ | Untested |
| End-to-end tests | manual | manual | Need Linux CI runner |

**Verdict:** The code is already largely cross-platform. Linux support would require:
1. A CI runner for Linux
2. Manual smoke tests on `x86_64-linux` and `aarch64-linux`
3. Possibly small adjustments to the Nix installer integration path

No architectural changes are needed. The main gap is CI/test coverage, not code.
