#!/bin/sh
# Example 3: Directory Navigation
# Shows how cd and pwd work in the virtual filesystem

echo "=== Directory Navigation Demo ==="

echo "Starting location:"
pwd

echo ""
echo "Moving to /tmp:"
cd /tmp
pwd

echo ""
echo "Moving to /home:"
cd /home
pwd

echo ""
echo "Back to root:"
cd /
pwd

echo ""
echo "Navigation complete!"
