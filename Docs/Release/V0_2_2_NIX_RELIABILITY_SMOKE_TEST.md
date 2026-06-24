# Root v0.2.2 Nix Reliability Smoke Test

Manual release validation focused on Nix reliability paths: missing Nix,
experimental features, clean install, restore, rollback, invalid lockfile
defense, multi-package update, and profile verification.

Run the full automated CI sequence first, then execute these checks on a
disposable Root directory with real Nix.

---

## Automated Gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build
target/debug/root --version
```

**Expected:** every command succeeds and the binary reports `root 0.2.2`.

---

## Prerequisites

- macOS (Apple Silicon or Intel) or Linux
- Nix installed (or run the missing-Nix test first to verify the error path)
- Internet access (for Nix builds and binary cache)
- No existing `~/.root` directory (or back it up before the clean install test)

---

## 1. Missing Nix

**Setup:** Temporarily remove Nix from PATH or test on a machine without Nix.

```bash
root doctor
```

**Expected:**
- Error-level issue: "Nix is not installed or not available on PATH."
- Clear explanation that Root uses Nix for reproducible, deterministic builds
  and package isolation.
- Suggestion mentions `root init --install-nix` and the NixOS download page.
- No panic, no raw Nix error output, no crash.
- Exit code 0 (doctor is informational; use `--check` for non-zero on issues).

```bash
root doctor --json
```

**Expected:**
- Valid JSON output.
- `"nix_installed": false`.
- `"healthy": false`.
- At least one issue in the `"issues"` array with `"category": "Nix"`.

---

## 2. Experimental Features Missing

**Setup:** Temporarily disable `nix-command` and `flakes` experimental features
(e.g., comment them out in `~/.config/nix/nix.conf` or rename the config to
simulate a fresh Nix installation that has not enabled them). Root passes
`--extra-experimental-features nix-command flakes` on every Nix invocation,
so this test validates the error-normalization path when those flags are
rejected by the Nix daemon or when the Nix version does not support them.

```bash
root doctor
```

**Expected:**
- Error-level issue: "Nix is installed but experimental features are not
  enabled. Root needs 'nix-command' and 'flakes' experimental features."
- Suggestion shows the exact config line to add:
  ```text
  experimental-features = nix-command flakes
  ```
- Suggests adding it to `~/.config/nix/nix.conf` and re-running `root doctor`.
- No panic, no raw Nix daemon error dump.

```bash
root doctor --json
```

**Expected:**
- Valid JSON output.
- Issue with `"category": "Nix"` mentioning `"experimental feature"`.
- `"healthy": false`.

**Cleanup:** Restore the experimental features config and verify:

```bash
root doctor
```

**Expected:** Nix check passes, system healthy.

---

## 3. Clean Install

```bash
# Start completely fresh
rm -rf ~/.root

# Verify the directory is gone
root doctor
```

**Expected:**
- Reports missing Root directory (`~/.root` does not exist).
- Suggests running `root init`.
- No crash or panic.

```bash
root init
root doctor
```

**Expected:**
- Root directory created at `~/.root`.
- Subdirectories created: `snapshots/`, `profiles/`, `logs/`, `cache/`.
- Profile directory NOT pre-created as a plain directory (Nix manages it as a
  symlink). Doctor may show a warning about the missing profile directory;
  this is acceptable and will resolve on first install.
- Nix detected (assuming Nix is installed and experimental features are on).

```bash
root plan install ffmpeg
```

**Expected:**
- Shows "Install plan for ffmpeg".
- Lists supported package, Nix attr (`nixpkgs#ffmpeg`), binaries, verify args.
- Lists 8 steps that will be performed.
- States rollback is available.
- Says "This is a preview. No changes have been made."

```bash
root install ffmpeg
```

**Expected:**
- Planning and install succeed.
- Reports installed ffmpeg with snapshot ID.
- Says "Rollback available with: root rollback --last".

```bash
root verify ffmpeg
```

**Expected:**
- Binary `ffmpeg` found and executable.
- Resolved path points to `~/.root/profiles/default/bin/ffmpeg` (NOT a global
  PATH location).
- `ffmpeg -version` executes successfully.
- No `.drv` output path errors.

```bash
root history
```

**Expected:**
- Lists at least one snapshot with timestamp, ID, reason ("install ffmpeg"),
  package count (1).
- Lists an install event for ffmpeg with status "verified".

### Lockfile inspection

```bash
cat ~/.root/root.lock | python3 -m json.tool | head -40
```

**Expected:**
- `version` is 2.
- `nixpkgs.rev` is a concrete commit hash (not `"unknown"`).
- `packages[0].store_path` does NOT end in `.drv`.
- `packages[0].outputs` values (e.g., `"out"`) have store paths that do NOT
  end in `.drv`.
- `packages[0].drv_path` (if present) DOES end in `.drv`.
- `installable` is pinned with a full flake URL.

---

## 4. Restore

```bash
# Copy the current lockfile to simulate a shared/Git-backed restore
cp ~/.root/root.lock /tmp/root-v0.2.2-restore.lock

# Now simulate a corrupted state by removing the profile or installing
# something outside Root, then restore from the saved lock
root restore --lock /tmp/root-v0.2.2-restore.lock
```

**Expected:**
- Reports "Restored Root profile from /tmp/root-v0.2.2-restore.lock".
- Shows packages installed or unchanged.
- Snapshot saved before restore.
- No `.drv` output path errors.

```bash
root status
```

**Expected:**
- Machine ID displayed.
- State is "Healthy" or "Aligned" (no drift detected).
- Rootfile, lockfile, and profile package counts match.

```bash
root verify ffmpeg
```

**Expected:**
- ffmpeg binary is functional from the Restored profile.
- Path points to `~/.root/profiles/default/bin/ffmpeg`.
- Verification SUCCESS.

```bash
root history --limit 5
```

**Expected:**
- Shows restore event in recent history (alongside install events).
- Restore event has type "restore" and status "completed".

---

## 5. Rollback

```bash
# Note the current state
root list

# Roll back the most recent operation (the restore)
root rollback --last
```

**Expected:**
- Reports rollback to a specific snapshot ID.
- Shows packages removed and/or restored.
- Lockfile and Rootfile reflect the rolled-back state.

```bash
root history --limit 5
```

**Expected:**
- Shows a rollback event with type "rollback" and status "completed".
- The rollback restored a specific snapshot ID.

```bash
root verify ffmpeg
```

**Expected:**
- ffmpeg verification succeeds.
- Rollback validated the restored state — the profile contains the locked
  store paths from the snapshot.

```bash
root list
```

**Expected:**
- The listed packages match the state after rollback (consistent with what
  was locked before the restore that was rolled back).

### Rollback failure path

```bash
# If no snapshots remain (or simulate by clearing snapshots)
mkdir -p /tmp/root-snapshots-backup
cp -r ~/.root/snapshots/* /tmp/root-snapshots-backup/ 2>/dev/null || true
rm -rf ~/.root/snapshots/*
root rollback --last
```

**Expected:**
- Clear error: "No snapshots available for rollback."
- Suggests running `root install` first.
- Lockfile and Rootfile are NOT corrupted.
- Exit code 6.

```bash
# Restore snapshots
cp -r /tmp/root-snapshots-backup/* ~/.root/snapshots/ 2>/dev/null || true
rm -rf /tmp/root-snapshots-backup
```

---

## 6. Invalid Lockfile (`.drv` Injection)

**Setup:** Manually inject a `.drv` path into the lockfile's output path to
simulate a corrupted or tampered lockfile. Root must refuse to proceed with
a mutation when output paths contain `.drv` suffixes.

```bash
# Read the current lockfile
LOCKFILE=~/.root/root.lock
cp "$LOCKFILE" /tmp/root-v0.2.2-clean.lock

# Inject a .drv path into the first package's store_path
python3 -c "
import json
with open('$LOCKFILE') as f:
    lock = json.load(f)
if lock['packages']:
    lock['packages'][0]['store_path'] = '/nix/store/xxxxx-fake-0.0.0.drv'
    # Also inject into outputs if they exist
    for output_name in lock['packages'][0].get('outputs', {}):
        lock['packages'][0]['outputs'][output_name] = '/nix/store/xxxxx-fake-0.0.0.drv'
with open('$LOCKFILE', 'w') as f:
    json.dump(lock, f, indent=2)
"

# Attempt an install (or any mutation)
root install ffmpeg
```

**Expected:**
- Root refuses the mutation with a clear error message about `.drv` output
  paths in the lockfile.
- Message explains that output paths must not end in `.drv`.
- Exit code 4 (verification failure).

```bash
# Attempt rollback
root rollback --last
```

**Expected:**
- Root refuses with the same `.drv` defense.
- Lockfile is NOT modified by the failed operation.

```bash
# Restore the clean lockfile
cp /tmp/root-v0.2.2-clean.lock "$LOCKFILE"

# Verify normal operations resume
root install ffmpeg
```

**Expected:** Install succeeds as normal.

**Cleanup:**

```bash
rm -f /tmp/root-v0.2.2-clean.lock
```

---

## 7. Multiple Package Install & Update

```bash
# Install three packages at once (separate commands since Root installs
# one package at a time)
root install ripgrep
root install fd
root install bat
```

**Expected:**
- Each install succeeds.
- Each creates a snapshot.
- Each shows "Rollback available with: root rollback --last".

```bash
root list
```

**Expected:**
- Lists ffmpeg, ripgrep, fd, bat with their versions.
- Total matches the expected package count.

```bash
# Verify each installed package
root verify ripgrep
root verify fd
root verify bat
```

**Expected:**
- Each binary is found in `~/.root/profiles/default/bin/` and executes
  successfully.
- `rg --version`, `fd --version`, `bat --version` all pass.
- Verification SUCCESS for all three.

```bash
# Update all managed packages
root update
```

**Expected:**
- Reports updated or unchanged packages.
- If any packages had newer versions available, they are updated.
- Snapshot saved (even if nothing changed, an update event is recorded).
- No `.drv` output path errors.
- `history --limit 3` shows update events.

```bash
# Verify packages are still functional after update
root verify ripgrep
```

**Expected:**
- ripgrep (and all other packages) still functional.
- Verification SUCCESS.

---

## 8. Profile Verification

```bash
root list
```

**Expected:**
- Lists all managed packages: ffmpeg, ripgrep, fd, bat (and any others).
- Shows version for each.
- Shows Nix Profile State (profile path and element count).

```bash
ls -la ~/.root/profiles/default/bin/
```

**Expected:**
- Lists binaries for all installed packages: `ffmpeg`, `rg`, `fd`, `bat`.
- Each binary is a symlink into the Nix store.
- No dangling symlinks.
- Number of binaries matches the packages' expected binaries:
  - ffmpeg: `ffmpeg`, `ffplay`, `ffprobe` (at minimum `ffmpeg`)
  - ripgrep: `rg`
  - fd: `fd`
  - bat: `bat`

```bash
# Cross-check that each binary resolves to the Root profile
for bin in ffmpeg rg fd bat; do
  echo "$bin -> $(which $bin 2>/dev/null || echo 'not in PATH')"
done
```

**Expected:**
- Each binary that is on PATH resolves to `~/.root/profiles/default/bin/<bin>`.
- If PATH is not configured, the `which` output is acceptable — `root verify`
  remains the authoritative check.

```bash
# Verify the binaries are real Nix store paths
ls -la ~/.root/profiles/default/bin/rg
```

**Expected:**
- Output shows the symlink target, e.g.:
  `/nix/store/<hash>-ripgrep-<version>/bin/rg`
- The target path does NOT contain `.drv`.

---

## JSON Output Check

Test `--json` on the Nix-reliability-relevant commands:

```bash
root doctor --json
root install ffmpeg --json
root verify ffmpeg --json
root rollback --last --json
root restore --lock /tmp/root-v0.2.2-restore.lock --json 2>/dev/null || true
root status --json
root history --json --limit 3
root list --json
```

**Expected:**
- Valid JSON is printed to stdout.
- Errors include `"success": false` and a `"message"` field.
- Exit codes match the CLI error code table (0 success, 4 verification,
  6 rollback, 7 Nix unavailable, etc.).

---

## Validation Checklist

| Test Path                              | Status | Notes |
|----------------------------------------|--------|-------|
| 1. Missing Nix — doctor message        | ☐      | |
| 2. Experimental features missing       | ☐      | |
| 3. Clean install — init, doctor, install, verify, history | ☐ | |
| 4. Restore — restore, status, verify   | ☐      | |
| 5. Rollback — rollback, history, verify| ☐      | |
| 5b. Rollback failure — no snapshots    | ☐      | |
| 6. Invalid lockfile — `.drv` rejection | ☐      | |
| 7. Multi-package install & update      | ☐      | |
| 8. Profile verification — list, `ls` bin, symlinks | ☐ | |
| JSON output — doctor, install, verify, rollback, restore, status, history, list | ☐ | |
| No `.drv` output paths in lockfile     | ☐      | |
| All binaries resolve to Root profile   | ☐      | |
| No panics or crashes on error paths    | ☐      | |

---

## Cleanup

```bash
rm -f /tmp/root-v0.2.2-restore.lock /tmp/root-v0.2.2-clean.lock 2>/dev/null || true
```

If you used a disposable Root directory via `ROOT_DIR`:

```bash
rm -rf "$ROOT_DIR"
unset ROOT_DIR
```
