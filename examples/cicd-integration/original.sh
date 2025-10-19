#!/bin/bash
# CI/CD Build Script - ORIGINAL (Problematic)
#
# Common issues in CI/CD scripts:
# - Non-deterministic build IDs (timestamps)
# - Environment-dependent paths
# - No artifact validation
# - Race conditions in parallel builds
# - Caching issues

set -e

# Non-deterministic build ID
BUILD_ID="build-$(date +%Y%m%d-%H%M%S)"
BUILD_DIR="/tmp/builds/$BUILD_ID"

echo "=== CI/CD Build Started ==="
echo "Build ID: $BUILD_ID"
echo "Commit: $GITHUB_SHA"

# Environment-specific (assumes specific tools)
if [[ -x "$(command -v npm)" ]]; then
    echo "Using npm"
    NPM_CMD="npm"
elif [[ -x "$(command -v yarn)" ]]; then
    echo "Using yarn"
    NPM_CMD="yarn"
else
    echo "Error: No package manager found"
    exit 1
fi

# Non-idempotent: fails if exists
mkdir $BUILD_DIR
cd $BUILD_DIR

# Unquoted variables
git clone $GITHUB_REPO .

# Cache directory with timestamp (non-deterministic)
CACHE_DIR="/tmp/cache-$(date +%s)"
mkdir $CACHE_DIR

# Install dependencies
echo "Installing dependencies..."
$NPM_CMD install

# Run tests
echo "Running tests..."
$NPM_CMD test

# Build
echo "Building..."
$NPM_CMD run build

# Create artifact (timestamp-based name)
ARTIFACT="artifact-$(date +%Y%m%d-%H%M%S).tar.gz"
tar -czf $ARTIFACT dist/

# Upload to S3 (hardcoded bucket)
aws s3 cp $ARTIFACT s3://my-artifacts/$ARTIFACT

# No checksum validation
# No artifact size check
# Build directory cleanup not guaranteed

echo "=== Build Complete ==="
echo "Artifact: $ARTIFACT"
