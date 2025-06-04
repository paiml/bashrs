#!/bin/sh
# Rash installer script
# Generated from src/install.rs
set -euf

VERSION="0.2.0"
GITHUB_REPO="paiml/rash"

echo "Rash installer v${VERSION}"
echo "========================"

# Detect platform
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${OS}" in
    linux) OS="linux" ;;
    darwin) OS="darwin" ;;
    *) echo "Unsupported OS: ${OS}"; exit 1 ;;
esac

case "${ARCH}" in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: ${ARCH}"; exit 1 ;;
esac

PLATFORM="${OS}-${ARCH}"
echo "Detected platform: ${PLATFORM}"

# Installation directory
PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${PREFIX}/bin"
echo "Installing to: ${BIN_DIR}"

# Create directory
mkdir -p "${BIN_DIR}"

# Download URL
URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/rash-${PLATFORM}.tar.gz"
echo "Downloading from: ${URL}"

# Download
if command -v curl >/dev/null 2>&1; then
    curl -sSfL "${URL}" -o rash.tar.gz
elif command -v wget >/dev/null 2>&1; then
    wget -q "${URL}" -O rash.tar.gz
else
    echo "Error: Neither curl nor wget found"
    exit 1
fi

# Extract
tar xzf rash.tar.gz -C "${BIN_DIR}"
rm rash.tar.gz

# Make executable
chmod +x "${BIN_DIR}/rash"

echo ""
echo "✓ Rash installed successfully!"
echo ""
echo "To get started, add this to your PATH:"
echo "  export PATH=\"${BIN_DIR}:\$PATH\""
echo ""
echo "Then run:"
echo "  rash --help"
