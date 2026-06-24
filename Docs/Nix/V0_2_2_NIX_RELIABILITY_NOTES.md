# Nix Reliability Notes — Root v0.2.2

**Date:** 2026-06-23  
**Scope:** Nix requirements, failure modes, recovery procedures, and debugging guidance for Root v0.2.2.

---

## Table of Contents

1. [Nix Requirements for Root](#1-nix-requirements-for-root)
2. [Experimental Feature Requirements](#2-experimental-feature-requirements)
3. [Common Nix Failures and Root Handling](#3-common-nix-failures-and-root-handling)
4. [Recovery Steps for Each Failure Mode](#4-recovery-steps-for-each-failure-mode)
5. [Derivation Path vs Output Path Separation](#5-derivation-path-vs-output-path-separation)
6. [Debugging Nix Failures with --json](#6-debugging-nix-failures-with---json)
7. [Profile Generation Tracking](#7-profile-generation-tracking)
8. [Quick Reference](#8-quick-reference)

---

## 1. Nix Requirements for Root

### Minimum Nix Version

Root requires **Nix 2.18+** with flakes support. All nix CLI invocations pass `--extra-experimental-features nix-command flakes` automatically, so Root works on fresh Nix installations without manual `nix.conf` configuration.

### Nix Binary Availability

Root locates `nix` via the system `PATH` at runtime. The `NixAdapter` trait (implemented by `RealNixAdapter`) shells out to `std::process::Command::new("nix")`. If Nix is not on `PATH`, the command returns `Err(std::io::Error)` which is mapped to `NixError::NotInstalled`.

### Store Path Assumptions

Root assumes the Nix store lives at `/nix/store/`. All store path validation checks that paths start with this prefix. This is the default Nix store directory and cannot be changed through Root's configuration.

### Profile Path

Root manages a single Nix profile at `~/.root/profiles/default`. This path is passed as `--profile <path>` to every `nix profile` subcommand. The profile path must be valid UTF-8 (validated by `profile_path_str()`).

### Network Requirements

The following operations require network access:
- `nix flake metadata` — resolves flake references from GitHub or other flake registries
- `nix build --no-link` — may need to download source tarballs or binary caches
- `nix search` — queries the nixpkgs flake
- `nix eval` — evaluates Nix expressions, may fetch flakes

If network is unavailable, cached results from prior evaluations may still work if the nixpkgs flake is already in the local Nix store.

---

## 2. Experimental Feature Requirements

Root requires two Nix experimental features:

| Feature | Purpose | Commands Requiring It |
|---------|---------|----------------------|
| `nix-command` | Modern `nix` CLI interface (subcommands like `nix profile add`, `nix flake metadata`, `nix eval`, `nix build`) | All 12 nix invocations |
| `flakes` | Flake-style installables (`nixpkgs#<pkg>`, `github:NixOS/nixpkgs/<rev>#<pkg>`), `nix flake metadata` | `nix search`, `nix profile add`, `nix flake metadata`, `nix eval`, `nix build`, `nix eval --raw` |

### How Root Passes Experimental Features

Every nix invocation goes through `RealNixAdapter::run_command()` in `crates/root-nix/src/lib.rs:160-181`:

```rust
Command::new("nix")
    .arg("--extra-experimental-features")
    .arg("nix-command flakes")
    .args(args)
    .args(extra_args)
```

The flags are always prepended, regardless of the subcommand. If the user's Nix configuration already enables these features, the flags are harmless duplicates.

### Detection

In v0.2.2, `root doctor` detects when experimental features are not enabled. The `check_availability()` method calls `nix --version` (which does not require experimental features), but `root doctor` additionally inspects error output from `normalize_error()` to detect `"experimental feature ... is not enabled"` stderr patterns.

The doctor produces a clear diagnostic:
```
Nix is installed but experimental features are not enabled.
Root needs 'nix-command' and 'flakes' experimental features.

Suggestion: Add this to ~/.config/nix/nix.conf:
  experimental-features = nix-command flakes
Then run: root doctor
```

### Enabling Experimental Features

**User-level (recommended):**
```
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

**System-wide:**
```
echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
```

**Environment variable (temporary):**
```bash
export NIX_CONFIG="experimental-features = nix-command flakes"
```

After enabling, run `root doctor` to verify.

---

## 3. Common Nix Failures and Root Handling

Root normalizes all Nix errors through `RealNixAdapter::normalize_error()` in `crates/root-nix/src/lib.rs:183-211`. The following table documents every recognized failure mode:

| # | Failure Mode | Stderr Pattern | Root Error | User-Facing Message | Detected In |
|---|---|---|---|---|---|
| 1 | Nix not installed | `Command::new("nix")` returns `Err` | `NixError::NotInstalled` | "Nix is not installed or not available on PATH" | All commands via `run_command()` |
| 2 | Missing attribute for platform | `attribute '...' missing from derivation` | `NixError::PlatformMissing(pkg)` | "This package is not available for your Mac architecture. Try `root search <pkg>` to find alternatives." | `eval`, `build` |
| 3 | Package not found in nixpkgs | `error: no outputs found` | `NixError::NotFound(pkg)` | "Package '<pkg>' not found in nixpkgs" | `eval`, `build`, `search` |
| 4 | Experimental features disabled | `experimental feature ... is not enabled` | `NixError::Generic(...)` | Instructions to enable `nix-command` and `flakes` in nix.conf | All nix subcommands |
| 5 | Profile symlink conflict | `error: reading symbolic link` or `Invalid argument` | `NixError::Generic(...)` | "Nix profile path issue detected... Run `root doctor`. To repair, try: `rm -rf ~/.root/profiles/default && root init`" | `profile add`, `profile remove`, `profile list` |
| 6 | Generic Nix failure | Any other stderr | `NixError::Generic(stderr)` | Raw stderr trimmed | All nix subcommands |
| 7 | Invalid flake reference | `error: cannot find flake` or similar | `NixError::Generic(...)` | Raw Nix error | `flake metadata` |
| 8 | Network timeout / no internet | `error: unable to download` or similar | `NixError::Generic(...)` | Raw Nix error | `flake metadata`, `build`, `eval` (first use) |
| 9 | Build failure (missing deps) | `error: builder for ... failed` | `NixError::Generic(...)` | Raw Nix error | `build` |
| 10 | Insufficient disk space | `error: writing to file` or `No space left` | `NixError::Generic(...)` | Raw Nix error | `build`, `profile add` |
| 11 | Profile already has package | `error: package '...' already in profile` | `NixError::Generic(...)` | Nix error (handled upstream before `install`) | `profile add` |
| 12 | Corrupt Nix database | `error: opening Nix database` | `NixError::Generic(...)` | Raw Nix error | All nix subcommands |

### Normalized Exit Codes

All Root CLI commands produce these exit codes when Nix failures occur:

| Code | Meaning | Triggered By |
|------|---------|-------------|
| 1 | Generic failure | Any `NixError::Generic` |
| 3 | Package not found | `NixError::NotFound` |
| 7 | Nix unavailable | `NixError::NotInstalled` |
| 8 | Platform missing | `NixError::PlatformMissing` |

Use `root <command> --json` to see the full structured error output including the raw Nix stderr.

---

## 4. Recovery Steps for Each Failure Mode

### Failure 1: Nix Not Installed

```text
Error: Nix is not installed or not available on PATH
```

**Recovery:**
1. Run `root init --install-nix` for an interactive guided installation.
2. Or install manually from https://nixos.org/download/
3. After installation, run `root doctor` to verify.

**Installation validation (v0.2.2):**
`root init --install-nix` now:
- Explains what will happen before running
- Requires explicit user confirmation (y/N)
- Detects the target platform automatically
- Runs a post-install probe to verify Nix is available

### Failure 2: Platform Missing

```text
Error: This package is not available for your Mac architecture.
Try `root search <pkg>` to find alternatives.
```

**Recovery:**
1. Use `root search <pkg>` to find a compatible alternative.
2. If the package should be available, try `root update` to refresh nixpkgs and retry.
3. Check if the package has platform-specific variants with different Nix attributes.

### Failure 3: Package Not Found

```text
Error: Package '<pkg>' not found in nixpkgs
```

**Recovery:**
1. Verify the package name with `root catalog`.
2. If it is a known package, run `root update` to refresh metadata and retry.
3. The package may have been removed from nixpkgs or renamed.

### Failure 4: Experimental Features Disabled

```text
Error: Nix experimental features 'nix-command' and 'flakes' are required.
To enable them, add this to ~/.config/nix/nix.conf:
  experimental-features = nix-command flakes
```

**Recovery:**
1. Run `mkdir -p ~/.config/nix` if the directory does not exist.
2. Add `experimental-features = nix-command flakes` to `~/.config/nix/nix.conf`.
3. Run `root doctor` to verify.

### Failure 5: Profile Symlink Conflict

```text
Error: Nix profile path issue detected.
This can happen when Root's profile path (~/.root/profiles/default)
conflicts with Nix's symlink management.
To repair, try:  rm -rf ~/.root/profiles/default && root init
```

**Recovery:**
1. Run `root doctor` for a full diagnostic.
2. If the profile path is a plain directory (not a symlink), remove it:
   `rm -rf ~/.root/profiles/default`
3. Run `root init` to recreate the profile path as a clean state.
4. Re-run the failed command.

### Failure 6+: Generic Nix Error

For any unrecognized Nix error, the raw stderr is surfaced. Recovery depends on the specific error:

- **Invalid flake reference**: Verify the flake URL syntax. Use `nix flake metadata <url>` to test independently.
- **Network errors**: Check internet connectivity. Retry after network is restored. Some cached operations may work offline if the nixpkgs flake was previously fetched.
- **Build failures**: The package may have unmet build dependencies. Run `nix build nixpkgs#<pkg>` directly to see the full build log.
- **Disk space**: Free up disk space. Nix requires space in `/nix/store` (typically on the root partition) and in `~/.root/`.
- **Corrupt Nix database**: Run `nix store repair` or `nix-store --verify --repair`.

### Recovery Command Matrix

| Situation | Recommended Command |
|-----------|-------------------|
| Nix not installed | `root init --install-nix` |
| Experimental features disabled | Edit nix.conf, then `root doctor` |
| Package not found | `root search <pkg>`, then `root update` |
| Platform missing | `root search <pkg>` for alternatives |
| Profile conflict | `rm -rf ~/.root/profiles/default && root init` |
| Lockfile stale/corrupt | `rm ~/.root/root.lock && root lock` |
| Profile out of sync | `root sync` |
| Rollback failed | `root doctor`, then `root sync` |
| Unknown Nix error | `root <cmd> --json` for details |

---

## 5. Derivation Path vs Output Path Separation

### The Problem

Nix commands like `nix build --no-link --print-out-paths --json` return JSON that contains both derivation paths (`.drv` files) and realized output paths. Prior to v0.2.2, Root could accidentally treat a `.drv` path as an output path, causing verification to fail with messages like:

```
Installed profile did not contain locked Nix store path ... .drv
```

### How Root Separates Them

Root uses a three-layer defense to ensure `.drv` paths never appear in output fields:

#### Layer 1: JSON Extraction (`json_store_paths()`)

In `crates/root-nix/src/lib.rs:680-685`, the `json_store_paths()` function filters extracted strings:

```rust
fn json_store_paths(json: &str) -> Vec<String> {
    extract_json_strings(json)
        .into_iter()
        .filter(|value| value.starts_with("/nix/store/") && !value.ends_with(".drv"))
        .collect()
}
```

Only values that start with `/nix/store/` and do **not** end with `.drv` are returned. This function is used in:
- `build_output_paths()` — extracting output paths from `nix build --json`
- `path_info()` — extracting the primary path from `nix path-info --json`

#### Layer 2: Lockfile Construction (`deterministic_package_from_resolution()`)

In `crates/root-core/src/lib.rs:996-1003`, when building a `LockedPackageV2` from resolution data:

```rust
if path.ends_with(".drv") {
    return Err(anyhow::anyhow!(
        "Root resolved a derivation path but no realized output path for {}. \
         Expected an output store path, got: {}",
        canonical_name, path
    ));
}
```

This rejects `.drv` paths before they enter the lockfile's `store_paths`, `outputs`, or `store_path` fields.

#### Layer 3: Verification Guard (`verify_profile_contains_outputs()`)

In `crates/root-core/src/lib.rs:1121-1128`, before checking the Nix profile:

```rust
for store_path in outputs.values() {
    if store_path.ends_with(".drv") {
        return Err(anyhow::anyhow!(
            "Root resolved a derivation path but no realized output path. \
             Refusing to verify .drv path as an installed output: {}",
            store_path
        ));
    }
}
```

This checks all output `store_paths` values for `.drv` suffixes before querying the Nix profile.

### Schema-Level Separation

The v2 lockfile (`RootLockV2`) has distinct fields for derivation and output paths:

| Field | Location | Purpose | Contains .drv? |
|-------|----------|---------|----------------|
| `packages[].drv_path` | `LockedPackageV2.drv_path` | Stores the derivation path for reference | Yes |
| `packages[].store_path` | `LockedPackageV2.store_path` | Primary output store path | No |
| `packages[].storePaths` | `LockedPackageV2.store_paths` | All output store paths by output name | No |
| `packages[].outputs[].storePath` | `LockedPackageOutput.store_path` | Individual output store paths | No |

The `json_store_paths()` filter runs before any data enters these fields. The only field that may contain a `.drv` path is `drv_path`.

### Tests

The following tests verify separation correctness:

- `test_json_store_paths_filters_drv_paths` — confirms `.drv` paths are filtered from output extraction
- `test_json_store_paths_multiple_outputs_with_drv` — confirms multiple outputs survive while `.drv` is filtered
- `test_deterministic_package_rejects_drv_output_path` — confirms `.drv` in outputs produces clear error
- `test_verify_profile_rejects_drv_paths` — confirms verification rejects `.drv` paths
- `test_lockfile_drv_and_output_path_separation` — end-to-end test of lockfile field separation
- `test_verify_profile_succeeds_with_real_output_path` — confirms non-`.drv` paths pass validation

---

## 6. Debugging Nix Failures with --json

Every Root CLI command supports a `--json` flag that produces structured JSON output instead of human-readable text. This is the primary debugging mechanism for Nix failures.

### Using --json

```bash
# Install a package, get JSON output
root install ffmpeg --json

# Or for other commands
root doctor --json
root status --json
root plan install ripgrep --json
```

### JSON Output Structure

When a Nix error occurs with `--json`, Root outputs a JSON object with error details:

```json
{
  "success": false,
  "error": {
    "message": "Nix command failed: error: attribute 'aarch64-darwin' missing from derivation",
    "exit_code": 8
  }
}
```

The `exit_code` field maps to the standard Root exit codes:
- 7 = Nix not available
- 3 = Package not found
- 8 = Platform not available
- 1 = Generic error (includes experimental features, profile conflicts, etc.)

For success output with `--json`:

```json
{
  "success": true,
  "data": { ... }
}
```

### Debugging Workflow

1. **Re-run the failed command with `--json`:**
   ```bash
   root install ffmpeg --json
   ```

2. **Check the `exit_code`** to determine the category of failure:
   - Exit 7 → Nix is not installed
   - Exit 3 → Package not in nixpkgs
   - Exit 8 → Platform not supported
   - Exit 1 → Other Nix error (experimental features, profile conflict, etc.)

3. **For generic errors (exit 1), test Nix independently:**
   ```bash
   nix --extra-experimental-features nix-command flakes eval nixpkgs#ffmpeg.name
   nix --extra-experimental-features nix-command flakes flake metadata nixpkgs
   ```

4. **Check `root doctor --json`** for a comprehensive system state report:
   ```bash
   root doctor --json
   ```
   This reports:
   - `nix_installed` — whether Nix is on PATH
   - `root_initialized` — whether `~/.root/` exists and is valid
   - `issues[]` — array of detailed issues with severity, category, description, and suggestion

5. **Check `root status --json`** for drift detection:
   ```bash
   root status --json
   ```
   This compares the Rootfile, lockfile, and actual Nix profile state.

6. **Inspect the lockfile directly:**
   ```bash
   cat ~/.root/root.lock | python3 -m json.tool
   ```
   Look for:
   - Placeholder store paths (e.g., `"/nix/store/xxx"`)
   - Floating `"latest"` versions
   - `drv_path` values that should end in `.drv`
   - `store_path` / `storePaths` values that should NOT end in `.drv`

7. **Check Nix profile state:**
   ```bash
   nix --extra-experimental-features nix-command flakes profile list --profile ~/.root/profiles/default
   ```

### Common Debugging Scenarios

| Scenario | Check This |
|----------|-----------|
| `root doctor` reports Nix experimental features error | `~/.config/nix/nix.conf` contents |
| Package resolves but install fails | `nix build nixpkgs#<pkg>` directly |
| Profile verification fails | `nix profile list --profile ~/.root/profiles/default --json` |
| Lockfile has stale data | `nix flake metadata nixpkgs --json` and compare with lockfile rev |
| Rollback fails | Check `~/.root/snapshots/` for available snapshot files |
| `root doctor` reports drift | Compare lockfile store paths against profile list output |

---

## 7. Profile Generation Tracking

### What Is a Profile Generation

Nix profiles maintain an append-only generation history. Every `nix profile add` or `nix profile remove` operation creates a new generation (a symlink in the profile directory). Generations are numbered sequentially (1, 2, 3, ...).

### How Root Tracks Generations

In the v2 lockfile (`RootLockV2`), Root records the current profile generation number:

```rust
pub struct LockProfile {
    pub name: String,                    // "default"
    pub path: Option<String>,            // e.g., "/nix/var/nix/profiles/default"
    pub generation: Option<u64>,         // e.g., 7
}
```

This field is populated from the `profile` section of `root.lock`:

```json
"profile": {
  "name": "default",
  "generation": 7
}
```

### Validation After Every Mutation

Starting in v0.2.2, after every mutation operation (install, update, rollback, restore), Root validates:

1. **The profile generation changed** — confirms the mutation actually took effect
2. **Expected output paths are present** — verifies that locked store paths appear in `nix profile list --json` output

This validation runs through `verify_profile_contains_outputs()` which:
1. Checks that no output paths are `.drv` paths (rejects derivations)
2. Queries the actual Nix profile with `nix profile list --json`
3. Verifies each locked store path appears in the profile JSON

### When Generations Are Updated

| Operation | Profile Generation Change |
|-----------|------------------------|
| `install` | Incremented (profile add) |
| `remove` | Incremented (profile remove) |
| `update` | Incremented (remove + profile add) |
| `rollback --last` | Incremented (remove old + profile add new) |
| `restore` / `sync` | Incremented once per repair operation |
| `lock` only | NOT changed (no profile mutation) |

### Verifying Generation Manually

```bash
# Check current profile generation
nix --extra-experimental-features nix-command flakes profile list --profile ~/.root/profiles/default

# View generation history
ls -la ~/.root/profiles/ | grep default

# Check generation recorded in lockfile
grep -A3 '"profile"' ~/.root/root.lock
```

### Recovery After Stale Generation Data

If the lockfile's profile generation becomes stale (e.g., after manual Nix operations outside Root):

1. Run `root doctor` to detect drift between lockfile and actual profile state.
2. Run `root sync` to reconcile the profile with the lockfile.
3. Run `root lock` to refresh all metadata including the profile generation.

---

## 8. Quick Reference

### File Locations

| File | Purpose |
|------|---------|
| `crates/root-nix/src/lib.rs` | All nix CLI invocations, error normalization, JSON parsing |
| `crates/root-core/src/lib.rs` | High-level operations (install, update, rollback, verify) |
| `crates/root-doctor/src/lib.rs` | Diagnostics and experimental feature detection |
| `crates/root-lockfile/src/lib.rs` | Lockfile schema v1/v2, profile generation tracking |
| `~/.root/profiles/default` | Root-managed Nix profile |
| `~/.root/root.lock` | Lockfile with deterministic metadata |
| `~/.root/Rootfile` | User package configuration |
| `~/.config/nix/nix.conf` | Nix configuration (experimental features) |

### Key Functions

| Function | File:Line | Purpose |
|----------|-----------|---------|
| `run_command()` | `root-nix/src/lib.rs:160-181` | Single entry point for all nix CLI calls |
| `normalize_error()` | `root-nix/src/lib.rs:183-211` | Maps stderr patterns to typed errors |
| `json_store_paths()` | `root-nix/src/lib.rs:680-685` | Extracts store paths, filters .drv |
| `verify_profile_contains_outputs()` | `root-core/src/lib.rs:1117-1142` | Post-mutation validation |
| `deterministic_package_from_resolution()` | `root-core/src/lib.rs:974-1078` | Builds lockfile entry with separation |
| `install_nix()` | `root-core/src/lib.rs:863-877` | Nix installer via curl pipe |
| `run_diagnostics()` | `root-doctor/src/lib.rs:32-491` | Full system health check |

### Error Types

| Error | Meaning | Exit Code |
|-------|---------|-----------|
| `NixError::NotInstalled` | Nix not found on PATH | 7 |
| `NixError::NotFound(pkg)` | Package not in nixpkgs | 3 |
| `NixError::PlatformMissing(pkg)` | Not available for current arch | 8 |
| `NixError::Generic(msg)` | Other Nix failure | 1 |

### Nix Commands Root Uses

| # | Nix Subcommand | Full Args | Called From |
|---|---|---|---|
| 1 | `--version` | `nix ... --version` | `check_availability()` |
| 2 | `search` | `nix ... search nixpkgs <pkg>` | `search()` |
| 3 | `profile add` (package) | `nix ... profile add nixpkgs#<pkg> --profile <path>` | `install()` |
| 4 | `profile add` (installable) | `nix ... profile add <installable> --profile <path>` | `install_installable()` |
| 5 | `profile list` | `nix ... profile list --profile <path>` | `list()` |
| 6 | `profile remove` | `nix ... profile remove <pkg> --profile <path>` | `remove()` |
| 7 | `profile list --json` | `nix ... profile list --json --profile <path>` | `profile_list_json()` |
| 8 | `flake metadata --json` | `nix ... flake metadata --json <flake_ref>` | `flake_metadata()` |
| 9 | `eval --json` | `nix ... eval --json <installable>.meta` | `eval_package_metadata()` |
| 10 | `build --no-link --print-out-paths --json` | `nix ... build --no-link --print-out-paths --json <installable>` | `build_output_paths()` |
| 11 | `eval --raw` | `nix ... eval --raw <installable>.drvPath` | `derivation_path()` |
| 12 | `path-info --json --closure-size` | `nix ... path-info --json --closure-size <path>` | `path_info()` |

### Testing

The `MockNixAdapter` (in `root-nix`) lets unit tests simulate all Nix failure modes without a real Nix installation:

| Special Package | Simulated Error |
|----------------|-----------------|
| `missing_pkg` | `NixError::NotFound` |
| `bad_platform_pkg` | `NixError::PlatformMissing` |
| Any other package with `installed = false` adapter | `NixError::NotInstalled` |

V0.2.2 adds 24+ new tests covering:
- Experimental feature detection in doctor (Phase 2)
- Profile validation after mutation (Phase 3)
- Store path validation (`.drv` rejection) (Phase 4)
- Error normalization for all 12+ failure modes (Phase 5)
- Installer validation (Phase 6)
