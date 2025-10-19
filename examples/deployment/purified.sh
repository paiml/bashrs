#!/bin/sh
# Deployment Script - PURIFIED by Rash v2.0.0
#
# Improvements:
# - Deterministic release names (version-based, not timestamp)
# - Deterministic session IDs (version-based, not $RANDOM)
# - Idempotent operations (mkdir -p, rm -f, ln -sf)
# - All variables quoted
# - Comprehensive error handling
# - POSIX compliant

APP_NAME="myapp"
DEPLOY_ROOT="/opt/apps"

# Version is required for determinism
VERSION="${1:-}"

if [ -z "${VERSION}" ]; then
    printf 'Usage: %s <version>\n' "$0"
    printf 'Example: %s v2.1.0\n' "$0"
    exit 1
fi

# Deterministic release name based on version
RELEASE_NAME="release-${VERSION}"
SESSION_ID="session-${VERSION}"

printf '=== Deployment Started ===\n'
printf 'App: %s\n' "${APP_NAME}"
printf 'Release: %s\n' "${RELEASE_NAME}"
printf 'Session: %s\n' "${SESSION_ID}"
printf 'Version: %s\n' "${VERSION}"

# Create release directory (idempotent)
RELEASE_DIR="${DEPLOY_ROOT}/${APP_NAME}/releases/${RELEASE_NAME}"
mkdir -p "${RELEASE_DIR}" || exit 1

# Copy build artifacts with error handling
printf 'Copying build artifacts...\n'
if [ ! -d "build" ]; then
    printf 'Error: build/ directory not found\n'
    exit 1
fi
cp -r build/* "${RELEASE_DIR}/" || exit 1

# Create/update symlink (idempotent)
CURRENT_LINK="${DEPLOY_ROOT}/${APP_NAME}/current"
rm -f "${CURRENT_LINK}"  # -f: force, no error if missing
ln -sf "${RELEASE_DIR}" "${CURRENT_LINK}" || exit 1

# Update config (idempotent)
mkdir -p "${DEPLOY_ROOT}/${APP_NAME}/config" || exit 1
if [ -f "config/production.yml" ]; then
    cp config/production.yml "${DEPLOY_ROOT}/${APP_NAME}/config/app.yml" || exit 1
else
    printf 'Warning: config/production.yml not found, skipping config update\n'
fi

# Restart service
printf 'Restarting service...\n'
if command -v systemctl > /dev/null 2>&1; then
    systemctl restart "${APP_NAME}" || exit 1
else
    printf 'Warning: systemctl not found, skipping service restart\n'
fi

# Log deployment (deterministic - version instead of timestamp)
LOG_FILE="/var/log/${APP_NAME}-deploy.log"
printf '%s: Deployed %s (session %s)\n' "${VERSION}" "${RELEASE_NAME}" "${SESSION_ID}" >> "${LOG_FILE}"

# Cleanup old releases (keep last 5)
cd "${DEPLOY_ROOT}/${APP_NAME}/releases" || exit 1
# Use ls -t to sort by time, tail to get old ones, xargs to remove
# shellcheck disable=SC2012
ls -t | tail -n +6 | while IFS= read -r old_release; do
    printf 'Removing old release: %s\n' "${old_release}"
    rm -rf "${old_release}"
done

printf '=== Deployment Complete ===\n'
printf 'Release: %s\n' "${RELEASE_NAME}"
printf 'Session: %s\n' "${SESSION_ID}"
printf 'Current: %s -> %s\n' "${CURRENT_LINK}" "${RELEASE_DIR}"
