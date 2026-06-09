# Root v0.1.9 Smoke Test

## Prerequisites

- macOS (Apple Silicon or Intel)
- Nix installed OR willing to install via `root init --install-nix`
- Internet access (for Nix builds and binary cache)

---

## 1. Fresh Machine Path

```bash
# Start clean
rm -rf ~/.root

# Init should create directories
root init

# Expect:
# - Root directory created at ~/.root
# - Nix detected (or not, depending on installation)
```

**Expected:**
- `~/.root` directory created
- `~/.root/snapshots/` created
- `~/.root/profiles/` created (but NOT `profiles/default`)
- `~/.root/logs/` created
- `~/.root/cache/` created

---

## 2. Missing Nix Path

```bash
# If Nix is NOT installed:
root init --install-nix

# Or with an explicit check:
root doctor
```

**Expected:**
- Clear message explaining Root needs Nix and why
- Suggestion to install Nix
- No raw Nix error output

---

## 3. Catalog Path

```bash
root catalog
root catalog --json
```

**Expected:**
- 42 packages listed across 11 categories
- JSON output valid
- All packages have name, description, category, binaries, aliases, verify commands

---

## 4. Representative Install Path

Test at least these packages:

```bash
root plan install ffmpeg
root install ffmpeg
root verify ffmpeg

root plan install ripgrep
root install ripgrep
root verify ripgrep

root plan install jq
root install jq
root verify jq

root plan install go
root install go
root verify go

root plan install terraform
root install terraform
root verify terraform

root plan install kubectl
root install kubectl
root verify kubectl

root plan install helm
root install helm
root verify helm

root plan install docker-client
root install docker-client
root verify docker-client

root plan install git-delta
root install git-delta
root verify git-delta

root plan install zoxide
root install zoxide
root verify zoxide

root plan install lazygit
root install lazygit
root verify lazygit

root plan install postgresql
root install postgresql
root verify postgresql

root plan install poppler
root install poppler
root verify poppler
```

**Expected for each:**
- Install succeeds
- Real Nix metadata is written (no "latest" versions)
- Output paths do not end in `.drv`
- Verification uses `~/.root/profiles/default/bin/<binary>`
- Verification reports the correct command args

---

## 5. Verification Path

```bash
# Test that verification uses Root profile, not global PATH
which ffmpeg              # note the path
root verify ffmpeg        # should report ~/.root/profiles/default/bin/ffmpeg

# Test verification failure for missing binary
# (remove binary temporarily from profile)
root verify nonexistent
# Expected: clear error "not found in Root profile"
```

**Expected:**
- `root verify ffmpeg` shows resolved path under `~/.root/profiles/default/bin/`
- Non-standard tools use correct args:
  - `go version`
  - `terraform version`
  - `kubectl version --client`
  - `helm version --short`
  - `tmux -V`
  - `direnv version`

---

## 6. History Path

```bash
root history
root history --json
```

**Expected:**
- Shows snapshots for each install
- Shows events for each operation
- JSON output is valid

---

## 7. Rollback Path

```bash
# Note current state
root list

# Rollback the most recent install
root rollback --last

# Verify state reverted
root list
root history
```

**Expected:**
- The most recently installed package is removed
- Prior packages are restored
- Lockfile and Rootfile reflect the rolled-back state
- History shows a rollback event
- JSON output from `--json` flag

---

## 8. Rollback Failure Path

```bash
# If no snapshots exist
rm -rf ~/.root/snapshots
root rollback --last
# Expected: "No snapshots available for rollback"

# If lockfile is corrupted
# Expected: clear error, lockfile and Rootfile preserved
```

**Expected:**
- Rollback failure does NOT corrupt existing state
- Lockfile and Rootfile are unchanged on failure
- Failure is reported to the user clearly

---

## 9. Alias Install Path

```bash
root plan install rg      # should resolve to ripgrep
root install rg
root verify rg

root plan install node    # should resolve to nodejs
root install node
root verify node

root plan install tf      # should resolve to terraform
root install tf
root verify tf

root plan install delta   # should resolve to git-delta
root install delta
root verify delta

root plan install z       # should resolve to zoxide
root install z
root verify z

root plan install lg      # should resolve to lazygit
root install lg
root verify lg
```

**Expected:**
- Plan shows alias → canonical name resolution
- Lockfile stores canonical name in `name` field
- Lockfile `requested` field preserves original alias
- Binaries match the canonical package

---

## 10. Lockfile Inspection Path

```bash
# Inspect the lockfile after installs
cat ~/.root/root.lock | python3 -m json.tool

# Verify:
# - version is 2
# - nixpkgs.rev is a concrete commit hash (not "unknown")
# - Each package has:
#   - name, requested, version
#   - storePath (does NOT end in .drv)
#   - drv_path (ends in .drv, stored separately)
#   - storePaths (none end in .drv)
#   - outputs (none have .drv store_path)
# - installable is pinned with full flake URL
```

**Expected:**
- No output path ends in `.drv`
- Drv paths are in `drv_path` field only
- Alias packages have canonical `name` and original `requested`

---

## 11. Legacy Lock Detection Path

```bash
# Create a legacy v1 lockfile
cat > ~/.root/root.lock << 'EOF'
{
  "version": 1,
  "platform": "aarch64-darwin",
  "nixpkgs": {
    "rev": "unknown",
    "source": "github:NixOS/nixpkgs"
  },
  "packages": [
    {
      "name": "ffmpeg",
      "requested": "ffmpeg",
      "version": "latest",
      "attribute": "ffmpeg",
      "storePath": "/nix/store/xxx",
      "binaries": ["ffmpeg"]
    }
  ]
}
EOF

root doctor

# Verify it detects:
# - Legacy schema version
# - Floating "latest" version
# - Placeholder store path
# - Unknown nixpkgs revision
```

**Expected:**
- Doctor reports all 4 legacy issues
- Doctor suggests `root lock` to regenerate
- `root lock` upgrades to v2 with deterministic metadata

---

## 12. Linux Investigation Checklist

If testing on Linux:

```bash
# Build
cargo build

# All tests must pass
cargo test --all

# Init
root init
root doctor

# Install (at least one package)
root install ffmpeg
root verify ffmpeg

# History and rollback
root history
root rollback --last
```

**Expected:**
- `cargo test --all` passes
- All Root commands produce meaningful output on Linux
- No macOS-specific errors

---

## Validation Summary

| Test Path                      | Status | Notes |
|--------------------------------|--------|-------|
| 1. Fresh machine path          | ☐      | |
| 2. Missing Nix path            | ☐      | |
| 3. Catalog path                | ☐      | |
| 4. Representative install path | ☐      | |
| 5. Verification path           | ☐      | |
| 6. History path                | ☐      | |
| 7. Rollback path               | ☐      | |
| 8. Rollback failure path       | ☐      | |
| 9. Alias install path          | ☐      | |
| 10. Lockfile inspection path   | ☐      | |
| 11. Legacy lock detection path | ☐      | |
| 12. Linux checklist            | ☐      | |
