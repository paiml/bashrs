#!/bin/bash
# Deployment Script - ORIGINAL (MESSY)
#
# Common deployment script with typical problems:
# - Non-deterministic release names (timestamps)
# - Non-deterministic session IDs ($RANDOM)
# - Non-idempotent symlink operations
# - Unsafe variable usage
# - Poor error handling

set -e

APP_NAME="myapp"
DEPLOY_ROOT="/opt/apps"

# Non-deterministic release name using timestamp
RELEASE_NAME="release-$(date +%s)"
SESSION_ID=$RANDOM

echo "=== Deployment Started ==="
echo "App: $APP_NAME"
echo "Release: $RELEASE_NAME"
echo "Session: $SESSION_ID"
echo "Timestamp: $(date)"

# Create release directory (non-idempotent)
RELEASE_DIR="$DEPLOY_ROOT/$APP_NAME/releases/$RELEASE_NAME"
mkdir -p $RELEASE_DIR

# Copy build artifacts
echo "Copying build artifacts..."
cp -r build/* $RELEASE_DIR/

# Create/update symlink (non-idempotent)
CURRENT_LINK="$DEPLOY_ROOT/$APP_NAME/current"
rm $CURRENT_LINK  # Will fail if link doesn't exist
ln -s $RELEASE_DIR $CURRENT_LINK

# Update config (non-idempotent)
mkdir $DEPLOY_ROOT/$APP_NAME/config
cp config/production.yml $DEPLOY_ROOT/$APP_NAME/config/app.yml

# Restart service
echo "Restarting service..."
systemctl restart $APP_NAME

# Log deployment (non-deterministic)
LOG_FILE="/var/log/${APP_NAME}-deploy.log"
echo "$(date): Deployed $RELEASE_NAME (session $SESSION_ID)" >> $LOG_FILE

# Cleanup old releases (keep last 5)
cd $DEPLOY_ROOT/$APP_NAME/releases
ls -t | tail -n +6 | xargs rm -rf

echo "=== Deployment Complete ==="
echo "Release: $RELEASE_NAME"
echo "Session: $SESSION_ID"
