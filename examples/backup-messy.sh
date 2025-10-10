#!/bin/bash
# backup-messy.sh - PROBLEMATIC database backup script
# This demonstrates common non-deterministic and non-idempotent patterns

# Random backup ID
BACKUP_ID="backup-$RANDOM-$(date +%s)"

# Process-dependent temp directory
TEMP_DIR="/tmp/dbbackup-$$"
mkdir $TEMP_DIR

# Non-deterministic backup filename
BACKUP_FILE="/backups/db-$(date +%Y%m%d-%H%M%S).sql.gz"

# Perform backup
pg_dump mydb > "$TEMP_DIR/dump.sql"
gzip "$TEMP_DIR/dump.sql"
mv "$TEMP_DIR/dump.sql.gz" "$BACKUP_FILE"

# Cleanup (non-idempotent)
rm -r $TEMP_DIR

# Log with timestamp
echo "[$(date)] Backup $BACKUP_ID completed: $BACKUP_FILE" >> /var/log/backups.log

echo "Backup ID: $BACKUP_ID"
echo "File: $BACKUP_FILE"
