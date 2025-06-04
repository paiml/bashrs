#!/usr/bin/env bash
# Check test coverage against threshold

set -euo pipefail

THRESHOLD=${1:-85}
COVERAGE_FILE="coverage-summary.txt"

if [ ! -f "$COVERAGE_FILE" ]; then
    echo "❌ Coverage file not found: $COVERAGE_FILE"
    exit 1
fi

# Extract coverage percentage from llvm-cov output
COVERAGE=$(grep -E "^\s*TOTAL" "$COVERAGE_FILE" | awk '{print $10}' | sed 's/%//' | head -1)

if [ -z "$COVERAGE" ]; then
    echo "❌ Could not extract coverage from $COVERAGE_FILE"
    exit 1
fi

echo "📊 Current coverage: ${COVERAGE}%"
echo "📊 Required threshold: ${THRESHOLD}%"

# Compare coverage with threshold
if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
    echo "❌ Coverage ${COVERAGE}% is below threshold ${THRESHOLD}%"
    exit 1
else
    echo "✅ Coverage ${COVERAGE}% meets threshold ${THRESHOLD}%"
    exit 0
fi