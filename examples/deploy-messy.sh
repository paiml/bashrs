#!/bin/bash
# deploy-messy.sh - PROBLEMATIC bash script demonstrating issues
# This is the "before" example showing non-deterministic and non-idempotent code

# Non-deterministic: uses $RANDOM
SESSION_ID=$RANDOM

# Non-deterministic: uses timestamps
RELEASE_TAG="release-$(date +%Y%m%d-%H%M%S)"

# Process-dependent paths
WORK_DIR="/tmp/deploy-$$"
LOG_FILE="/var/log/deploy-$SECONDS.log"

# Non-idempotent operations
rm /app/current
mkdir /app/releases/$RELEASE_TAG

# Extract archive
tar xzf app.tar.gz -C /app/releases/$RELEASE_TAG

# Create symlink (fails if exists)
ln -s /app/releases/$RELEASE_TAG /app/current

# Record deployment
echo "Session $SESSION_ID: Deployed $RELEASE_TAG at $(date)" >> $LOG_FILE

echo "Deployment complete: $RELEASE_TAG"
echo "Session: $SESSION_ID"
echo "Logs: $LOG_FILE"
