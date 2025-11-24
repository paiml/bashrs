#!/bin/sh
# Clean, safe POSIX shell script
set -euo pipefail

# Properly quoted variables
name="${1:-default}"
echo "Hello, ${name}"

# Safe file operations
if [ -f "config.txt" ]; then
    echo "Config file exists"
fi

# Idempotent operations
mkdir -p "/tmp/bashrs_test"
rm -f "/tmp/bashrs_test/old_file"

exit 0
