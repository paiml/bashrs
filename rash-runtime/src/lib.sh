#!/bin/sh
# Rash Runtime Library v1.0.0
# POSIX-compliant shell functions for Rash-generated scripts

# Error handling
rash_require() {
    if ! "$@"; then
        echo "FATAL: Requirement failed: $*" >&2
        exit 1
    fi
}

rash_assert() {
    if ! "$@"; then
        echo "ASSERTION FAILED: $*" >&2
        exit 2
    fi
}

# File operations
rash_file_exists() {
    test -f "$1"
}

rash_dir_exists() {
    test -d "$1"
}

rash_is_writable() {
    test -w "$1"
}

rash_is_readable() {
    test -r "$1"
}

rash_atomic_write() {
    local content="$1"
    local target="$2"
    local temp="${target}.tmp.$$"
    
    echo "$content" > "$temp"
    mv "$temp" "$target"
}

# Network operations with verification
rash_download_verified() {
    local url="$1"
    local dst="$2"
    local checksum="$3"
    
    # Try curl first, then wget
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 "$url" -o "$dst"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dst" "$url"
    else
        echo "FATAL: Neither curl nor wget found" >&2
        return 1
    fi
    
    # Verify checksum
    if command -v sha256sum >/dev/null 2>&1; then
        echo "$checksum  $dst" | sha256sum -c >/dev/null
    elif command -v shasum >/dev/null 2>&1; then
        echo "$checksum  $dst" | shasum -a 256 -c >/dev/null
    else
        echo "FATAL: No checksum utility found" >&2
        return 1
    fi
}

# Archive operations
rash_extract_tar() {
    local archive="$1"
    local dest="$2"
    
    # Security: extract with restricted permissions
    tar -xf "$archive" -C "$dest" --no-same-owner --no-same-permissions
}

rash_extract_tar_gz() {
    local archive="$1"
    local dest="$2"
    
    if command -v gzip >/dev/null 2>&1; then
        gzip -dc "$archive" | tar -xf - -C "$dest" --no-same-owner --no-same-permissions
    else
        # Try with built-in gz support
        tar -xzf "$archive" -C "$dest" --no-same-owner --no-same-permissions
    fi
}

# Platform detection
rash_detect_arch() {
    local arch
    arch="$(uname -m)"
    
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l|armhf)
            echo "armv7"
            ;;
        i386|i686)
            echo "i386"
            ;;
        *)
            echo "unknown"
            return 1
            ;;
    esac
}

rash_detect_os() {
    local os
    os="$(uname -s)"
    
    case "$os" in
        Linux)
            echo "linux"
            ;;
        Darwin)
            echo "macos"
            ;;
        FreeBSD)
            echo "freebsd"
            ;;
        OpenBSD)
            echo "openbsd"
            ;;
        NetBSD)
            echo "netbsd"
            ;;
        *)
            echo "unknown"
            return 1
            ;;
    esac
}

rash_detect_platform() {
    local os arch
    os="$(rash_detect_os)" || return 1
    arch="$(rash_detect_arch)" || return 1
    echo "${arch}-${os}"
}

# Logging
rash_log_info() {
    echo "INFO: $*" >&2
}

rash_log_warn() {
    echo "WARN: $*" >&2
}

rash_log_error() {
    echo "ERROR: $*" >&2
}

rash_log_debug() {
    if [ "${RASH_DEBUG:-}" = "1" ]; then
        echo "DEBUG: $*" >&2
    fi
}

# Command existence checking
rash_has_command() {
    command -v "$1" >/dev/null 2>&1
}

rash_require_command() {
    local cmd="$1"
    if ! rash_has_command "$cmd"; then
        echo "FATAL: Required command '$cmd' not found" >&2
        exit 1
    fi
}

# Temporary directory management
rash_mktemp_dir() {
    local template="${1:-rash.XXXXXX}"
    local tmpdir="${TMPDIR:-/tmp}"
    
    if command -v mktemp >/dev/null 2>&1; then
        mktemp -d "$tmpdir/$template"
    else
        # Fallback for systems without mktemp
        local dir="$tmpdir/$template.$$"
        mkdir -p "$dir"
        echo "$dir"
    fi
}

# Permission management
rash_chmod() {
    local perms="$1"
    local file="$2"
    
    # Validate permissions format (octal)
    case "$perms" in
        [0-7][0-7][0-7])
            chmod "$perms" "$file"
            ;;
        *)
            echo "ERROR: Invalid permission format: $perms" >&2
            return 1
            ;;
    esac
}

# String operations
rash_str_contains() {
    local string="$1"
    local substring="$2"
    
    case "$string" in
        *"$substring"*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

rash_str_starts_with() {
    local string="$1"
    local prefix="$2"
    
    case "$string" in
        "$prefix"*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

rash_str_ends_with() {
    local string="$1"
    local suffix="$2"
    
    case "$string" in
        *"$suffix")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Version comparison
rash_version_compare() {
    local version1="$1"
    local operator="$2"
    local version2="$3"
    
    # Simple version comparison - can be enhanced
    case "$operator" in
        "="|"==")
            [ "$version1" = "$version2" ]
            ;;
        "!="|"<>")
            [ "$version1" != "$version2" ]
            ;;
        *)
            echo "ERROR: Unsupported version operator: $operator" >&2
            return 1
            ;;
    esac
}