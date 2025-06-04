#!/bin/sh
# Rash installer script v0.2.0
# Auto-generated install script
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

# Download and verify
if command -v curl >/dev/null 2>&1; then
    if ! curl -sSfL "${URL}" -o rash.tar.gz; then
        echo "Error: Failed to download from ${URL}"
        exit 1
    fi
elif command -v wget >/dev/null 2>&1; then
    if ! wget -q "${URL}" -O rash.tar.gz; then
        echo "Error: Failed to download from ${URL}"
        exit 1
    fi
else
    echo "Error: Neither curl nor wget found"
    exit 1
fi

# Verify download
if [ ! -f rash.tar.gz ] || [ ! -s rash.tar.gz ]; then
    echo "Error: Download failed or file is empty"
    exit 1
fi

# Extract
if ! tar xzf rash.tar.gz -C "${BIN_DIR}"; then
    echo "Error: Failed to extract archive"
    exit 1
fi

# Cleanup
rm rash.tar.gz

# Make executable
chmod +x "${BIN_DIR}/rash"

# Verify installation
if ! "${BIN_DIR}/rash" --version >/dev/null 2>&1; then
    echo "Error: Installation verification failed"
    exit 1
fi

echo ""
echo "✅ Rash installed successfully!"
echo ""
echo "To get started, add this to your PATH:"
echo "  export PATH=\"${BIN_DIR}:\$PATH\""
echo ""
echo "Add to your shell profile for permanent access:"
echo "  echo 'export PATH=\"${BIN_DIR}:\$PATH\"' >> ~/.bashrc  # or ~/.zshrc"
echo "  source ~/.bashrc  # or ~/.zshrc"
echo ""
echo "Then run:"
echo "  rash --help"