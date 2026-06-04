# Root v0.1.3 — Smoke Test

This document contains the manual smoke-test checklist for Root v0.1.3,
including alias resolution and catalog coverage.

Run each test on a **clean machine** (or a throw-away home directory) to
validate the fresh-install experience, basic operations, and error handling.

---

## Prerequisites

- macOS (Apple Silicon or Intel)
- `curl` available
- No previous `~/.root` directory (or `rm -rf ~/.root` before starting)

---

## 1. Fresh Install (no Nix)

**Setup:** Remove the Root directory (if any) and ensure Nix is **not** installed.

```bash
rm -rf ~/.root
```

**Test: run doctor without Nix**

```bash
root doctor
```

**Expected:**
- Error-level issue: Nix is not installed or not available on PATH.
- Suggestion mentions `root init --install-nix`.
- Exit code 0 (doctor reports issues but is informational).

---

## 2. Initialize Root

```bash
root init
```

**Expected:**
- Reports Root directory created.
- Reports Nix not detected (if Nix not installed).
- Shows next steps: `root doctor`, `root install ffmpeg`, etc.

**Test: init with --install-nix flag**

```bash
root init --install-nix
```

**Expected:**
- Installs Nix automatically (requires sudo).
- Reports Nix detected after install.

---

## 3. Doctor With Healthy State

```bash
root doctor
```

**Expected:**
- All checks pass (Nix available, Root profile ready, event ledger writable).
- Root is ready.
- Shows next steps.

---

## 4. Plan Install

```bash
root plan install ffmpeg
```

**Expected:**
- Shows "Install plan for ffmpeg".
- Lists supported package, Nix attr, binaries, verify args.
- Lists 8 steps that will be performed.
- States rollback is available.
- Says "This is a preview. No changes have been made."

```bash
root plan install ripgrep
```

**Expected:**
- Same as above but for ripgrep.
- Binary listed as `rg`.

```bash
root plan install jq
```

**Expected:**
- Same as above but for jq.
- Binary listed as `jq`.

---

## 5. Alias Resolution

```bash
root plan install rg
```

**Expected:**
- Shows "Install plan for rg → ripgrep".
- Nix attr shows `nixpkgs#ripgrep`.
- Binary listed as `rg`.
- Verify listed as `rg --version`.

```bash
root plan install node
```

**Expected:**
- Shows "Install plan for node → nodejs".
- Nix attr shows `nixpkgs#nodejs`.
- Binary listed as `node, npm`.

```bash
root plan install make
```

**Expected:**
- Shows "Install plan for make → gnumake".
- Nix attr shows `nixpkgs#gnumake`.

```bash
root plan install python
```

**Expected:**
- Shows "Install plan for python → python3".
- Nix attr shows `nixpkgs#python3`.

---

## 6. Unsupported Package Rejection (no Nix call)

```bash
root plan install imagemagick
```

**Expected:**
- Error message (not JSON): does not support "imagemagick" yet.
- Lists supported packages.
- No Nix commands are executed (check with `nix profile list`).
- Exit code 1 or 2.

```bash
root install imagemagick
```

**Expected:**
- Same unsupported-package error.
- Exit code 2.

```bash
root install missing_pkg
```

**Expected:**
- If the name is on the supported list, Nix is called and reports not found.
- If the name is not supported, the allowlist rejects it first.

---

## 7. Install Packages

```bash
root install ffmpeg
```

**Expected:**
- "Planning install..." message.
- Reports installed ffmpeg.
- Shows snapshot ID.
- Says "Rollback available with: root rollback --last".

```bash
root install poppler
```

**Expected:**
- Same as above.

```bash
root install ripgrep
```

**Expected:**
- Same as above.

```bash
root install jq
```

**Expected:**
- Same as above.

```bash
root install rg
```

**Expected:**
- Installs `ripgrep` (canonical name) via Nix.
- Lockfile stores `"name": "ripgrep"`, `"requested": "rg"`.
- Lockfile does NOT store `"name": "rg"`.

---

## 8. History

```bash
root history
```

**Expected:**
- Lists 1+ snapshots with timestamps, IDs, reasons, package counts.
- Lists events for each install.
- Each event shows timestamp, type (install), status (verified), package name.

---

## 9. Verify Packages

```bash
root verify ffmpeg
```

**Expected:**
- Binary ffmpeg shown as Executable (Exit code 0).
- Path points to `~/.root/profiles/default/bin/ffmpeg`.
- Verification SUCCESS.

```bash
root verify poppler
```

**Expected:**
- Binaries `pdftotext` and `pdfinfo` shown as Executable.
- Both pass `-v` check.
- Verification SUCCESS.

```bash
root verify ripgrep
```

**Expected:**
- Binary `rg` shown as Executable.
- Passes `--version` check.
- Verification SUCCESS.

```bash
root verify jq
```

**Expected:**
- Binary `jq` shown as Executable.
- Passes `--version` check.
- Verification SUCCESS.

---

## 10. Rollback

```bash
root rollback --last
```

**Expected:**
- Reports rolled back to a snapshot.
- Lists removed packages (jq, ripgrep, poppler, ffmpeg — whichever was last).
- Rootfile no longer contains the rolled-back package.

```bash
root history
```

**Expected:**
- Shows pre-rollback snapshot and rollback event.
- Rollback event type is `rollback`, status is `completed`.

---

## 11. Doctor With Missing/Stale State

**Test: stale lockfile**

```bash
root install ffmpeg
# Simulate crash by creating .lockfile manually
touch ~/.root/root.lockfile
root install poppler
```

**Expected:**
- Error about mutation in progress.
- Suggests deleting `~/.root/root.lockfile` if no other operation is running.

```bash
rm ~/.root/root.lockfile
root install poppler
```

**Expected:**
- Succeeds after lockfile is removed.

**Test: doctor with legacy state**

To test legacy detection, you would need a v1 lockfile with "latest" versions
and placeholder store paths. Use `root doctor` to verify it catches:

- Legacy schema version warning
- Floating "latest" version warning
- Placeholder store path warning
- Unknown nixpkgs revision warning

---

## 12. Doctor With Issues

```bash
# Remove the Rootfile to trigger a warning
mv ~/.root/Rootfile ~/.root/Rootfile.bak
root doctor
```

**Expected:**
- Warning about missing Rootfile.
- Suggests running `root install ffmpeg` to create one.

```bash
mv ~/.root/Rootfile.bak ~/.root/Rootfile
```

**Test: PATH warning**

If `~/.root/profiles/default/bin` is not in PATH, doctor should warn:

**Expected:**
- Warning about Root profile binary path not in PATH.
- Suggests adding it to shell config.

---

## 13. JSON Output

Every command supports `--json`. Test a few:

```bash
root doctor --json
root install ffmpeg --json
root plan install ffmpeg --json
root history --json
root verify ffmpeg --json
root rollback --last --json
```

**Expected:**
- Valid JSON is printed to stdout.
- Errors include `success: false` and a `message` field.
- Exit codes match the CLI error code table.
