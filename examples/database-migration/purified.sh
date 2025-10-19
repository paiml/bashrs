#!/bin/sh
# Database Migration Script - PURIFIED by Rash v2.0.0
#
# Improvements:
# - Deterministic backup names (version-based)
# - Rollback capability
# - Pre-migration validation
# - Idempotent (tracks applied migrations)
# - Transactional migration
# - All variables quoted
# - Secure password handling
# - POSIX compliant

set -e

DB_HOST="${DB_HOST:-localhost}"
DB_NAME="${DB_NAME:-production_db}"
DB_USER="${DB_USER:-admin}"
MIGRATION_VERSION="${1:-}"

# Validate migration version provided
if [ -z "${MIGRATION_VERSION}" ]; then
    printf 'Usage: %s <migration_version>\n' "$0"
    printf 'Example: %s 001_add_user_preferences\n' "$0"
    exit 1
fi

# Deterministic backup file (version-based, not timestamp)
BACKUP_DIR="/var/backups/db"
BACKUP_FILE="${BACKUP_DIR}/backup_${DB_NAME}_${MIGRATION_VERSION}.sql"
MIGRATION_LOG="/var/log/migrations.log"
MIGRATION_TRACKER="${BACKUP_DIR}/migrations_applied.txt"

printf '=== Database Migration Started ===\n'
printf 'Migration: %s\n' "${MIGRATION_VERSION}"
printf 'Database: %s\n' "${DB_NAME}"

# Create backup directory (idempotent)
mkdir -p "${BACKUP_DIR}" || exit 1

# Check if migration already applied (idempotent)
if [ -f "${MIGRATION_TRACKER}" ] && grep -q "^${MIGRATION_VERSION}\$" "${MIGRATION_TRACKER}"; then
    printf 'Migration %s already applied, skipping\n' "${MIGRATION_VERSION}"
    exit 0
fi

# Secure password handling (from environment or file, not command line)
if [ -z "${DB_PASSWORD:-}" ]; then
    if [ -f "${HOME}/.db_password" ]; then
        DB_PASSWORD=$(cat "${HOME}/.db_password")
    else
        printf 'Error: DB_PASSWORD not set and ~/.db_password not found\n' >&2
        exit 1
    fi
fi

# Pre-migration validation: check database connectivity
printf 'Validating database connection...\n'
if ! mysql -h "${DB_HOST}" -u "${DB_USER}" -p"${DB_PASSWORD}" \
    -e "SELECT 1" "${DB_NAME}" > /dev/null 2>&1; then
    printf 'Error: Cannot connect to database\n' >&2
    exit 1
fi
printf 'Database connection verified\n'

# Create backup (deterministic name for rollback)
printf 'Creating backup: %s\n' "${BACKUP_FILE}"
mysqldump -h "${DB_HOST}" -u "${DB_USER}" -p"${DB_PASSWORD}" \
    "${DB_NAME}" > "${BACKUP_FILE}" || {
    printf 'Error: Backup failed\n' >&2
    exit 1
}

# Verify backup created successfully
if [ ! -f "${BACKUP_FILE}" ]; then
    printf 'Error: Backup file not created\n' >&2
    exit 1
fi

backup_size=$(wc -c < "${BACKUP_FILE}")
if [ "${backup_size}" -lt 100 ]; then
    printf 'Error: Backup file too small (%d bytes), may be invalid\n' "${backup_size}" >&2
    exit 1
fi
printf 'Backup verified (%d bytes)\n' "${backup_size}"

# Apply migration in transaction (rollback on failure)
printf 'Applying migration...\n'

# Create migration SQL file with transaction
MIGRATION_SQL="/tmp/migration_${MIGRATION_VERSION}.sql"
cat > "${MIGRATION_SQL}" <<'EOF'
START TRANSACTION;

-- Create user_preferences table (idempotent with IF NOT EXISTS)
CREATE TABLE IF NOT EXISTS user_preferences (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    theme VARCHAR(50),
    language VARCHAR(10),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index (idempotent - only if doesn't exist)
CREATE INDEX IF NOT EXISTS idx_user_id ON user_preferences(user_id);

-- Alter users table (safe - checks if column exists)
-- Note: MySQL doesn't have IF NOT EXISTS for ALTER TABLE ADD COLUMN
-- So we use a stored procedure for safety
DELIMITER //
CREATE PROCEDURE add_preferences_column()
BEGIN
    IF NOT EXISTS (
        SELECT * FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
        AND TABLE_NAME = 'users'
        AND COLUMN_NAME = 'preferences_id'
    ) THEN
        ALTER TABLE users ADD COLUMN preferences_id INT;
    END IF;
END//
DELIMITER ;

CALL add_preferences_column();
DROP PROCEDURE add_preferences_column;

COMMIT;
EOF

# Execute migration
if mysql -h "${DB_HOST}" -u "${DB_USER}" -p"${DB_PASSWORD}" \
    "${DB_NAME}" < "${MIGRATION_SQL}"; then
    printf 'Migration applied successfully\n'

    # Record migration as applied (idempotent tracking)
    printf '%s\n' "${MIGRATION_VERSION}" >> "${MIGRATION_TRACKER}"

    # Log migration (deterministic)
    printf '%s: Migration applied successfully\n' "${MIGRATION_VERSION}" >> "${MIGRATION_LOG}"

    # Clean up temp SQL file
    rm -f "${MIGRATION_SQL}"

    printf '=== Migration Complete ===\n'
    printf 'Backup available at: %s\n' "${BACKUP_FILE}"
else
    printf 'Error: Migration failed\n' >&2
    printf 'Rolling back is available via: %s\n' "${BACKUP_FILE}"
    rm -f "${MIGRATION_SQL}"
    exit 1
fi

# Verify migration succeeded
printf 'Verifying migration...\n'
table_count=$(mysql -h "${DB_HOST}" -u "${DB_USER}" -p"${DB_PASSWORD}" \
    "${DB_NAME}" -sN -e "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'user_preferences'")

if [ "${table_count}" -eq 1 ]; then
    printf 'Verification passed: user_preferences table exists\n'
else
    printf 'Warning: Verification failed, table may not exist\n' >&2
fi
