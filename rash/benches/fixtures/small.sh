#!/bin/bash
# Small bash script fixture for purification benchmarks
# Contains basic purification opportunities (determinism, idempotency)
# Target: ~50 lines

# Non-deterministic: $RANDOM
function generate_id() {
    local id=$RANDOM
    echo "ID-$id"
}

# Non-deterministic: $$
function create_temp() {
    TEMP_DIR="/tmp/build-$$"
    mkdir $TEMP_DIR
}

# Non-idempotent: mkdir without -p
function setup_dirs() {
    mkdir logs
    mkdir data
    mkdir cache
}

# Unquoted variables (safety issue)
function process_file() {
    local filename=$1
    cat $filename > output.txt
    rm $filename
}

# Non-deterministic: date
function log_message() {
    local msg=$1
    echo "[$(date)] $msg" >> app.log
}

# Non-idempotent: ln without -sf
function create_link() {
    ln config.txt current_config.txt
}

# Main execution
generate_id
create_temp
setup_dirs
process_file "test.txt"
log_message "Application started"
create_link

# Non-deterministic: timestamp
BUILD_TIME=$(date)
echo "Build completed at $BUILD_TIME"
