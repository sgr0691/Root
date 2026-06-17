#!/bin/sh
set -eu

# Build Root release archives for all supported platforms.
# Usage:  sh scripts/build-release.sh [--target TARGET] [--version X.Y.Z]
#
# If --target is not specified, builds for the current host target.
# If --version is not specified, reads from Cargo.toml.
#
# Output: dist/root-<target>.tar.gz  +  dist/checksums.txt

SCRIPT_DIR="$(CDPATH='' cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DIST_DIR="${PROJECT_DIR}/dist"

# --- Discover version ---
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "${PROJECT_DIR}/Cargo.toml" | head -1)"
if [ -z "$VERSION" ]; then
    echo "Error: could not read version from Cargo.toml" >&2
    exit 1
fi

# --- Parse args ---
TARGET_SPEC=""
while [ $# -gt 0 ]; do
    case "$1" in
        --target) TARGET_SPEC="$2"; shift 2 ;;
        --version) VERSION="$2"; shift 2 ;;
        *) echo "Usage: $0 [--target TARGET] [--version X.Y.Z]" >&2; exit 1 ;;
    esac
done

# --- Detect host target if not specified ---
detect_host_target() {
    ARCH="$(uname -m)"
    case "$(uname -s)" in
        Darwin)
            case "$ARCH" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                x86_64)       echo "x86_64-apple-darwin" ;;
                *) echo "Error: unsupported arch $ARCH on macOS" >&2; exit 1 ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                *) echo "Error: unsupported arch $ARCH on Linux" >&2; exit 1 ;;
            esac
            ;;
        *) echo "Error: unsupported OS $(uname -s)" >&2; exit 1 ;;
    esac
}

ALL_TARGETS="aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu"

if [ -z "$TARGET_SPEC" ]; then
    TARGETS="$(detect_host_target)"
    echo "No --target specified; building for host: ${TARGETS}"
else
    case "$TARGET_SPEC" in
        all) TARGETS="$ALL_TARGETS" ;;
        *)   TARGETS="$TARGET_SPEC" ;;
    esac
fi

# --- Build ---
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

for target in $TARGETS; do
    echo ""
    echo "=== Building for ${target} ==="

    cargo build --release --target "$target"

    ARCHIVE="root-${target}.tar.gz"
    ARCHIVE_PATH="${DIST_DIR}/${ARCHIVE}"

    # Compute archive name prefix: strip first entry inside tarball
    # so extracting gives ./root (and ./LICENSE if available)
    TMPDIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'root-release')"
    cp "target/${target}/release/root" "$TMPDIR/"
    cp "${PROJECT_DIR}/LICENSE" "$TMPDIR/"
    chmod 755 "$TMPDIR/root"
    chmod 644 "$TMPDIR/LICENSE"

    tar czf "$ARCHIVE_PATH" -C "$TMPDIR" root LICENSE
    rm -rf "$TMPDIR"

    echo "Created ${DIST_DIR}/${ARCHIVE}"
done

# --- Checksums ---
echo ""
echo "=== Generating checksums ==="
CHECKSUMS="${DIST_DIR}/checksums.txt"
cd "$DIST_DIR"
for f in *.tar.gz; do
    sha256_hex="$((sha256sum "$f" 2>/dev/null || shasum -a 256 "$f") | awk '{print $1}')"
    echo "${sha256_hex}  ${f}" >> "$CHECKSUMS"
done

echo "Created ${CHECKSUMS}"
echo ""
echo "=== Release artifacts in ${DIST_DIR} ==="
ls -lh "$DIST_DIR"
