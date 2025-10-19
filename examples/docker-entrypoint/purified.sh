#!/bin/sh
# Docker Entrypoint Script - PURIFIED by Rash v2.0.0
#
# Improvements:
# - POSIX compliant (works on Alpine Linux with busybox sh)
# - No bash-specific features
# - All variables quoted
# - Idempotent operations
# - Safe signal handling
# - No eval/source (security)
# - Proper error handling

set -e

# POSIX-compatible: space-separated list instead of array
CONFIG_FILES="/etc/myapp/config.yml /etc/myapp/secrets.yml"

# POSIX test: [ ] instead of [[ ]]
if [ -n "${DEBUG:-}" ]; then
    set -x
fi

# POSIX function syntax (no 'function' keyword)
setup_logging() {
    log_dir="/var/log/myapp"

    # Idempotent: mkdir -p
    mkdir -p "${log_dir}" || exit 1

    # Quoted variable
    touch "${log_dir}/app.log" || exit 1

    # POSIX-compatible: direct redirection (no process substitution)
    # Note: Can't use tee in POSIX without subshell
    # Alternative: log to file, tail -f in separate process
    exec >> "${log_dir}/app.log" 2>&1
}

# Cleanup handler (called on exit)
cleanup() {
    printf 'Shutting down...\n'
    # Send SIGTERM to all child processes
    # Use jobs -p for POSIX compatibility
    for pid in $(jobs -p); do
        kill "${pid}" 2>/dev/null || true
    done
}

# Setup signal handlers (idempotent)
trap cleanup TERM INT

# Initialize app
setup_logging

printf 'Starting application...\n'

# POSIX-compatible: iterate space-separated list
for config in ${CONFIG_FILES}; do
    if [ -f "${config}" ]; then
        printf 'Found config: %s\n' "${config}"
        # Security: Don't source config files
        # Instead, expect environment variables to be set
        printf 'Note: Config files not sourced for security\n'
    else
        printf 'Warning: Config file not found: %s\n' "${config}"
    fi
done

# POSIX test for required environment
if [ -z "${DATABASE_URL:-}" ]; then
    printf 'Error: DATABASE_URL not set\n' >&2
    exit 1
fi

# Start application with exec
# All variables quoted
# Use ${VAR:-default} for optional vars
printf 'Starting myapp...\n'
printf 'Database: %s\n' "${DATABASE_URL}"
printf 'Port: %s\n' "${PORT:-8080}"

exec /usr/local/bin/myapp \
    --db "${DATABASE_URL}" \
    --port "${PORT:-8080}"
