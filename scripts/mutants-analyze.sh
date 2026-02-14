#!/bin/bash
# comply:disable=COMPLY-001
# Analyze mutation testing results from Sprint 26
# Usage: ./scripts/mutants-analyze.sh [output-dir]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${1:-$PROJECT_ROOT/mutants-sprint26}"

if [ ! -d "$OUTPUT_DIR" ]; then
    echo "Error: Output directory not found: $OUTPUT_DIR"
    echo "Usage: $0 [output-dir]"
    exit 1
fi

echo "=== Sprint 26: Mutation Testing Results ==="
echo ""
echo "Output Directory: $OUTPUT_DIR"
echo ""

# Count mutants by outcome
CAUGHT=$(cat "$OUTPUT_DIR/caught.txt" 2>/dev/null | wc -l)
MISSED=$(cat "$OUTPUT_DIR/missed.txt" 2>/dev/null | wc -l)
TIMEOUT=$(cat "$OUTPUT_DIR/timeout.txt" 2>/dev/null | wc -l)
UNVIABLE=$(cat "$OUTPUT_DIR/unviable.txt" 2>/dev/null | wc -l)

TESTED=$((CAUGHT + MISSED + TIMEOUT))
TOTAL=$((TESTED + UNVIABLE))

echo "Total Mutants:   $TOTAL"
echo "  Tested:        $TESTED"
echo "  Unviable:      $UNVIABLE"
echo ""
echo "Test Results:"
echo "  ✓ Caught:      $CAUGHT"
echo "  ✗ Missed:      $MISSED"
echo "  ⏱ Timeout:      $TIMEOUT"
echo ""

# Calculate kill rate
if [ "$TESTED" -gt 0 ]; then
    KILL_RATE=$(echo "scale=2; ($CAUGHT * 100) / $TESTED" | bc)
    echo "Kill Rate:       ${KILL_RATE}%"

    if (( $(echo "$KILL_RATE >= 90" | bc -l) )); then
        echo "Status:          ✅ TARGET ACHIEVED (≥90%)"
    else
        GAP=$(echo "scale=2; 90 - $KILL_RATE" | bc)
        echo "Status:          🟡 GAP: ${GAP}% below target"
    fi
else
    echo "Kill Rate:       N/A (no mutants tested)"
fi

echo ""
echo "=== Survivors by Module ==="
echo ""

for module in bash_parser ir emitter verifier linter ast stdlib; do
    if [ -f "$OUTPUT_DIR/missed.txt" ]; then
        COUNT=$(grep "src/$module/" "$OUTPUT_DIR/missed.txt" 2>/dev/null | wc -l)
        if [ "$COUNT" -gt 0 ]; then
            printf "%-15s: %3d survivors\n" "$module" "$COUNT"
        fi
    fi
done

echo ""
echo "=== Next Steps ==="
echo ""
echo "1. Review survivors: cat $OUTPUT_DIR/missed.txt"
echo "2. Categorize by pattern: ./scripts/mutants-categorize.sh"
echo "3. Write targeted tests to kill survivors"
echo "4. Re-run: ./scripts/mutants-run.sh"
echo ""
