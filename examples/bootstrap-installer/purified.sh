#!/bin/sh
# Bootstrap Installer - PURIFIED by Rash v2.0.0
#
# Improvements:
# - Deterministic temp directory (version-based, not $$)
# - Version passed as argument (not fetched)
# - Idempotent operations (mkdir -p, rm -f)
# - All variables quoted
# - Proper error handling
# - POSIX compliant

# Version is now a required argument for determinism
VERSION="${1:-}"

if [ -z "${VERSION}" ]; then
    printf 'Usage: %s <version>\n' "$0"
    printf 'Example: %s v1.2.3\n' "$0"
    exit 1
fi

printf 'Installing myapp version %s\n' "${VERSION}"

# Deterministic temp directory based on version
TEMP_DIR="/tmp/myapp-install-${VERSION}"

# Idempotent directory creation
mkdir -p "${TEMP_DIR}" || exit 1
cd "${TEMP_DIR}" || exit 1

# Download release with error handling
printf 'Downloading %s...\n' "${VERSION}"
curl -L "https://github.com/myorg/myapp/releases/download/${VERSION}/myapp-linux.tar.gz" \
  -o myapp.tar.gz || exit 1

# Extract with error handling
tar -xzf myapp.tar.gz || exit 1

# Install binary
cp myapp /usr/local/bin/myapp || exit 1
chmod +x /usr/local/bin/myapp || exit 1

# Idempotent config installation
mkdir -p /etc/myapp || exit 1
cp config.yml /etc/myapp/config.yml || exit 1

# Cleanup
cd / || exit 1
rm -rf "${TEMP_DIR}"

printf 'Successfully installed myapp %s!\n' "${VERSION}"
printf "Run 'myapp --version' to verify\n"
