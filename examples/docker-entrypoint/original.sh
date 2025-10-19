#!/bin/bash
# Docker Entrypoint Script - ORIGINAL (Problematic)
#
# Common issues in container entrypoint scripts:
# - Bash-specific features (Alpine doesn't have bash)
# - Process substitution
# - Arrays
# - [[  ]] test syntax
# - Non-idempotent signal handlers
# - Hardcoded paths

set -e

# Bash-specific: declare array
declare -a CONFIG_FILES=("/etc/myapp/config.yml" "/etc/myapp/secrets.yml")

# Bash-specific: [[ ]] test
if [[ -n "$DEBUG" ]]; then
    set -x
fi

# Function with bash-isms
function setup_logging() {
    local log_dir="/var/log/myapp"

    # Non-idempotent: fails if exists
    mkdir $log_dir

    # Unquoted variable
    touch $log_dir/app.log

    # Bash-specific: process substitution
    exec > >(tee -a $log_dir/app.log)
    exec 2>&1
}

# Setup signal handlers (non-idempotent)
trap 'echo "Shutting down..."; pkill -P $$' SIGTERM SIGINT

# Initialize app
setup_logging

echo "Starting application..."

# Bash-specific: iterate array
for config in "${CONFIG_FILES[@]}"; do
    if [[ -f "$config" ]]; then
        echo "Loading config: $config"
        source $config  # Dangerous: eval-like behavior
    fi
done

# Check environment (bash-specific test)
if [[ -z "${DATABASE_URL}" ]]; then
    echo "Error: DATABASE_URL not set"
    exit 1
fi

# Start application with exec
# Unquoted variables
exec /usr/local/bin/myapp --db $DATABASE_URL --port ${PORT:-8080}
