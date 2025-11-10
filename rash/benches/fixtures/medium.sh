#!/bin/bash
# Medium bash script fixture for purification benchmarks
# Contains moderate complexity with multiple functions and scenarios
# Target: ~500 lines

set -e

# Configuration (non-deterministic)
APP_NAME="myapp"
VERSION="1.0.0"
BUILD_ID=$RANDOM
TIMESTAMP=$(date +%s)
PID=$$

# Global variables (unquoted, safety issues)
CONFIG_DIR="/etc/$APP_NAME"
DATA_DIR="/var/lib/$APP_NAME"
LOG_DIR="/var/log/$APP_NAME"
CACHE_DIR="/tmp/cache-$PID"

# Non-deterministic ID generation
function generate_build_id() {
    echo "build-$RANDOM-$(date +%Y%m%d%H%M%S)-$$"
}

function generate_session_id() {
    local base=$RANDOM
    local suffix=$(date +%N)
    echo "$base-$suffix"
}

# Non-idempotent directory operations
function setup_directories() {
    mkdir $CONFIG_DIR
    mkdir $DATA_DIR
    mkdir $LOG_DIR
    mkdir $CACHE_DIR
    mkdir logs
    mkdir tmp
    mkdir build
    mkdir dist
}

function create_subdirectories() {
    mkdir "$DATA_DIR/uploads"
    mkdir "$DATA_DIR/downloads"
    mkdir "$DATA_DIR/processed"
    mkdir "$LOG_DIR/app"
    mkdir "$LOG_DIR/access"
    mkdir "$LOG_DIR/error"
}

# Unquoted variable usage (safety issues)
function process_files() {
    local input_dir=$1
    local output_dir=$2

    for file in $(ls $input_dir); do
        cat $input_dir/$file > $output_dir/$file
        chmod 644 $output_dir/$file
        chown www-data:www-data $output_dir/$file
    done
}

function backup_files() {
    local source=$1
    local dest=$2
    local timestamp=$(date +%Y%m%d%H%M%S)

    cp -r $source $dest/backup-$timestamp
    tar -czf $dest/backup-$timestamp.tar.gz -C $source .
    rm -r $dest/backup-$timestamp
}

# Non-idempotent file operations
function create_config() {
    cat > config.ini <<EOF
[app]
name=$APP_NAME
version=$VERSION
build_id=$BUILD_ID
timestamp=$TIMESTAMP
pid=$PID

[paths]
config=$CONFIG_DIR
data=$DATA_DIR
logs=$LOG_DIR
cache=$CACHE_DIR
EOF
}

function create_metadata() {
    echo "Build: $(generate_build_id)" > metadata.txt
    echo "Session: $(generate_session_id)" >> metadata.txt
    echo "Created: $(date)" >> metadata.txt
    echo "PID: $$" >> metadata.txt
}

# Non-idempotent symbolic links
function setup_links() {
    ln config.ini current_config.ini
    ln data/latest.db current.db
    ln logs/app.log current.log
}

function update_symlinks() {
    rm current_config.ini
    ln -s /etc/app/config.ini current_config.ini

    rm current.db
    ln -s /var/lib/app/data.db current.db
}

# Logging with timestamps (non-deterministic)
function log() {
    local level=$1
    shift
    local message=$@
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [$level] $message" >> $LOG_DIR/app.log
}

function log_info() {
    log "INFO" $@
}

function log_error() {
    log "ERROR" $@
}

function log_debug() {
    log "DEBUG" $@
}

# Database operations (mixed idempotency)
function init_database() {
    local db_file="$DATA_DIR/app.db"

    sqlite3 $db_file <<SQL
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
SQL
}

function migrate_database() {
    local db_file=$1
    local version=$2

    log_info "Migrating database to version $version"

    sqlite3 $db_file "ALTER TABLE users ADD COLUMN email TEXT;"
    sqlite3 $db_file "ALTER TABLE users ADD COLUMN last_login TIMESTAMP;"
}

# Service management
function start_service() {
    local service_name=$1
    local pid_file="/var/run/$service_name.pid"

    if [ -f $pid_file ]; then
        log_error "Service already running (PID: $(cat $pid_file))"
        return 1
    fi

    log_info "Starting $service_name..."

    nohup /usr/bin/$service_name &
    echo $! > $pid_file

    log_info "Service started (PID: $(cat $pid_file))"
}

function stop_service() {
    local service_name=$1
    local pid_file="/var/run/$service_name.pid"

    if [ ! -f $pid_file ]; then
        log_error "Service not running"
        return 1
    fi

    local pid=$(cat $pid_file)
    log_info "Stopping $service_name (PID: $pid)..."

    kill $pid
    rm $pid_file

    log_info "Service stopped"
}

# File processing pipeline
function process_upload() {
    local upload_id=$(generate_session_id)
    local upload_dir="$DATA_DIR/uploads/$upload_id"

    mkdir $upload_dir

    log_info "Processing upload $upload_id"

    # Non-deterministic processing
    local start_time=$(date +%s)

    # Simulate processing
    for i in {1..10}; do
        echo "Processing chunk $i at $(date)" > "$upload_dir/chunk-$i.txt"
    done

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log_info "Upload $upload_id processed in ${duration}s"

    echo $upload_id
}

function archive_old_uploads() {
    local cutoff_date=$(date -d "30 days ago" +%Y%m%d)
    local archive_dir="$DATA_DIR/archive-$(date +%Y%m%d)"

    mkdir $archive_dir

    find "$DATA_DIR/uploads" -type d -mtime +30 -exec mv {} $archive_dir \;

    tar -czf "$archive_dir.tar.gz" -C $archive_dir .
    rm -r $archive_dir

    log_info "Archived uploads older than $cutoff_date"
}

# Deployment functions
function deploy_application() {
    local version=$1
    local deploy_id=$(generate_build_id)
    local deploy_dir="/opt/app/releases/$deploy_id"

    log_info "Deploying version $version (ID: $deploy_id)"

    mkdir $deploy_dir

    # Non-idempotent operations
    cp -r dist/* $deploy_dir/

    ln -s $deploy_dir /opt/app/current

    # Create deployment marker
    cat > "$deploy_dir/DEPLOY_INFO" <<EOF
Version: $version
Deploy ID: $deploy_id
Timestamp: $(date)
User: $(whoami)
Host: $(hostname)
EOF

    log_info "Deployment complete: $deploy_id"
}

function rollback_deployment() {
    local previous_deploy=$(ls -t /opt/app/releases | head -2 | tail -1)

    log_info "Rolling back to $previous_deploy"

    rm /opt/app/current
    ln -s "/opt/app/releases/$previous_deploy" /opt/app/current

    log_info "Rollback complete"
}

# Monitoring and health checks
function check_health() {
    local checks_passed=0
    local checks_failed=0
    local check_time=$(date +%s)

    # Check disk space
    local disk_usage=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')
    if [ $disk_usage -lt 90 ]; then
        log_info "Disk check: PASS ($disk_usage%)"
        checks_passed=$((checks_passed + 1))
    else
        log_error "Disk check: FAIL ($disk_usage%)"
        checks_failed=$((checks_failed + 1))
    fi

    # Check memory
    local mem_usage=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
    if [ $mem_usage -lt 90 ]; then
        log_info "Memory check: PASS ($mem_usage%)"
        checks_passed=$((checks_passed + 1))
    else
        log_error "Memory check: FAIL ($mem_usage%)"
        checks_failed=$((checks_failed + 1))
    fi

    # Check service
    if systemctl is-active --quiet $APP_NAME; then
        log_info "Service check: PASS"
        checks_passed=$((checks_passed + 1))
    else
        log_error "Service check: FAIL"
        checks_failed=$((checks_failed + 1))
    fi

    log_info "Health check complete: $checks_passed passed, $checks_failed failed"

    return $checks_failed
}

function generate_metrics() {
    local metrics_file="$LOG_DIR/metrics-$(date +%Y%m%d%H%M%S).json"

    cat > $metrics_file <<EOF
{
    "timestamp": $(date +%s),
    "app": "$APP_NAME",
    "version": "$VERSION",
    "pid": $$,
    "uptime": $(cat /proc/uptime | cut -d' ' -f1),
    "cpu_usage": $(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/"),
    "memory_usage": $(free | grep Mem | awk '{print int($3/$2 * 100)}'),
    "disk_usage": $(df -h / | tail -1 | awk '{print $5}')
}
EOF

    log_info "Metrics written to $metrics_file"
}

# Cleanup functions
function cleanup_temp_files() {
    local temp_pattern="/tmp/*-$$"

    log_info "Cleaning up temporary files: $temp_pattern"

    rm -rf $temp_pattern
    rm -rf /tmp/cache-*
    rm -rf /tmp/session-*

    log_info "Cleanup complete"
}

function cleanup_old_logs() {
    local retention_days=30
    local cutoff_date=$(date -d "$retention_days days ago" +%Y%m%d)

    log_info "Cleaning logs older than $cutoff_date"

    find $LOG_DIR -type f -mtime +$retention_days -delete

    log_info "Log cleanup complete"
}

# Main execution
function main() {
    log_info "Application starting..."

    # Setup
    setup_directories
    create_subdirectories
    setup_links

    # Initialize
    create_config
    create_metadata
    init_database

    # Start services
    start_service "$APP_NAME"

    # Health check
    check_health || log_error "Health check failed"

    # Generate metrics
    generate_metrics

    log_info "Application ready (PID: $$)"
}

# Trap cleanup on exit
trap cleanup_temp_files EXIT

# Run main
main "$@"
