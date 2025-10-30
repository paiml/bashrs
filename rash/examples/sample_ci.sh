#!/bin/bash
# Sample CI/CD pipeline script
# Common patterns in continuous integration

set -euo pipefail

# CI Configuration
readonly CI="${CI:-false}"
readonly BRANCH="${BRANCH:-main}"
readonly BUILD_NUMBER="${BUILD_NUMBER:-local}"

# Logging
log_section() {
    printf '\n======================================\n'
    printf '%s\n' "$1"
    printf '======================================\n'
}

# Environment setup
setup_environment() {
    log_section "Setting up environment"

    printf 'CI Mode: %s\n' "${CI}"
    printf 'Branch: %s\n' "${BRANCH}"
    printf 'Build: #%s\n' "${BUILD_NUMBER}"

    # Install dependencies based on project type
    if [[ -f "package.json" ]]; then
        npm ci
    elif [[ -f "Cargo.toml" ]]; then
        cargo fetch
    fi
}

# Linting
lint_code() {
    log_section "Running linters"

    local exit_code=0

    if [[ -f "Cargo.toml" ]]; then
        cargo clippy -- -D warnings || exit_code=1
        cargo fmt -- --check || exit_code=1
    fi

    return "${exit_code}"
}

# Testing
run_tests() {
    log_section "Running tests"

    if [[ -f "package.json" ]]; then
        npm test
    elif [[ -f "Cargo.toml" ]]; then
        cargo test --all-features
    fi
}

# Build
build_project() {
    log_section "Building project"

    if [[ -f "package.json" ]]; then
        npm run build
    elif [[ -f "Cargo.toml" ]]; then
        cargo build --release
    fi
}

# Main pipeline
run_pipeline() {
    log_section "Starting CI/CD Pipeline"

    setup_environment
    lint_code
    run_tests
    build_project

    log_section "Pipeline completed successfully"
}

# TEST: test_ci_variable_is_set
test_ci_variable_is_set() {
    [[ -n "${CI}" ]] || return 1
    return 0
}

# TEST: test_branch_is_set
test_branch_is_set() {
    [[ -n "${BRANCH}" ]] || return 1
    return 0
}

# TEST: test_build_number_is_set
test_build_number_is_set() {
    [[ -n "${BUILD_NUMBER}" ]] || return 1
    return 0
}

# Run only if not being sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    run_pipeline "$@"
fi
