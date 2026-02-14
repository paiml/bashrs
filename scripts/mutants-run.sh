#!/bin/bash
# comply:disable=COMPLY-001
# bashrs disable-file=DET002,SEC010
# Run full mutation testing baseline for Sprint 26
# Usage: ./scripts/mutants-run.sh
# Note: DET002 disabled - timestamps are intentional for logging
# Note: SEC010 disabled - paths are internal/controlled

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/mutants-sprint26"
LOG_FILE="$PROJECT_ROOT/mutants-sprint26.log"

echo "=== Sprint 26: Mutation Testing Baseline ==="
echo ""
echo "Project: $(basename "$PROJECT_ROOT")"
echo "Output:  $OUTPUT_DIR"
echo "Log:     $LOG_FILE"
echo ""
echo "This will take 3-5 hours to complete..."
echo "Press Ctrl+C within 5 seconds to cancel"
sleep 5

cd "$PROJECT_ROOT/rash" || exit 1

echo ""
echo "Starting mutation testing at $(date)"
echo ""

cargo mutants \
  --exclude 'src/bash_parser/tests.rs' \
  --exclude 'src/bash_parser/property_tests.rs' \
  --exclude 'src/bash_parser/generators.rs' \
  --exclude 'src/bash_transpiler/tests.rs' \
  --exclude 'src/bin/*' \
  --exclude 'tests/*' \
  --timeout 60 \
  --jobs 4 \
  --output "$OUTPUT_DIR" \
  2>&1 | tee "$LOG_FILE"

echo ""
echo "Mutation testing completed at $(date)"
echo ""
echo "Results available in: $OUTPUT_DIR"
echo "Log saved to: $LOG_FILE"
echo ""
echo "Run ./scripts/mutants-analyze.sh to see summary"
