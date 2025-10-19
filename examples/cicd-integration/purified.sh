#!/bin/sh
# CI/CD Build Script - PURIFIED by Rash v2.0.0
#
# Improvements:
# - Deterministic build IDs (commit-based)
# - Portable (works on any CI system)
# - Validated artifacts (checksums)
# - Idempotent operations
# - Clean error handling
# - POSIX compliant

set -e

# Required environment variables validation
GITHUB_SHA="${GITHUB_SHA:-}"
GITHUB_REPO="${GITHUB_REPO:-}"
S3_BUCKET="${S3_BUCKET:-my-artifacts}"

if [ -z "${GITHUB_SHA}" ]; then
    printf 'Error: GITHUB_SHA environment variable not set\n' >&2
    exit 1
fi

if [ -z "${GITHUB_REPO}" ]; then
    printf 'Error: GITHUB_REPO environment variable not set\n' >&2
    exit 1
fi

# Deterministic build ID based on commit SHA (not timestamp)
BUILD_ID="build-${GITHUB_SHA}"
BUILD_DIR="${BUILD_DIR:-/tmp/builds/${BUILD_ID}}"
CACHE_DIR="${CACHE_DIR:-/tmp/cache}"

printf '=== CI/CD Build Started ===\n'
printf 'Build ID: %s\n' "${BUILD_ID}"
printf 'Commit: %s\n' "${GITHUB_SHA}"
printf 'Repository: %s\n' "${GITHUB_REPO}"

# Detect package manager (portable)
detect_package_manager() {
    if command -v npm > /dev/null 2>&1; then
        printf 'npm'
    elif command -v yarn > /dev/null 2>&1; then
        printf 'yarn'
    else
        printf 'Error: No package manager found (npm or yarn required)\n' >&2
        exit 1
    fi
}

NPM_CMD=$(detect_package_manager)
printf 'Using package manager: %s\n' "${NPM_CMD}"

# Create build directory (idempotent)
mkdir -p "${BUILD_DIR}" || exit 1
cd "${BUILD_DIR}" || exit 1

# Clone repository (idempotent - remove if exists first)
if [ -d ".git" ]; then
    printf 'Repository already cloned, updating...\n'
    git fetch origin || exit 1
    git reset --hard "${GITHUB_SHA}" || exit 1
else
    printf 'Cloning repository...\n'
    git clone "${GITHUB_REPO}" . || exit 1
    git checkout "${GITHUB_SHA}" || exit 1
fi

# Create cache directory (idempotent, deterministic location)
mkdir -p "${CACHE_DIR}" || exit 1

# Install dependencies with caching
printf 'Installing dependencies...\n'
if [ "${NPM_CMD}" = "npm" ]; then
    npm ci --cache "${CACHE_DIR}" || exit 1
else
    yarn install --frozen-lockfile --cache-folder "${CACHE_DIR}" || exit 1
fi

# Run tests
printf 'Running tests...\n'
"${NPM_CMD}" test || {
    printf 'Error: Tests failed\n' >&2
    exit 1
}

# Build
printf 'Building...\n'
"${NPM_CMD}" run build || {
    printf 'Error: Build failed\n' >&2
    exit 1
}

# Verify build output exists
if [ ! -d "dist" ]; then
    printf 'Error: dist/ directory not created\n' >&2
    exit 1
fi

# Deterministic artifact name (commit-based, not timestamp)
ARTIFACT="artifact-${GITHUB_SHA}.tar.gz"
CHECKSUM_FILE="${ARTIFACT}.sha256"

# Create artifact
printf 'Creating artifact: %s\n' "${ARTIFACT}"
tar -czf "${ARTIFACT}" dist/ || {
    printf 'Error: Failed to create artifact\n' >&2
    exit 1
}

# Validate artifact was created
if [ ! -f "${ARTIFACT}" ]; then
    printf 'Error: Artifact file not created\n' >&2
    exit 1
fi

# Check artifact size (should be reasonable)
artifact_size=$(wc -c < "${ARTIFACT}")
if [ "${artifact_size}" -lt 100 ]; then
    printf 'Error: Artifact too small (%d bytes), may be invalid\n' "${artifact_size}" >&2
    exit 1
fi

printf 'Artifact size: %d bytes\n' "${artifact_size}"

# Generate checksum for integrity verification
printf 'Generating checksum...\n'
sha256sum "${ARTIFACT}" > "${CHECKSUM_FILE}" || {
    printf 'Error: Failed to generate checksum\n' >&2
    exit 1
}

checksum=$(cut -d' ' -f1 < "${CHECKSUM_FILE}")
printf 'SHA256: %s\n' "${checksum}"

# Upload to S3 with checksum
printf 'Uploading to S3...\n'
if command -v aws > /dev/null 2>&1; then
    # Upload artifact
    aws s3 cp "${ARTIFACT}" "s3://${S3_BUCKET}/${ARTIFACT}" \
        --metadata "sha256=${checksum},commit=${GITHUB_SHA}" || {
        printf 'Error: Failed to upload artifact to S3\n' >&2
        exit 1
    }

    # Upload checksum file
    aws s3 cp "${CHECKSUM_FILE}" "s3://${S3_BUCKET}/${CHECKSUM_FILE}" || {
        printf 'Error: Failed to upload checksum to S3\n' >&2
        exit 1
    }

    printf 'Uploaded to: s3://%s/%s\n' "${S3_BUCKET}" "${ARTIFACT}"
else
    printf 'Warning: AWS CLI not found, skipping upload\n' >&2
    printf 'Artifact available locally: %s\n' "${BUILD_DIR}/${ARTIFACT}"
fi

# Clean up old builds (keep last 5)
printf 'Cleaning up old builds...\n'
cd /tmp/builds || exit 0  # Don't fail if directory doesn't exist

# Use ls -t to sort by time, keep newest 5
# shellcheck disable=SC2012
ls -t | tail -n +6 | while IFS= read -r old_build; do
    printf 'Removing old build: %s\n' "${old_build}"
    rm -rf "${old_build}"
done

printf '=== Build Complete ===\n'
printf 'Artifact: %s\n' "${ARTIFACT}"
printf 'Checksum: %s\n' "${checksum}"
printf 'Build ID: %s\n' "${BUILD_ID}"
printf 'Location: s3://%s/%s\n' "${S3_BUCKET}" "${ARTIFACT}"
