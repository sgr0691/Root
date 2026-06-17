#!/bin/sh
# Test harness for scripts/install.sh
#
# Tests the installer's argument parsing, dry-run mode, and error paths
# without actually downloading or installing anything.
#
# Usage:  sh scripts/test-install.sh

set -eu

PASS=0
FAIL=0
INSTALLER="$(dirname "$0")/install.sh"

banner() {
    printf "\n━━━ %s ━━━\n" "$1"
}

pass() {
    PASS=$((PASS + 1))
    printf "  ✓ %s\n" "$1"
}

fail() {
    FAIL=$((FAIL + 1))
    printf "  ✗ %s\n" "$1"
}

# --- Help flag ---

banner "help (--help)"

if "$INSTALLER" --help 2>&1 | grep -q "Install Root"; then
    pass "--help shows usage"
else
    fail "--help does not show usage"
fi

# --- Help flag (-h) ---

banner "help (-h)"

if "$INSTALLER" -h 2>&1 | grep -q "Install Root"; then
    pass "-h shows usage"
else
    fail "-h does not show usage"
fi

# --- Dry-run produces expected output ---

banner "dry-run"

DRY_OUT=$("$INSTALLER" --dry-run 2>&1 || true)
if echo "$DRY_OUT" | grep -q "Dry Run"; then
    pass "--dry-run shows dry-run output"
else
    fail "--dry-run missing dry-run output"
fi

# Dry-run should NOT contain nix-related prompts
if echo "$DRY_OUT" | grep -q "Root requires Nix"; then
    fail "--dry-run should not check for Nix"
else
    pass "--dry-run skips Nix check"
fi

# Dry-run with version
DRY_VER=$("$INSTALLER" --dry-run --version 1.0.0 2>&1 || true)
if echo "$DRY_VER" | grep -q "v1.0.0\|1.0.0"; then
    pass "--dry-run --version shows specified version"
else
    fail "--dry-run --version missing specified version"
fi

# --- Unknown option ---

banner "unknown option"

if "$INSTALLER" --bogus 2>&1; then
    fail "unknown option should exit non-zero"
else
    pass "unknown option exits non-zero"
fi

# --- Non-interactive (no TTY) should default to "no" ---

banner "non-interactive Nix prompt"

# Simulate no /dev/tty by piping empty stdin — the script should
# detect /dev/tty is not readable (or read fails) and default to "n"
# Note: we only test that it doesn't hang. Full validation is manual.
MISSING_NIX_OUT=$(echo "" | "$INSTALLER" 2>&1) || true
if echo "$MISSING_NIX_OUT" | grep -q "Installation cancelled"; then
    pass "non-interactive mode shows cancellation message"
else
    # If nix IS available on this machine, this test is skipped
    if command -v nix >/dev/null 2>&1; then
        pass "nix is available — skipping non-interactive test"
    else
        fail "non-interactive mode did not show cancellation message"
    fi
fi

# --- OS/arch detection ---

banner "OS/arch detection"

OS_OUT=$("$INSTALLER" --dry-run 2>&1 || true)
case "$(uname -s)" in
    Darwin)
        if echo "$OS_OUT" | grep -q "apple-darwin"; then
            pass "detects macOS (apple-darwin)"
        else
            fail "should detect apple-darwin on macOS"
        fi
        ;;
    Linux)
        if echo "$OS_OUT" | grep -q "unknown-linux-gnu"; then
            pass "detects Linux (unknown-linux-gnu)"
        else
            fail "should detect unknown-linux-gnu on Linux"
        fi
        ;;
esac

case "$(uname -m)" in
    arm64|aarch64)
        if echo "$OS_OUT" | grep -q "aarch64"; then
            pass "detects aarch64 architecture"
        else
            fail "should detect aarch64"
        fi
        ;;
    x86_64|amd64)
        if echo "$OS_OUT" | grep -q "x86_64"; then
            pass "detects x86_64 architecture"
        else
            fail "should detect x86_64"
        fi
        ;;
esac

# --- Summary ---

banner "RESULTS"
printf "  Passed: %d\n" "$PASS"
printf "  Failed: %d\n" "$FAIL"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
