#!/bin/bash
# Sample deployment script
# Common deployment automation patterns

set -euo pipefail

# Configuration
readonly APP_NAME='myapp'
readonly DEPLOY_DIR="/var/www/${APP_NAME}"
readonly VERSION="${1:-latest}"

# Logging functions
log() {
    printf '[INFO] %s\n' "$*"
}

error() {
    printf '[ERROR] %s\n' "$*" >&2
    exit 1
}

# Pre-deployment checks
check_requirements() {
    log "Checking requirements..."

    command -v git >/dev/null 2>&1 || error "git is required"
    command -v docker >/dev/null 2>&1 || error "docker is required"

    [[ -d "${DEPLOY_DIR}" ]] || error "Deploy directory not found: ${DEPLOY_DIR}"
}

# Deploy new version
deploy() {
    log "Deploying version: ${VERSION}"

    cd "${DEPLOY_DIR}" || error "Cannot cd to ${DEPLOY_DIR}"
    git fetch origin || error "Git fetch failed"
    git checkout "${VERSION}" || error "Version ${VERSION} not found"

    docker-compose build || error "Docker build failed"
    docker-compose up -d || error "Docker deployment failed"

    log "Deployment successful!"
}

# Health check
health_check() {
    log "Running health check..."

    local max_attempts=30
    local attempt=0

    while [[ "${attempt}" -lt "${max_attempts}" ]]; do
        if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
            log "Health check passed!"
            return 0
        fi

        attempt=$((attempt + 1))
        sleep 1
    done

    error "Health check failed after ${max_attempts} attempts"
}

# Main deployment workflow
deploy_app() {
    log "Starting deployment of ${APP_NAME}"

    check_requirements
    deploy
    health_check

    log "Deployment completed successfully!"
}

# TEST: test_check_requirements_exists
test_check_requirements_exists() {
    type check_requirements >/dev/null 2>&1 || return 1
    return 0
}

# TEST: test_version_is_set
test_version_is_set() {
    [[ -n "${VERSION}" ]] || return 1
    return 0
}

# TEST: test_app_name_is_set
test_app_name_is_set() {
    [[ -n "${APP_NAME}" ]] || return 1
    return 0
}

# Run only if not being sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    deploy_app "$@"
fi
