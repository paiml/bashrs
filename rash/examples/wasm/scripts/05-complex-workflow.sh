#!/bin/sh
# Example 5: Complex Workflow
# Demonstrates a multi-step workflow with variables and navigation

echo "=== Complex Workflow Demo ==="
echo ""

# Step 1: Initialize
echo "Step 1: Initialize"
PROJECT="webapp"
STAGE="development"
BUILD_ID="build-12345"
echo "  Project: $PROJECT"
echo "  Stage: $STAGE"
echo "  Build ID: $BUILD_ID"
echo ""

# Step 2: Setup environment
echo "Step 2: Setup Environment"
BASE_DIR="/tmp"
WORK_DIR="$PROJECT-$STAGE"
echo "  Base directory: $BASE_DIR"
echo "  Working directory: $WORK_DIR"
cd $BASE_DIR
pwd
echo ""

# Step 3: Configure
echo "Step 3: Configure"
CONFIG_FILE="config.env"
echo "  Configuration file: $CONFIG_FILE"
TIMESTAMP="2024-10-24"
echo "  Timestamp: $TIMESTAMP"
echo ""

# Step 4: Process
echo "Step 4: Process"
PROCESS_COUNT="4"
THREAD_COUNT="8"
echo "  Processes: $PROCESS_COUNT"
echo "  Threads: $THREAD_COUNT"
echo ""

# Step 5: Validate
echo "Step 5: Validate"
VALIDATION="passed"
echo "  Validation status: $VALIDATION"
echo ""

# Step 6: Deploy
echo "Step 6: Deploy"
DEPLOY_TARGET="/home"
echo "  Deploying to: $DEPLOY_TARGET"
cd $DEPLOY_TARGET
pwd
echo ""

# Step 7: Verify
echo "Step 7: Verify"
STATUS="active"
HEALTH="healthy"
echo "  Status: $STATUS"
echo "  Health: $HEALTH"
echo ""

# Summary
echo "=== Workflow Summary ==="
echo "Project $PROJECT ($STAGE)"
echo "Build: $BUILD_ID"
echo "Status: $STATUS ($HEALTH)"
echo "Location: $DEPLOY_TARGET"
echo ""
echo "Workflow completed successfully!"
