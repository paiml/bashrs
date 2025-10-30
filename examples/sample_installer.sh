#!/bin/bash
# Sample installer script
# Common software installation patterns

set -euo pipefail

# Configuration
readonly TOOL_NAME='mytool'
readonly VERSION='1.0.0'
readonly INSTALL_DIR="${HOME}/.local/bin"

# Logging
log() {
    printf '[INFO] %s\n' "$*"
}

error() {
    printf '[ERROR] %s\n' "$*" >&2
    exit 1
}

# Detect operating system
detect_os() {
    log "Detecting operating system..."

    if [[ -f /etc/os-release ]]; then
        # shellcheck source=/dev/null
        source /etc/os-release
        printf '%s\n' "${ID}"
    else
        uname -s
    fi
}

# Detect architecture
detect_arch() {
    log "Detecting architecture..."

    local arch
    arch="$(uname -m)"

    case "${arch}" in
        x86_64)
            printf 'x86_64\n'
            ;;
        aarch64|arm64)
            printf 'arm64\n'
            ;;
        *)
            error "Unsupported architecture: ${arch}"
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."

    local missing=()

    if ! command -v curl >/dev/null 2>&1; then
        missing+=('curl')
    fi

    if ! command -v tar >/dev/null 2>&1; then
        missing+=('tar')
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing[*]}"
    fi
}

# Install binary
install_binary() {
    log "Installing ${TOOL_NAME} to ${INSTALL_DIR}..."

    mkdir -p "${INSTALL_DIR}"

    # Create placeholder for demonstration
    printf '#!/bin/sh\necho "%s version %s"\n' "${TOOL_NAME}" "${VERSION}" > "${INSTALL_DIR}/${TOOL_NAME}"
    chmod +x "${INSTALL_DIR}/${TOOL_NAME}"

    log "Installation complete!"
}

# Verify installation
verify_installation() {
    log "Verifying installation..."

    if [[ -x "${INSTALL_DIR}/${TOOL_NAME}" ]]; then
        log "Installation successful!"
        return 0
    else
        error "Installation verification failed"
    fi
}

# Main installation workflow
install_tool() {
    log "Installing ${TOOL_NAME} ${VERSION}"

    detect_os
    detect_arch
    check_dependencies
    install_binary
    verify_installation

    log "Run '${TOOL_NAME} --help' to get started"
}

# TEST: test_detect_os_returns_value
test_detect_os_returns_value() {
    local os
    os="$(detect_os 2>/dev/null)"
    [[ -n "${os}" ]] || return 1
    return 0
}

# TEST: test_detect_arch_returns_value
test_detect_arch_returns_value() {
    local arch
    arch="$(detect_arch 2>/dev/null)"
    [[ -n "${arch}" ]] || return 1
    return 0
}

# TEST: test_install_dir_is_set
test_install_dir_is_set() {
    [[ -n "${INSTALL_DIR}" ]] || return 1
    return 0
}

# Run only if not being sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    install_tool "$@"
fi
