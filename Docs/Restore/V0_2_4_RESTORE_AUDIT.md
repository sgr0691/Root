# Restore Audit — Root v0.2.4

## 1. Restore Entry Points

### CLI: `Commands::Restore`

**File:** `crates/root-cli/src/main.rs:95-99`

```rust
Restore {
    #[arg(long, value_name = "PATH")]
    lock: Option<std::path::PathBuf>,
}
```

Accepts an optional `--lock <PATH>` argument. Defaults to `~/.root/root.lock`.  
Dispatched at `main.rs:874-893` — calls `root_core::restore(&adapter, lock.as_deref())`.

Output is formatted via `handle_structured` (JSON or human-readable). The human output lists `Installed`, `Removed`, `Unchanged`, and `Snapshot saved`.

### Core entry point: `pub fn restore()`

**File:** `crates/root-core/src/lib.rs:2819-2855`

```
fn restore(adapter: &impl NixAdapter, lock_path: Option<&Path>) -> Result<RestoreReport>
```

Steps in order:
1. `root_lockfile::init_root_dir()` — ensures `~/.root` tree exists
2. Resolve lock path (`selected_lock_path`): use provided path or default `~/.root/root.lock`
3. Read lockfile: `RootLockV2::read_from_file()` with `.or_else(|_| RootLock::read_from_file(&...).map(|lock| lock.to_v2()))`
4. `validate_store_paths(&target_lock)` — reject invalid lockfiles **before** any mutation
5. `enforce_policy(PolicyAction::Restore, None)` — check policy allows restore at all
6. Per-package policy check: `enforce_policy(PolicyAction::Restore, Some(&package.name))`
7. `MutationGuard::acquire()` — acquire the mutation lockfile (PID-based)
8. `reconcile_profile_to_lock(adapter, &target_lock, ...)` — the core mutation logic
9. Return `RestoreReport { success, lock_path, installed, removed, unchanged, snapshot_id }`

### `RestoreReport` struct

**File:** `crates/root-core/src/lib.rs:2318-2325`

```rust
pub struct RestoreReport {
    pub success: bool,
    pub lock_path: String,
    pub installed: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
}
```

### Test coverage

- `test_restore_from_shared_v2_lock` (lib.rs:3577-3627) — installs "fd", creates a separate v2 lockfile with "ripgrep", runs restore, asserts success, verifies lockfile, Rootfile, and event recording
- `test_restore_rejects_invalid_lockfile_before_mutation` (lib.rs:5333-5353) — writes an invalid lockfile (`.drv` in output path), asserts restore fails **before** any Nix mutation
- CLI parse test: `parses_phase_one_commands` (main.rs:1131-1137) — verifies `--lock ./root.lock` parses correctly

---

## 2. Lockfile Validation Flow

### Reading: v2 with v1 fallback

**File:** `crates/root-core/src/lib.rs:1098-1112`

```rust
fn get_or_create_lock_v2() -> Result<RootLockV2> {
    let lock = RootLockV2::read_from_file(&path)
        .or_else(|_| RootLock::read_from_file(&path).map(|lock| lock.to_v2()))?;
    root_lockfile::validate_store_paths(&lock)?;
    Ok(lock)
}
```

- First tries `RootLockV2::read_from_file` (JSON deserialization)
- On failure, falls back to v1 (`RootLock::read_from_file`) and converts via `to_v2()`
- Calls `validate_store_paths` after successful read
- Used by: `restore()`, `sync()`, `reconcile_profile_to_lock()`, `get_or_create_lock_v2()`, `status()`

### `RootLockV2::read_from_file`

**File:** `crates/root-lockfile/src/lib.rs:334-337`

```rust
pub fn read_from_file(path: &Path) -> Result<Self> {
    let content = fs::read_to_string(path)?;
    Self::read_from_str(&content)
}
```

- Reads file to string, deserializes via `serde_json`
- Deserialization failures return generic `context("Failed to parse root.lock v2 JSON")` error

### `RootLock::read_from_file` (v1 fallback)

**File:** `crates/root-lockfile/src/lib.rs:238-241`

Same pattern but for v1 schema. `to_v2()` conversion (lib.rs:307-325) maps `LockedPackage` → `LockedPackageV2`, creating synthetic `outputs` and `store_paths` maps from the single `store_path` field. The `installable` field is set to the package's `attribute` value.

### `validate_store_paths`

**File:** `crates/root-lockfile/src/lib.rs:629-703`

Validates every package in the lock:
1. `drv_path` (if present/non-empty) → must end in `.drv`
2. `outputs.*.store_path` → must NOT end in `.drv`, must start with `/nix/store/`
3. `store_paths.*` → same as outputs
4. `store_path` (primary) → same as outputs

### Error types

**File:** `crates/root-lockfile/src/lib.rs:598-612`

| Variant | Meaning |
|---------|---------|
| `DrvInOutputField { package, package_short, found }` | A `.drv` path where a realized output was expected |
| `OutputNotInStore { package, found }` | Path doesn't start with `/nix/store/` |

### Where validation gates mutation

| Caller | Location | Timing |
|--------|----------|--------|
| `restore()` | `lib.rs:2827-2830` | Before `MutationGuard`, before `reconcile_profile_to_lock` |
| `sync()` | `lib.rs:2735` | Before `reconcile_profile_to_lock` |
| `install()` | `lib.rs:1601-1603` | After Nix install, before saving lockfile |
| `rollback_last()` | `lib.rs:1915-1918` | Before any Nix mutation |
| `get_or_create_lock_v2()` | `lib.rs:1104` | On every lockfile read |

---

## 3. Nix Operations Used

### RealNixAdapter implementations

**File:** `crates/root-nix/src/lib.rs`

| Operation | Method | Shell command | Line |
|-----------|--------|---------------|------|
| Check availability | `check_availability()` | `nix --version` | 528-534 |
| Search | `search()` | `nix search nixpkgs <pkg>` | 567-569 |
| Install (by attribute) | `install()` | `nix profile add nixpkgs#<pkg> --profile <path>` | 571-580 |
| Install (by installable) | `install_installable()` | `nix profile add <installable> --profile <path>` | 582-590 |
| List | `list()` | `nix profile list --profile <path>` | 592-595 |
| Remove | `remove()` | `nix profile remove <pkg> --profile <path>` | 597-605 |
| Profile JSON list | `profile_list_json()` | `nix profile list --json --profile <path>` | 607-614 |
| Flake metadata | `flake_metadata()` | `nix flake metadata --json nixpkgs` | 616-628 |
| Eval metadata | `eval_package_metadata()` | `nix eval --json <pkg>.meta` | 630-649 |
| Build outputs | `build_output_paths()` | `nix build --no-link --print-out-paths --json <installable>` | 651-689 |
| Derivation path | `derivation_path()` | `nix eval --raw <pkg>.drvPath` | 700-717 |
| Path info | `path_info()` | `nix path-info --json --closure-size <path>` | 719-739 |
| Profile generation | `profile_generation()` | Reads profile symlink target (parses `-NNN-link` suffix) | 500-518 |

**Profile directory:** `~/.root/profiles/default`  
**All commands pass:** `--profile <profile_path>` for profile operations.

### MockNixAdapter (testing)

**File:** `crates/root-nix/src/lib.rs:742-914`

In-memory adapter with a `Vec<String>` of installed packages and atomic generation counter. Special packages:
- `"missing_pkg"` → `NixError::NotFound`
- `"bad_platform_pkg"` → `NixError::PlatformMissing`

`profile_list_json` generates synthetic JSON from internal state; `profile_list_json_override` allows injecting arbitrary JSON for edge case testing.

---

## 4. Profile Mutation Flow — `reconcile_profile_to_lock()`

**File:** `crates/root-core/src/lib.rs:2544-2713`

This is the core reconciliation engine used by `restore()`, `sync()`, and (conceptually) `install()`.

### Parameters
- `adapter: &impl NixAdapter`
- `target_lock: &RootLockV2` — the desired state
- `snapshot_reason: &str` — label for the pre-mutation snapshot
- `command: &str` — command string for event recording (e.g., `"root restore"`)
- `event_type: events::RootEventType` — event type discrimination (`Restore`, `Update`, etc.)

### Step-by-step flow

**Step 1 — Snapshot current state (lib.rs:2551-2553):**
```rust
let current_lock = get_or_create_lock_v2()?;
let snapshot = Snapshot::create_from_v2(snapshot_reason, &current_lock)?;
```
Captures the current lockfile state as a snapshot file in `~/.root/snapshots/`.

**Step 2 — Profile inspection (lib.rs:2555):**
```rust
let profile_entries = profile_packages(adapter)?;
```
- First tries `adapter.profile_list_json()` to get structured JSON with store paths
- Falls back to `adapter.list()` (legacy text format) with empty store paths

**Step 3 — Build set of locked package names (lib.rs:2556-2560):**
```rust
let locked_names: BTreeSet<&str> = target_lock.packages.iter().map(...).collect();
```

**Step 4 — Install missing packages (lib.rs:2564-2665):**
For each target package not already installed (`locked_package_installed` checks name + store_paths):

1. **Get before-generation** via `adapter.profile_generation()` — reads profile symlink to get generation number
2. **Install** using adapter — prefers `install_installable` if package has an `installable` field, else uses `install` (by name)
3. **Verify profile contains outputs** — `verify_profile_contains_outputs()` checks each store path is present in `nix profile list --json`
4. **Validate mutation result** — `validate_mutation_result()` checks:
   - Generation actually changed
   - All expected store paths are in profile (no .drv paths in outputs)
5. On failure at any sub-step: records a `RootEventStatus::Failed` event with the error, then propagates the error up

**Step 5 — Remove extra packages (lib.rs:2667-2687):**
For each profile entry not in the target lock:
- Calls `adapter.remove(&entry.package)`
- On failure: records a failed event, propagates error

**Step 6 — Save state (lib.rs:2689-2690):**
```rust
save_lock_v2(target_lock)?;
write_rootfile_from_v2_lock(target_lock)?;
```
- Writes the target lockfile to disk
- Rebuilds `Rootfile` from scratch: clears existing Rootfile, reinserts all packages from the target lock

**Step 7 — Record completion event (lib.rs:2692-2705):**
```rust
record_event(event_type, RootEventStatus::Completed, command, ...)
```

### Return type: `ProfileReconcileReport`
```rust
struct ProfileReconcileReport {
    installed: Vec<String>,
    removed: Vec<String>,
    unchanged: Vec<String>,
    snapshot_id: String,
}
```

### Key observation: Non-atomic between Nix mutations and file writes

If the process crashes between a successful Nix install/remove and `save_lock_v2()` / `write_rootfile_from_v2_lock()`, the Nix profile and lockfile/Rootfile will be inconsistent. The pre-mutation snapshot preserves the ability to reconstruct the prior state, but post-crash recovery depends on manual intervention (`root sync`).

---

## 5. Event Recording Flow

**File:** `crates/root-core/src/events.rs`

### Storage format
- Append-only JSONL file at `~/.root/events.jsonl`
- Each event is a single JSON object on one line

### Event types relevant to restore

```rust
pub enum RootEventType {
    Restore,
    Install,
    Update,          // also used by sync
    Remove,
    Rollback,
    Verification,
    VerificationFailed,
    Doctor,
    Execution,
    Policy,
    Sandbox,
}
```

### Event statuses

```rust
pub enum RootEventStatus {
    Started,
    Completed,
    Failed,
    Verified,
    Timeout,
}
```

### Recording during restore

In `reconcile_profile_to_lock()`, events are recorded at:

| Point | Status | Notes |
|-------|--------|-------|
| Per-package generation check failure | `Failed` | Before any install attempt |
| Per-package install failure | `Failed` | Nix install error |
| Per-package verification failure | `Failed` | `verify_profile_contains_outputs` |
| Per-package mutation validation failure | `Failed` | `validate_mutation_result` |
| Per-package remove failure | `Failed` | `adapter.remove` |
| Final completion | `Completed` | After all Nix ops + lockfile save |

The `record_event` function (events.rs:217-237) creates a `RootEvent` and appends it:

```rust
pub fn record_event(
    event_type, status, command,
    package, snapshot_id, restored_snapshot_id, message
) -> Result<RootEvent>
```

---

## 6. Rollback/Recovery Behavior

### `rollback_last()`

**File:** `crates/root-core/src/lib.rs:1904-2085`

`restore()` does **not** auto-rollback on failure. The user must manually run `root rollback --last`.

Rollback steps:
1. **Acquire mutation lock** — ensures exclusive access
2. **List snapshots** — reads `~/.root/snapshots/*.json`, takes most recent (`snaps[0]`)
3. **Get target lock** from snapshot via `last_snap.restored_lock()`:
   - If `lock.version == 0` and `packages` is non-empty: reconstruct a v2 lock from the legacy `packages` vec (root-snapshot/src/lib.rs:126-141)
   - Otherwise: use stored `lock: RootLockV2` directly
4. **Validate** snapshot lock store paths
5. **Compute diff** between current and target lock:
   - `packages_to_remove`: packages in current but not in target (or changed)
   - `packages_to_install`: packages in target but not in current (or changed)
6. **Create pre-rollback snapshot** — ensures rollback can be rolled-forward
7. **Execute Nix changes first**:
   - Remove packages
   - Install packages (with generation check + `verify_profile_contains_outputs` + `validate_mutation_result`)
8. **Update lockfile and Rootfile** — only after Nix operations succeed
9. **Record rollback event**

**Critical design choice:** Step 8 happens **after** all Nix mutations succeed, meaning a crash between steps 7 and 8 leaves Nix profile in the rolled-back state while lockfile/Rootfile still reflect the pre-rollback state. Running `root sync` would fix inconsistency.

### Snapshot integrity

**File:** `crates/root-snapshot/src/lib.rs:102-124`

When reading a snapshot, `Snapshot::read()` verifies the `lock_content_hash`:
```rust
let lock_content = serde_json::to_vec(&snapshot.lock)?;
let computed = compute_sha256(&lock_content);
if computed != snapshot.lock_content_hash {
    bail!("Snapshot ... lock content hash mismatch ... corrupted or tampered");
}
```

### Snapshot content

A snapshot stores a full serialized `RootLockV2` plus metadata:
- `id` (e.g., `snap_20250101_120000_123456`)
- `created_at` (UTC timestamp)
- `reason` (e.g., `"before restore from /path/to/lock"`)
- `package_count`
- `lock_content_hash` (SHA-256 of the JSON lock)
- `lock: RootLockV2` (full lock state)
- `packages: Vec<LockedPackage>` (legacy v1 field for backward compat)

---

## 7. Drift Detection Behavior

### `status()` function

**File:** `crates/root-core/src/lib.rs:3090-3194`

Compares three sources of truth:

1. **Rootfile contents** (user intent) — via `get_or_create_rootfile()`
2. **root.lock** (deterministic lock) — via `get_or_create_lock_v2()`
3. **Nix profile** (actual installed state) — via `profile_packages(adapter)`

Drift categories detected:

| Category | When | Severity | Suggestion |
|----------|------|----------|------------|
| `rootfile-lockfile-mismatch` | Package in Rootfile not in lockfile | Unhealthy | `root lock` |
| `profile-unavailable` | Nix profile cannot be inspected | Unhealthy | `root doctor` |
| `lockfile-profile-mismatch` | Package in lockfile not in Nix profile | Unhealthy | `root sync` |
| `profile-lockfile-mismatch` | Package in Nix profile not in lockfile | Unhealthy | `root sync` |

State classification:
- `"Healthy"` — no issues
- `"NeedsAttention"` — lockfile-profile mismatch or profile unavailable
- `"Drifted"` — other mismatches

### `doctor()` diagnostics

**File:** `crates/root-doctor/src/lib.rs:32-543`

Called via `root doctor` → `root_core::doctor(&adapter)` → `root_doctor::run_diagnostics(&adapter)`

Checks in order:
1. **Nix availability** — `nix --version` probe
2. **Experimental features** — `nix eval nixpkgs#hello`, parses stderr for feature-missing messages
3. **Root directory structure** — subdirectories `snapshots`, `profiles`, `logs`, `cache`
4. **Config files** — reads Rootfile and root.lock, checks for:
   - Corrupted/unparseable files
   - Legacy schema version
   - Floating `"latest"` versions
   - Placeholder store paths
   - Unknown nixpkgs revision
5. **Drift detection** — compares Rootfile ↔ lockfile ↔ profile using `profile_list_json()`
6. **PATH & shadow detection** — checks `~/.root/profiles/default/bin` is in PATH and no other binary shadows Root-managed ones

---

## 8. Known Failure Modes

### 8.1 Invalid lockfile

**What happens:** `validate_store_paths()` is called before any mutation.  
**Code path:** `restore()` → `lib.rs:2827-2830` → `root-lockfile/src/lib.rs:629-703`  
**Error messages:**
- `"Invalid Root lockfile: package X.Y has a derivation path where an output path was expected"`
- `"Lockfile at {} failed validation before restore"`  
**Test:** `test_restore_rejects_invalid_lockfile_before_mutation` (lib.rs:5333-5353)  
**Exit code:** 1 (generic failure)

### 8.2 `.drv` path in output path

**What happens:** Caught at two layers:
1. At resolution time: `deterministic_package_from_resolution()` (lib.rs:1165-1172) rejects `.drv` outputs
2. At validation time: `validate_store_paths()` (root-lockfile/src/lib.rs:667-669) rejects `.drv` in `store_paths`

**Error:**
- `"Root resolved a derivation path but no realized output path for {}. Expected an output store path, got: {}"`
- `"Invalid Root lockfile: package {}.out has a derivation path where an output path was expected"`
**Recovery:** User must fix the lockfile or provide a valid one. The restore is gated.

### 8.3 Missing package metadata (unsupported package)

**What happens:** Restore from an externally-created lockfile can contain packages Root doesn't know about via `resolve_package()`.  
**Code path:** `reconcile_profile_to_lock()` does NOT call `resolve_package()` to gate installation — it installs whatever packages are in the lockfile. `resolve_package()` is only used for `binaries` in `validate_mutation_result()` (lib.rs:2634), returning `&[]` for unknown packages.  
**Impact:** Binary validation is skipped for unknown packages. Install and verify still run. Minimal risk.

### 8.4 Missing Nix

**What happens:** `NixAdapter` methods return `NixError::NotInstalled` (exit code 7).  
**Code path:** Any Nix call from `reconcile_profile_to_lock()` → adapter method → `run_command()` → `Command::new("nix")` fails.  
**Error:** `"Failed to install '{}': Nix is not installed or not available on PATH."`  
**Exit code:** 7  
**Test:** `test_diagnostics_no_nix` (root-doctor/tests)

### 8.5 Missing experimental features

**What happens:** Nix commands like `nix profile` and `nix eval` fail with experimental feature errors.  
**Code path:** `doctor()` detects this via `probe_experimental_features()`. For restore: the first Nix subcommand will fail.  
**Error:** Varies by Nix version — `"experimental feature 'nix-command' is not enabled"` or similar.  
**Recovery:** Follow doctor suggestion to add `experimental-features = nix-command flakes` to `nix.conf`.

### 8.6 Package resolution failure

**What happens:** Not applicable to restore — restore does NOT call `resolve_locked_package()`. It uses the installable string already stored in the lockfile (`package.installable`).  
**Potential failure:** If the installable references a nixpkgs revision that no longer exists (e.g., a Git SHA that was garbage-collected), `adapter.install_installable()` will fail.  
**Error:** From Nix — store path not found, or build failure.

### 8.7 Install failure mid-restore

**What happens:** `reconcile_profile_to_lock()` loops over target packages. If the third of five packages fails to install:  
1. A `Failed` event is recorded for that package  
2. The **entire function** returns an error  
3. **No rollback** of the previously installed packages occurs  
4. The mutation lock is released (via `Drop`)  

**Impact:** Partial install — some target packages may be installed, others not.  
**Code path:** lib.rs:2600-2610 (install error handling)  
**Error:** `"root restore failed to install '{}': {}"`  
**Recovery:** User must run `root rollback --last` (pre-rollback snapshot exists). Or run `root restore` again (idempotent for already-installed packages).  
**Test gap:** No test verifies mid-restore failure atomically reverts changes.

### 8.8 Profile mutation failure

**What happens:** After a successful `adapter.install_installable()`, `validate_mutation_result()` checks generation changed and expected paths exist.  
**Code path:** lib.rs:2643-2662  
**Error:** `"root restore mutation validation failed for '{}': Profile mutation validation failed: ..."`  
**Cause (non-exhaustive):** Generation didn't change (Nix profile wasn't actually updated). Missing output paths. `.drv` paths in outputs.  
**Recovery:** Same as 8.7 — manual rollback or retry.

### 8.9 Verification failure after restore

**What happens:** `verify_profile_contains_outputs()` (lib.rs:1286-1311) checks every store path in the package's `store_paths` map appears in `nix profile list --json` output.  
**Code path:** lib.rs:2612-2631  
**Error:** `"Installed profile did not contain locked Nix store path {}"` or `"Refusing to verify .drv path as an installed output"`  
**Recovery:** Manual rollback or retry.

### 8.10 Interrupted restore (crash between Nix mutations and lockfile write)

**What happens:** If the process is killed after `save_lock_v2(target_lock)?` but before the final event record (or after some packages installed but before `save_lock_v2`):  
- **Case A (installed some packages, crash before save_lock_v2):** Nix profile has packages that lockfile doesn't know about. `root sync` will detect drift (`profile-lockfile-mismatch`) and attempt to remove extra packages.  
- **Case B (installed some packages, crash after save_lock_v2):** Lockfile is written but Rootfile may be in the wrong state (file ops are atomic via `atomic_write` but separated). Running `root sync` will detect rootfile-lockfile drift.  
- **Case C (crash during remove loop):** Some packages removed from profile but still in lockfile. `root sync` will re-install them.  

**Safeguards:**  
- Pre-mutation snapshot preserves the initial state  
- `root rollback --last` can restore the snapshot  
- `root sync` reconciles profile with lockfile  

**Current limitation:** No crash recovery mechanism is automatically triggered. Manual intervention required.

### 8.11 Stale mutation lock

**What happens:** `MutationGuard::acquire()` (lib.rs:600-637) at restore start.  
- If lock file exists: checks if the PID listed in the lockfile is alive via `kill -0 <PID>`  
- If PID is alive: error — `"Another Root mutation is in progress (PID {}). If this is unexpected, delete ~/.root/root.lockfile and try again."`  
- If PID is dead: removes stale lock, retries  
- If lock file is unreadable: error — `"Lock file ~/.root/root.lockfile exists and could not be read. Delete it manually and try again."`  
**Drop:** Lock file is removed on `MutationGuard` drop (lib.rs:674-678).

### 8.12 Profile drift before restore

**What happens:** `reconcile_profile_to_lock()` reads current profile via `profile_packages(adapter)` (lib.rs:2555). If the profile was manually modified (e.g., `nix profile remove` outside Root), restore will correctly handle it — extra packages are removed, missing packages installed.  
**If profile is completely broken** (e.g., symlink corrupted): `profile_packages()` returns error → `reconcile_profile_to_lock()` fails fast.  
**Recovery:** `root doctor` to diagnose profile issues.

### 8.13 Profile drift after restore

**What happens:** The restore is a point-in-time reconciliation. After restore succeeds, any subsequent manual Nix operations will cause drift.  
**Detection:** `root status` or `root doctor --check` will detect `lockfile-profile-mismatch` or `profile-lockfile-mismatch`.  
**Recovery:** `root sync` re-reconciles.

---

## 9. Current Gaps and Limitations

### 9.1 No automatic rollback on failure

If `reconcile_profile_to_lock()` fails mid-way (e.g., 3 of 5 packages installed successfully then a 4th fails), the partially-applied state is not automatically rolled back. The pre-mutation snapshot makes manual rollback possible (`root rollback --last`), but recovery is not automatic.

### 9.2 Non-atomic Nix + file writes

The sequence in `reconcile_profile_to_lock()` is:
1. Nix installs (multiple, sequential)
2. Nix removes (multiple, sequential)
3. Write lockfile
4. Write Rootfile

A crash between any of these steps leaves inconsistent state. The pre-mutation snapshot is the sole recovery mechanism.

### 9.3 No atomicity between multiple Nix operations

Each `adapter.install_installable()` and `adapter.remove()` is a separate `nix profile` invocation. If the user kills the process during the loop, some packages may be installed but not others. Nix profiles do not support transactional batch operations.

### 9.4 Legacy v1 lockfallback can produce nondeterministic locks

When restoring from a v1 lockfile, `to_v2()` sets `installable: Some(package.attribute)` where `attribute` is just the package name (e.g., `"ffmpeg"`). The resulting lock lacks a pinned nixpkgs revision in the installable (no `github:NixOS/nixpkgs/<rev>#ffmpeg`), meaning the resolved package depends on what `nixpkgs` currently points to.

### 9.5 No timeout for Nix operations during restore

Long-running Nix builds during restore have no configurable timeout. A stuck build blocks the mutation lock indefinitely.

### 9.6 No restore from snapshot file

Restore only accepts a lockfile path. Restoring from a snapshot (which also contains a full lock) requires manually extracting the lock from the snapshot JSON file.

### 9.7 No `--dry-run` mode for restore

There is no way to preview what changes a restore would make. Users must read the lockfile manually or infer from `root status`.

### 9.8 Test coverage gaps

| Scenario | Tested? |
|----------|---------|
| Restore from shared v2 lock | Yes |
| Restore rejects invalid lockfile pre-mutation | Yes |
| Rollback verifies store paths | Yes |
| Sync rejects invalid lockfile | Yes |
| Mid-restore failure recovery | **No** — no test verifies behavior |
| Crash between Nix ops and file writes | **No** |
| Restore with no existing lockfile | **No** |
| Restore from v1 lockfile | **No** |
| Mutation lock stale recovery | **No** |
| Concurrent restore (second process blocked) | **No** |

### 9.9 Policy enforcement only at entry points

`enforce_policy(PolicyAction::Restore, ...)` is called at the `restore()` top level, not within `reconcile_profile_to_lock()`. The function `reconcile_profile_to_lock()` is shared with `sync()`, which also does its own policy check at its entry point.

### 9.10 Snapshot deduplication

Every call to `reconcile_profile_to_lock()` creates a snapshot before any change, even if the current lock hasn't changed since the last snapshot. There is no periodic cleanup or retention policy for snapshots.
