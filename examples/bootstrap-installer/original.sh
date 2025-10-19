#!/bin/bash
# Bootstrap Installer - ORIGINAL (MESSY)
#
# This is a typical bootstrap installer script with common problems:
# - Non-deterministic temp directory ($$)
# - Non-deterministic version fetch
# - Non-idempotent operations
# - Unquoted variables
# - No error handling
# - Not POSIX compliant

set -e

# Non-deterministic temp directory using process ID
TEMP_DIR="/tmp/myapp-install-$$"

# Fetch latest version from GitHub API (network call)
echo "Fetching latest version..."
VERSION=$(curl -s https://api.github.com/repos/myorg/myapp/releases/latest | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$VERSION" ]; then
    echo "Error: Could not fetch version"
    exit 1
fi

echo "Installing myapp version $VERSION"

# Non-idempotent directory creation
mkdir $TEMP_DIR
cd $TEMP_DIR

# Download release
echo "Downloading $VERSION..."
curl -L https://github.com/myorg/myapp/releases/download/$VERSION/myapp-linux.tar.gz -o myapp.tar.gz

# Extract
tar -xzf myapp.tar.gz

# Install binary
cp myapp /usr/local/bin/myapp
chmod +x /usr/local/bin/myapp

# Install config (non-idempotent)
mkdir /etc/myapp
cp config.yml /etc/myapp/config.yml

# Cleanup
cd /
rm -r $TEMP_DIR

echo "Successfully installed myapp $VERSION!"
echo "Run 'myapp --version' to verify"
