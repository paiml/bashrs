#!/bin/sh
# Example 4: Deployment Script
# A realistic deployment scenario using variables and navigation

# Configuration
APP_NAME="myapp"
VERSION="2.1.0"
ENV="production"

echo "=== Deployment Script ==="
echo ""

# Display configuration
echo "Application: $APP_NAME"
echo "Version: $VERSION"
echo "Environment: $ENV"
echo ""

# Prepare release
RELEASE="release-$VERSION"
echo "Creating release: $RELEASE"

# Navigate to deployment location
echo "Navigating to deployment directory..."
cd /tmp
pwd

# Create release info
echo ""
echo "Release Information:"
echo "  Name: $RELEASE"
echo "  App: $APP_NAME"
echo "  Version: $VERSION"
echo "  Environment: $ENV"

echo ""
echo "=== Deployment Complete ==="
