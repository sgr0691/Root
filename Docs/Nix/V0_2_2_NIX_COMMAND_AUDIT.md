# Nix Command Audit — Root v0.2.2

**Date:** 2026-06-23  
**Scope:** Every invocation of `nix` CLI commands across the entire codebase  
**Auditor:** Automated codebase analysis

---

## Executive Summary

- **Total distinct nix CLI invocations (Rust):** 12 distinct commands, all routed through a single `run_command()` function in `crates/root-nix/src/lib.rs`
- **Direct shell nix invocations:** 0 (installer script fetches Nix via `curl | sh`, never calls `nix` directly)
- **Total distinct nix subcommands used:** `--version`, `search`, `profile add`, `profile list`, `profile remove`, `eval`, `build`, `path-info`, `flake metadata`
- **Experimental features required:** `nix-command` and `flakes` (always passed via `--extra-experimental-features`)
- **Flakes required:** Yes — all commands use flake-style installables (`nixpkgs#<pkg>`, `github:NixOS/nixpkgs/<rev>#<pkg>`)
- **Key gaps identified:** 4 (see section below)

---

## Invocation Map

All 12 nix commands flow through `RealNixAdapter::run_command()` in **`/Users/sergio/Developer/side-projects/Root/crates/root-nix/src/lib.rs:160-181`**.

### Common structure of every nix invocation

```
nix --extra-experimental-features nix-command flakes <subcommand> [args...] [extra_args...]
```

The `--extra-experimental-features nix-command flakes` flags are **always** prepended. If the user's Nix does not have these enabled, the command will fail with a generic error about experimental features not being enabled — but this is caught by `normalize_error`.

---

## Complete Nix Commands Table

| # | Nix Command | Full Args | Rust File:Line | Trait Method | Root CLI Commands | Expected Stdout | Expected Stderr | Expected Exit Code | Exp Features | Flakes | Common Failure Modes | Error Handling | Gaps |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `nix --version` | `nix --extra-experimental-features nix-command flakes --version` | `root-nix/src/lib.rs:229` | `check_availability()` | `init`, `doctor` | Version string like `nix (Nix) 2.24.0` | (none on success) | 0 | Required | No | Nix not installed → `Command::new()` returns `Err` → `NixError::NotInstalled` | Mapped to `Ok(false)` — non-fatal. If other error, propagated as `Err`. | None |
| 2 | `nix search` | `nix --extra-experimental-features nix-command flakes search nixpkgs <package>` | `root-nix/src/lib.rs:237` | `search(package)` | `search` (search_catalog uses curated list, not nix search directly) | Lines matching `* nixpkgs#<package> (<version>)` | Nix error text on failure | 0 success, 1 failure | Required | Yes (uses `nixpkgs` flake) | Package not found, nixpkgs flake not available, network error | `normalize_error()` parses stderr; NotFound → `NixError::NotFound`, PlatformMissing → `NixError::PlatformMissing`, else `NixError::Generic` | Limited to nixpkgs flake only; no support for custom flakes |
| 3 | `nix profile add` (by package) | `nix --extra-experimental-features nix-command flakes profile add nixpkgs#<pkg> --profile <path>` | `root-nix/src/lib.rs:243-247` | `install(package)` | `install` (legacy fallback from `sync`/`restore`) | (empty on success) | Nix stderr on failure | 0 success, 1 failure | Required | Yes (uses `nixpkgs#` installable) | Package not found, platform not available, profile path issues, experimental features not enabled | `normalize_error()` — see error patterns below | **Gap 1:** RealNixAdapter::install() uses `nixpkgs#` directly — does NOT support pinned installables |
| 4 | `nix profile add` (by installable) | `nix --extra-experimental-features nix-command flakes profile add <installable> --profile <path>` | `root-nix/src/lib.rs:253-257` | `install_installable(package, installable)` | `install`, `update`, `sync`, `restore`, `rollback` | (empty on success) | Nix stderr on failure | 0 success, 1 failure | Required | Yes (accepts any flake installable) | Same as #3 plus: invalid flake ref, network error resolving flake | Same as #3 | **Gap 2:** `--profile` path is assumed valid UTF-8 but has a check via `profile_path_str()` returning an error |
| 5 | `nix profile list` | `nix --extra-experimental-features nix-command flakes profile list --profile <path>` | `root-nix/src/lib.rs:262-263` | `list()` | `list`, `sync` (legacy), `profile_packages()` (fallback) | Text table of profile entries | Nix stderr on failure | 0 success, 1 failure | Required | No (just `--profile`) | Profile does not exist, profile path invalid, experimental features not enabled | `normalize_error()` | None |
| 6 | `nix profile remove` | `nix --extra-experimental-features nix-command flakes profile remove <package_or_index> --profile <path>` | `root-nix/src/lib.rs:268-273` | `remove(package_or_index)` | `remove`, `update`, `sync`, `restore`, `rollback` | (empty on success) | Nix stderr on failure | 0 success, 1 failure | Required | No | Package not in profile, index out of range, profile path invalid | `normalize_error()` | **Gap 3:** No validation of whether the package/index exists before removal — relies on nix error message |
| 7 | `nix profile list --json` | `nix --extra-experimental-features nix-command flakes profile list --json --profile <path>` | `root-nix/src/lib.rs:278-282` | `profile_list_json()` | `doctor`, `status`, `verify_profile_contains_outputs()`, `profile_packages()` | JSON array of profile entries with store paths | Nix stderr on failure | 0 success, 1 failure | Required | No | Profile not found, permission denied | `normalize_error()` | **Gap 4:** JSON output is parsed manually (string matching) not via serde — fragile parsing |
| 8 | `nix flake metadata --json` | `nix --extra-experimental-features nix-command flakes flake metadata --json <flake_ref>` | `root-nix/src/lib.rs:286` | `flake_metadata(flake_ref)` | `install`, `update`, `lock`, `plan` | JSON with `originalUrl`, `lockedUrl`, `rev`, `narHash`, `lastModified` | Nix stderr on failure | 0 success, 1 failure | Required | Yes (requires flakes) | Invalid flake ref, network timeout, no internet, nixpkgs flake not cached | `normalize_error()`; JSON parsed manually | Manual JSON parsing; no `serde` deserialization |
| 9 | `nix eval --json` | `nix --extra-experimental-features nix-command flakes eval --json <installable>.meta` | `root-nix/src/lib.rs:305-308` | `eval_package_metadata(package, pinned_installable)` | `install`, `update`, `lock`, `plan` | JSON with `description`, `homepage`, `license`, etc. | Nix stderr on failure | 0 success, 1 failure | Required | Yes (requires flakes to resolve installable) | Package not found, attribute missing, platform not supported | `normalize_error()`; `eval_json_attr()` swallows `Generic` errors for optional attrs (name/version) | None |
| 10 | `nix build --no-link --print-out-paths --json` | `nix --extra-experimental-features nix-command flakes build --no-link --print-out-paths --json <installable>` | `root-nix/src/lib.rs:326-335` | `build_output_paths(package, pinned_installable)` | `install`, `update`, `lock`, `plan` | JSON array of build plans/drvPath/outputs | Nix stderr on failure | 0 success, 1 failure | Required | Yes (requires flakes) | Build failure, missing dependencies, platform not supported, insufficient disk space | `normalize_error()` | None |
| 11 | `nix eval --raw` | `nix --extra-experimental-features nix-command flakes eval --raw <installable>.drvPath` | `root-nix/src/lib.rs:357-360` | `derivation_path(package, pinned_installable)` | `install`, `update`, `lock`, `plan` | Path string like `/nix/store/<hash>-<name>-<version>.drv` | Nix stderr on failure | 0 success, 1 failure | Required | Yes (requires flakes) | Package not found, attribute not a derivation | `normalize_error()` | Does not validate the returned path is a `.drv` path |
| 12 | `nix path-info --json --closure-size` | `nix --extra-experimental-features nix-command flakes path-info --json --closure-size <path_or_installable>` | `root-nix/src/lib.rs:372-375` | `path_info(path_or_installable)` | `resolve_locked_package()` (used by `install`, `update`, `lock`) | JSON with `path`, `narHash`, `narSize`, `closureSize`, `references` | Nix stderr on failure | 0 success, 1 failure | Required | No (accepts store paths and installables) | Invalid store path, path not in Nix store | `normalize_error()` | None |

---

## Error Normalization Patterns

All errors flow through `RealNixAdapter::normalize_error()` at **`/Users/sergio/Developer/side-projects/Root/crates/root-nix/src/lib.rs:183-211`**.

| Stderr Pattern | Mapped Error | User-Facing Message |
|---|---|---|
| `attribute ... missing from derivation` (any arch) | `NixError::PlatformMissing(pkg)` | "This package is not available for your Mac architecture. Try `root search <pkg>` to find alternatives." |
| `error: no outputs found` | `NixError::NotFound(pkg)` | "Package '<pkg>' not found in nixpkgs" |
| `experimental feature ... is not enabled` | `NixError::Generic(...)` | Instructions to enable `nix-command` and `flakes` in nix.conf |
| `error: reading symbolic link` or `Invalid argument` | `NixError::Generic(...)` | Profile path issue detected; suggests `root doctor` or `rm -rf ~/.root/profiles/default && root init` |
| Anything else | `NixError::Generic(stderr)` | Raw stderr trimmed |

### Exit Code Mapping (from `root-cli/src/main.rs`)

| Exit Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Generic failure / anyhow error |
| 2 | Invalid arguments / unsupported import |
| 3 | Package not found |
| 4 | Verification failed |
| 5 | Drift detected |
| 6 | Rollback failed |
| 7 | Nix unavailable |
| 8 | Platform missing |

---

## Root CLI Command → Nix Subcommand Flow

| Root CLI Command | NixAdapter Methods Called | Nix Subcommands (in order) |
|---|---|---|
| `root init` | `check_availability()` | `nix --version` |
| `root init --install-nix` | `check_availability()` then **`curl \| sh`** (not nix) | `nix --version` then shell Nix installer |
| `root search <q>` | (uses curated catalog, not `nix search`) | — |
| `root plan install <pkg>` | `flake_metadata()`, `eval_package_metadata()`, `build_output_paths()` | `nix flake metadata`, `nix eval --json`, `nix build --no-link --print-out-paths --json` |
| `root install <pkg>` | `flake_metadata()`, `eval_package_metadata()`, `derivation_path()`, `build_output_paths()`, `path_info()`, `install_installable()`, `profile_list_json()` | `nix flake metadata`, `nix eval --json`, `nix eval --raw`, `nix build --no-link --print-out-paths --json`, `nix path-info --json --closure-size`, `nix profile add`, `nix profile list --json` |
| `root list` | `list()` | `nix profile list` |
| `root remove <pkg>` | `remove()` | `nix profile remove` |
| `root update [<pkg>]` | `flake_metadata()`, `resolve_locked_package()` (which calls eval/build/path-info), `remove()`, `install_installable()`, `profile_list_json()` | `nix flake metadata`, `nix eval`, `nix build`, `nix path-info`, `nix profile remove`, `nix profile add`, `nix profile list --json` |
| `root lock` | `flake_metadata()`, `resolve_locked_package()` for each package | `nix flake metadata`, then per package: `nix eval`, `nix build`, `nix path-info` |
| `root sync` | `profile_list_json()`, `list()` (fallback), then per discrepancy: `install_installable()` / `install()` / `remove()`, `profile_list_json()` | `nix profile list --json`, `nix profile list`, `nix profile add`, `nix profile remove`, `nix profile list --json` |
| `root restore [<path>]` | Same as `sync` via `reconcile_profile_to_lock()` | Same as `sync` |
| `root doctor` | `check_availability()`, `profile_list_json()` | `nix --version`, `nix profile list --json` |
| `root status` | `profile_packages()` → `profile_list_json()`, `list()` (fallback) | `nix profile list --json`, `nix profile list` |
| `root rollback --last` | `remove()`, `install_installable()`, `profile_list_json()` | `nix profile remove`, `nix profile add`, `nix profile list --json` |

---

## Non-`nix` Binary: Nix Installer in `root-core`

**`/Users/sergio/Developer/side-projects/Root/crates/root-core/src/lib.rs:863-877`**

```rust
pub fn install_nix() -> Result<()> {
    let status = std::process::Command::new("sh")
        .args(["-c", "curl -L https://nixos.org/nix/install | sh"])
        .status()
        .context("Failed to run Nix installer")?;
    if !status.success() {
        Err(anyhow::anyhow!("Nix installer exited with code {:?}", status.code()))
    } else {
        Ok(())
    }
}
```

| Aspect | Detail |
|---|---|
| File | `crates/root-core/src/lib.rs:863` |
| Root CLI command | `root init --install-nix` |
| Command | `sh -c "curl -L https://nixos.org/nix/install \| sh"` |
| Expected exit code | 0 |
| Stdout | Untracked (only captures status, not output) |
| Stderr | Untracked (only captures status, not output) |
| Error handling | Any non-zero exit → generic anyhow error with code |
| Failure modes | Network failure, curl not installed, installer script failure, permission denied |
| **Gap** | No stdout/stderr captured for diagnostics; no --version pinning; uses the official Nix installer URL (not Determinate Systems like `scripts/install.sh` does) |

---

## Shell Script Nix Handling

### `scripts/install.sh`
- **No nix binary commands called** — only `command -v nix` for availability checking (line 114, 193)
- Nix installation uses Determinate Systems installer: `https://install.determinate.systems/nix` (line 164)
- **Notable:** The shell installer uses a DIFFERENT Nix installer (Determinate Systems) than the Rust `install_nix()` function (official Nix installer). This is an inconsistency.

### `scripts/test-install.sh`
- Only `command -v nix` on line 97 for test skip logic

### `scripts/build-release.sh`
- No Nix commands — pure `cargo build` cross-compilation

---

## Key Gaps Identified

### Gap 1: `RealNixAdapter::install()` does not support pinned installables
- **File:** `crates/root-nix/src/lib.rs:240-248`
- **Issue:** The `install()` method always uses `nixpkgs#<package>` instead of a pinned flake ref. This is only called from legacy paths (`sync_legacy_lock`). The main install path uses `install_installable()` instead.
- **Impact:** Low (legacy code path only).

### Gap 2: Profile path UTF-8 validation
- **File:** `crates/root-nix/src/lib.rs:154-158`
- **Issue:** `profile_path_str()` validates UTF-8 but returns a `NixError::Generic`, not a typed error.
- **Impact:** Low (home dirs are almost always UTF-8).

### Gap 3: No pre-removal validation
- **File:** `crates/root-nix/src/lib.rs:266-273`
- **Issue:** `remove()` doesn't check if the package/index exists in the profile before calling `nix profile remove`. Relies on Nix error messages.
- **Impact:** Low (Nix handles this gracefully).

### Gap 4: Manual JSON parsing
- **File:** `crates/root-nix/src/lib.rs:629-699`
- **Issue:** All JSON output from nix commands is parsed with hand-rolled string parsing functions (`json_field_string`, `json_field_u64`, `json_array_strings`, `json_store_paths`, `extract_json_strings`), not with `serde_json`.
- **Impact:** Medium — parsing is fragile against Nix output format changes. If Nix's JSON output format changes even slightly (whitespace, field ordering, escaping), the custom parser could silently return wrong data or `None`.
- **Mitigation:** Tests exist for the JSON helpers, but no integration tests verify against real Nix output.

### Gap 5: Inconsistent Nix installer URLs
- **File:** `crates/root-core/src/lib.rs:865` vs `scripts/install.sh:164`
- **Issue:** Rust code uses `curl -L https://nixos.org/nix/install | sh` (official Nix installer), while the shell installer uses `https://install.determinate.systems/nix` (Determinate Systems installer).
- **Impact:** Medium — different installation behavior on the user's machine depending on which path they take. The Determinate Systems installer is more robust (uninstall support, SELinux support, etc.).

### Gap 6: No stdout/stderr capture from Nix installer
- **File:** `crates/root-core/src/lib.rs:864-866`
- **Issue:** `install_nix()` uses `.status()` instead of `.output()`, discarding all stdout/stderr from the Nix installation process.
- **Impact:** Low — on failure, the user only gets an exit code, making debugging difficult.

### Gap 7: `nix eval --raw` .drvPath not validated
- **File:** `crates/root-nix/src/lib.rs:357-360`
- **Issue:** The derivation path returned by `nix eval --raw` is not validated to be a `.drv` path. If Nix returns garbage, it will be used as-is.
- **Impact:** Low (Nix is unlikely to return an invalid path).

### Gap 8: `PlatformMissing` error message hardcoded for macOS
- **File:** `crates/root-nix/src/lib.rs:10-11`
- **Issue:** The `PlatformMissing` error message says "for your Mac architecture". This is incorrect on Linux.
- **Impact:** Low (Linux is not officially supported yet).

---

## Summary

| Metric | Value |
|---|---|
| Total unique nix commands | 12 |
| Files containing nix command invocations | 1 Rust file (`crates/root-nix/src/lib.rs`) |
| Files calling the NixAdapter | 3 (`root-cli`, `root-core`, `root-doctor`) |
| Shell scripts checking for nix | 2 (`install.sh`, `test-install.sh`) |
| Shell scripts calling nix commands | 0 |
| CI workflows calling nix | 0 (no .github directory) |
| Experimental features required | Always (`nix-command` + `flakes`) |
| Flakes required | Yes (for most commands) |
| Key gaps identified | 8 |
| High-severity gaps | 1 (Gap 4: manual JSON parsing) |
| Medium-severity gaps | 1 (Gap 5: inconsistent installer URLs) |
| Low-severity gaps | 6 (Gaps 1-3, 6-8) |

---

## File Locations Reference

| File | Relevant Lines | Purpose |
|---|---|---|
| `crates/root-nix/src/lib.rs` | 160-181 | `run_command()` — single entry point for all nix calls |
| `crates/root-nix/src/lib.rs` | 183-211 | `normalize_error()` — error handling for all nix calls |
| `crates/root-nix/src/lib.rs` | 227-391 | All `RealNixAdapter` trait impl methods |
| `crates/root-nix/src/lib.rs` | 629-699 | Manual JSON parser helpers |
| `crates/root-core/src/lib.rs` | 863-877 | `install_nix()` — Nix installer (curl pipe to sh) |
| `crates/root-core/src/lib.rs` | 879-893 | `init()` → `check_availability()` |
| `crates/root-core/src/lib.rs` | 954-969 | `locked_installable_for()` → `flake_metadata()` |
| `crates/root-core/src/lib.rs` | 1117-1142 | `verify_profile_contains_outputs()` → `profile_list_json()` |
| `crates/root-core/src/lib.rs` | 1330-1409 | `install()` → multiple nix calls |
| `crates/root-core/src/lib.rs` | 1452-1519 | `update()` → multiple nix calls |
| `crates/root-core/src/lib.rs` | 1592-1607 | `list()` → `list()` |
| `crates/root-core/src/lib.rs` | 1609-1645 | `remove()` → `remove()` |
| `crates/root-core/src/lib.rs` | 1659-1797 | `rollback_last()` → remove + install + verify |
| `crates/root-core/src/lib.rs` | 1799-1817 | `doctor()` → check_availability + profile_list_json |
| `crates/root-core/src/lib.rs` | 1896-2192 | `lock()` → flake_metadata + resolve_locked_package |
| `crates/root-core/src/lib.rs` | 2195-2212 | `profile_packages()` → profile_list_json + list |
| `crates/root-core/src/lib.rs` | 2243-2359 | `reconcile_profile_to_lock()` → install/remove/list |
| `crates/root-core/src/lib.rs` | 2361-2462 | `sync()` → reconcile_profile_to_lock |
| `crates/root-core/src/lib.rs` | 2464-2496 | `restore()` → reconcile_profile_to_lock |
| `crates/root-core/src/lib.rs` | 2621-2725 | `status()` → profile_packages |
| `crates/root-doctor/src/lib.rs` | 32-491 | `run_diagnostics()` → check_availability + profile_list_json |
| `crates/root-cli/src/main.rs` | 317-323 | `RealNixAdapter::new()` — adapter creation |
| `scripts/install.sh` | 113-200 | Nix detection and installation (Determinate Systems installer) |
| `scripts/test-install.sh` | 96-101 | Nix availability check for test skip |
