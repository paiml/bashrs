#!/bin/bash
# Database Migration Script - ORIGINAL (Problematic)
#
# Common issues in database migration scripts:
# - No rollback capability
# - Non-deterministic backup names (timestamps)
# - No validation before migration
# - Unsafe error handling
# - No idempotency

set -e

DB_HOST="localhost"
DB_NAME="production_db"
DB_USER="admin"

# Non-deterministic backup name
BACKUP_FILE="/var/backups/db_backup_$(date +%Y%m%d_%H%M%S).sql"
MIGRATION_LOG="/var/log/migrations.log"

echo "=== Database Migration Started ==="
echo "Date: $(date)"

# Non-idempotent: no check if migration already applied
echo "Running migration to add user_preferences table..."

# No backup verification
echo "Creating backup: $BACKUP_FILE"
mysqldump -h $DB_HOST -u $DB_USER -p$DB_PASSWORD $DB_NAME > $BACKUP_FILE

# Unsafe: password in process list
# Unquoted variables

# Check if migration needed (but doesn't prevent re-run)
TABLE_EXISTS=$(mysql -h $DB_HOST -u $DB_USER -p$DB_PASSWORD $DB_NAME \
    -e "SHOW TABLES LIKE 'user_preferences'" | wc -l)

if [[ $TABLE_EXISTS -eq 0 ]]; then
    echo "Applying migration..."

    # Migration SQL (no transaction, partial failure possible)
    mysql -h $DB_HOST -u $DB_USER -p$DB_PASSWORD $DB_NAME <<EOF
CREATE TABLE user_preferences (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    theme VARCHAR(50),
    language VARCHAR(10)
);

CREATE INDEX idx_user_id ON user_preferences(user_id);

ALTER TABLE users ADD COLUMN preferences_id INT;
EOF

    echo "Migration completed successfully"
    echo "$(date): Migration applied - user_preferences table created" >> $MIGRATION_LOG
else
    echo "Table already exists, skipping migration"
fi

# No verification that migration succeeded
# No rollback on failure
# Backup file name is lost (timestamp-based)

echo "=== Migration Complete ==="
