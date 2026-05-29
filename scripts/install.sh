#!/bin/sh
set -eu

REPO="sgr0691/Root"
VERSION=""
DRY_RUN=false

usage() {
    cat <<EOF
Usage: $(basename "$0") [--version VERSION] [--dry-run]

Install the Root binary.

Options:
  --version VERSION   Install a specific version (default: latest)
  --dry-run           Print what would be done without doing it
  -h, --help          Show this help message
EOF
    exit 0
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Error: Unknown option: $1" >&2
            echo "Usage: $(basename "$0") [--version VERSION] [--dry-run]" >&2
            exit 1
            ;;
    esac
done

detect_os_arch() {
    OS=""
    ARCH=""

    case "$(uname -s)" in
        Darwin) OS="apple-darwin" ;;
        Linux)  OS="unknown-linux-gnu" ;;
        *)
            echo "Error: Unsupported operating system: $(uname -s)" >&2
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        arm64|aarch64) ARCH="aarch64" ;;
        x86_64|amd64)  ARCH="x86_64" ;;
        *)
            echo "Error: Unsupported architecture: $(uname -m)" >&2
            exit 1
            ;;
    esac
}

fetch_latest_version() {
    url="https://api.github.com/repos/${REPO}/releases/latest"
    fetch_cmd=""
    
    if command -v curl >/dev/null 2>&1; then
        fetch_cmd="curl -sL"
    elif command -v wget >/dev/null 2>&1; then
        fetch_cmd="wget -qO-"
    else
        echo "Error: Neither curl nor wget found. Please install one of them." >&2
        exit 1
    fi

    $fetch_cmd "$url" | sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p'
}

download_archive() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -sL -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$output" "$url"
    fi
}

detect_os_arch

if [ -z "$VERSION" ]; then
    echo "Fetching latest version..."
    VERSION=$(fetch_latest_version)
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine the latest version." >&2
        exit 1
    fi
    echo "Latest version: v${VERSION}"
fi

FILENAME="root-${ARCH}-${OS}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${FILENAME}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/v${VERSION}/checksums.txt"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="root"

if [ "$DRY_RUN" = true ]; then
    echo "=== Dry Run ==="
    echo "Repository:     ${REPO}"
    echo "Version:        v${VERSION}"
    echo "Archive:        ${FILENAME}"
    echo "Download URL:   ${DOWNLOAD_URL}"
    echo "Checksum URL:   ${CHECKSUM_URL}"
    echo "Install path:   ${INSTALL_DIR}/${BINARY_NAME}"
    echo "================"
    exit 0
fi

TMPDIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'root-install')
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${FILENAME}..."
download_archive "$DOWNLOAD_URL" "${TMPDIR}/${FILENAME}"

if [ ! -f "${TMPDIR}/${FILENAME}" ] || [ ! -s "${TMPDIR}/${FILENAME}" ]; then
    echo "Error: Failed to download ${DOWNLOAD_URL}" >&2
    exit 1
fi

echo "Downloading checksums..."
CHECKSUMS_FILE="${TMPDIR}/checksums.txt"
download_archive "$CHECKSUM_URL" "$CHECKSUMS_FILE"

if [ -f "$CHECKSUMS_FILE" ] && [ -s "$CHECKSUMS_FILE" ]; then
    EXPECTED_SHA=$(grep "${FILENAME}" "$CHECKSUMS_FILE" | head -n1 | awk '{print $1}')
    if [ -n "$EXPECTED_SHA" ]; then
        COMPUTED_SHA=$(sha256sum < "${TMPDIR}/${FILENAME}" 2>/dev/null || shasum -a 256 < "${TMPDIR}/${FILENAME}" 2>/dev/null | awk '{print $1}')
        COMPUTED_SHA=$(echo "$COMPUTED_SHA" | tr -d ' ')
        if [ "$COMPUTED_SHA" != "$EXPECTED_SHA" ]; then
            echo "Error: SHA256 mismatch!" >&2
            echo "  Expected: ${EXPECTED_SHA}" >&2
            echo "  Got:      ${COMPUTED_SHA}" >&2
            exit 1
        fi
        echo "Checksum verified successfully."
    else
        echo "Warning: Could not find checksum for ${FILENAME}. Skipping verification." >&2
    fi
else
    echo "Warning: Could not download checksums file. Skipping verification." >&2
fi

echo "Extracting..."
cd "$TMPDIR"
if command -v tar >/dev/null 2>&1; then
    tar xzf "${FILENAME}"
else
    echo "Error: tar is required to extract the archive." >&2
    exit 1
fi

if [ ! -f "$BINARY_NAME" ]; then
    echo "Error: Binary '$BINARY_NAME' not found in archive." >&2
    exit 1
fi

if [ ! -w "$INSTALL_DIR" ]; then
    echo "Error: No write permission to ${INSTALL_DIR}. Try running with sudo." >&2
    exit 1
fi

cp "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
chmod 755 "${INSTALL_DIR}/${BINARY_NAME}"
echo "Installed root v${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"
