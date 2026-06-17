# Root v0.2.0 Install Script Validation

Manual test cases for the `scripts/install.sh` install script, which now handles
Nix detection, optional Nix installation, Root binary install, and `root doctor`
verification.

---

## Setup

Each test case starts from a clean environment. Use `docker` or a disposable VM.

```bash
# Clone the repo (use a pinned version for consistency):
REPO_ROOT=$(mktemp -d)
git clone https://github.com/sgr0691/Root.git "$REPO_ROOT"
INSTALLER="$REPO_ROOT/scripts/install.sh"

# Or test the raw URL version:
# curl -fsSL https://raw.githubusercontent.com/sgr0691/Root/main/scripts/install.sh > /tmp/install.sh
# INSTALLER=/tmp/install.sh
```

---

## Test 1: macOS with Nix already installed

| Step | Action | Expected |
|------|--------|----------|
| 1 | `command -v nix` returns path to nix binary | `/run/current-system/sw/bin/nix` or similar |
| 2 | `sh "$INSTALLER" --dry-run` | Shows dry-run info, does not prompt about Nix |
| 3 | `sh "$INSTALLER" --version 0.2.0` | Downloads Root v0.2.0, runs `root doctor`, says "Root is ready." |

**Pass/Fail:** ☐

---

## Test 2: macOS without Nix installed

| Step | Action | Expected |
|------|--------|----------|
| 1 | Remove/uninstall Nix, ensure `command -v nix` fails | nix not found |
| 2 | `sh "$INSTALLER"` | Shows "Root requires Nix." and prompts `Continue? [y/N]` |
| 3 | Type `y` | Installs Nix via Determinate Systems, installs Root, runs `root doctor`, says "Root is ready." |

**Pass/Fail:** ☐

---

## Test 3: Linux with Nix already installed

| Step | Action | Expected |
|------|--------|----------|
| 1 | Ensure Nix is installed and `command -v nix` succeeds | nix found |
| 2 | `sh "$INSTALLER" --version 0.2.0` | Downloads Root, runs `root doctor`, says "Root is ready." |

**Pass/Fail:** ☐

---

## Test 4: Linux without Nix installed

| Step | Action | Expected |
|------|--------|----------|
| 1 | Ensure Nix is not installed | nix not found |
| 2 | `sh "$INSTALLER"` | Shows "Root requires Nix." and prompts |
| 3 | Type `y` | Installs Nix, then Root, runs `root doctor` |

**Pass/Fail:** ☐

---

## Test 5: User declines Nix installation

| Step | Action | Expected |
|------|--------|----------|
| 1 | Ensure Nix is not installed | nix not found |
| 2 | `sh "$INSTALLER"` | Shows "Root requires Nix." and prompts |
| 3 | Type `n` or press Enter | Shows "Installation cancelled." and "Install Nix first, then rerun the Root installer." |
| 4 | `command -v nix` is still empty | nix was not installed |

**Pass/Fail:** ☐

---

## Test 6: Nix installer fails

| Step | Action | Expected |
|------|--------|----------|
| 1 | Simulate Nix installer failure (e.g., cut network, or mock the installer to return non-zero) | Script shows "Error: Nix installation failed." and exits non-zero |

**Pass/Fail:** ☐

---

## Test 7: Nix installs but is unavailable until shell restart

| Step | Action | Expected |
|------|--------|----------|
| 1 | After successful Nix install, simulate nix not being available in the current shell (e.g., remove the profile sourcing) | Script shows "Nix was installed but is not available in the current shell. Please restart your shell and rerun the Root installer." |

**Note:** This is hard to test directly. To validate the code path, comment
out the sourcing lines in `install_nix()` and verify the fallback message.

**Pass/Fail:** ☐

---

## Test 8: Root install succeeds but `root doctor` fails

| Step | Action | Expected |
|------|--------|----------|
| 1 | After Root install, simulate a doctor failure | Script shows "Warning: 'root doctor' reported issues." and continues with exit 0 |

**Note:** Test by temporarily modifying the `root` binary to exit non-zero on
`doctor`, or by removing Nix after installing Root.

**Pass/Fail:** ☐

---

## Test 9: Dry-run without Nix

| Step | Action | Expected |
|------|--------|----------|
| 1 | Ensure Nix is not installed | nix not found |
| 2 | `sh "$INSTALLER" --dry-run` | Shows dry-run info, does NOT prompt about Nix, exits 0 |

**Pass/Fail:** ☐

---

## Test 10: Non-interactive (curl | sh) Nix prompt

| Step | Action | Expected |
|------|--------|----------|
| 1 | Ensure Nix is not installed | nix not found |
| 2 | Simulate non-interactive stdin: `echo "" | sh "$INSTALLER"` | Should detect no /dev/tty or read failure, default to "n", show cancellation message |

**Pass/Fail:** ☐
