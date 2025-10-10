#!/bin/bash
# legacy-deploy.sh - PROBLEMATIC deployment script
# This script has multiple safety violations that need purification

# ISSUE 1: Non-deterministic (SC2086 potential)
SESSION_ID=$RANDOM
TIMESTAMP=$(date +%s)

# ISSUE 2: Unquoted variable expansion (SC2086)
TARGET_DIR=/app/releases/release-$TIMESTAMP
mkdir $TARGET_DIR

# ISSUE 3: Unquoted command substitution (SC2046)
FILES=$(ls /app/build)
cp -r $FILES $TARGET_DIR/

# ISSUE 4: Useless echo in command substitution (SC2116)
RELEASE_TAG=$(echo release-$TIMESTAMP)

# ISSUE 5: Non-idempotent operations
rm /app/current
ln -s $TARGET_DIR /app/current

echo "Deployed $RELEASE_TAG to $TARGET_DIR"
